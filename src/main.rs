use anyhow::Error;
use is_executable::IsExecutable;
use std::ffi::OsString;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
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
}

impl Command {
    fn handle_type(input: &str) {
        match Command::from_str(input) {
            Ok(_) => println!("{input} is a shell builtin"),
            Err(_) => match find_executable(input) {
                Some(exec_path) => println!("{input} is {}", exec_path.display()),
                None => println!("{input}: not found"),
            },
        }
    }

    fn handle_pwd() {
        match env::current_dir() {
            Ok(pwd) => println!("{}", pwd.display()),
            Err(e) => eprintln!("unexpected error: {e}"),
        }
    }
}

// evaluate commands
fn eval(input: &str) {
    let (mut command, mut remainder) = input.split_once(" ").unwrap_or((input, ""));

    // do nothing if they hit enter.
    if let Ok(Command::Enter) = Command::from_str(command) {
        return;
    }

    // tidy inputs so I don't need to trim throughout.
    command = command.trim();
    remainder = remainder.trim();

    // get args
    let args: Vec<&str> = remainder
        .split(" ")
        .filter(|x| !x.is_empty())
        .map(|x| x.trim())
        .collect();

    match Command::from_str(command) {
        Ok(Command::Exit) => std::process::exit(0),
        Ok(Command::Echo) => println!("{}", remainder),
        Ok(Command::Pwd) => Command::handle_pwd(),
        Ok(Command::Type) => Command::handle_type(remainder),
        _ => exec(command, &args),
    }
}

// execute a command
fn exec(input: &str, args: &[&str]) {
    match find_executable(input) {
        Some(exec_path) => {
            if let Some(exec_name) = exec_path.file_name() {
                let mut exec_command = std::process::Command::new(exec_name);
                let result: Result<process::Child, io::Error> = if args.is_empty() {
                    exec_command.args(args).spawn()
                } else {
                    exec_command.spawn()
                };
                match result {
                    Ok(mut child) => {
                        child.wait().ok();
                    }
                    Err(err) => eprintln!("unable to execute command: {err}"),
                }
            }
        }
        None => println!("{}: command not found", input.trim()),
    }
}

// returns executable path if one is found.
fn find_executable(input: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|path| {
        env::split_paths(&path).find_map(|dir| {
            let exec_path = dir.join(input);
            exec_path.is_executable().then_some(exec_path)
        })
    })
}
