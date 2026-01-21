use std::{
    fmt::{Debug, Display},
    iter::Peekable,
    str::CharIndices,
};

use itertools::{Itertools, PeekNth};

#[cfg(test)]
mod test;

#[derive(Clone, Copy)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    fn end(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    fn last(start: usize, last: usize) -> Self {
        Self::end(start, last + 1)
    }

    fn one(start: usize) -> Self {
        Self::last(start, start)
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'src> {
    kind: TokenKind<'src>,
    span: Span,
}

#[derive(Debug, Clone, Copy)]
pub enum TokenKind<'src> {
    Func,
    Ident(&'src str),
    OpenParen,
    CloseParen,
    Arrow,
    OpenBrace,
    CloseBrace,
    Number(usize),
    Semi,
}

#[derive(Clone, Default)]
pub struct TokenTree<'src> {
    tokens: Vec<Token<'src>>,
}

impl Debug for TokenTree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for token in &self.tokens {
            writeln!(f, "{} {:?}", token.span, token.kind)?;
        }
        Ok(())
    }
}

struct TokenCtx<'src> {
    source: &'src str,
    chars: PeekNth<CharIndices<'src>>,
    tree: TokenTree<'src>,

    char: char,
    span: Span,
}

impl<'src> TokenCtx<'src> {
    fn eat_if(&mut self, accept: impl FnOnce(char) -> bool) -> bool {
        let Some((last, _)) = self.chars.next_if(|c| accept(c.1)) else {
            return false;
        };
        self.span.end = last + 1;
        true
    }

    fn eat_eq(&mut self, needle: &str) -> bool {
        let start = self.span.end - 1;
        let end = start + needle.len();

        if end > self.source.len() {
            return false;
        }

        if &self.source[start..end] == needle {
            self.span.end = end;

            // We need to update the iterator state since the comparison went around it
            for _ in 0..needle.len() {
                self.chars.next().unwrap();
            }
            true
        } else {
            false
        }
    }

    fn eat_while(&mut self, mut accept: impl FnMut(char) -> bool) -> bool {
        let Some((last, _)) = self.chars.peeking_take_while(|c| accept(c.1)).last() else {
            return false;
        };
        self.span.end = last + 1;
        true
    }

    fn current_str(&self) -> &'src str {
        &self.source[self.span.start..self.span.end]
    }

    fn push(&mut self, kind: TokenKind<'src>) {
        self.current_str();
        self.tree.tokens.push(Token {
            kind,
            span: self.span,
        });
    }

    pub fn tokenize_once(&mut self) {
        if self.eat_while(|c| c.is_ascii_whitespace()) {
            return;
        }

        if self.eat_eq("->") {
            self.push(TokenKind::Arrow);
            return;
        }

        let singles = [
            ("(", TokenKind::OpenParen),
            (")", TokenKind::CloseParen),
            ("{", TokenKind::OpenBrace),
            ("}", TokenKind::CloseBrace),
            (";", TokenKind::Semi),
        ];

        for (needle, kind) in singles {
            if self.eat_eq(needle) {
                self.push(kind);
                return;
            }
        }

        if self.eat_if(|c| c.is_ascii_alphabetic()) {
            self.eat_while(|c| c.is_ascii_alphanumeric());

            let kind = match self.current_str() {
                "func" => TokenKind::Func,
                ident => TokenKind::Ident(ident),
            };

            self.push(kind);
            return;
        }

        if self.eat_while(|c| c.is_ascii_digit()) {
            self.push(TokenKind::Number(self.current_str().parse().unwrap()));
            return;
        }

        panic!("Invalid token '{}' at {}", self.char, self.span);
    }
}

pub fn tokenize<'src>(source: &'src str) -> TokenTree<'src> {
    let mut chars = itertools::peek_nth(source.char_indices());

    let Some((start, char)) = chars.next() else {
        return TokenTree::default();
    };

    let mut ctx = TokenCtx {
        source,
        chars,
        tree: TokenTree::default(),

        char,
        span: Span::one(start),
    };

    while let Some(&(start, char)) = ctx.chars.peek() {
        ctx.span = Span::one(start);
        ctx.char = char;

        ctx.tokenize_once();
    }
    ctx.tree
}

fn run(input: &str) -> usize {
    let tokens = tokenize(input);
    tokens
        .tokens
        .iter()
        .filter_map(|token| match token.kind {
            TokenKind::Number(num) => Some(num),
            _ => None,
        })
        .last()
        .unwrap_or(0)
}
