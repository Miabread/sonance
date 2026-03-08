use std::fmt::Display;

use logos::Logos;

#[derive(Debug, Clone, Logos, PartialEq)]
pub enum Token<'src> {
    Error,
    #[regex(r"\s+", logos::skip)]
    Whitespace,

    #[regex(r"[0-9]+")]
    Int(&'src str),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident(&'src str),
    #[regex(r#""([^"\\]*(?:\\[\s\S][^"\\]*)*)""#, |lex| {
        let mut chars = lex.slice().chars();
        chars.next();
        chars.next_back();
        chars.as_str()
    })]
    String(&'src str),

    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,

    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("->")]
    Arrow,
    #[token("!")]
    Bang,
    #[token(";")]
    Semi,
    #[token(",")]
    Comma,

    #[token("func")]
    Func,
    #[token("int")]
    TInt,
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Error => write!(f, "<error>"),
            Token::Whitespace => write!(f, "<whitespace>"),
            Token::Int(i) => write!(f, "{i}"),
            Token::Ident(i) => write!(f, "{i}"),
            Token::String(s) => write!(f, "{s}"),
            Token::OpenParen => write!(f, "("),
            Token::CloseParen => write!(f, ")"),
            Token::OpenBracket => write!(f, "["),
            Token::CloseBracket => write!(f, "]"),
            Token::OpenBrace => write!(f, "{{"),
            Token::CloseBrace => write!(f, "}}"),
            Token::Add => write!(f, "+"),
            Token::Sub => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Arrow => write!(f, "->"),
            Token::Bang => write!(f, "!"),
            Token::Semi => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::Func => write!(f, "func"),
            Token::TInt => write!(f, "int"),
        }
    }
}
