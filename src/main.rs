use std::io::{self, Write};

fn main() {
    loop {
        print_prompt();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(_) => println!("read stdin error!")
        }

        match input.trim() {
            ".exit" => {
                println!("bye");
                break;
            }
            _ => println!("unrecgonized command {}", input)
        }
    }
}

fn print_prompt() {
    print!("db > ");
    io::stdout().flush().unwrap();
}