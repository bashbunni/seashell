#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut command = String::new();
    match io::stdin().read_line(&mut command) {
        _ => print!("{}: command not found", command.trim()),
    };
}
