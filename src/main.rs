mod table;

use std::{
    io::{self, Write},
    process,
};

use table::{SyntaxErr};

fn main() {
    let mut tbl = table::Table::db_open("abc.db");
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
            match do_meta_command(input, &mut tbl) {
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
                    SyntaxErr::WrongArgsNum => println!("syntax error near {}, err: wrong number of args.", input),
                    SyntaxErr::StringTooLong => println!("syntax error, string too long."),
                    SyntaxErr::InvalidID => println!("syntax err, invalid id format.")
                }
                
                continue;
            }
        };

        execute_statement(statement, &mut tbl);
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

fn do_meta_command(input: &str, tbl: &mut table::Table) -> MetaCommandResult {
    match input {
        ".exit" => {
            tbl.db_close();
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
            let mut cursor = table::Cursor::table_end(tbl);
            match cursor.insert(row) {
                table::ExecuteResult::ExecuteSuccess => println!("execute insert 1 row."),
                table::ExecuteResult::ExecuteTableFull => println!("table has full."),
            };
        }
        StatementType::StatementSelect => {
            let mut cursor = table::Cursor::table_start(tbl);
            cursor.select();
        }
    }
}
