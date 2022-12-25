use crate::interpreter::array2string;

// global function name
pub const GLOBAL: &'static str = "_global";
// main function name
pub const MAIN: &'static str = "main";

pub const TRUE: &'static str = "true";
pub const FALSE: &'static str = "false";

#[derive(PartialEq, Clone, Debug, Copy)]
pub enum TokId {
    FUNCTION,
    IMPORT,
    END,
    AS,
    RET,
    WHILE,
    DO,
    IF,
    ELSE,
    PLUS,
    MINUS,
    BLOCK,
    LOOP,
    MULTIPLY,
    DIVIDE,
    MOD,
    IS,
    ASSIGNMENT,
    RETURNINGASSIGNMENT,
    EQUALS,
    BIGGER,
    SMALLER,
    BIGGEREQUALS,
    SMALLEREQUALS,
    STRING,
    BOOLEAN,
    INT,
    FLOAT,
    ARRAYBEGIN,
    ARRAYEND,
    ARRAY,
    // raw types
    TINT,
    TFLOAT,
    TSTRING,
    TBOOL,
    TARRAY,
    // for cool visualizations
    LINEBREAK,
    UNKNOWN,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lexeme<T> {
    pub id: TokId,
    pub rep: T,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    STR(String),
    ARR(Vec<Object>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub id: TokId,
    pub rep: Value,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub arguments: Vec<Object>,
    pub stack: Vec<Object>,
}

impl std::fmt::Display for TokId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", *self as u32)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::STR(s) => {
                write!(f, "{}", s)
            }
            Value::ARR(arr) => {
                write!(f, "{}", array2string(arr.clone()))
            }
        }
    }
}
