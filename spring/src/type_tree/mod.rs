pub mod data;
pub mod error;

use chumsky::span::Spanned;

use crate::{parse_tree, type_tree::error::TypeMismatchError};

pub use data::*;

pub struct Context<'src> {
    pub source: &'src str,
    pub error_count: u64,
}

pub fn type_statement<'src>(
    stmt: Spanned<parse_tree::Statement<'src>>,
    ctx: &mut Context<'src>,
) -> Statement<'src> {
    let kind = match stmt.inner {
        parse_tree::Statement::Expr(expr) => StatementKind::Expr(type_expr(expr, ctx)),
        parse_tree::Statement::Macro(ident, args) => {
            let ident = Ident {
                name: ident.inner,
                span: ident.span,
            };

            let args = args.into_iter().map(|expr| type_expr(expr, ctx)).collect();

            StatementKind::Macro(ident, args)
        }
    };

    let ty = match &kind {
        StatementKind::Expr(expr) => expr.ty.clone(),
        StatementKind::Macro(_, _) => Type::Unit,
    };

    Statement {
        kind,
        ty,
        span: stmt.span,
    }
}

pub fn type_expr<'src>(
    expr: Spanned<parse_tree::Expr<'src>>,
    ctx: &mut Context<'src>,
) -> Expr<'src> {
    let kind = match expr.inner {
        parse_tree::Expr::Int(i) => ExprKind::Int(i),
        parse_tree::Expr::Float(f) => ExprKind::Float(f),
        parse_tree::Expr::String(s) => ExprKind::String(s),
        parse_tree::Expr::BinOp(op, lhs, rhs) => {
            let lhs = type_expr(*lhs, ctx);
            let rhs = type_expr(*rhs, ctx);
            ExprKind::BinOp(op, Box::new(lhs), Box::new(rhs))
        }
    };

    let ty = match &kind {
        ExprKind::Int(_) => Type::Int,
        ExprKind::Float(_) => Type::Float,
        ExprKind::String(_) => Type::String,
        ExprKind::BinOp(_, lhs, rhs) => 'block: {
            let Type::Int = lhs.ty else {
                TypeMismatchError {
                    produce_expr: lhs.span,
                    consume_expr: expr.span,
                    expected: Type::Int,
                    received: lhs.ty.clone(),
                }
                .report(ctx);
                break 'block Type::Error;
            };

            let Type::Int = rhs.ty else {
                TypeMismatchError {
                    produce_expr: lhs.span,
                    consume_expr: expr.span,
                    expected: Type::Int,
                    received: rhs.ty.clone(),
                }
                .report(ctx);
                break 'block Type::Error;
            };

            Type::Int
        }
    };

    Expr {
        kind,
        ty,
        span: expr.span,
    }
}
