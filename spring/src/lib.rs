use std::fmt::Display;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::{SimpleSpan, Spanned};

use crate::parse_tree::{Expr, Op, Statement};

pub mod parse_tree;

#[derive(Debug, Clone)]
pub struct DummyError;

pub struct Context<'src> {
    pub source: &'src str,
}

trait Eval<'src> {
    type Output;
    fn eval(&self, ctx: &mut Context<'src>) -> Result<Self::Output, DummyError>;
}

impl<'src> Statement<'src> {
    pub fn eval(self, ctx: &mut Context<'src>) -> Result<Value<'src>, DummyError> {
        match self {
            Statement::Expr(expr) => expr.eval(ctx),
            Statement::Macro(name, exprs) => match name.inner {
                "print" => {
                    for expr in exprs {
                        print!("{}", expr.eval(ctx)?);
                    }
                    println!();
                    Ok(Value::Unit)
                }
                _ => {
                    UnknownBuiltinError { name }.report(ctx);
                    Err(DummyError)
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

impl<'src> Eval<'src> for Spanned<Expr<'src>> {
    type Output = Value<'src>;

    fn eval(&self, ctx: &mut Context<'src>) -> Result<Value<'src>, DummyError> {
        Ok(match &self.inner {
            Expr::Int(i) => Value::Int(*i),
            Expr::String(s) => Value::String(s),
            Expr::BinOp(op, lhs, rhs) => {
                let Value::Int(lhs_value) = lhs.eval(ctx)? else {
                    TypeMismatchError {
                        produce_expr: lhs.span,
                        consume_expr: self.span,
                        expected: Type::Int,
                        received: Type::Unit,
                    }
                    .report(ctx);
                    return Err(DummyError);
                };
                let Value::Int(rhs_value) = rhs.eval(ctx)? else {
                    TypeMismatchError {
                        produce_expr: rhs.span,
                        consume_expr: self.span,
                        expected: Type::Int,
                        received: Type::Unit,
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

#[derive(Debug, Clone)]
pub enum Type {
    Unit,
    Int,
    String,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unit => write!(f, "Unit"),
            Type::Int => write!(f, "Int"),
            Type::String => write!(f, "String"),
        }
    }
}
