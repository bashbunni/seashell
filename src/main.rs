use is_executable::IsExecutable;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;
use std::{
    env::{self},
    str::FromStr,
};
use std::ffi::{OsString};
use strum_macros::EnumString;

// for type:
// 1. check for builtin; use current handling for that
// 2. look at all dirs in path, if ! exist command not found...
//   a. check file name
//   b. is executable? permissions. If not, skip

fn main() {
    // eval loop
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("unable to read user input");
        eval(&input);
    }
}

#[derive(EnumString)]
enum Command {
    #[strum(serialize = "exit")]
    Exit,
    #[strum(serialize = "echo")]
    Echo,
    #[strum(serialize = "type")]
    Type,
}

impl Command {
    fn handle_echo(input: &str) {
        println!("{input}");
    }

    fn handle_type(input: &str) {
        // check if it's a built-in
        if Command::from_str(input).is_ok() {
            println!("{input} is a shell builtin")
        } else if let Some(path) = env::var_os("PATH") {
            handle_path(path, input);
        } else {
            println!("{input}: not found")
        }
    }
}

fn eval(input: &str) {
    let (command, remainder) = input.split_once(" ").unwrap_or((input, ""));

    match Command::from_str(command.trim()) {
        Ok(Command::Exit) => std::process::exit(0),
        Ok(Command::Echo) => Command::handle_echo(remainder.trim()),
        Ok(Command::Type) => Command::handle_type(remainder.trim()),
        _ => println!("{}: command not found", input.trim()),
    }
}

// TODO make this logic/operation happen within a map of the original collection.
// A path was provided, so we split it up, check the dirs for an executable.
fn handle_path(path: OsString, input: &str) {
            env::split_paths(&path).find(|p| {
                if p.join(input).is_executable()
                    && let Ok(path_str) = p.join(input).into_os_string().into_string()
                {
                    println!("{input} is {path_str}");
                    return true;
                }
                return false;
            });
   }
