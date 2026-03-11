pub mod data;

use chumsky::span::Spanned;

use crate::parse_tree;

pub use data::*;

pub fn type_statement<'src>(stmt: Spanned<parse_tree::Statement<'src>>) -> Statement<'src> {
    let kind = match stmt.inner {
        parse_tree::Statement::Expr(expr) => StatementKind::Expr(type_expr(expr)),
        parse_tree::Statement::Macro(ident, args) => {
            let ident = Ident {
                name: ident.inner,
                span: ident.span,
            };

            let args = args.into_iter().map(|expr| type_expr(expr)).collect();

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

pub fn type_expr<'src>(expr: Spanned<parse_tree::Expr<'src>>) -> Expr<'src> {
    let kind = match expr.inner {
        parse_tree::Expr::Int(i) => ExprKind::Int(i),
        parse_tree::Expr::String(s) => ExprKind::String(s),
        parse_tree::Expr::BinOp(op, lhs, rhs) => {
            ExprKind::BinOp(op, Box::new(type_expr(*lhs)), Box::new(type_expr(*rhs)))
        }
    };

    let ty = match &kind {
        ExprKind::Int(_) => Type::Int,
        ExprKind::String(_) => Type::String,
        ExprKind::BinOp(..) => Type::Int,
    };

    Expr {
        kind,
        ty,
        span: expr.span,
    }
}
