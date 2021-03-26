mod table;

use std::{
    io::{self, Write},
    process,
};

use table::{SyntaxErr};

fn main() {
    let mut tbl = table::Table::new();
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
            }
            PrepareResult::PrepareSyntaxError(err) => {
                match err {
                    SyntaxErr::WrongArgsNum => println!("sytax error near {}, err: wrong number of args.", input),
                    SyntaxErr::StringTooLong => println!("sytax error, string too long.")
                }
                
                continue;
            }
        };

        execute_statement(statement, &mut tbl);
        println!("executed.");
    }
}

fn print_prompt() {
    print!("db >");
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
    StatementInsert(table::Row),
    StatementSelect,
}

enum PrepareResult {
    PrepareSuccess(StatementType),
    PrepareUnRecognizedStatement,
    PrepareSyntaxError(SyntaxErr),
}

fn prepare_statement(input: &str) -> PrepareResult {
    if input.starts_with("insert") {
        let statement = match table::Row::new(input) {
            Ok(row) => row,
            Err(syntax_err) => return PrepareResult::PrepareSyntaxError(syntax_err),
        };
        return PrepareResult::PrepareSuccess(StatementType::StatementInsert(statement));
    }
    if input == "select" {
        return PrepareResult::PrepareSuccess(StatementType::StatementSelect);
    }

    return PrepareResult::PrepareUnRecognizedStatement;
}

fn execute_statement(statement: StatementType, tbl: &mut table::Table) {
    match statement {
        StatementType::StatementInsert(row) => {
            match tbl.insert(row) {
                table::ExecuteResult::ExecuteSuccess => println!("execute insert 1 row."),
                table::ExecuteResult::ExecuteTableFull => println!("table has full."),
            };
        }
        StatementType::StatementSelect => {
            tbl.select();
        }
    }
}
