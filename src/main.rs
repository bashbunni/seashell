use is_executable::IsExecutable;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::{env, str::FromStr};
use strum_macros::{Display, EnumString};

// for type:
// 1. check for builtin; use current handling for that
// 2. look at all dirs in path, if ! exist command not found...
//   a. check file name
//   b. is executable? permissions. If not, skip

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
    let (mut command, remainder) = input.split_once(" ").unwrap_or((input, ""));

    // do nothing if they hit enter.
    if let Ok(Command::Enter) = Command::from_str(command) {
        return;
    }

    // tidy inputs so I don't need to trim throughout.
    command = command.trim();

    // get args
    let args = parse_args(remainder);

    match Command::from_str(command) {
        Ok(Command::Exit) => std::process::exit(0),
        Ok(Command::Echo) => println!("{}", args.join("")),
        Ok(Command::Type) => Command::handle_type(args),
        Ok(Command::Pwd) => Command::handle_pwd(),
        Ok(Command::Cd) => Command::handle_cd(args),
        _ => {
            // this returns a different type than the other commands for testing
            // purposes... TODO might update the others to do something similar
            // as I get better with adding tests along the way.
            exec(command, args);
        }
    }
}

// retain exact characters if within quotes. all args are separated by spaces,
// if there is no space, just quotes, they are still treated as the same
// argument.
fn parse_args(input: &str) -> Vec<String> {
    let mut in_quote: bool = false;
    let mut arg: String = String::new();
    let mut args: Vec<String> = vec![];
    let mut prev_char: char = char::default();
    for ch in input.chars() {
        if ch == '\'' {
            // TODO check this. We should only split on spaces... but not within quotes
            //            if in_quote && !arg.is_empty() {
            //                // we've reached the end of the quoted text.
            //                args.push(arg.clone());
            //                arg.clear();
            //            }
            in_quote = !in_quote;
        } else if !in_quote {
            // ignore multiple spaces.
            if prev_char == ' ' && ch == ' ' || ch.is_ascii_whitespace() && ch != ' ' {
                continue;
            } else if ch == ' ' {
                // split on spaces, don't include them as args, keep one though if there are multiple spaces, we need to keep a space between them to print
                arg.push(ch);
                args.push(arg.clone());
                arg.clear();
            } else {
                arg.push_str(&handle_special_chars(ch));
            }
        } else {
            arg.push(ch);
        }
        prev_char = ch;
    }

    if !arg.is_empty() {
        args.push(arg.clone());
        arg.clear();
    }
    args
}

fn handle_special_chars(ch: char) -> String {
    match ch {
        '~' => format!("{}", env::home_dir().unwrap_or_default().display()),
        _ => ch.to_string(),
    }
}

// execute a command
fn exec(input: &str, args: Vec<String>) -> Option<process::Output> {
    match find_executable(input) {
        Some(exec_path) => {
            let mut exec_command = std::process::Command::new(exec_path);
            let result = exec_command.args(args).output();
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
            println!("{}: command not found", input.trim());
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

    // echo
    #[test]
    fn test_parse_spaces_no_quotes() {
        let args = parse_args("world     shell");
        assert_eq!(args, vec!["world", "shell"]);
    }

    // quotes
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

// #[cfg(test)]
// TODO add quote tests:
// input: $ echo 'world     shell' 'example''hello' test''script
// expect: world     shell examplehello testscript
// input: $echo world     shell
// expect: worldshell
//
// TODO add cat tests
// input: $ cat '/tmp/owl/f   43' '/tmp/owl/f   72' '/tmp/owl/f   8'
//
//
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
