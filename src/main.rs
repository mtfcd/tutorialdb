mod data_struct;

use std::{io::{self, Write}, process};

fn main() {
    loop {
        print_prompt();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(_) => {
                println!("read stdin error!");
                continue;
            }
        }
        let input = input.trim();

        if input.starts_with(".") {
            match do_meta_command(input) {
                MetaCommandResult::MetaCommandSuccess => continue,
                MetaCommandResult::MetaCommandUnrecognizedCommand => {
                    println!("unrecgonized command {}", input);
                    continue;
                }
            }
        }

        let statement = match prepare_statement(input) {
            PrepareResult::PrepareSuccess(statement_type) => statement_type,
            PrepareResult::PrepareUnRecognizedStatement => {
                println!("Unrecognized keyword at start of {}.", input);
                continue;
            },
            PrepareResult::PrepareSyntaxError => {
                println!("sytax error near {}.", input);
                continue;
            }
        };

        execute_statement(statement);
        println!("executed.");
    }
}

fn print_prompt() {
    print!("db > ");
    io::stdout().flush().unwrap();
}

enum MetaCommandResult {
    MetaCommandSuccess,
    MetaCommandUnrecognizedCommand,
}

fn do_meta_command(input: &str) -> MetaCommandResult {
    match input {
        ".exit" => {
            process::exit(0);
        }
        _ => return MetaCommandResult::MetaCommandUnrecognizedCommand,
    }
}

enum StatementType {
    StatementInsert(data_struct::Row),
    StatementSelect,
}

enum PrepareResult {
    PrepareSuccess(StatementType),
    PrepareUnRecognizedStatement,
    PrepareSyntaxError,
}

fn prepare_statement(input: &str) -> PrepareResult {
    if input.starts_with("insert") {
        let statement = match data_struct::Row::new(input) {
            Ok(row) => row,
            Err(_) => return PrepareResult::PrepareSyntaxError
        };
        return PrepareResult::PrepareSuccess(StatementType::StatementInsert(statement));
    }
    if input == "select" {
        return PrepareResult::PrepareSuccess(StatementType::StatementSelect);
    }

    return PrepareResult::PrepareUnRecognizedStatement;
}

fn execute_statement(statement: StatementType) {
    match statement {
        StatementType::StatementInsert(row) => {
            println!("execute insert {:?}", row);
        }
        StatementType::StatementSelect => {
            println!("execute select");
        }
    }
}
