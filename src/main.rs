#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("unable to read user input");
        // we are okay to panic on this failure right now.
        let (command, remainder) = input.split_once(" ").unwrap_or((&input, "")); // TODO if ok, then use it...

        match command.trim() {
            "exit" => return,
            "echo" => println!("{}", remainder.trim()),
            _ => println!("{}: command not found", input.trim()),
        }
    }
}
