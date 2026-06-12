pub mod error;

use std::fmt::Display;

use ariadne::{Color, ColorGenerator, Label, Report, ReportKind, Source};
use chumsky::span::SpanWrap;

use crate::{
    interpret::error::InterpretError,
    type_tree::{
        Block, Expr, ExprKind, Ident, ItemKind, Module, Op, Pattern, Statement, StatementKind, Type,
    },
};

#[derive(Debug, Clone, PartialEq)]
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

pub struct Interpreter<'src> {
    pub src: &'src str,
}

impl<'src> Interpreter<'src> {
    pub fn new(src: &'src str) -> Self {
        Self { src }
    }

    pub fn eval_module(
        &mut self,
        module: &Module<'src>,
    ) -> Result<Value<'src>, InterpretError<'src>> {
        let body = module
            .items
            .iter()
            .find_map(|item| {
                if let ItemKind::Func { name, body } = &item.kind
                    && name.name == "main"
                {
                    Some(body)
                } else {
                    None
                }
            })
            .expect("meow");

        self.eval_block(body)
    }

    pub fn eval_block(&mut self, block: &Block<'src>) -> Result<Value<'src>, InterpretError<'src>> {
        let mut output = None;

        for stmt in &block.body {
            output = Some(self.eval_stmt(stmt)?);
        }

        Ok(output.unwrap_or(Value::Unit))
    }

    pub fn eval_stmt(
        &mut self,
        stmt: &Statement<'src>,
    ) -> Result<Value<'src>, InterpretError<'src>> {
        match &stmt.kind {
            StatementKind::Expr(expr) => self.eval_expr(expr),
        }
    }

    fn eval_expr(&mut self, expr: &Expr<'src>) -> Result<Value<'src>, InterpretError<'src>> {
        Ok(match &expr.kind {
            ExprKind::Int(i) => Value::Int(*i),
            ExprKind::Float(f) => Value::Float(*f),
            ExprKind::String(s) => Value::String(s),
            ExprKind::BinOp { op, lhs, rhs } => match lhs.ty {
                Type::Int => {
                    let Value::Int(lhs_value) = self.eval_expr(lhs)? else {
                        panic!("expected int value");
                    };

                    let Value::Int(rhs_value) = self.eval_expr(rhs)? else {
                        panic!("expected int value");
                    };

                    Value::Int(match op {
                        Op::Add => lhs_value + rhs_value,
                        Op::Sub => lhs_value - rhs_value,
                        Op::Mul => lhs_value * rhs_value,
                        Op::Div => {
                            if rhs_value == 0 {
                                return Err(InterpretError::DivideByZeroError { span: rhs.span }
                                    .report(self));
                            }
                            lhs_value / rhs_value
                        }
                    })
                }
                Type::Float => {
                    let Value::Float(lhs_value) = self.eval_expr(lhs)? else {
                        panic!("expected float value");
                    };

                    let Value::Float(rhs_value) = self.eval_expr(rhs)? else {
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
            ExprKind::Match { scrutinee, arms } => 'block: {
                let Value::Int(scrutinee) = self.eval_expr(scrutinee)? else {
                    panic!("expected int value");
                };

                for (pat, expr) in arms {
                    match pat.inner {
                        Pattern::Int(i) => {
                            if scrutinee == i {
                                break 'block self.eval_expr(expr)?;
                            }
                        }
                        Pattern::Discard => {
                            break 'block self.eval_expr(expr)?;
                        }
                    }
                }

                panic!("hit end of match");
            }
            ExprKind::Macro { name, args } => return self.eval_macro(expr, name, args),
        })
    }

    fn eval_macro(
        &mut self,
        expr: &Expr<'src>,
        name: &Ident<'src>,
        args: &Vec<Expr<'src>>,
    ) -> Result<Value<'src>, InterpretError<'src>> {
        match name.name {
            "print" => {
                let args = args
                    .iter()
                    .map(|expr| self.eval_expr(expr))
                    .collect::<Result<Vec<_>, _>>()?;

                for arg in args {
                    print!("{}", arg);
                }

                println!();
                Ok(Value::Unit)
            }

            "dbg" => {
                let mut colors = ColorGenerator::new();

                let labels = args
                    .iter()
                    .map(|expr| {
                        Ok(Label::new(((), expr.span.into_range()))
                            .with_message(format!("{}", self.eval_expr(expr)?))
                            .with_color(colors.next()))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Report::build(
                    ReportKind::Custom("Debug", Color::Blue),
                    ((), expr.span.into_range()),
                )
                .with_labels(labels)
                .finish()
                .eprint(Source::from(self.src))
                .unwrap();

                Ok(Value::Unit)
            }

            "ty" => {
                let mut colors = ColorGenerator::new();

                let labels = args.iter().map(|expr| {
                    Label::new(((), expr.span.into_range()))
                        .with_message(format!("{}", expr.ty))
                        .with_color(colors.next())
                });

                Report::build(
                    ReportKind::Custom("Debug Types", Color::Blue),
                    ((), expr.span.into_range()),
                )
                .with_labels(labels)
                .finish()
                .eprint(Source::from(self.src))
                .unwrap();

                Ok(Value::Unit)
            }

            "error" => {
                let args = args
                    .iter()
                    .map(|expr| Ok(self.eval_expr(expr)?.with_span(expr.span)))
                    .collect::<Result<_, _>>()?;

                Err(InterpretError::CustomError {
                    args,
                    span: expr.span,
                }
                .report(self))
            }

            _ => Err(InterpretError::UnknownBuiltinError { name: name.clone() }.report(self)),
        }
    }
}
