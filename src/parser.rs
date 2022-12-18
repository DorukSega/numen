use crate::head::{Function, Lexeme, TokId, GLOBAL};
use std::collections::HashMap;

use crate::lexer::lexmap_contains_value;

pub fn parse_file(mut lexed: Vec<Lexeme<String>>) -> HashMap<String, Function> {
    //parse types
    parse_type(&mut lexed);
    // function hash map
    let mut function_map: HashMap<String, Function> = HashMap::new();
    // global func, representing global scope
    function_map.insert(
        GLOBAL.clone().to_string(),
        Function {
            arguments: vec![],
            stack: vec![],
        },
    );

    let mut iter = lexed.iter();
    let mut fname: String = String::new();
    let mut block_count = 0; // for stuff like if and while
                             // parsing functions
    while let Some(lex) = iter.next() {
        //inside the function
        if !fname.is_empty() {
            let Some(funcref) = function_map.get_mut(&fname) else {
                panic!("PARSER: the function {} is not declared!", fname);
            };
            match lex.id {
                TokId::FUNCTION => {
                    //other than global, one should not declare functions inside functions
                    panic!(
                        "PARSER: can't declare a function inside one\n {} inside {}",
                        fname,
                        iter.next().unwrap().rep
                    )
                }
                TokId::END => {
                    if block_count > 0 {
                        block_count -= 1;
                        funcref.stack.push(lex.clone());
                    } else {
                        fname.clear();
                    }
                }
                TokId::WHILE | TokId::IF | TokId::BLOCK => {
                    //BLOCK CHECK
                    block_count += 1;
                    funcref.stack.push(lex.clone());
                }
                _ => {
                    funcref.stack.push(lex.clone());
                }
            }
            continue;
        }
        // global
        match lex.id {
            TokId::FUNCTION => {
                let nameref = iter.next().unwrap();
                if nameref.id != TokId::UNKNOWN {
                    panic!(
                        "PARSER: function name {} is alredy used as {:?}",
                        nameref.rep, nameref.id
                    )
                }
                fname = nameref.rep.clone();

                validate_name(&fname);

                // handle function parameters
                let mut new_func = Function {
                    stack: vec![],
                    arguments: vec![],
                };
                let mut param = iter.next().unwrap();
                if param.id != TokId::AS {
                    while param.id != TokId::AS {
                        new_func.arguments.push(param.clone());
                        param = iter.next().unwrap();
                    }
                }

                function_map.insert(fname.clone(), new_func);
                continue;
                /*parse_function(function_map, iter.as_slice(), &nfname)*/
            }
            TokId::END => {
                panic!("PARSER: Two many ends!");
            }
            _ => {
                if let Some(func) = function_map.get_mut(GLOBAL.clone()) {
                    func.stack.push(lex.clone());
                } else {
                    panic!("PARSER: the function {} is not declared!", fname);
                }
            }
        }
    }

    function_map
} // end of parse

fn parse_type(lexed: &mut Vec<Lexeme<String>>) {
    for lex in lexed {
        if lex.id == TokId::UNKNOWN {
            if lex.rep == "true" {
                lex.id = TokId::BOOLEAN;
            } else if lex.rep == "false" {
                lex.id = TokId::BOOLEAN;
            } else if let Ok(_) = lex.rep.parse::<i32>() {
                lex.id = TokId::INT;
            } else if let Ok(_) = lex.rep.parse::<f64>() {
                lex.id = TokId::FLOAT;
            }
        }
    }
}

fn validate_name(name: &String) {
    if lexmap_contains_value(name.as_str()).is_some() {
        panic!("PARSER: \"{}\" name can't be a reserved word", name)
    }
    for (i, char) in name.chars().enumerate() {
        if i == 0 && (!char.is_alphabetic() && char != '_') {
            panic!("PARSER: \"{}\" first char of name is not valid", name)
        }
        if !char.is_alphabetic() && char != '_' && !char.is_numeric() {
            panic!("PARSER: \"{}\" char at {} is not valid", name, i)
        }
    }
}
