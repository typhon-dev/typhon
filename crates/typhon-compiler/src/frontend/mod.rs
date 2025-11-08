//! Frontend components of the Typhon compiler, including lexer, parser, and AST.

/// Abstract Syntax Tree (AST) definitions for the Typhon programming language.
pub mod ast;

/// Lexical analysis for the Typhon programming language.
pub mod lexer;

/// Parser for the Typhon programming language.
pub mod parser;

// Re-exports for commonly used components
pub use self::ast::{
    BinaryOperator,
    DefaultVisitor,
    Expression,
    Identifier,
    Literal,
    Module,
    MutVisitor,
    Parameter,
    SourceInfo,
    Statement,
    TypeExpression,
    UnaryOperator,
    Visitor,
};
pub use self::lexer::{
    Lexer,
    token::{
        Token,
        TokenKind,
        TokenSpan,
    },
};
pub use self::parser::{
    Parser,
    error::{
        ParseError,
        ParseResult,
    },
};
