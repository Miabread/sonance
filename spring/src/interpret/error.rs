use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::SimpleSpan;

use crate::{interpret::Context, type_tree::Ident};

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
