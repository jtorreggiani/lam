// src/machine/arithmetic.rs
//! Arithmetic evaluation and expression parsing for the LAM.

use crate::term::Term;
use crate::error_handling::MachineError;

/// Represents an arithmetic expression.
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    /// Constant integer.
    Const(i32),
    /// Variable (by register index).
    Var(usize),
    /// Addition of two expressions.
    Add(Box<Expression>, Box<Expression>),
    /// Subtraction of two expressions.
    Sub(Box<Expression>, Box<Expression>),
    /// Multiplication of two expressions.
    Mul(Box<Expression>, Box<Expression>),
    /// Division of two expressions.
    Div(Box<Expression>, Box<Expression>),
}

/// Evaluates an arithmetic expression using the given registers.
/// Returns the computed integer or a MachineError.
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

/// Parses a string expression into an Expression.
/// Supports a single constant or a simple binary operation.
pub fn parse_expression(expr: &str) -> Result<Expression, String> {
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    if tokens.len() == 1 {
        let n = tokens[0]
            .parse::<i32>()
            .map_err(|_| format!("Invalid constant: {}", tokens[0]))?;
        Ok(Expression::Const(n))
    } else if tokens.len() == 3 {
        let left = tokens[0]
            .parse::<i32>()
            .map_err(|_| format!("Invalid constant: {}", tokens[0]))?;
        let right = tokens[2]
            .parse::<i32>()
            .map_err(|_| format!("Invalid constant: {}", tokens[2]))?;
        match tokens[1] {
            "+" => Ok(Expression::Add(
                Box::new(Expression::Const(left)),
                Box::new(Expression::Const(right)),
            )),
            "-" => Ok(Expression::Sub(
                Box::new(Expression::Const(left)),
                Box::new(Expression::Const(right)),
            )),
            "*" => Ok(Expression::Mul(
                Box::new(Expression::Const(left)),
                Box::new(Expression::Const(right)),
            )),
            "/" => Ok(Expression::Div(
                Box::new(Expression::Const(left)),
                Box::new(Expression::Const(right)),
            )),
            op => Err(format!("Unsupported operator: {}", op)),
        }
    } else {
        Err("Expression format not recognized".to_string())
    }
}
