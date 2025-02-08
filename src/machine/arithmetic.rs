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
    /// Addition.
    Add(Box<Expression>, Box<Expression>),
    /// Subtraction.
    Sub(Box<Expression>, Box<Expression>),
    /// Multiplication.
    Mul(Box<Expression>, Box<Expression>),
    /// Division.
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
        Expression::Div(e1, e2) => {
            let denominator = evaluate(e2, registers)?;
            if denominator == 0 {
                return Err(MachineError::UnificationFailed("Division by zero".to_string()));
            }
            Ok(evaluate(e1, registers)? / denominator)
        },
    }
}

/// Parses a string expression into an Expression.
/// The parser supports numbers, parentheses, and the operators +, -, *, and / with standard precedence.
/// Grammar:
///   Expr   → Term { ("+" | "-") Term }
///   Term   → Factor { ("*" | "/") Factor }
///   Factor → number | "(" Expr ")" | "-" Factor
pub fn parse_expression(input: &str) -> Result<Expression, String> {
    let tokens = tokenize(input)?;
    let (expr, pos) = parse_expr(&tokens, 0)?;
    if pos != tokens.len() {
        return Err(format!("Unexpected token at position {}", pos));
    }
    Ok(expr)
}

/// Token types for arithmetic parsing.
#[derive(Debug, PartialEq)]
enum Token {
    Number(i32),
    Plus,
    Minus,
    Mul,
    Div,
    LParen,
    RParen,
}

/// Tokenizes the input arithmetic expression.
fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' => { chars.next(); },
            '+' => { tokens.push(Token::Plus); chars.next(); },
            '-' => { tokens.push(Token::Minus); chars.next(); },
            '*' => { tokens.push(Token::Mul); chars.next(); },
            '/' => { tokens.push(Token::Div); chars.next(); },
            '(' => { tokens.push(Token::LParen); chars.next(); },
            ')' => { tokens.push(Token::RParen); chars.next(); },
            '0'..='9' => {
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_digit(10) {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let num = num_str.parse::<i32>().map_err(|e| format!("Invalid number: {}", e))?;
                tokens.push(Token::Number(num));
            },
            _ => return Err(format!("Unexpected character: {}", ch)),
        }
    }
    Ok(tokens)
}

/// Parses an expression: Expr → Term { ("+" | "-") Term }
fn parse_expr(tokens: &[Token], pos: usize) -> Result<(Expression, usize), String> {
    let (mut expr, mut pos) = parse_term(tokens, pos)?;
    while pos < tokens.len() {
        match tokens[pos] {
            Token::Plus => {
                pos += 1;
                let (right, new_pos) = parse_term(tokens, pos)?;
                expr = Expression::Add(Box::new(expr), Box::new(right));
                pos = new_pos;
            },
            Token::Minus => {
                pos += 1;
                let (right, new_pos) = parse_term(tokens, pos)?;
                expr = Expression::Sub(Box::new(expr), Box::new(right));
                pos = new_pos;
            },
            _ => break,
        }
    }
    Ok((expr, pos))
}

/// Parses a term: Term → Factor { ("*" | "/") Factor }
fn parse_term(tokens: &[Token], pos: usize) -> Result<(Expression, usize), String> {
    let (mut term, mut pos) = parse_factor(tokens, pos)?;
    while pos < tokens.len() {
        match tokens[pos] {
            Token::Mul => {
                pos += 1;
                let (right, new_pos) = parse_factor(tokens, pos)?;
                term = Expression::Mul(Box::new(term), Box::new(right));
                pos = new_pos;
            },
            Token::Div => {
                pos += 1;
                let (right, new_pos) = parse_factor(tokens, pos)?;
                term = Expression::Div(Box::new(term), Box::new(right));
                pos = new_pos;
            },
            _ => break,
        }
    }
    Ok((term, pos))
}

/// Parses a factor: Factor → number | "(" Expr ")" | "-" Factor
fn parse_factor(tokens: &[Token], pos: usize) -> Result<(Expression, usize), String> {
    if pos >= tokens.len() {
        return Err("Unexpected end of input".to_string());
    }
    match tokens[pos] {
        Token::Number(n) => Ok((Expression::Const(n), pos + 1)),
        Token::LParen => {
            let (expr, new_pos) = parse_expr(tokens, pos + 1)?;
            if new_pos >= tokens.len() || tokens[new_pos] != Token::RParen {
                return Err("Missing closing parenthesis".to_string());
            }
            Ok((expr, new_pos + 1))
        },
        Token::Minus => {
            // Handle unary minus as 0 - Factor.
            let (factor, new_pos) = parse_factor(tokens, pos + 1)?;
            Ok((Expression::Sub(Box::new(Expression::Const(0)), Box::new(factor)), new_pos))
        },
        _ => Err(format!("Unexpected token at position {}", pos)),
    }
}
