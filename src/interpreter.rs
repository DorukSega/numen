use crate::head::{Function, Lexeme, TokId, GLOBAL, MAIN};
use std::collections::HashMap;
use std::process::exit;


pub fn interpret(mut function_map: HashMap<String, Function>) {
    let mut global_heap: HashMap<String, Lexeme<String>> = HashMap::new();

    interpret_func(
        &mut function_map,
        GLOBAL.to_string(),
        &mut global_heap,
        None,
        None,
        None,
    );
    interpret_func(
        &mut function_map,
        MAIN.to_string(),
        &mut global_heap,
        None,
        None,
        None,
    );
}

fn interpret_func(
    function_map: &mut HashMap<String, Function>,
    fname: String,
    global_heap: &mut HashMap<String, Lexeme<String>>,
    parent_stack_option: Option<&mut Vec<Lexeme<String>>>,
    custom_stack: Option<Vec<Lexeme<String>>>,
    custom_heap: Option<&mut HashMap<String, Lexeme<String>>>,
) -> Vec<Lexeme<String>> {
    let Some(mut func) = function_map.get_mut(&fname).cloned() else {
        panic!("INTERP: {} function does not exist", fname)
    };

    if custom_stack.is_some() {
        func.stack = custom_stack.unwrap();
    }

    let mut live_heap = &mut HashMap::new(); // HEAP
    if custom_heap.is_some() {
        live_heap = custom_heap.unwrap();
    }

    let mut live_stack: Vec<Lexeme<String>> = Vec::new(); // STACK
    let mut parent_stack: Option<&mut Vec<Lexeme<String>>> = None;

    if let Some(par_stack) = parent_stack_option {
        // ARGUMENT PASSING
        for (i, arg) in func.arguments.iter().enumerate() {
            let value = par_stack.pop().unwrap_or_else(|| {
                panic!(
                    "Function {} expected {} arguments passed but got {}",
                    fname,
                    func.arguments.len(),
                    i
                )
            });
            if arg.id == TokId::UNKNOWN {
                // variable name case
                live_heap.insert(arg.rep.clone(), value);
            } else {
                // type names, int float so on
                match arg.id {
                    TokId::TINT => {
                        if value.id == TokId::INT {
                            live_stack.push(value)
                        }
                    }
                    TokId::TFLOAT => {
                        if value.id == TokId::FLOAT {
                            live_stack.push(value)
                        }
                    }
                    TokId::TSTRING => {
                        if value.id == TokId::STRING {
                            live_stack.push(value)
                        }
                    }
                    TokId::TBOOL => {
                        if value.id == TokId::BOOLEAN {
                            live_stack.push(value)
                        }
                    }
                    _ => {
                        panic!("{} is not a name of a type", arg.id)
                    }
                }
            }
        }
        parent_stack = Some(par_stack);
    }

    #[derive(PartialEq)]
    enum BlockType {
        NONE,
        IF,
        ELSE,
    }
    let mut block_types: Vec<BlockType> = Vec::new();
    let mut vector_heap: Vec<HashMap<String, Lexeme<String>>> = Vec::new();
    let mut block_level: i32 = -1;

    let mut iter = func.stack.iter().enumerate();
    'main: while let Some((_index, tok)) = iter.next() {
        // blocking
        if block_level > -1 {
            if block_types[block_level as usize] == BlockType::IF {
                if tok.id == TokId::ELSE {
                    block_types[block_level as usize] = BlockType::NONE;
                    vector_heap[block_level as usize].clear();
                    block_level -= 1;
                    // This will skip everything until end
                    let mut item = iter.next().expect("INTERP: no argument to skip").1;
                    let mut block_count = 0;
                    while item.id != TokId::END || block_count != 0 {
                        match item.id {
                            TokId::IF | TokId::WHILE => block_count += 1,
                            TokId::END => block_count -= 1,
                            _ => {}
                        }
                        item = iter
                            .next()
                            .expect("INTERP: 'end' is missing for the if statement")
                            .1;
                    }
                    continue;
                } else if tok.id == TokId::END {
                    block_types[block_level as usize] = BlockType::NONE;
                    vector_heap[block_level as usize].clear();
                    block_level -= 1;
                    continue;
                }
            } else if block_types[block_level as usize] == BlockType::ELSE {
                if tok.id == TokId::END {
                    block_types[block_level as usize] = BlockType::NONE;
                    vector_heap[block_level as usize].clear();
                    block_level -= 1;
                    continue;
                }
            }
        }

        match tok.id {
            TokId::LINEBREAK => {} // should not use linebreak
            TokId::IMPORT => {}
            TokId::WHILE => {
                let mut while_cond: Vec<Lexeme<String>> = Vec::new();
                let mut do_stack: Vec<Lexeme<String>> = Vec::new();
                let mut item = iter.next().expect("INTERP: no condition to evaluate for while").1;
                let mut block_count = 0;
                while item.id != TokId::DO || block_count != 0 {
                    match item.id {
                        TokId::IF | TokId::WHILE => block_count += 1,
                        TokId::END => block_count -= 1,
                        _ => {}
                    }
                    while_cond.push(item.clone());
                    item = iter.next().expect("INTERP: 'do' is missing for the while statement").1;
                }
                item = iter.next().expect("INTERP: no argument to evaluate for while ... do").1;
                block_count = 0;
                while item.id != TokId::END || block_count != 0 {
                    match item.id {
                        TokId::IF | TokId::WHILE => block_count += 1,
                        TokId::END => block_count -= 1,
                        _ => {}
                    }
                    do_stack.push(item.clone());
                    item = iter.next().expect("INTERP: 'end' is missing for the while ... do statement").1;
                }

                let mut result = interpret_func(
                    function_map, fname.clone(), global_heap, parent_stack.as_deref_mut(),
                    Some(while_cond.clone()), Some(&mut live_heap),
                );
                let mut condition = result.pop().expect("INTERP: no condition for while");
                while condition.rep == "true" {
                    let mut runned_stack = interpret_func(
                        function_map, fname.clone(), global_heap, parent_stack.as_deref_mut(),
                        Some(do_stack.clone()), Some(&mut live_heap),
                    );
                    while let Some(item) = runned_stack.pop() {
                        live_stack.push(item)
                    }
                    result = interpret_func(
                        function_map, fname.clone(), global_heap, parent_stack.as_deref_mut(),
                        Some(while_cond.clone()), Some(&mut live_heap),
                    );
                    condition = result.pop().expect("INTERP: no condition for while");
                }
            }
            TokId::IF => {
                let condition = live_stack.pop().expect("INTERP: no condition argument for if");
                if condition.id != TokId::BOOLEAN {
                    panic!("INTERP: argument {} is not the type boolean", condition.id)
                }
                if condition.rep == "true" {
                    block_level += 1;
                    block_types.push(BlockType::IF);
                    vector_heap.insert(block_level as usize, HashMap::new());
                    continue;
                } else if condition.rep == "false" {
                    let mut item = iter.next().expect("INTERP: no argument to evaluate for if").1;
                    let mut block_count = 0;
                    // This will skip everything until else or end
                    while (item.id != TokId::END && item.id != TokId::ELSE) || block_count != 0 {
                        match item.id {
                            TokId::IF | TokId::WHILE => block_count += 1,
                            TokId::END => block_count -= 1,
                            _ => {}
                        }
                        item = iter.next().expect("INTERP: 'end' is missing for the if statement").1;
                    }
                    if item.id == TokId::ELSE {
                        block_level += 1;
                        block_types.push(BlockType::ELSE);
                        vector_heap.insert(block_level as usize, HashMap::new());
                    }
                    continue;
                } else {
                    panic!("INTERP: condition {} is invalid", condition.id)
                }
            }
            TokId::PLUS => {
                let second = live_stack.pop().expect("INTERP: error no argument to add");
                let first = live_stack.pop().expect("INTERP: error no argument to add");
                match first.id {
                    TokId::STRING => {
                        //string
                        if second.id == TokId::STRING {
                            live_stack.push(Lexeme {
                                id: TokId::STRING,
                                rep: (first.rep + second.rep.as_str()),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be added", first, second);
                        }
                    }
                    TokId::INT => {
                        if second.id == TokId::INT {
                            live_stack.push(Lexeme {
                                id: TokId::INT,
                                rep: (cast2int(&first.rep) + cast2int(&second.rep)).to_string(),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2int(&first.rep) as f64 + cast2float(&second.rep))
                                    .to_string(),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be added", first, second);
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2float(&first.rep) + cast2int(&second.rep) as f64)
                                    .to_string(),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2float(&first.rep) + cast2float(&second.rep)).to_string(),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be added", first, second);
                        }
                    }
                    typ => {
                        panic!("INTERP: {:?} can't be added", typ);
                    }
                }
            }
            TokId::MINUS => {
                let second = live_stack
                    .pop()
                    .expect("INTERP: error no argument to subtract");
                let first = live_stack
                    .pop()
                    .expect("INTERP: error no argument to subtract");
                match first.id {
                    TokId::INT => {
                        if second.id == TokId::INT {
                            live_stack.push(Lexeme {
                                id: TokId::INT,
                                rep: (cast2int(&first.rep) - cast2int(&second.rep)).to_string(),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2int(&first.rep) as f64 - cast2float(&second.rep))
                                    .to_string(),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be subtracted", first, second);
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2float(&first.rep) - cast2int(&second.rep) as f64)
                                    .to_string(),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2float(&first.rep) - cast2float(&second.rep)).to_string(),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be subtracted", first, second);
                        }
                    }
                    typ => {
                        panic!("INTERP: {:?} can't be subtracted", typ);
                    }
                }
            }
            TokId::MULTIPLY => {
                let second = live_stack.pop().expect("INTERP: error no argument to multiply");
                let first = live_stack.pop().expect("INTERP: error no argument to multiply");
                match first.id {
                    TokId::INT => {
                        if second.id == TokId::INT {
                            live_stack.push(Lexeme {
                                id: TokId::INT,
                                rep: (cast2int(&first.rep) * cast2int(&second.rep)).to_string(),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2int(&first.rep) as f64 * cast2float(&second.rep))
                                    .to_string(),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be multiplied", first, second);
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2float(&first.rep) * cast2int(&second.rep) as f64)
                                    .to_string(),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2float(&first.rep) * cast2float(&second.rep)).to_string(),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be multiplied", first, second);
                        }
                    }
                    typ => {
                        panic!("INTERP: {:?} can't be multiplied", typ);
                    }
                }
            }
            TokId::DIVIDE => {
                let second = live_stack.pop().expect("INTERP: error no argument to divide");
                let first = live_stack.pop().expect("INTERP: error no argument to divide");
                match first.id {
                    TokId::INT => {
                        if second.id == TokId::INT {
                            live_stack.push(Lexeme {
                                id: TokId::INT,
                                rep: (cast2int(&first.rep) / cast2int(&second.rep)).to_string(),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2int(&first.rep) as f64 / cast2float(&second.rep)).to_string(),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be divided", first, second);
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2float(&first.rep) / cast2int(&second.rep) as f64).to_string(),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2float(&first.rep) / cast2float(&second.rep)).to_string(),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be divided", first, second);
                        }
                    }
                    typ => {
                        panic!("INTERP: {:?} can't be divided", typ);
                    }
                }
            }
            TokId::MOD => {
                let second = live_stack.pop().expect("INTERP: error no argument to mod");
                let first = live_stack.pop().expect("INTERP: error no argument to mod");
                match first.id {
                    TokId::INT => {
                        if second.id == TokId::INT {
                            live_stack.push(Lexeme {
                                id: TokId::INT,
                                rep: (cast2int(&first.rep) % cast2int(&second.rep)).to_string(),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2int(&first.rep) as f64 % cast2float(&second.rep)).to_string(),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be used to mod", first, second);
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2float(&first.rep) % cast2int(&second.rep) as f64).to_string(),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Lexeme {
                                id: TokId::FLOAT,
                                rep: (cast2float(&first.rep) % cast2float(&second.rep)).to_string(),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be used to mod", first, second);
                        }
                    }
                    typ => {
                        panic!("INTERP: {:?} can't be used to mod", typ);
                    }
                }
            }
            TokId::IS => {
                let second = live_stack.pop().expect("INTERP: error no argument to typecheck");
                let first = live_stack.pop().expect("INTERP: error no argument to typecheck");
                if (first.id == TokId::INT && second.id == TokId::TINT)
                    || (first.id == TokId::FLOAT && second.id == TokId::TFLOAT)
                    || (first.id == TokId::BOOLEAN && second.id == TokId::TBOOL)
                    || (first.id == TokId::STRING && second.id == TokId::TSTRING)
                {
                    live_stack.push(Lexeme {
                        id: TokId::BOOLEAN,
                        rep: "true".to_string(),
                    });
                } else {
                    live_stack.push(Lexeme {
                        id: TokId::BOOLEAN,
                        rep: "false".to_string(),
                    });
                }
            }
            TokId::ASSIGNMENT | TokId::RETURNINGASSIGNMENT => {
                let Some((_, mut var)) = iter.next() else {
                    panic!("INTERP: no variable name to assign in to");
                };
                if var.id == TokId::LINEBREAK {
                    var = iter.next().expect("INTERP: no variable to assign in to").1
                }
                if var.id != TokId::UNKNOWN {
                    panic!("INTERP: {} -> {} is not a variable name", var.id, var.rep)
                }
                let popped = live_stack.pop().expect("INTERP: no argument to assign");
                if tok.id == TokId::RETURNINGASSIGNMENT {
                    live_stack.push(popped.clone());
                }
                if fname == GLOBAL {
                    global_heap.insert(var.rep.clone(), popped);
                } else {
                    if block_level > -1 {
                        // INSIDE A BLOCK
                        if global_heap.contains_key(var.rep.as_str()) {
                            global_heap.insert(var.rep.clone(), popped);
                        } else if live_heap.contains_key(var.rep.as_str()) {
                            live_heap.insert(var.rep.clone(), popped);
                        } else {
                            for i in (0..block_level + 1).rev() {
                                if vector_heap[i as usize].contains_key(var.rep.as_str()) {
                                    vector_heap[i as usize].insert(var.rep.clone(), popped);
                                    continue 'main;
                                }
                            }
                            vector_heap[block_level as usize].insert(var.rep.clone(), popped);
                        }
                    } else {
                        if global_heap.contains_key(var.rep.as_str()) {
                            global_heap.insert(var.rep.clone(), popped);
                        } else {
                            live_heap.insert(var.rep.clone(), popped);
                        }
                    }
                }
            }
            TokId::EQUALS => {
                let second = live_stack
                    .pop()
                    .expect("INTERP: error no argument to check for equation");
                let first = live_stack
                    .pop()
                    .expect("INTERP: error no argument to check for equation");
                if first == second {
                    live_stack.push(Lexeme {
                        id: TokId::BOOLEAN,
                        rep: "true".to_string(),
                    });
                } else {
                    live_stack.push(Lexeme {
                        id: TokId::BOOLEAN,
                        rep: "false".to_string(),
                    });
                }
            }
            TokId::BIGGER => {
                let second = live_stack
                    .pop()
                    .expect("INTERP: error no argument to compare for bigger");
                let first = live_stack
                    .pop()
                    .expect("INTERP: error no argument to compare for bigger");
                match first.id {
                    TokId::INT => {
                        if second.id == TokId::INT && cast2int(&first.rep) > cast2int(&second.rep) {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2int(&first.rep) as f64 > cast2float(&second.rep)
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "false".to_string(),
                            })
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT
                            && cast2float(&first.rep) > cast2int(&second.rep) as f64
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2float(&first.rep) > cast2float(&second.rep)
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "false".to_string(),
                            })
                        }
                    }
                    typ => {
                        panic!("INTERP: {:?} can't be compared for bigger", typ);
                    }
                }
            }
            TokId::SMALLER => {
                let second = live_stack
                    .pop()
                    .expect("INTERP: error no argument to compare for smaller");
                let first = live_stack
                    .pop()
                    .expect("INTERP: error no argument to compare for smaller");
                match first.id {
                    TokId::INT => {
                        if second.id == TokId::INT && cast2int(&first.rep) < cast2int(&second.rep) {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2float(&second.rep) > cast2int(&first.rep) as f64
                        {
                            // hack because other way don't work
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "false".to_string(),
                            })
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT
                            && cast2float(&first.rep) < cast2int(&second.rep) as f64
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2float(&first.rep) < cast2float(&second.rep)
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "false".to_string(),
                            })
                        }
                    }
                    typ => {
                        panic!("INTERP: {:?} can't be compared for smaller", typ);
                    }
                }
            }
            TokId::BIGGEREQUALS => {
                let second = live_stack
                    .pop()
                    .expect("INTERP: error no argument to compare for bigger equals");
                let first = live_stack
                    .pop()
                    .expect("INTERP: error no argument to compare for bigger equals");
                match first.id {
                    TokId::INT => {
                        if second.id == TokId::INT && cast2int(&first.rep) >= cast2int(&second.rep)
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2int(&first.rep) as f64 >= cast2float(&second.rep)
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "false".to_string(),
                            })
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT
                            && cast2float(&first.rep) >= cast2int(&second.rep) as f64
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2float(&first.rep) >= cast2float(&second.rep)
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "false".to_string(),
                            })
                        }
                    }
                    typ => {
                        panic!("INTERP: {:?} can't be compared for bigger equals", typ);
                    }
                }
            }
            TokId::SMALLEREQUALS => {
                let second = live_stack
                    .pop()
                    .expect("INTERP: error no argument to compare for smaller equals");
                let first = live_stack
                    .pop()
                    .expect("INTERP: error no argument to compare for smaller equals");
                match first.id {
                    TokId::INT => {
                        if second.id == TokId::INT && cast2int(&first.rep) <= cast2int(&second.rep)
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2int(&first.rep) as f64 <= cast2float(&second.rep)
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "false".to_string(),
                            })
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT
                            && cast2float(&first.rep) <= cast2int(&second.rep) as f64
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2float(&first.rep) <= cast2float(&second.rep)
                        {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "true".to_string(),
                            })
                        } else {
                            live_stack.push(Lexeme {
                                id: TokId::BOOLEAN,
                                rep: "false".to_string(),
                            })
                        }
                    }
                    typ => {
                        panic!("INTERP: {:?} can't be compared for smaller equals", typ);
                    }
                }
            }
            TokId::RET => {
                if let Some(par_stack) = parent_stack.as_mut() {
                    par_stack.push(
                        live_stack
                            .pop()
                            .expect("INTERP: error no argument to return"),
                    );
                }
            }
            TokId::UNKNOWN => {
                match tok.rep.as_str() {
                    "print" => {
                        let value = live_stack
                            .pop()
                            .expect("INTERP: error no argument to print");
                        match value.id {
                            TokId::STRING | TokId::INT | TokId::FLOAT | TokId::BOOLEAN => {
                                println!("{}", value.rep)
                            }
                            _ => {
                                panic!("INTERP: can't print {}", tok.rep);
                            }
                        }
                    }
                    "assert" => {
                        let second = live_stack.pop().expect("INTERP: error no argument to assert");
                        let first = live_stack.pop().expect("INTERP: error no argument to assert");
                        if first != second {
                            println!("\"{}\" != \"{}\"", first.rep, second.rep);
                            exit(1);
                        }
                    }
                    "swap" => {
                        let second = live_stack.pop().expect("INTERP: error no argument to swap");
                        let first = live_stack.pop().expect("INTERP: error no argument to swap");
                        live_stack.push(second);
                        live_stack.push(first);
                    }
                    "drop" => {
                        live_stack.pop().expect("INTERP: error no argument to drop");
                    }
                    ";" => {
                        //risky!
                        live_stack.clear();
                    }
                    "rot" => {
                        let third = live_stack.pop().expect("INTERP: error no argument to rot");
                        let second = live_stack.pop().expect("INTERP: error no argument to rot");
                        let first = live_stack.pop().expect("INTERP: error no argument to rot");
                        live_stack.push(first);
                        live_stack.push(third);
                        live_stack.push(second);
                    }
                    "copy" => {
                        let top = live_stack.last().expect("INTERP: error no argument to rot");
                        live_stack.push(top.clone());
                    }
                    "carry" => {
                        let second = live_stack
                            .get(live_stack.len() - 2)
                            .expect("INTERP: error no argument to rot");
                        live_stack.push(second.clone());
                    }
                    "sqrt" => {
                        let item = live_stack
                            .pop()
                            .expect("INTERP: error no argument to take square of");
                        match item.id {
                            TokId::INT => {
                                live_stack.push(Lexeme {
                                    rep: (f64::sqrt(cast2int(&item.rep) as f64) as i32).to_string(),
                                    id: TokId::INT,
                                });
                            }
                            TokId::FLOAT => {
                                live_stack.push(Lexeme {
                                    rep: (f64::sqrt(cast2float(&item.rep))).to_string(),
                                    id: TokId::FLOAT,
                                });
                            }
                            typ => {
                                panic!("INTERP: {:?} can't take the square root of this type", typ);
                            }
                        }
                    }
                    def => {
                        // variable casting or function call
                        if block_level > -1 {
                            // INSIDE A BLOCK
                            for i in (0..block_level + 1).rev() {
                                if vector_heap[i as usize].contains_key(def) {
                                    let Some(value) = vector_heap[i as usize].get(def) else {
                                        panic!("INTERP: {} value does not exist in local block's heap", def)
                                    };
                                    live_stack.push(value.clone());
                                    continue 'main;
                                }
                            }
                        }

                        if live_heap.contains_key(def) {
                            // LOCAL VARIABLE
                            let Some(value) = live_heap.get(def) else {
                                panic!("INTERP: {} value does not exist in local heap", def)
                            };
                            live_stack.push(value.clone())
                        } else if global_heap.contains_key(def) {
                            // GLOBAL VARIABLE
                            let Some(value) = global_heap.get(def) else {
                                panic!("INTERP: {} value does not exist in local heap", def)
                            };
                            live_stack.push(value.clone())
                        } else if function_map.contains_key(def) {
                            //FUNCTION CALL
                            interpret_func(
                                function_map,
                                def.to_string(),
                                global_heap,
                                Some(&mut live_stack),
                                None,
                                None,
                            );
                        }
                    }
                }
            }
            TokId::FUNCTION => {
                panic!("INTERP: can't declare functions inside a functions") // Not implemented
            }

            _ => {
                live_stack.push(tok.clone());
            }
        }
    }
    return live_stack;
}

fn cast2int(data: &String) -> i32 {
    data.parse::<i32>().unwrap_or_else(|_|
        panic!("INTERP: can't parse {} to int",data))
}

fn cast2float(data: &String) -> f64 {
    data.parse::<f64>()
        .expect("INTERP: can't parse value to int")
}
