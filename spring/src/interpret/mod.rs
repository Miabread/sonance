pub mod error;
pub mod scope;

use std::fmt::Display;

use ariadne::{Color, ColorGenerator, Label, Report, ReportKind, Source};
use chumsky::span::SpanWrap;

use crate::{
    interpret::{error::InterpretError, scope::Scope},
    type_tree::{
        BinOp, BinOpExpr, Block, Expr, ExprKind, FuncItem, Ident, ItemKind, MacroCallExpr,
        MatchExpr, Module, Pattern, Statement, StatementKind, TypeKind,
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
                if let ItemKind::Func(FuncItem { name, body }) = &item.kind
                    && name.name == "main"
                {
                    Some(body)
                } else {
                    None
                }
            })
            .expect("meow");

        let mut scope = Scope::new();
        self.eval_block(body, &mut scope)
    }

    pub fn eval_block(
        &mut self,
        block: &Block<'src>,
        scope: &mut Scope<'src, '_>,
    ) -> Result<Value<'src>, InterpretError<'src>> {
        for stmt in &block.body {
            self.eval_stmt(stmt, scope)?;
        }

        self.eval_expr(&block.trailing, scope)
    }

    pub fn eval_stmt(
        &mut self,
        stmt: &Statement<'src>,
        scope: &mut Scope<'src, '_>,
    ) -> Result<Value<'src>, InterpretError<'src>> {
        match &stmt.kind {
            StatementKind::Expr(expr) => self.eval_expr(expr, scope),
        }
    }

    fn eval_expr(
        &mut self,
        expr: &Expr<'src>,
        scope: &mut Scope<'src, '_>,
    ) -> Result<Value<'src>, InterpretError<'src>> {
        Ok(match &expr.kind {
            ExprKind::Unit => Value::Unit,
            ExprKind::Int(i) => Value::Int(*i),
            ExprKind::Float(f) => Value::Float(*f),
            ExprKind::String(s) => Value::String(s),
            ExprKind::Var(v) => scope.get(v.name).expect("variable").clone(),
            ExprKind::BinOp(BinOpExpr { op, lhs, rhs }) => match lhs.ty.kind {
                TypeKind::Int => {
                    let Value::Int(lhs_value) = self.eval_expr(lhs, scope)? else {
                        panic!("expected int value");
                    };

                    let Value::Int(rhs_value) = self.eval_expr(rhs, scope)? else {
                        panic!("expected int value");
                    };

                    Value::Int(match op {
                        BinOp::Add => lhs_value + rhs_value,
                        BinOp::Sub => lhs_value - rhs_value,
                        BinOp::Mul => lhs_value * rhs_value,
                        BinOp::Div => {
                            if rhs_value == 0 {
                                return Err(InterpretError::DivideByZeroError { span: rhs.span }
                                    .report(self));
                            }
                            lhs_value / rhs_value
                        }
                    })
                }
                TypeKind::Float => {
                    let Value::Float(lhs_value) = self.eval_expr(lhs, scope)? else {
                        panic!("expected float value");
                    };

                    let Value::Float(rhs_value) = self.eval_expr(rhs, scope)? else {
                        panic!("expected float value");
                    };

                    Value::Float(match op {
                        BinOp::Add => lhs_value + rhs_value,
                        BinOp::Sub => lhs_value - rhs_value,
                        BinOp::Mul => lhs_value * rhs_value,
                        BinOp::Div => lhs_value / rhs_value,
                    })
                }
                _ => panic!("bin op unsupported type"),
            },
            ExprKind::Match(MatchExpr { scrutinee, arms }) => 'block: {
                let Value::Int(scrutinee) = self.eval_expr(scrutinee, scope)? else {
                    panic!("expected int value");
                };

                for (pat, block) in arms {
                    match pat.inner {
                        Pattern::Int(i) => {
                            if scrutinee == i {
                                break 'block self.eval_block(block, scope)?;
                            }
                        }
                        Pattern::Discard => {
                            break 'block self.eval_block(block, scope)?;
                        }
                    }
                }

                panic!("hit end of match");
            }
            ExprKind::MacroCall(MacroCallExpr { name, args }) => {
                return self.eval_macro(expr, name, args, scope);
            }
        })
    }

    fn eval_macro(
        &mut self,
        expr: &Expr<'src>,
        name: &Ident<'src>,
        args: &Vec<Expr<'src>>,
        scope: &mut Scope<'src, '_>,
    ) -> Result<Value<'src>, InterpretError<'src>> {
        match name.name {
            "print" => {
                let args = args
                    .iter()
                    .map(|expr| self.eval_expr(expr, scope))
                    .collect::<Result<Vec<_>, _>>()?;

                for arg in args {
                    print!("{}", arg);
                }

                println!();
                Ok(Value::Unit)
            }

            "debug" => {
                let mut colors = ColorGenerator::new();

                let labels = args
                    .iter()
                    .map(|expr| {
                        Ok(Label::new(((), expr.span.into_range()))
                            .with_message(format!("{}", self.eval_expr(expr, scope)?))
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

            "type" => {
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
                    .map(|expr| Ok(self.eval_expr(expr, scope)?.with_span(expr.span)))
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
