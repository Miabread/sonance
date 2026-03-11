use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::SimpleSpan;

use crate::type_tree::{Context, Type};

pub struct TypeMismatchError {
    pub produce_expr: SimpleSpan,
    pub consume_expr: SimpleSpan,
    pub expected: Type,
    pub received: Type,
}

impl TypeMismatchError {
    pub fn report(self, ctx: &mut Context<'_>) {
        ctx.error_count += 1;

        Report::build(ReportKind::Error, ((), self.produce_expr.into_range()))
            .with_message(format!(
                "expected type {} but got type {}",
                self.expected, self.received
            ))
            .with_label(
                Label::new(((), self.produce_expr.into_range()))
                    .with_message(format!("got type {} here", self.received))
                    .with_color(Color::Red),
            )
            .with_label(
                Label::new(((), self.consume_expr.into_range()))
                    .with_message(format!("expected type {} here", self.expected))
                    .with_color(Color::Blue),
            )
            .finish()
            .eprint(Source::from(ctx.source))
            .unwrap();
    }
}
