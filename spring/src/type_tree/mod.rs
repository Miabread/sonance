pub mod data;
pub mod error;
pub mod scope;

use std::collections::HashMap;

use chumsky::span::{SpanWrap, Spanned};

use crate::{
    DummyError,
    parse_tree::{self},
    type_tree::{error::TypeError, scope::Scope},
};

pub use data::*;

#[derive(Debug, Clone)]
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
    pub fn unify(&self, a: Type, b: Type) -> Result<Type, DummyError> {
        match a.kind {
            TypeKind::Error
            | TypeKind::Unit
            | TypeKind::Int
            | TypeKind::Float
            | TypeKind::String => {
                todo!()
            }
            TypeKind::Func { args, return_type } => todo!(),
        }
    }

    pub fn type_module(
        &mut self,
        module: Spanned<parse_tree::Module<'src>>,
    ) -> Result<Module<'src>, DummyError> {
        let mut scope = Scope::new();

        for item in &module.items {
            match &item.inner {
                parse_tree::Item::Func(func_item) => {
                    scope.set(func_item.name.0, self.type_func_signature(func_item)?);
                }
            }
        }

        let items = module
            .inner
            .items
            .into_iter()
            .map(|item| self.type_item(item))
            .collect::<Result<_, _>>()?;

        Ok(Module {
            items,
            span: module.span,
        })
    }

    pub fn type_item(
        &mut self,
        item: Spanned<parse_tree::Item<'src>>,
    ) -> Result<Item<'src>, DummyError> {
        Ok(match item.inner {
            parse_tree::Item::Func(func) => Item {
                kind: ItemKind::Func(self.type_func_item(func)?),
                span: item.span,
            },
        })
    }

    pub fn type_func_signature(
        &mut self,
        func: &parse_tree::FuncItem<'src>,
    ) -> Result<Type, DummyError> {
        Ok(Type {
            kind: TypeKind::Func {
                args: func
                    .args
                    .iter()
                    .map(|arg| self.type_type(arg.ty.clone()))
                    .collect::<Result<_, _>>()?,
                return_type: Box::new(self.type_type(func.return_type.clone())?),
            },
        })
    }

    pub fn type_func_item(
        &mut self,
        func: parse_tree::FuncItem<'src>,
    ) -> Result<FuncItem<'src>, DummyError> {
        let mut scope = Scope::new();
        for arg in func.args.inner {
            scope.set(arg.name.0, self.type_type(arg.ty)?);
        }
        Ok(FuncItem {
            name: self.type_ident(func.name),
            body: self.type_block(func.body, &scope)?,
        })
    }

    pub fn type_type(&mut self, ty: Spanned<parse_tree::Type>) -> Result<Type, DummyError> {
        Ok(Type {
            kind: match ty.inner {
                parse_tree::Type::Unit => TypeKind::Unit,
                parse_tree::Type::Int => TypeKind::Int,
                parse_tree::Type::Float => TypeKind::Float,
                parse_tree::Type::String => TypeKind::String,
            },
        })
    }

    pub fn type_block(
        &mut self,
        block: Spanned<parse_tree::Block<'src>>,
        scope: &Scope,
    ) -> Result<Block<'src>, DummyError> {
        let body = block
            .inner
            .body
            .into_iter()
            .map(|stmt| self.type_statement(stmt, scope))
            .collect::<Result<_, _>>()?;

        let trailing = if let Some(trailing) = block.inner.trailing.inner {
            self.type_expr(trailing, scope)?
        } else {
            Expr {
                kind: ExprKind::Unit,
                ty: Type {
                    kind: TypeKind::Unit,
                },
                span: block.inner.trailing.span,
            }
        };

        Ok(Block {
            body,
            trailing,
            span: block.span,
        })
    }

    pub fn type_ident(&mut self, ident: Spanned<parse_tree::Ident<'src>>) -> Ident<'src> {
        Ident {
            name: ident.inner.0,
            span: ident.span,
        }
    }

    pub fn type_statement(
        &mut self,
        stmt: Spanned<parse_tree::Statement<'src>>,
        scope: &Scope,
    ) -> Result<Statement<'src>, DummyError> {
        let kind = match stmt.inner {
            parse_tree::Statement::Expr(expr) => StatementKind::Expr(self.type_expr(expr, scope)?),
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
        scope: &Scope,
    ) -> Result<Expr<'src>, DummyError> {
        Ok(match expr.inner {
            parse_tree::Expr::Int(i) => Expr {
                kind: ExprKind::Int(i),
                ty: Type {
                    kind: TypeKind::Int,
                },
                span: expr.span,
            },
            parse_tree::Expr::Float(f) => Expr {
                kind: ExprKind::Float(f),
                ty: Type {
                    kind: TypeKind::Float,
                },
                span: expr.span,
            },
            parse_tree::Expr::String(s) => Expr {
                kind: ExprKind::String(s),
                ty: Type {
                    kind: TypeKind::String,
                },
                span: expr.span,
            },
            parse_tree::Expr::Var(ident) => {
                let ident = self.type_ident(ident.with_span(expr.span));
                let ty = scope.get(&ident.name).cloned().unwrap_or_else(|| {
                    TypeError::UnknownVariableError {
                        ident_span: ident.span,
                    }
                    .report(self);
                    Type {
                        kind: TypeKind::Error,
                    }
                });

                Expr {
                    kind: ExprKind::Var(ident),
                    ty,
                    span: expr.span,
                }
            }
            parse_tree::Expr::BinOp(parse_tree::BinOpExpr { op, lhs, rhs }) => {
                let lhs = Box::new(self.type_expr(*lhs, scope)?);
                let rhs = Box::new(self.type_expr(*rhs, scope)?);

                let ty = if lhs.ty.kind != TypeKind::Int && lhs.ty.kind != TypeKind::Float {
                    TypeError::TypeMismatchError {
                        receive_expr: lhs.span,
                        expected_expr: expr.span,
                        expected: vec![TypeKind::Int, TypeKind::Float],
                        received: lhs.ty.kind.clone(),
                    }
                    .report(self);
                    TypeKind::Error
                } else if lhs.ty != rhs.ty {
                    TypeError::TypeMismatchError {
                        receive_expr: rhs.span,
                        expected_expr: expr.span,
                        expected: vec![lhs.ty.kind.clone()],
                        received: rhs.ty.kind.clone(),
                    }
                    .report(self);
                    TypeKind::Error
                } else {
                    TypeKind::Int
                };

                Expr {
                    kind: ExprKind::BinOp(BinOpExpr { op, lhs, rhs }),
                    ty: Type { kind: ty },
                    span: expr.span,
                }
            }
            parse_tree::Expr::Match(parse_tree::MatchExpr { scrutinee, arms }) => {
                let scrutinee = Box::new(self.type_expr(*scrutinee, scope)?);

                let arms = arms
                    .into_iter()
                    .map(|(pat, block)| Ok((pat, self.type_block(block, scope)?)))
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

                if scrutinee.ty.kind != TypeKind::Int {
                    TypeError::TypeMismatchError {
                        received: scrutinee.ty.kind.clone(),
                        receive_expr: scrutinee.span,
                        expected: vec![TypeKind::Int],
                        expected_expr: scrutinee.span,
                    }
                    .report(self);
                }

                let mut arms_iter = arms.iter();
                let first = &arms_iter.next().unwrap().1.trailing;
                let ty = 'block: {
                    for (_, block) in arms_iter {
                        let expr = &block.trailing;
                        if expr.ty != first.ty {
                            TypeError::TypeMismatchError {
                                received: expr.ty.kind.clone(),
                                receive_expr: expr.span,
                                expected: vec![first.ty.kind.clone()],
                                expected_expr: first.span,
                            }
                            .report(self);
                            break 'block Type {
                                kind: TypeKind::Error,
                            };
                        }
                    }
                    first.ty.clone()
                };

                Expr {
                    kind: ExprKind::Match(MatchExpr { scrutinee, arms }),
                    ty,
                    span: expr.span,
                }
            }

            parse_tree::Expr::FuncCall(parse_tree::FuncCallExpr { name, args }) => {
                todo!()
            }

            parse_tree::Expr::MacroCall(parse_tree::MacroCallExpr { name, args }) => {
                let name = self.type_ident(name);

                let args = args
                    .into_iter()
                    .map(|expr| self.type_expr(expr, scope))
                    .collect::<Result<_, _>>()?;

                Expr {
                    kind: ExprKind::MacroCall(MacroCallExpr { name, args }),
                    ty: Type {
                        kind: TypeKind::Unit,
                    },
                    span: expr.span,
                }
            }
        })
    }
}
