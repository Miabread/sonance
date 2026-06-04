pub mod data;
pub mod error;

use std::collections::HashMap;

use chumsky::span::Spanned;

use crate::{
    DummyError,
    parse_tree::{self},
    type_tree::error::TypeError,
};

pub use data::*;

pub struct TypeContext<'src> {
    pub src: &'src str,
    pub errors: Vec<TypeError>,
}

impl<'src> TypeContext<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            src,
            errors: vec![],
        }
    }
}

impl<'src> TypeContext<'src> {
    pub fn type_module(
        &mut self,
        module: Spanned<parse_tree::Module<'src>>,
    ) -> Result<Module<'src>, DummyError> {
        let span = module.span;
        let items = module
            .inner
            .items
            .into_iter()
            .map(|item| self.type_item(item))
            .collect::<Result<_, _>>()?;

        Ok(Module { items, span })
    }

    pub fn type_item(
        &mut self,
        item: Spanned<parse_tree::Item<'src>>,
    ) -> Result<Item<'src>, DummyError> {
        Ok(match item.inner {
            parse_tree::Item::Func { name, body } => Item {
                kind: ItemKind::Func {
                    name: self.type_ident(name),
                    body: self.type_block(body)?,
                },
                span: item.span,
            },
        })
    }

    pub fn type_block(
        &mut self,
        block: Spanned<parse_tree::Block<'src>>,
    ) -> Result<Block<'src>, DummyError> {
        Ok(Block {
            body: block
                .inner
                .body
                .into_iter()
                .map(|stmt| self.type_statement(stmt))
                .collect::<Result<_, _>>()?,
            span: block.span,
        })
    }

    pub fn type_ident(&mut self, ident: Spanned<&'src str>) -> Ident<'src> {
        Ident {
            name: ident.inner,
            span: ident.span,
        }
    }

    pub fn type_statement(
        &mut self,
        stmt: Spanned<parse_tree::Statement<'src>>,
    ) -> Result<Statement<'src>, DummyError> {
        let kind = match stmt.inner {
            parse_tree::Statement::Expr(expr) => StatementKind::Expr(self.type_expr(expr)?),
        };

        let ty = match &kind {
            StatementKind::Expr(expr) => expr.ty.clone(),
        };

        Ok(Statement {
            kind,
            ty,
            span: stmt.span,
        })
    }

    pub fn type_expr(
        &mut self,
        expr: Spanned<parse_tree::Expr<'src>>,
    ) -> Result<Expr<'src>, DummyError> {
        let kind = match expr.inner {
            parse_tree::Expr::Int(i) => ExprKind::Int(i),
            parse_tree::Expr::Float(f) => ExprKind::Float(f),
            parse_tree::Expr::String(s) => ExprKind::String(s),
            parse_tree::Expr::BinOp { op, lhs, rhs } => {
                let lhs = Box::new(self.type_expr(*lhs)?);
                let rhs = Box::new(self.type_expr(*rhs)?);
                ExprKind::BinOp { op, lhs, rhs }
            }
            parse_tree::Expr::Match { scrutinee, arms } => {
                let scrutinee = Box::new(self.type_expr(*scrutinee)?);

                let arms = arms
                    .into_iter()
                    .map(|(pat, expr)| Ok((pat, self.type_expr(expr)?)))
                    .collect::<Result<Vec<_>, _>>()?;

                let mut has_discard = false;
                let mut ints_covered = HashMap::with_capacity(arms.len());

                for (pat, _) in &arms {
                    match pat.inner {
                        Pattern::Int(i) => {
                            if let Some(first_span) = ints_covered.insert(i, pat.span) {
                                TypeError::MatchOverlapError {
                                    match_span: expr.span,
                                    first_span,
                                    repeat_span: pat.span,
                                }
                                .report(self);
                                return Err(DummyError);
                            }
                        }
                        Pattern::Discard => has_discard = true,
                    }
                }

                if !has_discard {
                    TypeError::MatchMissingDiscardError {
                        match_span: expr.span,
                    }
                    .report(self);
                    return Err(DummyError);
                }

                ExprKind::Match { scrutinee, arms }
            }

            parse_tree::Expr::Macro { name, args } => {
                let name = self.type_ident(name);

                let args = args
                    .into_iter()
                    .map(|expr| self.type_expr(expr))
                    .collect::<Result<_, _>>()?;

                ExprKind::Macro { name, args }
            }
        };

        let ty = match &kind {
            ExprKind::Int(_) => Type::Int,
            ExprKind::Float(_) => Type::Float,
            ExprKind::String(_) => Type::String,
            ExprKind::BinOp { lhs, rhs, .. } => 'block: {
                if lhs.ty != Type::Int && lhs.ty != Type::Float {
                    TypeError::TypeMismatchError {
                        receive_expr: lhs.span,
                        expected_expr: expr.span,
                        expected: vec![Type::Int, Type::Float],
                        received: lhs.ty.clone(),
                    }
                    .report(self);
                    break 'block Type::Error;
                }

                if lhs.ty != rhs.ty {
                    TypeError::TypeMismatchError {
                        receive_expr: rhs.span,
                        expected_expr: expr.span,
                        expected: vec![lhs.ty.clone()],
                        received: rhs.ty.clone(),
                    }
                    .report(self);
                    break 'block Type::Error;
                };

                Type::Int
            }
            ExprKind::Match { scrutinee, arms } => 'block: {
                if scrutinee.ty != Type::Int {
                    TypeError::TypeMismatchError {
                        received: scrutinee.ty.clone(),
                        receive_expr: scrutinee.span,
                        expected: vec![Type::Int],
                        expected_expr: scrutinee.span,
                    }
                    .report(self);
                }

                let mut arms = arms.iter();
                let first = &arms.next().unwrap().1;
                for (_, arm) in arms {
                    if arm.ty != first.ty {
                        TypeError::TypeMismatchError {
                            received: arm.ty.clone(),
                            receive_expr: arm.span,
                            expected: vec![first.ty.clone()],
                            expected_expr: first.span,
                        }
                        .report(self);
                        break 'block Type::Error;
                    }
                }
                first.ty.clone()
            }
            ExprKind::Macro { .. } => Type::Unit,
        };

        Ok(Expr {
            kind,
            ty,
            span: expr.span,
        })
    }
}
