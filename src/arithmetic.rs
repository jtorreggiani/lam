#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Const(i32),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
}

pub fn evaluate(expr: &Expression) -> i32 {
    match expr {
        Expression::Const(n) => *n,
        Expression::Add(e1, e2) => evaluate(e1) + evaluate(e2),
        Expression::Sub(e1, e2) => evaluate(e1) - evaluate(e2),
        Expression::Mul(e1, e2) => evaluate(e1) * evaluate(e2),
        Expression::Div(e1, e2) => evaluate(e1) / evaluate(e2), // note: division by zero not checked
    }
}
