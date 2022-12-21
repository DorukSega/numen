use crate::head::{Function, TokId, GLOBAL, MAIN, Object, Value};
use std::collections::HashMap;
use std::process::{exit};


pub fn interpret(mut function_map: HashMap<String, Function>) {
    let mut global_heap: HashMap<String, Object> = HashMap::new();

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
    global_heap: &mut HashMap<String, Object>,
    parent_stack_option: Option<&mut Vec<Object>>,
    custom_stack: Option<Vec<Object>>,
    custom_heap: Option<&mut HashMap<String, Object>>,
) -> Vec<Object> {
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

    let mut live_stack: Vec<Object> = Vec::new(); // STACK
    let mut parent_stack: Option<&mut Vec<Object>> = None;

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
                live_heap.insert(match arg.rep.clone() {
                    Value::STR(s) => s,
                    Value::ARR(_) => unreachable!()
                }, value);
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
    let mut vector_heap: Vec<HashMap<String, Object>> = Vec::new();
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
                let mut while_cond: Vec<Object> = Vec::new();
                let mut do_stack: Vec<Object> = Vec::new();
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
                while cast2string(&condition.rep) == "true" {
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
                if cast2string(&condition.rep) == "true" {
                    block_level += 1;
                    block_types.push(BlockType::IF);
                    vector_heap.insert(block_level as usize, HashMap::new());
                    continue;
                } else if cast2string(&condition.rep) == "false" {
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
                    TokId::ARRAY => {
                        if second.id == TokId::ARRAY {
                            let Value::ARR(mut first_arr) = first.rep else {
                                panic!("INTERP: expected Array but got this {}", first.rep);
                            };
                            let Value::ARR(second_arr) = second.rep else {
                                panic!("INTERP: expected Array but got this {}", second.rep);
                            };
                            first_arr.extend_from_slice(&second_arr);
                            live_stack.push(Object {
                                id: TokId::ARRAY,
                                rep: Value::ARR(first_arr),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be added", first, second);
                        }
                    }
                    TokId::STRING => {
                        //string
                        if second.id == TokId::STRING {
                            live_stack.push(Object {
                                id: TokId::STRING,
                                rep: Value::STR(cast2str(first.rep) + cast2str(second.rep).as_str()),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be added", first, second);
                        }
                    }
                    TokId::INT => {
                        if second.id == TokId::INT {
                            live_stack.push(Object {
                                id: TokId::INT,
                                rep: Value::STR((cast2int(cast2string(&first.rep)) + cast2int(cast2string(&second.rep))).to_string()), //Value::STR((cast2int(cast2string(&first.rep)) + cast2int(cast2string(&second.rep))).to_string())
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2int(cast2string(&first.rep)) as f64 + cast2float(cast2string(&second.rep)))
                                    .to_string()),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be added", first, second);
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2float(cast2string(&first.rep)) + cast2int(cast2string(&second.rep)) as f64)
                                    .to_string()),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2float(cast2string(&first.rep)) + cast2float(cast2string(&second.rep))).to_string()),
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
                            live_stack.push(Object {
                                id: TokId::INT,
                                rep: Value::STR((cast2int(cast2string(&first.rep)) - cast2int(cast2string(&second.rep))).to_string()),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2int(cast2string(&first.rep)) as f64 - cast2float(cast2string(&second.rep)))
                                    .to_string()),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be subtracted", first, second);
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2float(cast2string(&first.rep)) - cast2int(cast2string(&second.rep)) as f64)
                                    .to_string()),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2float(cast2string(&first.rep)) - cast2float(cast2string(&second.rep))).to_string()),
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
                            live_stack.push(Object {
                                id: TokId::INT,
                                rep: Value::STR((cast2int(cast2string(&first.rep)) * cast2int(cast2string(&second.rep))).to_string()),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2int(cast2string(&first.rep)) as f64 * cast2float(cast2string(&second.rep)))
                                    .to_string()),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be multiplied", first, second);
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2float(cast2string(&first.rep)) * cast2int(cast2string(&second.rep)) as f64)
                                    .to_string()),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2float(cast2string(&first.rep)) * cast2float(cast2string(&second.rep))).to_string()),
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
                            live_stack.push(Object {
                                id: TokId::INT,
                                rep: Value::STR((cast2int(cast2string(&first.rep)) / cast2int(cast2string(&second.rep))).to_string()),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2int(cast2string(&first.rep)) as f64 / cast2float(cast2string(&second.rep))).to_string()),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be divided", first, second);
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2float(cast2string(&first.rep)) / cast2int(cast2string(&second.rep)) as f64).to_string()),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2float(cast2string(&first.rep)) / cast2float(cast2string(&second.rep))).to_string()),
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
                            live_stack.push(Object {
                                id: TokId::INT,
                                rep: Value::STR((cast2int(cast2string(&first.rep)) % cast2int(cast2string(&second.rep))).to_string()),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2int(cast2string(&first.rep)) as f64 % cast2float(cast2string(&second.rep))).to_string()),
                            })
                        } else {
                            panic!("INTERP: {:?} and {:?} can't be used to mod", first, second);
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2float(cast2string(&first.rep)) % cast2int(cast2string(&second.rep)) as f64).to_string()),
                            })
                        } else if second.id == TokId::FLOAT {
                            live_stack.push(Object {
                                id: TokId::FLOAT,
                                rep: Value::STR((cast2float(cast2string(&first.rep)) % cast2float(cast2string(&second.rep))).to_string()),
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
                    live_stack.push(Object {
                        id: TokId::BOOLEAN,
                        rep: Value::STR("true".to_string()),
                    });
                } else {
                    live_stack.push(Object {
                        id: TokId::BOOLEAN,
                        rep: Value::STR("false".to_string()),
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
                    global_heap.insert(cast2str(var.rep.clone()), popped);
                } else {
                    if block_level > -1 {
                        // INSIDE A BLOCK
                        if global_heap.contains_key(cast2string(&var.rep)) {
                            global_heap.insert(cast2str(var.rep.clone()), popped);
                        } else if live_heap.contains_key(cast2string(&var.rep)) {
                            live_heap.insert(cast2str(var.rep.clone()), popped);
                        } else {
                            for i in (0..block_level + 1).rev() {
                                if vector_heap[i as usize].contains_key(cast2string(&var.rep)) {
                                    vector_heap[i as usize].insert(cast2str(var.rep.clone()), popped);
                                    continue 'main;
                                }
                            }
                            vector_heap[block_level as usize].insert(cast2str(var.rep.clone()), popped);
                        }
                    } else {
                        if global_heap.contains_key(cast2string(&var.rep)) {
                            global_heap.insert(cast2str(var.rep.clone()), popped);
                        } else {
                            live_heap.insert(cast2str(var.rep.clone()), popped);
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
                    live_stack.push(Object {
                        id: TokId::BOOLEAN,
                        rep: Value::STR("true".to_string()),
                    });
                } else {
                    live_stack.push(Object {
                        id: TokId::BOOLEAN,
                        rep: Value::STR("false".to_string()),
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
                        if second.id == TokId::INT && cast2int(cast2string(&first.rep)) > cast2int(cast2string(&second.rep)) {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2int(cast2string(&first.rep)) as f64 > cast2float(cast2string(&second.rep))
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("false".to_string()),
                            })
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT
                            && cast2float(cast2string(&first.rep)) > cast2int(cast2string(&second.rep)) as f64
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2float(cast2string(&first.rep)) > cast2float(cast2string(&second.rep))
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("false".to_string()),
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
                        if second.id == TokId::INT && cast2int(cast2string(&first.rep)) < cast2int(cast2string(&second.rep)) {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2float(cast2string(&second.rep)) > cast2int(cast2string(&first.rep)) as f64
                        {
                            // hack because other way don't work
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("false".to_string()),
                            })
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT
                            && cast2float(cast2string(&first.rep)) < cast2int(cast2string(&second.rep)) as f64
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2float(cast2string(&first.rep)) < cast2float(cast2string(&second.rep))
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("false".to_string()),
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
                        if second.id == TokId::INT && cast2int(cast2string(&first.rep)) >= cast2int(cast2string(&second.rep))
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2int(cast2string(&first.rep)) as f64 >= cast2float(cast2string(&second.rep))
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("false".to_string()),
                            })
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT
                            && cast2float(cast2string(&first.rep)) >= cast2int(cast2string(&second.rep)) as f64
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2float(cast2string(&first.rep)) >= cast2float(cast2string(&second.rep))
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("false".to_string()),
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
                        if second.id == TokId::INT && cast2int(cast2string(&first.rep)) <= cast2int(cast2string(&second.rep))
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2int(cast2string(&first.rep)) as f64 <= cast2float(cast2string(&second.rep))
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("false".to_string()),
                            })
                        }
                    }
                    TokId::FLOAT => {
                        if second.id == TokId::INT
                            && cast2float(cast2string(&first.rep)) <= cast2int(cast2string(&second.rep)) as f64
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else if second.id == TokId::FLOAT
                            && cast2float(cast2string(&first.rep)) <= cast2float(cast2string(&second.rep))
                        {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("true".to_string()),
                            })
                        } else {
                            live_stack.push(Object {
                                id: TokId::BOOLEAN,
                                rep: Value::STR("false".to_string()),
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
                match cast2string(&tok.rep).as_str() {
                    "print" => {
                        let value = live_stack
                            .pop()
                            .expect("INTERP: error no argument to print");
                        match value.id {
                            TokId::STRING | TokId::INT | TokId::FLOAT | TokId::BOOLEAN => {
                                println!("{}", value.rep)
                            }
                            TokId::ARRAY => {
                                match value.rep {
                                    Value::STR(sr) => {
                                        panic!("INTERP: argument defined as Array is not an Array {}", sr)
                                    }
                                    Value::ARR(arr) => {
                                        println!("{}", array2string(arr))
                                    }
                                }
                            }
                            _ => {
                                panic!("INTERP: can't print {}", value.rep);
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
                    "clear" => { //risky! clears the entire stack
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
                                live_stack.push(Object {
                                    rep: Value::STR((f64::sqrt(cast2int(cast2string(&item.rep)) as f64) as i32).to_string()),
                                    id: TokId::INT,
                                });
                            }
                            TokId::FLOAT => {
                                live_stack.push(Object {
                                    rep: Value::STR((f64::sqrt(cast2float(cast2string(&item.rep)))).to_string()),
                                    id: TokId::FLOAT,
                                });
                            }
                            typ => {
                                panic!("INTERP: {:?} can't take the square root of this type", typ);
                            }
                        }
                    }
                    "push" => {
                        let second = live_stack.pop().expect("INTERP: error no argument to push");
                        let first = live_stack.pop().expect("INTERP: error no argument to push");
                        if first.id == TokId::ARRAY {
                            match second.id {
                                TokId::INT | TokId::FLOAT | TokId::BOOLEAN | TokId::ARRAY | TokId::STRING => { // Base Types
                                    let Value::ARR(mut first_arr) = first.rep else {
                                        panic!("INTERP: expected Array but got this {}", first.rep);
                                    };
                                    first_arr.push(second);
                                    live_stack.push(Object {
                                        id: TokId::ARRAY,
                                        rep: Value::ARR(first_arr),
                                    })
                                }
                                _ => panic!("INTERP: {} can't be pushed into {}", second.rep, first.rep)
                            }
                        } else if second.id == TokId::ARRAY {
                            match first.id {
                                TokId::INT | TokId::FLOAT | TokId::BOOLEAN | TokId::ARRAY | TokId::STRING => { // Base Types
                                    let Value::ARR(mut second_arr) = second.rep else {
                                        panic!("INTERP: expected Array but got this {}", first.rep);
                                    };
                                    second_arr.push(first);
                                    live_stack.push(Object {
                                        id: TokId::ARRAY,
                                        rep: Value::ARR(second_arr),
                                    })
                                }
                                _ => panic!("INTERP: {} can't be pushed into {}", first.rep, second.rep)
                            }
                        } else {
                            panic!("INTERP: no Array provided for push")
                        }
                    }
                    "pop" => {
                        let item = live_stack.pop().expect("INTERP: error no argument to pop");
                        if item.id == TokId::ARRAY {
                            let Value::ARR(mut arr) = item.rep else {
                                panic!("INTERP: expected Array but got this {}", item.rep);
                            };
                            if let Some(popped) = arr.pop() {
                                live_stack.push(Object {
                                    id: TokId::ARRAY,
                                    rep: Value::ARR(arr),
                                });
                                live_stack.push(popped)
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
        panic!("INTERP: can't parse {} to int", data))
}

fn cast2float(data: &String) -> f64 {
    data.parse::<f64>()
        .expect("INTERP: can't parse value to int")
}


fn cast2string(val: &Value) -> &String {
    match &val {
        Value::STR(s) => s,
        Value::ARR(_) => panic!("INTERP: can't cast array as string")
    }
}

fn cast2str(val: Value) -> String {
    match val {
        Value::STR(s) => s,
        Value::ARR(_) => panic!("INTERP: can't cast array as string")
    }
}

pub fn array2string(arr: Vec<Object>) -> String {
    let mut result: String = String::from("[ ");
    for item in arr {
        match item.rep {
            Value::STR(s) => {
                if item.id == TokId::STRING {
                    result += ("\"".to_string() + s.as_str() + "\" ").as_str()
                } else {
                    result += (s + " ").as_str()
                }
            }
            Value::ARR(a) => result += (array2string(a) + " ").as_str(),
        }
    }
    result += "]";
    return result;
}