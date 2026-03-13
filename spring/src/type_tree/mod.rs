pub mod data;
pub mod error;

use std::collections::HashMap;

use chumsky::span::Spanned;

use crate::{DummyError, parse_tree};

pub use data::*;
use error::*;

pub struct Context<'src> {
    pub source: &'src str,
    pub error_count: u64,
}

pub fn type_statement<'src>(
    stmt: Spanned<parse_tree::Statement<'src>>,
    ctx: &mut Context<'src>,
) -> Result<Statement<'src>, DummyError> {
    let kind = match stmt.inner {
        parse_tree::Statement::Expr(expr) => StatementKind::Expr(type_expr(expr, ctx)?),
        parse_tree::Statement::Macro(ident, args) => {
            let ident = Ident {
                name: ident.inner,
                span: ident.span,
            };

            let args = args
                .into_iter()
                .map(|expr| type_expr(expr, ctx))
                .collect::<Result<_, _>>()?;

            StatementKind::Macro(ident, args)
        }
    };

    let ty = match &kind {
        StatementKind::Expr(expr) => expr.ty.clone(),
        StatementKind::Macro(_, _) => Type::Unit,
    };

    Ok(Statement {
        kind,
        ty,
        span: stmt.span,
    })
}

pub fn type_expr<'src>(
    expr: Spanned<parse_tree::Expr<'src>>,
    ctx: &mut Context<'src>,
) -> Result<Expr<'src>, DummyError> {
    let kind = match expr.inner {
        parse_tree::Expr::Int(i) => ExprKind::Int(i),
        parse_tree::Expr::Float(f) => ExprKind::Float(f),
        parse_tree::Expr::String(s) => ExprKind::String(s),
        parse_tree::Expr::BinOp { op, lhs, rhs } => {
            let lhs = Box::new(type_expr(*lhs, ctx)?);
            let rhs = Box::new(type_expr(*rhs, ctx)?);
            ExprKind::BinOp { op, lhs, rhs }
        }
        parse_tree::Expr::Match { scrutinee, arms } => {
            let scrutinee = Box::new(type_expr(*scrutinee, ctx)?);

            let arms = arms
                .into_iter()
                .map(|(pat, expr)| Ok((pat, type_expr(expr, ctx)?)))
                .collect::<Result<Vec<_>, _>>()?;

            let mut has_discard = false;
            let mut ints_covered = HashMap::with_capacity(arms.len());

            for (pat, _) in &arms {
                match pat.inner {
                    Pattern::Int(i) => {
                        if let Some(first_span) = ints_covered.insert(i, pat.span) {
                            MatchOverlapError {
                                match_span: expr.span,
                                first_span,
                                repeat_span: pat.span,
                            }
                            .report(ctx);
                            return Err(DummyError);
                        }
                    }
                    Pattern::Discard => has_discard = true,
                }
            }

            if !has_discard {
                MatchMissingDiscardError {
                    match_span: expr.span,
                }
                .report(ctx);
                return Err(DummyError);
            }

            ExprKind::Match { scrutinee, arms }
        }
    };

    let ty = match &kind {
        ExprKind::Int(_) => Type::Int,
        ExprKind::Float(_) => Type::Float,
        ExprKind::String(_) => Type::String,
        ExprKind::BinOp { lhs, rhs, .. } => 'block: {
            if lhs.ty != Type::Int && lhs.ty != Type::Float {
                TypeMismatchError {
                    receive_expr: lhs.span,
                    expected_expr: expr.span,
                    expected: vec![Type::Int, Type::Float],
                    received: lhs.ty.clone(),
                }
                .report(ctx);
                break 'block Type::Error;
            }

            if lhs.ty != rhs.ty {
                TypeMismatchError {
                    receive_expr: rhs.span,
                    expected_expr: expr.span,
                    expected: vec![lhs.ty.clone()],
                    received: rhs.ty.clone(),
                }
                .report(ctx);
                break 'block Type::Error;
            };

            Type::Int
        }
        ExprKind::Match { scrutinee, arms } => 'block: {
            if scrutinee.ty != Type::Int {
                TypeMismatchError {
                    received: scrutinee.ty.clone(),
                    receive_expr: scrutinee.span,
                    expected: vec![Type::Int],
                    expected_expr: scrutinee.span,
                }
                .report(ctx);
            }

            let mut arms = arms.iter();
            let first = &arms.next().unwrap().1;
            for (_, arm) in arms {
                if arm.ty != first.ty {
                    TypeMismatchError {
                        received: arm.ty.clone(),
                        receive_expr: arm.span,
                        expected: vec![first.ty.clone()],
                        expected_expr: first.span,
                    }
                    .report(ctx);
                    break 'block Type::Error;
                }
            }
            first.ty.clone()
        }
    };

    Ok(Expr {
        kind,
        ty,
        span: expr.span,
    })
}
