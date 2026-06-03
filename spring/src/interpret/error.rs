use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::SimpleSpan;

use crate::{interpret::Context, type_tree::Ident};

#[derive(Debug, Clone, PartialEq)]
pub enum InterpretError<'src> {
    UnknownBuiltinError { name: Ident<'src> },
    DivideByZeroError { span: SimpleSpan },
}

impl InterpretError<'_> {
    pub fn report(self, ctx: &mut Context<'_>) -> Self {
        match self.clone() {
            InterpretError::UnknownBuiltinError { name } => {
                Report::build(ReportKind::Error, ((), name.span.into_range()))
                    .with_message(format!("unknown builtin `{}`", name.name))
                    .with_label(
                        Label::new(((), name.span.into_range()))
                            .with_message("used here")
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(Source::from(ctx.src))
                    .unwrap()
            }

            InterpretError::DivideByZeroError { span } => {
                Report::build(ReportKind::Error, ((), span.into_range()))
                    .with_message("divide by 0 by error")
                    .with_label(
                        Label::new(((), span.into_range()))
                            .with_message("value of 0")
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(Source::from(ctx.src))
                    .unwrap();
            }
        }
        self
    }
}
