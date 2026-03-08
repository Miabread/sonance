use std::fmt::Display;

use crate::parse_tree::{Expr, Op, Statement};

pub mod parse_tree;

impl<'src> Statement<'src> {
    pub fn eval(self) -> Result<Value<'src>, String> {
        match self {
            Statement::Expr(expr) => expr.eval(),
            Statement::Macro(name, exprs) => match name {
                "print" => {
                    for expr in exprs {
                        print!("{}", expr.eval()?);
                    }
                    println!();
                    Ok(Value::Unit)
                }
                _ => panic!("unknown macro {name}"),
            },
        }
    }
}

impl<'src> Expr<'src> {
    pub fn eval(self) -> Result<Value<'src>, String> {
        Ok(match self {
            Expr::Int(i) => Value::Int(i),
            Expr::String(s) => Value::String(s),
            Expr::BinOp(op, lhs, rhs) => {
                let Value::Int(lhs) = lhs.eval()? else {
                    return Err("lhs not int".into());
                };
                let Value::Int(rhs) = rhs.eval()? else {
                    return Err("rhs not int".into());
                };

                Value::Int(match op {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Mul => lhs * rhs,
                    Op::Div => {
                        if rhs == 0 {
                            return Err("div by 0".into());
                        }
                        lhs / rhs
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
