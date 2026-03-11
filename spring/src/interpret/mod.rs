use std::fmt::Display;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::SimpleSpan;

use crate::{
    DummyError,
    type_tree::{Expr, ExprKind, Ident, Op, Statement, StatementKind, Type},
};

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
                for expr in exprs {
                    print!("{}", eval_expr(expr, ctx)?);
                }
                println!();
                Ok(Value::Unit)
            }
            _ => {
                UnknownBuiltinError { name: name.clone() }.report(ctx);
                Err(DummyError)
            }
        },
    }
}

struct UnknownBuiltinError<'src> {
    name: Ident<'src>,
}

impl UnknownBuiltinError<'_> {
    fn report(self, ctx: &mut Context<'_>) {
        Report::build(ReportKind::Error, ((), self.name.span.into_range()))
            .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
            .with_code(3)
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

struct TypeMismatchError {
    produce_expr: SimpleSpan,
    consume_expr: SimpleSpan,
    expected: Type,
    received: Type,
}

impl TypeMismatchError {
    fn report(self, ctx: &mut Context<'_>) {
        Report::build(ReportKind::Error, ((), self.produce_expr.into_range()))
            .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
            .with_code(3)
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

struct DivideByZeroError {
    span: SimpleSpan,
}

impl DivideByZeroError {
    fn report(self, ctx: &mut Context<'_>) {
        Report::build(ReportKind::Error, ((), self.span.into_range()))
            .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
            .with_code(3)
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
