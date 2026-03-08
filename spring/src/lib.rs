use crate::parse_tree::{Expr, Op};

pub mod parse_tree;

impl Expr {
    pub fn eval(self) -> Result<u64, String> {
        Ok(match self {
            Expr::Int(i) => i,
            Expr::BinOp(op, lhs, rhs) => match op {
                Op::Add => lhs.eval()? + rhs.eval()?,
                Op::Sub => lhs.eval()? - rhs.eval()?,
                Op::Mul => lhs.eval()? * rhs.eval()?,
                Op::Div => {
                    let rhs = rhs.eval()?;
                    if rhs == 0 {
                        return Err("div by 0".into());
                    }
                    lhs.eval()? / rhs
                }
            },
        })
    }
}
