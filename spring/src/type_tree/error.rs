use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::SimpleSpan;

use crate::type_tree::{Context, Type};

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    TypeMismatchError {
        received: Type,
        receive_expr: SimpleSpan,
        expected: Vec<Type>,
        expected_expr: SimpleSpan,
    },
    MatchOverlapError {
        match_span: SimpleSpan,
        first_span: SimpleSpan,
        repeat_span: SimpleSpan,
    },
    MatchMissingDiscardError {
        match_span: SimpleSpan,
    },
}

impl TypeError {
    pub fn report(self, ctx: &mut Context<'_>) {
        ctx.errors.push(self.clone());
        match self {
            TypeError::TypeMismatchError {
                received,
                receive_expr,
                expected,
                expected_expr,
            } => {
                if received == Type::Error {
                    return;
                }

                let expected = expected
                    .into_iter()
                    .map(|ty| format!("{ty}"))
                    .collect::<Vec<_>>()
                    .join(" or ");

                Report::build(ReportKind::Error, ((), receive_expr.into_range()))
                    .with_message(format!(
                        "expected type {} but got type {}",
                        expected, received
                    ))
                    .with_label(
                        Label::new(((), receive_expr.into_range()))
                            .with_message(format!("got type {} here", received))
                            .with_color(Color::Red),
                    )
                    .with_label(
                        Label::new(((), expected_expr.into_range()))
                            .with_message(format!("expected type {} here", expected))
                            .with_color(Color::Blue),
                    )
            }

            TypeError::MatchOverlapError {
                match_span,
                first_span,
                repeat_span,
            } => Report::build(ReportKind::Error, ((), match_span.into_range()))
                .with_message("pattern overlaps")
                .with_label(
                    Label::new(((), first_span.into_range()))
                        .with_message("pattern first used here")
                        .with_color(Color::Blue),
                )
                .with_label(
                    Label::new(((), repeat_span.into_range()))
                        .with_message("patten repeated here")
                        .with_color(Color::Red),
                ),

            TypeError::MatchMissingDiscardError { match_span } => {
                Report::build(ReportKind::Error, ((), match_span.into_range()))
                    .with_message("expected discard inside `match` expression")
                    .with_label(
                        Label::new(((), match_span.into_range()))
                            .with_message("`match` expression here")
                            .with_color(Color::Red),
                    )
            }
        }
        .finish()
        .eprint(Source::from(ctx.src))
        .unwrap();
    }
}
