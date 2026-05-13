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
        // we are okay to panic on this failure right now.

        let (command, remainder) = input.split_once(" ").unwrap_or((&input, ""));

        match Command::from_str(command.trim()) {
            Ok(Command::Exit) => return,
            Ok(Command::Echo) => println!("{}", remainder.trim()),
            Ok(Command::Type) => match Command::from_str(remainder.trim()) {
                Ok(_) => println!("{} is a shell builtin.", remainder.trim()),
                Err(_) => println!("{}: command not found", remainder.trim()),
            },
            _ => println!("{}: command not found", input.trim()),
        }
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

// I need some way to check if the command exists. I want to use pattern matching.
//
// map{ command: function } -> all of the available commands
// when I get type, I see if the remainder exists
// current match restrictions: it's only on command.trim() that we're matching, not callable.
