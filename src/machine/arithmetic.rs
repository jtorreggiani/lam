use crate::term::Term;
use crate::error_handling::MachineError;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Const(i32),
    Var(usize),
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

pub fn parse_expression(expr: &str) -> Result<Expression, String> {
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    if tokens.len() == 1 {
        let n = tokens[0].parse::<i32>().map_err(|_| format!("Invalid constant: {}", tokens[0]))?;
        Ok(Expression::Const(n))
    } else if tokens.len() == 3 {
        let left = tokens[0].parse::<i32>().map_err(|_| format!("Invalid constant: {}", tokens[0]))?;
        let right = tokens[2].parse::<i32>().map_err(|_| format!("Invalid constant: {}", tokens[2]))?;
        match tokens[1] {
            "+" => Ok(Expression::Add(Box::new(Expression::Const(left)), Box::new(Expression::Const(right)))),
            "-" => Ok(Expression::Sub(Box::new(Expression::Const(left)), Box::new(Expression::Const(right)))),
            "*" => Ok(Expression::Mul(Box::new(Expression::Const(left)), Box::new(Expression::Const(right)))),
            "/" => Ok(Expression::Div(Box::new(Expression::Const(left)), Box::new(Expression::Const(right)))),
            op => Err(format!("Unsupported operator: {}", op)),
        }
    } else {
        Err("Expression format not recognized".to_string())
    }
}
