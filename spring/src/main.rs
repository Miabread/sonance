use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{input::Stream, prelude::*};
use logos::Logos;
use spring::{Token, parser};

const SRC: &str = r"
    1 / 0
";

fn main() {
    // Create a logos lexer over the source code
    let token_iter = Token::lexer(SRC)
        .spanned()
        // Convert logos errors into tokens. We want parsing to be recoverable and not fail at the lexing stage, so
        // we have a dedicated `Token::Error` variant that represents a token error that was previously encountered
        .map(|(tok, span)| match tok {
            // Turn the `Range<usize>` spans logos gives us into chumsky's `SimpleSpan` via `Into`, because it's easier to work with
            Ok(tok) => (tok, span.into()),
            Err(()) => (Token::Error, span.into()),
        });

    // Turn the token iterator into a stream that chumsky can use for things like backtracking
    let token_stream = Stream::from_iter(token_iter)
        // Tell chumsky to split the (Token, SimpleSpan) stream into its parts so that it can handle the spans for us
        // This involves giving chumsky an 'end of input' span: we just use a zero-width span at the end of the string
        .map((0..SRC.len()).into(), |(t, s): (_, _)| (t, s));

    // Parse the token stream with our chumsky parser
    let expr = match parser().parse(token_stream).into_result() {
        Ok(expr) => expr,
        // If parsing was unsuccessful, generate a nice user-friendly diagnostic with ariadne. You could also use
        // codespan, or whatever other diagnostic library you care about. You could even just display-print the errors
        // with Rust's built-in `Display` trait, but it's a little crude
        Err(errs) => {
            for err in errs {
                Report::build(ReportKind::Error, ((), err.span().into_range()))
                    .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                    .with_code(3)
                    .with_message(err.to_string())
                    .with_label(
                        Label::new(((), err.span().into_range()))
                            .with_message(err.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(Source::from(SRC))
                    .unwrap();
            }
            return;
        }
    };

    match expr.eval() {
        Ok(i) => println!("result = {i}"),
        Err(e) => eprintln!("error = {e}"),
    }
}
