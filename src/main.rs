use is_executable::IsExecutable;
use std::ffi::OsString;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;
use std::{
    env::{self},
    str::FromStr,
};
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
            println!("{input} is a shell builtin");
            return;
        }

        // check the path
        let path_env = env::var_os("PATH").and_then(|path| {
            env::split_paths(&path).find_map(|dir| {
                let exec_path = dir.join(input);
                exec_path.is_executable().then_some(exec_path)
            })
        });

        if let Some(exec_path) = path_env {
            println!("{input} is {}", exec_path.display());
            return;
        }

        // nothing found
        println!("{input}: not found")
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
