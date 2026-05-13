#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
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
    // 1. Read: Display a prompt and wait for user input
    // 2. Eval: Parse and execute the command
    // 3. Print: Display the output or error message
    // 4. Loop: Return to step 1 and wait for the next command
}
