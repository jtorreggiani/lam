use crate::term::Term;
use crate::machine::MachineError;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Const(i32),
    Var(usize), // new: allows us to refer to a register by its index
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
}

pub fn evaluate(expr: &Expression, registers: &[Option<Term>]) -> Result<i32, MachineError> {
    match expr {
        Expression::Const(n) => Ok(*n),
        Expression::Var(idx) => {
            if let Some(Term::Const(val)) = registers.get(*idx).and_then(|opt| opt.as_ref()) {
                Ok(*val)
            } else {
                Err(MachineError::UninitializedRegister(*idx))
            }
        },
        Expression::Add(e1, e2) => Ok(evaluate(e1, registers)? + evaluate(e2, registers)?),
        Expression::Sub(e1, e2) => Ok(evaluate(e1, registers)? - evaluate(e2, registers)?),
        Expression::Mul(e1, e2) => Ok(evaluate(e1, registers)? * evaluate(e2, registers)?),
        Expression::Div(e1, e2) => Ok(evaluate(e1, registers)? / evaluate(e2, registers)?),
    }
}
