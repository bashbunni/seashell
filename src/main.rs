#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::FromStr;
use strum_macros::EnumString;

fn main() {
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
        if Command::from_str(input).is_ok() {
            println!("{input} is a shell builtin")
        } else {
            println!("{input}: not found")
        }
    }
}

fn eval(input: &str) {
    let (command, remainder) = input.split_once(" ").unwrap_or((&input, ""));

    match Command::from_str(command.trim()) {
        Ok(Command::Exit) => return,
        Ok(Command::Echo) => Command::handle_echo(remainder.trim()),
        Ok(Command::Type) => Command::handle_type(remainder.trim()),
        _ => println!("{}: command not found", input.trim()),
    }
}
