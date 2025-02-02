#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Const(i32),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

// Recursively evaluates an arithmetic expression.
pub fn evaluate(expr: &Expr) -> i32 {
  match expr {
      Expr::Const(n) => *n,
      Expr::Add(e1, e2) => evaluate(e1) + evaluate(e2),
      Expr::Sub(e1, e2) => evaluate(e1) - evaluate(e2),
      Expr::Mul(e1, e2) => evaluate(e1) * evaluate(e2),
      // Note: does not check for division by zero.
      Expr::Div(e1, e2) => evaluate(e1) / evaluate(e2),
  }
}
