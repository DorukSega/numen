extern crate core;

// root that all files share
mod head;
// lexer, parses indiviual lexemes
mod lexer;
// parser, parses function blocks and raw types
mod interpreter;
mod parser;

use crate::head::{Function, TokId};
use crate::interpreter::interpret;
use lexer::lexer_file;
use parser::parse_file;
use std::env;
use std::fs;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let (_, filenames) = args.split_first().unwrap();
    for filename in filenames {
        run_file(filename);
    }
}

//pub fn get_std(libname: &String) {}

pub fn get_path(filename: &String) -> String {
    match env::current_exe() {
        Ok(exe_path) => {
            let dir = exe_path
                .parent()
                .expect("MAIN: Executable has no parent directory");
            let dir = dir.join(filename);
            dir.to_str().unwrap().to_string()
        }
        Err(_) => filename.clone(),
    }
}

pub fn read_file(filename: &String) -> String {
    if cfg!(any(debug_assertions)) { // IS COMPILED AS DEBUG MODE
        println!("MAIN: Reading file {}", filename);
    }
    //TODO: check standard library and see if it is referenced
    let filepath = get_path(filename);

    let contents = fs::read_to_string(filepath).expect("Should have been able to read the file");
    contents
}

pub fn run_file(filename: &String) {
    let file = read_file(filename);
    //println!("{}", file);
    let lexed = lexer_file(&file);
    //dbg!(lexed.clone());
    let fmap = parse_file(lexed);

    for (name, fun) in &fmap {
        if cfg!(any(debug_assertions)) { // IS COMPILED AS DEBUG MODE
            print_function(name, fun);
        }

        /* println!("{}:", name);
        println!("{:?}\n", fun);*/
    }

    interpret(fmap);
}


fn print_function(name: &String, fun: &Function) {
    println!("\x1b[31;1m{}: \x1b[0m", name);
    for item in &fun.stack {
        match item.id {
            TokId::WHILE | TokId::DO | TokId::IF | TokId::BLOCK
            | TokId::ELSE | TokId::FUNCTION | TokId::IMPORT | TokId::END
            | TokId::AS | TokId::RET | TokId::ASSIGNMENT | TokId::RETURNINGASSIGNMENT
            | TokId::ARRAY | TokId::LOOP => {
                print!("\x1b[35m{} \x1b[0m", item.rep);
            }
            TokId::PLUS | TokId::MINUS | TokId::MULTIPLY | TokId::DIVIDE
            | TokId::MOD | TokId::EQUALS | TokId::BIGGER | TokId::SMALLER
            | TokId::BIGGEREQUALS | TokId::SMALLEREQUALS | TokId::IS => {
                print!("\x1b[31m{} \x1b[0m", item.rep);
            }
            TokId::ARRAYBEGIN => {
                print!("[ ");
            }
            TokId::ARRAYEND => {
                print!("] ");
            }
            TokId::STRING => {
                print!("\x1b[32m\"{}\" \x1b[0m", item.rep);
            }
            TokId::BOOLEAN => {
                print!("\x1b[94m{} \x1b[0m", item.rep);
            }
            TokId::INT | TokId::FLOAT => {
                print!("\x1b[33m{} \x1b[0m", item.rep);
            }
            TokId::TINT | TokId::TFLOAT | TokId::TSTRING | TokId::TBOOL | TokId::TARRAY => {
                print!("\x1b[95m{} \x1b[0m", item.rep);
            }
            TokId::LINEBREAK => {
                print!("\n\t");
            }
            TokId::UNKNOWN => {
                print!("\x1b[34m{} \x1b[0m", item.rep);
            }
        }
    }
    print!("\n")
}
