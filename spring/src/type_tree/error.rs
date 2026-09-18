use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::SimpleSpan;

use crate::type_tree::{Type, TypeContext};

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    UnifyError {
        expected: Type,
        received: Type,
    },
    MatchOverlapError {
        match_span: SimpleSpan,
        first_span: SimpleSpan,
        repeat_span: SimpleSpan,
    },
    MatchMissingDiscardError {
        match_span: SimpleSpan,
    },
    UnknownVariableError {
        ident_span: SimpleSpan,
    },
}

impl TypeError {
    pub fn report(self, ctx: &mut TypeContext<'_>) {
        ctx.errors.push(self.clone());
        match self {
            TypeError::UnifyError { expected, received } => {
                Report::build(ReportKind::Error, ((), received.span.into_range()))
                    .with_message(format!(
                        "expected type {} but got type {}",
                        expected, received
                    ))
                    .with_label(
                        Label::new(((), received.span.into_range()))
                            .with_message(format!("got type {} here", received))
                            .with_color(Color::Red),
                    )
                    .with_label(
                        Label::new(((), expected.span.into_range()))
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

            TypeError::UnknownVariableError { ident_span } => {
                Report::build(ReportKind::Error, ((), ident_span.into_range()))
                    .with_message("unknown variable")
                    .with_label(
                        Label::new(((), ident_span.into_range()))
                            .with_message("used here")
                            .with_color(Color::Red),
                    )
            }
        }
        .finish()
        .eprint(Source::from(ctx.src))
        .unwrap();
    }
}
