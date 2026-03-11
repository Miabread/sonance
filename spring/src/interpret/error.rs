use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::SimpleSpan;

use crate::{
    interpret::Context,
    type_tree::{Ident, Type},
};

pub struct UnknownBuiltinError<'src> {
    pub name: Ident<'src>,
}

impl UnknownBuiltinError<'_> {
    pub fn report(self, ctx: &mut Context<'_>) {
        Report::build(ReportKind::Error, ((), self.name.span.into_range()))
            .with_message(format!("unknown builtin `{}`", self.name.name))
            .with_label(
                Label::new(((), self.name.span.into_range()))
                    .with_message("used here")
                    .with_color(Color::Red),
            )
            .finish()
            .eprint(Source::from(ctx.source))
            .unwrap();
    }
}

pub struct DivideByZeroError {
    pub span: SimpleSpan,
}

impl DivideByZeroError {
    pub fn report(self, ctx: &mut Context<'_>) {
        Report::build(ReportKind::Error, ((), self.span.into_range()))
            .with_message("divide by 0 by error")
            .with_label(
                Label::new(((), self.span.into_range()))
                    .with_message("value of 0")
                    .with_color(Color::Red),
            )
            .finish()
            .eprint(Source::from(ctx.source))
            .unwrap();
    }
}

pub struct TypeMismatchError {
    pub produce_expr: SimpleSpan,
    pub consume_expr: SimpleSpan,
    pub expected: Type,
    pub received: Type,
}

impl TypeMismatchError {
    pub fn report(self, ctx: &mut Context<'_>) {
        Report::build(ReportKind::Error, ((), self.produce_expr.into_range()))
            .with_message(format!(
                "expected type {} but got type {}",
                self.expected, self.received
            ))
            .with_label(
                Label::new(((), self.produce_expr.into_range()))
                    .with_message(format!("type {} produced here", self.received))
                    .with_color(Color::Red)
                    .with_order(-1),
            )
            .with_label(
                Label::new(((), self.consume_expr.into_range()))
                    .with_message(format!("type {} expected here", self.expected))
                    .with_color(Color::Blue),
            )
            .finish()
            .eprint(Source::from(ctx.source))
            .unwrap();
    }
}
