use std::fmt::Display;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::Spanned;

use crate::parse_tree::{Expr, Op, Statement};

pub mod parse_tree;

pub struct Context<'src> {
    pub source: &'src str,
}

impl<'src> Statement<'src> {
    pub fn eval(self, ctx: &mut Context<'src>) -> Result<Value<'src>, String> {
        match self {
            Statement::Expr(expr) => expr.inner.eval(ctx),
            Statement::Macro(name, exprs) => match name.inner {
                "print" => {
                    for expr in exprs {
                        print!("{}", expr.inner.eval(ctx)?);
                    }
                    println!();
                    Ok(Value::Unit)
                }
                _ => {
                    UnknownBuiltinError { name }.report(ctx);
                    Err("meow".to_owned())
                }
            },
        }
    }
}

struct UnknownBuiltinError<'src> {
    name: Spanned<&'src str>,
}

impl UnknownBuiltinError<'_> {
    fn report(self, ctx: &mut Context<'_>) {
        Report::build(ReportKind::Error, ((), self.name.span.into_range()))
            .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
            .with_code(3)
            .with_message(format!("unknown builtin `{}`", self.name.inner))
            .with_label(
                Label::new(((), self.name.span.into_range()))
                    .with_message("unknown builtin")
                    .with_color(Color::Red),
            )
            .finish()
            .eprint(Source::from(ctx.source))
            .unwrap();
    }
}

impl<'src> Expr<'src> {
    pub fn eval(self, ctx: &mut Context<'src>) -> Result<Value<'src>, String> {
        Ok(match self {
            Expr::Int(i) => Value::Int(i),
            Expr::String(s) => Value::String(s),
            Expr::BinOp(op, lhs, rhs) => {
                let Value::Int(lhs) = lhs.inner.eval(ctx)? else {
                    return Err("lhs not int".into());
                };
                let Value::Int(rhs) = rhs.inner.eval(ctx)? else {
                    return Err("rhs not int".into());
                };

                Value::Int(match op {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Mul => lhs * rhs,
                    Op::Div => {
                        if rhs == 0 {
                            return Err("div by 0".into());
                        }
                        lhs / rhs
                    }
                })
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum Value<'src> {
    Unit,
    Int(u64),
    String(&'src str),
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => write!(f, "Unit"),
            Value::Int(i) => write!(f, "{i}"),
            Value::String(s) => write!(f, "{s}"),
        }
    }
}
