use is_executable::IsExecutable;
use std::io::{self, Write};
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
    fn handle_type(args: Vec<String>) {
        for arg in args {
            match Command::from_str(&arg) {
                Ok(_) => println!("{arg} is a shell builtin"),
                Err(_) => match find_executable(&arg) {
                    Some(exec_path) => println!("{arg} is {}", exec_path.display()),
                    None => println!("{arg}: not found"),
                },
            }
        }
    }

    fn handle_pwd() {
        match env::current_dir() {
            Ok(pwd) => println!("{}", pwd.display()),
            Err(e) => eprintln!("unexpected error: {e}"),
        }
    }

    fn handle_cd(args: Vec<String>) {
        for arg in args {
            let path = Path::new(&arg);
            if path.is_dir() && env::set_current_dir(&arg).is_ok() {
                return;
            };
            println!("cd: {}: No such file or directory", path.display());
        }
    }
}

// evaluate commands
fn eval(input: &str) {
    let (mut cmd, remainder) = input.split_once(" ").unwrap_or((input, ""));
    // do nothing if they hit enter.
    if let Ok(Command::Enter) = Command::from_str(cmd) {
        return;
    }

    cmd = cmd.trim();
    let args = parse_args(remainder);

    match Command::from_str(cmd) {
        Ok(Command::Exit) => std::process::exit(0),
        Ok(Command::Echo) => println!("{}", args.join(" ")),
        Ok(Command::Type) => Command::handle_type(args),
        Ok(Command::Pwd) => Command::handle_pwd(),
        Ok(Command::Cd) => Command::handle_cd(args),
        _ => {
            exec(cmd, args);
        }
    }
}

// retain exact characters if within quotes. all args are separated by spaces,
// if there is no space, just quotes, they are still treated as the same
// argument.
fn parse_args(input: &str) -> Vec<String> {
    let mut in_single_quote: bool = false;
    let mut in_double_quote: bool = false;
    let mut arg: String = String::new();
    let mut args: Vec<String> = vec![];
    let mut prev_char: char = char::default();
    for ch in input.chars() {
        if prev_char == '\\' {
            arg.push(ch);
            prev_char = ch;
            continue;
        }
        match ch {
            // don't write backslash, but preserve it as prev_char
            '\\' => (),
            '\'' => {
                if in_double_quote {
                    // treat quotes as literal inside existing quoted text.
                    arg.push(ch);
                } else {
                    // otherwise, toggle start and end of quotes.
                    in_single_quote = !in_single_quote;
                }
            }
            '\"' => {
                if in_single_quote {
                    arg.push(ch);
                } else {
                    in_double_quote = !in_double_quote;
                }
            }
            _ => match is_quoted(in_single_quote, in_double_quote) {
                true => arg.push(ch), // if quoted, add as-is shown.
                false => {
                    if skip_char(&mut args, &mut arg, ch, prev_char) {
                        continue;
                    }
                }
            },
        }
        prev_char = ch;
    }

    // add final word, if it exists (args split on spaces otherwise).
    if !arg.is_empty() {
        push_arg(&mut args, &mut arg);
    }
    args
}

fn skip_char(args: &mut Vec<String>, arg: &mut String, ch: char, prev_char: char) -> bool {
    if is_ignored_whitespace(ch, prev_char) {
        return true;
    } else if ch == ' ' {
        push_arg(args, arg);
    } else {
        arg.push_str(&handle_special_chars(ch));
    }
    false
}

fn is_quoted(in_single_quote: bool, in_double_quote: bool) -> bool {
    if in_single_quote || in_double_quote {
        return true;
    }
    false
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

// execute a command
fn exec(cmd: &str, args: Vec<String>) -> Option<process::Output> {
    match find_executable(cmd) {
        Some(exec_path) => {
            let mut exec_cmd = std::process::Command::new(exec_path.file_name().unwrap());
            let result = exec_cmd.args(args).output();
            match result {
                Ok(output) => {
                    io::stdout().write_all(&output.stdout).ok();
                    io::stdout().flush().ok();
                    io::stderr().write_all(&output.stderr).ok();
                    io::stderr().flush().ok();
                    Some(output)
                }
                Err(err) => {
                    eprintln!("unable to execute command: {err}");
                    None
                }
            }
        }
        None => {
            println!("{}: command not found", cmd.trim());
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // backslashes
    #[test]
    fn test_backslash() {
        let result = format!("{}", parse_args("multiple\\ \\ \\ \\ spaces").join(" "));
        assert_eq!(result, "multiple    spaces");

        let result = parse_args("hello \'hello\'");
        assert_eq!(result, vec!["hello", "\'hello\'"]);

        //        let result = format!("{}", parse_args("\'\"literal quotes\"\'").join(" "));
        //        assert_eq!(result, "\'\"literal quotes\"\'");
    }

    // double quotes
    #[test]
    fn test_double_quotes() {
        let result = parse_args("\"hello    world\"");
        assert_eq!(result, vec!["hello    world"]);

        let result = parse_args("\"hello\"\"world\"");
        assert_eq!(result, vec!["helloworld"]);

        let result = parse_args("\"hello\"world");
        assert_eq!(result, vec!["helloworld"]);

        let result = parse_args("\"hello\" \"world\"");
        assert_eq!(result, vec!["hello", "world"]);

        let result = parse_args("\"shell\'s  test\"");
        assert_eq!(result, vec!["shell\'s  test"]);

        let result = parse_args("\"hello \'world~\' yes     \"");
        assert_eq!(result, vec!["hello 'world~' yes     "]);

        let result = parse_args("\"hello \'world~\' yes     \"");
        assert_eq!(result, vec!["hello 'world~' yes     "]);
    }

    // single quotes
    #[test]
    fn test_parse_spaces_no_quotes() {
        let args = parse_args("world     shell");
        assert_eq!(args, vec!["world", "shell"]);
    }

    #[test]
    fn test_single_quotes() {
        let args = parse_args("'test     example' 'world''shell' script''hello");
        assert_eq!(args, vec!["test     example", "worldshell", "scripthello"]);

        let args = parse_args("\'hello    \"worlds \"   \'");
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

        let args = parse_args("'/tmp/ant/f   61' '/tmp/ant/f   95' '/tmp/ant/f   36'");
        assert_eq!(
            args,
            vec![
                "/tmp/ant/f   61".to_string(),
                "/tmp/ant/f   95".to_string(),
                "/tmp/ant/f   36".to_string()
            ]
        );

        let output = exec("cat", args).expect("unable to execute cat");
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
}

//mod tests {
//    use super::*;
//
//    #[test]
//    fn basic_requirements() {
//        let result = quoted_text("'hello    world'");
//        assert_eq!(result, "hello    world");
//
//        let result2 = quoted_text("'hello''world'");
//        assert_eq!(result2, "helloworld");
//
//        let result3 = quoted_text("hello''world");
//        assert_eq!(result3, "helloworld");
//    }
//
//    #[test]
//    fn quoted_retains_spaces() {
//        let result = quoted_text("\'hello       \' world");
//        assert_eq!(result, "hello        world");
//    }
//
//    #[test]
//    fn quoted_ignores_carriage_returns() {
//        let result = quoted_text("hello world\r");
//        assert_eq!(result, "hello world");
//
//        let result2 = quoted_text("hello \'world\r\'");
//        assert_eq!(result2, "hello world");
//    }
//}
