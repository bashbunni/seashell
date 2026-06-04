use is_executable::IsExecutable;
use std::fs::File;
use std::io::{self, stderr, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::{env, str::FromStr};
use strum_macros::{Display, EnumString};

fn main() {
    // eval loop
    loop {
        run()
    }
}

fn run() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("unable to read user input");
    eval(&input);
}

/* commands */

// evaluate commands
fn eval(input: &str) {
    let result = parse_quotes(input);
    let (cmd, mut args) = tokenize_input(&result);
    // do nothing if they only hit enter.
    if let Ok(Command::Enter) = Command::from_str(&cmd) {
        return;
    }
    if cmd.is_empty() {
        // TODO eventually return a messsage stating the command is empty. (Not
        // sure if this will affect CodeCrafters tests)
        return;
    }

    // send all outputs to stdout so we can redirect them to a file as needed.
    let mut buf: BufWriter<Box<dyn Write>>;
    if let Some(out_path) = redirect_path(&mut args) {
        let file = File::create(&out_path)
            .unwrap_or_else(|err| panic!("unable to open file {out_path} for writing: {err}"));
        buf = BufWriter::new(Box::new(file));
    } else {
        buf = BufWriter::new(Box::new(io::stdout()));
    }

    match Command::from_str(&cmd) {
        Ok(Command::Exit) => std::process::exit(0),
        Ok(Command::Echo) => writeln!(buf, "{}", args.join(" ")).unwrap(),
        Ok(Command::Type) => Command::handle_type(&mut buf, args),
        Ok(Command::Pwd) => Command::handle_pwd(&mut buf),
        Ok(Command::Cd) => Command::handle_cd(&mut buf, args),
        _ => {
            exec(&mut buf, &cmd, args);
        }
    }
}

// return redirect output path
fn redirect_path(input: &mut Vec<String>) -> Option<String> {
    if let Some(index) = input
        .iter()
        .position(|x| **x == String::from(">") || **x == String::from("1>"))
    {
        let out_path = input.get(index + 1).cloned();
        input.remove(index + 1); // rm the file name
        input.remove(index); // rm '>'
        return out_path;
    }
    None
}

// get command and args
fn tokenize_input(input: &[String]) -> (String, Vec<String>) {
    let (mut cmd, mut args) = (String::new(), vec![]);
    if let Some(first_arg) = input.first() {
        cmd = first_arg.to_string();
        if input.len() > 1 {
            args = input[1..]
                .iter()
                .filter_map(|arg| {
                    if !arg.is_empty() {
                        // note: slices will always work with references
                        return Some(arg.to_owned());
                    }
                    None
                })
                .collect();
        }
    }
    (cmd, args)
}

// execute a command
fn exec(
    buf: &mut BufWriter<Box<dyn Write>>,
    cmd: &str,
    args: Vec<String>,
) -> Option<process::Output> {
    match find_executable(cmd) {
        Some(exec_path) => {
            let mut exec_cmd = std::process::Command::new(exec_path.file_name().unwrap());
            let result = exec_cmd.args(args).output();
            match result {
                // TODO this is causing me PAIN
                // e.g. ls -1 <path> > my_file doesn't print to the correct writer
                Ok(output) => {
                    buf.write_all(&output.stdout).ok();
                    buf.flush().ok();
                    stderr().write_all(&output.stderr).ok();
                    stderr().flush().ok();
                    Some(output)
                }
                Err(err) => {
                    eprintln!("unable to execute command: {err}");
                    None
                }
            }
        }
        None => {
            eprintln!("{}: command not found", cmd.trim());
            None
        }
    }
}

// return executable path if one is found.
fn find_executable(input: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|path| {
        env::split_paths(&path).find_map(|dir| {
            let exec_path = dir.join(input);
            exec_path.is_executable().then_some(exec_path)
        })
    })
}

#[derive(EnumString, Display)]
enum Command {
    #[strum(serialize = "\n")]
    Enter,
    #[strum(serialize = "exit")]
    Exit,
    #[strum(serialize = "echo")]
    Echo,
    #[strum(serialize = "type")]
    Type,
    #[strum(serialize = "pwd")]
    Pwd,
    #[strum(serialize = "cd")]
    Cd,
}

impl Command {
    fn handle_type(buf: &mut BufWriter<Box<dyn Write>>, args: Vec<String>) {
        for arg in args {
            match Command::from_str(&arg) {
                Ok(_) => writeln!(buf, "{arg} is a shell builtin").unwrap(),
                Err(_) => match find_executable(&arg) {
                    Some(exec_path) => writeln!(buf, "{arg} is {}", exec_path.display()).unwrap(),
                    None => writeln!(buf, "{arg}: not found").unwrap(),
                },
            }
        }
    }

    fn handle_pwd(buf: &mut BufWriter<Box<dyn Write>>) {
        match env::current_dir() {
            Ok(pwd) => writeln!(buf, "{}", pwd.display()).unwrap(),
            Err(e) => eprintln!("unexpected error: {e}"),
        }
    }

    fn handle_cd(buf: &mut BufWriter<Box<dyn Write>>, args: Vec<String>) {
        for arg in args {
            let path = Path::new(&arg);
            if path.is_dir() && env::set_current_dir(&arg).is_ok() {
                return;
            };
            writeln!(buf, "cd: {}: No such file or directory", path.display()).unwrap();
        }
    }
}

/* parsing */

enum Mode {
    SingleQuote,
    DoubleQuote,
    None,
}

fn parse_quotes(input: &str) -> Vec<String> {
    let mut mode = Mode::None;
    let mut arg: String = String::new();
    let mut args: Vec<String> = vec![];
    let mut prev_char: char = char::default();
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        match (&mode, ch) {
            (Mode::SingleQuote, '\'') => mode = Mode::None, // this is the end
            (Mode::SingleQuote, _) => arg.push(ch),

            (Mode::DoubleQuote, '\"') => mode = Mode::None,
            (Mode::DoubleQuote, '\\') => {
                if let Some(next) = chars.next() {
                    arg.push(next);
                }
            }
            (Mode::DoubleQuote, _) => arg.push(ch),

            (Mode::None, '\'') => mode = Mode::SingleQuote,
            (Mode::None, '\"') => mode = Mode::DoubleQuote,
            (Mode::None, '\\') => {
                if let Some(next) = chars.next() {
                    arg.push(next);
                }
            }
            (Mode::None, '>') => {
                // always set redirect command as its own arg, splitting existing arg if needed.
                // e.g. echo hello>world
                // TODO check if prev char is 1, if so remove it from arg before push_arg
                if let Some(last_arg) = arg.chars().last() {
                    if last_arg == '1' {
                        arg.remove(arg.len() - 1);
                        if !arg.is_empty() {
                            push_arg(&mut args, &mut arg);
                        }
                        push_arg(&mut args, &mut String::from("1>"))
                    }
                } else {
                    push_arg(&mut args, &mut String::from(">"))
                }
            }
            (Mode::None, _) => {
                if is_ignored_whitespace(ch, prev_char) {
                } else if ch == ' ' {
                    push_arg(&mut args, &mut arg);
                } else {
                    arg.push_str(&handle_special_chars(ch));
                }
            }
        }
        prev_char = ch;
    }

    // add final word, if it exists (args split on spaces otherwise).
    if !arg.is_empty() {
        push_arg(&mut args, &mut arg);
    }
    args
}

fn is_ignored_whitespace(ch: char, prev_char: char) -> bool {
    if prev_char == ' ' && ch == ' ' || ch.is_ascii_whitespace() && ch != ' ' {
        return true;
    }
    false
}

// push and clear arg
fn push_arg(args: &mut Vec<String>, arg: &mut String) {
    args.push(arg.to_string());
    arg.clear();
}

fn handle_special_chars(ch: char) -> String {
    match ch {
        '~' => format!("{}", env::home_dir().unwrap_or_default().display()),
        _ => ch.to_string(),
    }
}

/* tests */

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // backslash in double quotes
    #[test]
    fn test_double_quote_escape() {
        let result = parse_quotes(r#""just'one'\\n'backslash""#);
        assert_eq!(result, vec![r#"just'one'\n'backslash"#]);

        let result = parse_quotes(r#""inside\"literal_quote."outside\"""#);
        assert_eq!(result, vec![r#"inside"literal_quote.outside""#]);
    }

    // backslashes
    #[test]
    fn test_backslash() {
        let result = format!("{}", parse_quotes(r#"multiple\ \ \ \ spaces"#).join(" "));
        assert_eq!(result, "multiple    spaces");

        // inside quotes
        let result = parse_quotes(r"'shell\\\nscript'");
        assert_eq!(result, vec![r"shell\\\nscript"]);

        let result = parse_quotes(r#"'example\"test'"#);
        assert_eq!(result, vec![r#"example\"test"#]);

        let result = parse_quotes(r#"'multiple\\slashes'"#);
        assert_eq!(result, vec![r"multiple\\slashes"]);

        let result = parse_quotes(r#"'every\"thing_is\"literal'"#);
        assert_eq!(result, vec![r#"every\"thing_is\"literal"#]);
    }

    // double quotes
    #[test]
    fn test_double_quotes() {
        let result = parse_quotes("\"hello    world\"");
        assert_eq!(result, vec!["hello    world"]);

        let result = parse_quotes("\"hello\"\"world\"");
        assert_eq!(result, vec!["helloworld"]);

        let result = parse_quotes("\"hello\"world");
        assert_eq!(result, vec!["helloworld"]);

        let result = parse_quotes("\"hello\" \"world\"");
        assert_eq!(result, vec!["hello", "world"]);

        let result = parse_quotes("\"shell\'s  test\"");
        assert_eq!(result, vec!["shell\'s  test"]);

        let result = parse_quotes("\"hello \'world~\' yes     \"");
        assert_eq!(result, vec!["hello 'world~' yes     "]);

        let result = parse_quotes("\"hello \'world~\' yes     \"");
        assert_eq!(result, vec!["hello 'world~' yes     "]);
    }

    // single quotes
    #[test]
    fn test_parse_spaces_no_quotes() {
        let args = parse_quotes("world     shell");
        assert_eq!(args, vec!["world", "shell"]);
    }

    #[test]
    fn test_single_quotes() {
        let args = parse_quotes("'test     example' 'world''shell' script''hello");
        assert_eq!(args, vec!["test     example", "worldshell", "scripthello"]);

        let args = parse_quotes("\'hello    \"worlds \"   \'");
        assert_eq!(args, vec!["hello    \"worlds \"   "]);
    }

    #[test]
    fn test_cat_with_quoted_file_paths() {
        let base_dir = Path::new("/tmp/ant");
        fs::remove_dir_all(base_dir).ok();
        fs::create_dir_all(base_dir).expect("unable to create test directory");

        let first = base_dir.join("f   61");
        let second = base_dir.join("f   95");
        let third = base_dir.join("f   36");

        fs::write(&first, "apple strawberry.").expect("unable to write first file");
        fs::write(&second, "strawberry banana.").expect("unable to write second file");
        fs::write(&third, "pineapple banana.\n").expect("unable to write third file");

        let args = parse_quotes("'/tmp/ant/f   61' '/tmp/ant/f   95' '/tmp/ant/f   36'");
        assert_eq!(
            args,
            vec![
                "/tmp/ant/f   61".to_string(),
                "/tmp/ant/f   95".to_string(),
                "/tmp/ant/f   36".to_string()
            ]
        );

        // TODO fix this test
        let mut buf: BufWriter<Box<dyn Write>> = BufWriter::new(Box::new(io::stdout()));
        let output = exec(&mut buf, "cat", args).expect("unable to execute cat");
        let process::Output {
            status,
            stdout,
            stderr,
        } = output;
        let stdout = String::from_utf8(stdout).expect("cat stdout should be valid utf-8");
        let stderr = String::from_utf8(stderr).expect("cat stderr should be valid utf-8");

        assert!(
            status.success(),
            "cat failed\nstdout: {stdout:?}\nstderr: {stderr:?}"
        );
        assert_eq!(
            stdout.trim_end_matches('\n'),
            "apple strawberry.strawberry banana.pineapple banana.",
            "unexpected cat output\nstdout: {stdout:?}\nstderr: {stderr:?}"
        );

        fs::remove_dir_all(base_dir).ok();
    }

    #[test]
    fn basic_requirements() {
        let result = parse_quotes("'hello    world'");
        assert_eq!(result, vec!["hello    world"]);

        let result2 = parse_quotes("'hello''world'");
        assert_eq!(result2, vec!["helloworld"]);

        let result3 = parse_quotes("hello''world");
        assert_eq!(result3, vec!["helloworld"]);
    }

    #[test]
    fn quoted_retains_spaces() {
        let result = parse_quotes("\'hello       \' world");
        assert_eq!(result, vec!["hello       ", "world"]);
    }

    // TODO clarify + clean up these tests
    //    #[test]
    //    fn quoted_ignores_carriage_returns() {
    //        let result = parse_quotes("hello world\r");
    //        assert_eq!(result, vec!["hello world"]);
    //
    //        let result2 = parse_quotes("hello \'world\r\'");
    //        assert_eq!(result2, vec!["hello", r#"world\r"#]);
    //    }
}
