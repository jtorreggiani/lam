use crate::term::Term;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Const(i32),
    Var(usize), // new: allows us to refer to a register by its index
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
}

pub fn evaluate(expr: &Expression, registers: &[Option<Term>]) -> i32 {
  match expr {
      Expression::Const(n) => *n,
      Expression::Var(idx) => {
          // In this example we assume that the register stores a constant.
          // In a robust implementation you’d need to resolve variables properly.
          if let Some(Term::Const(val)) = registers.get(*idx).and_then(|opt| opt.as_ref()) {
              *val
          } else {
              panic!("Register {} does not contain a constant", idx)
          }
      },
      Expression::Add(e1, e2) => evaluate(e1, registers) + evaluate(e2, registers),
      Expression::Sub(e1, e2) => evaluate(e1, registers) - evaluate(e2, registers),
      Expression::Mul(e1, e2) => evaluate(e1, registers) * evaluate(e2, registers),
      Expression::Div(e1, e2) => evaluate(e1, registers) / evaluate(e2, registers),
  }
}
