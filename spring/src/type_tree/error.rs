use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::SimpleSpan;

use crate::type_tree::{Context, Type};

pub struct TypeMismatchError {
    pub received: Type,
    pub receive_expr: SimpleSpan,
    pub expected: Vec<Type>,
    pub expected_expr: SimpleSpan,
}

impl TypeMismatchError {
    pub fn report(self, ctx: &mut Context<'_>) {
        if self.received == Type::Error {
            return;
        }

        let expected = self
            .expected
            .into_iter()
            .map(|ty| format!("{ty}"))
            .collect::<Vec<_>>()
            .join(" or ");

        ctx.error_count += 1;

        Report::build(ReportKind::Error, ((), self.receive_expr.into_range()))
            .with_message(format!(
                "expected type {} but got type {}",
                expected, self.received
            ))
            .with_label(
                Label::new(((), self.receive_expr.into_range()))
                    .with_message(format!("got type {} here", self.received))
                    .with_color(Color::Red),
            )
            .with_label(
                Label::new(((), self.expected_expr.into_range()))
                    .with_message(format!("expected type {} here", expected))
                    .with_color(Color::Blue),
            )
            .finish()
            .eprint(Source::from(ctx.source))
            .unwrap();
    }
}

pub struct MatchOverlapError {
    pub match_span: SimpleSpan,
    pub first_span: SimpleSpan,
    pub repeat_span: SimpleSpan,
}

impl MatchOverlapError {
    pub fn report(self, ctx: &mut Context<'_>) {
        Report::build(ReportKind::Error, ((), self.match_span.into_range()))
            .with_message("pattern overlaps")
            .with_label(
                Label::new(((), self.first_span.into_range()))
                    .with_message("pattern first used here")
                    .with_color(Color::Blue),
            )
            .with_label(
                Label::new(((), self.repeat_span.into_range()))
                    .with_message("patten repeated here")
                    .with_color(Color::Red),
            )
            .finish()
            .eprint(Source::from(ctx.source))
            .unwrap();
    }
}

pub struct MatchMissingDiscardError {
    pub match_span: SimpleSpan,
}

impl MatchMissingDiscardError {
    pub fn report(self, ctx: &mut Context<'_>) {
        Report::build(ReportKind::Error, ((), self.match_span.into_range()))
            .with_message("expected discard inside `match` expression")
            .with_label(
                Label::new(((), self.match_span.into_range()))
                    .with_message("`match` expression here")
                    .with_color(Color::Red),
            )
            .finish()
            .eprint(Source::from(ctx.source))
            .unwrap();
    }
}
