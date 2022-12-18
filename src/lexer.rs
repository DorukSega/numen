use crate::head;
use head::{Lexeme, TokId};

const LEXMAP: [Lexeme<&'static str>; 27] = [
    Lexeme { id: TokId::FUNCTION, rep: "fun" },
    Lexeme { id: TokId::IMPORT, rep: "import" },
    Lexeme { id: TokId::END, rep: "end" },
    Lexeme { id: TokId::AS, rep: "as" },
    Lexeme { id: TokId::RET, rep: "ret" },
    Lexeme { id: TokId::WHILE, rep: "while" },
    Lexeme { id: TokId::DO, rep: "do" },
    Lexeme { id: TokId::IF, rep: "if" },
    Lexeme { id: TokId::BLOCK, rep: "block" },
    Lexeme { id: TokId::ELSE, rep: "else" },
    Lexeme { id: TokId::PLUS, rep: "+" },
    Lexeme { id: TokId::MINUS, rep: "-" },
    Lexeme { id: TokId::MULTIPLY, rep: "*", },
    Lexeme { id: TokId::DIVIDE, rep: "/", },
    Lexeme { id: TokId::MOD, rep: "%", },
    Lexeme { id: TokId::ASSIGNMENT, rep: "=", },
    Lexeme { id: TokId::RETURNINGASSIGNMENT, rep: "=>", },
    Lexeme { id: TokId::EQUALS, rep: "==", },
    Lexeme { id: TokId::BIGGER, rep: ">", },
    Lexeme { id: TokId::SMALLER, rep: "<", },
    Lexeme { id: TokId::SMALLEREQUALS, rep: "<=", },
    Lexeme { id: TokId::BIGGEREQUALS, rep: ">=", },
    Lexeme { id: TokId::TINT, rep: "int", },
    Lexeme { id: TokId::TFLOAT, rep: "float", },
    Lexeme { id: TokId::TSTRING, rep: "str", },
    Lexeme { id: TokId::TBOOL, rep: "bool", },
    Lexeme { id: TokId::IS, rep: "is", },
];


pub fn lexmap_contains_value(comp: &str) -> Option<TokId> {
    for lex in LEXMAP {
        if lex.rep == comp {
            return Some(lex.id.clone());
        }
    }
    return None;
}

// converts the raw file string to a lexed vector (semi parsed)
pub fn lexer_file(file: &String) -> Vec<Lexeme<String>> {
    let mut result: Vec<Lexeme<String>> = Vec::new();
    let mut word: Vec<char> = Vec::new();
    //let mut raw_string: Vec<char> = Vec::new();
    let mut string_mode: Option<char> = None;

    for char in file.chars() {
        // in raw string mode
        if string_mode.is_some() {
            // end of raw string
            if char == string_mode.unwrap() {
                result.push(Lexeme {
                    id: TokId::STRING,
                    rep: word_to_string(&word).clone(),
                });
                word.clear();
                string_mode = None;
            } else {
                // continue to push into word
                word.push(char);
            }
            continue; // must continue to avoid parsing
        }

        // if there is a raw string
        if char == '"' || char == '\'' {
            if word.len() > 0 {
                // this will only run when the word is not empty
                result.push(Lexeme {
                    id: TokId::UNKNOWN,
                    rep: word_to_string(&word).clone(),
                });
                word.clear();
            }
            string_mode = Some(char);
            continue; // continue into raw string
        }

        // word is something
        if let Some(_id_of) = lexmap_contains_value(&word_to_string(&word)) {
            let mut word_c = word.clone(); // created temporary for word + char
            word_c.push(char);
            // word + char is something
            if let Some(id_of) = lexmap_contains_value(&word_to_string(&word_c)) {
                result.push(Lexeme {
                    id: id_of,
                    rep: word_to_string(&word_c).clone(),
                });
                word.clear();
                word_c.clear();
                continue; // continue to ignore vacant char
            }
        }

        //char is something
        if let Some(_) = lexmap_contains_value(&char.to_string()) {
            // small token
            if word.len() > 0 {
                // this will only run when the word is not empty
                result.push(Lexeme {
                    id: TokId::UNKNOWN,
                    rep: word_to_string(&word).clone(),
                });
                word.clear();
            }
            word.push(char); // push into the word for next time
        } else {
            // char is not a token
            //char is a break
            if char == '\r' || char == '\t' || char.is_whitespace() {
                if char == '\n' {
                    result.push(Lexeme {
                        id: TokId::LINEBREAK,
                        rep: '\n'.to_string(),
                    });
                }
                // word is something
                if let Some(id_of) = lexmap_contains_value(&word_to_string(&word)) {
                    result.push(Lexeme {
                        id: id_of,
                        rep: word_to_string(&word).clone(),
                    });
                    word.clear();
                }
                // word exists and is unknown
                if word.len() > 0 {
                    // this will only run when the word is something
                    result.push(Lexeme {
                        id: TokId::UNKNOWN,
                        rep: word_to_string(&word).clone(),
                    });
                    word.clear();
                }
            } else {
                // char is not a known thing, pushed to word
                word.push(char);
            }
        }
    } // end of for
    // if something is left
    if word.len() > 0 {
        // something important
        if let Some(id_of) = lexmap_contains_value(&word_to_string(&word)) {
            result.push(Lexeme {
                id: id_of,
                rep: word_to_string(&word).clone(),
            });
        } else {
            // unknown
            result.push(Lexeme {
                id: TokId::UNKNOWN,
                rep: word_to_string(&word).clone(),
            });
        }
    }
    word.clear();
    result
}

fn word_to_string(word: &Vec<char>) -> String {
    word.iter().collect()
}
