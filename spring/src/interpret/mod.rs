pub mod error;

use std::fmt::Display;

use ariadne::{Color, ColorGenerator, Label, Report, ReportKind, Source};

use crate::{
    DummyError,
    type_tree::{Expr, ExprKind, Op, Statement, StatementKind, Type},
};

use error::*;

pub struct Context<'src> {
    pub source: &'src str,
}

pub fn eval_stmt<'src>(
    stmt: &Statement<'src>,
    ctx: &mut Context<'src>,
) -> Result<Value<'src>, DummyError> {
    match &stmt.kind {
        StatementKind::Expr(expr) => eval_expr(expr, ctx),
        StatementKind::Macro(name, exprs) => match name.name {
            "print" => {
                let exprs = exprs
                    .iter()
                    .map(|expr| eval_expr(expr, ctx))
                    .collect::<Result<Vec<_>, DummyError>>()?;

                for expr in exprs {
                    print!("{}", expr);
                }

                println!();
                Ok(Value::Unit)
            }

            "dbg" => {
                let mut colors = ColorGenerator::new();

                let exprs = exprs
                    .iter()
                    .map(|expr| {
                        Ok(Label::new(((), expr.span.into_range()))
                            .with_message(format!("{}", eval_expr(expr, ctx)?))
                            .with_color(colors.next()))
                    })
                    .collect::<Result<Vec<_>, DummyError>>()?;

                Report::build(
                    ReportKind::Custom("Debug", Color::Blue),
                    ((), stmt.span.into_range()),
                )
                .with_labels(exprs)
                .finish()
                .eprint(Source::from(ctx.source))
                .unwrap();

                Ok(Value::Unit)
            }

            _ => {
                UnknownBuiltinError { name: name.clone() }.report(ctx);
                Err(DummyError)
            }
        },
    }
}

fn eval_expr<'src>(expr: &Expr<'src>, ctx: &mut Context<'src>) -> Result<Value<'src>, DummyError> {
    Ok(match &expr.kind {
        ExprKind::Int(i) => Value::Int(*i),
        ExprKind::String(s) => Value::String(s),
        ExprKind::BinOp(op, lhs, rhs) => {
            let lhs_value = eval_expr(lhs, ctx)?;
            let Value::Int(lhs_value) = lhs_value else {
                TypeMismatchError {
                    produce_expr: lhs.span,
                    consume_expr: expr.span,
                    expected: Type::Int,
                    received: lhs_value.ty(),
                }
                .report(ctx);
                return Err(DummyError);
            };

            let rhs_value = eval_expr(rhs, ctx)?;
            let Value::Int(rhs_value) = rhs_value else {
                TypeMismatchError {
                    produce_expr: rhs.span,
                    consume_expr: expr.span,
                    expected: Type::Int,
                    received: rhs_value.ty(),
                }
                .report(ctx);
                return Err(DummyError);
            };

            Value::Int(match op {
                Op::Add => lhs_value + rhs_value,
                Op::Sub => lhs_value - rhs_value,
                Op::Mul => lhs_value * rhs_value,
                Op::Div => {
                    if rhs_value == 0 {
                        DivideByZeroError { span: rhs.span }.report(ctx);
                        return Err(DummyError);
                    }
                    lhs_value / rhs_value
                }
            })
        }
    })
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

impl Value<'_> {
    fn ty(&self) -> Type {
        match self {
            Value::Unit => Type::Unit,
            Value::Int(_) => Type::Int,
            Value::String(_) => Type::String,
        }
    }
}
