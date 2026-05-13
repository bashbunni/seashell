#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin()
            .read_line(&mut command)
            .expect("unable to read user input");
        // we are okay to panic on this failure right now.
        match command.trim() {
            "exit" => return,
            _ => println!("{}: command not found", command.trim()),
        }
    }
}
