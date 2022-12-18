// global function name
pub const GLOBAL: &'static str = "_global";
// main function name
pub const MAIN: &'static str = "main";

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
    TINT,
    // raw types
    TFLOAT,
    TSTRING,
    TBOOL,
    // for cool visualizations
    LINEBREAK,
    UNKNOWN,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lexeme<T> {
    pub id: TokId,
    pub rep: T,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub arguments: Vec<Lexeme<String>>,
    pub stack: Vec<Lexeme<String>>,
}

impl std::fmt::Display for TokId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", *self as u32)
    }
}
