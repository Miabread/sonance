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

            "ty" => {
                let mut colors = ColorGenerator::new();

                let exprs = exprs.iter().map(|expr| {
                    Label::new(((), expr.span.into_range()))
                        .with_message(format!("{}", expr.ty))
                        .with_color(colors.next())
                });

                Report::build(
                    ReportKind::Custom("Debug Types", Color::Blue),
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
        ExprKind::Float(f) => Value::Float(*f),
        ExprKind::String(s) => Value::String(s),
        ExprKind::BinOp(op, lhs, rhs) => match lhs.ty {
            Type::Int => {
                let Value::Int(lhs_value) = eval_expr(lhs, ctx)? else {
                    panic!("expected int value");
                };

                let Value::Int(rhs_value) = eval_expr(rhs, ctx)? else {
                    panic!("expected int value");
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
            Type::Float => {
                let Value::Float(lhs_value) = eval_expr(lhs, ctx)? else {
                    panic!("expected float value");
                };

                let Value::Float(rhs_value) = eval_expr(rhs, ctx)? else {
                    panic!("expected float value");
                };

                Value::Float(match op {
                    Op::Add => lhs_value + rhs_value,
                    Op::Sub => lhs_value - rhs_value,
                    Op::Mul => lhs_value * rhs_value,
                    Op::Div => lhs_value / rhs_value,
                })
            }
            _ => panic!("bin op unsupported type"),
        },
    })
}

#[derive(Debug, Clone)]
pub enum Value<'src> {
    Unit,
    Int(u64),
    Float(f64),
    String(&'src str),
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => write!(f, "Unit"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(i) => write!(f, "{i}"),
            Value::String(s) => write!(f, "{s}"),
        }
    }
}
