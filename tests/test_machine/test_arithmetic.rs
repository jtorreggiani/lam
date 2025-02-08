// tests/test_machine/test_arithmetic.rs

use lam::machine::arithmetic::{Expression, evaluate, parse_expression};
use lam::machine::instruction::Instruction;
use lam::machine::core::Machine;
use lam::term::Term;

#[test]
fn test_arithmetic_is_simple() {
    let code = vec![
        Instruction::ArithmeticIs { 
            target: 0, 
            expression: Expression::Mul(
                Box::new(Expression::Add(
                    Box::new(Expression::Const(3)), 
                    Box::new(Expression::Const(4))
                )),
                Box::new(Expression::Const(2))
            )
        },
    ];
    let mut machine = Machine::new(1, code);
    let _ = machine.run();
    
    // The result should be stored in register 0.
    assert_eq!(machine.registers[0], Some(Term::Const(14)));
}

#[test]
fn test_evaluate_const() {
    let expr = Expression::Const(42);
    assert_eq!(evaluate(&expr, &[]).unwrap(), 42);
}

#[test]
fn test_evaluate_add() {
    let expr = Expression::Add(
        Box::new(Expression::Const(3)), 
        Box::new(Expression::Const(4))
    );
    assert_eq!(evaluate(&expr, &[]).unwrap(), 7);
}

#[test]
fn test_evaluate_complex() {
    // (10 - 2) * (1 + 3) = 32.
    let expr = Expression::Mul(
        Box::new(Expression::Sub(
            Box::new(Expression::Const(10)), 
            Box::new(Expression::Const(2))
        )),
        Box::new(Expression::Add(
            Box::new(Expression::Const(1)), 
            Box::new(Expression::Const(3))
        ))
    );
    assert_eq!(evaluate(&expr, &[]).unwrap(), 32);
}

#[test]
fn test_parse_simple_expression() {
    let input = "3 + 4 * 2";
    let expr = parse_expression(input).expect("Failed to parse expression");
    // 4 * 2 is evaluated first, so the expression is equivalent to 3 + (4 * 2) = 11.
    assert_eq!(evaluate(&expr, &[]).unwrap(), 11);
}

#[test]
fn test_parse_expression_with_parentheses() {
    let input = "(3 + 4) * 2";
    let expr = parse_expression(input).expect("Failed to parse expression with parentheses");
    // Parentheses force addition first: (3 + 4) * 2 = 14.
    assert_eq!(evaluate(&expr, &[]).unwrap(), 14);
}

#[test]
fn test_parse_unary_minus() {
    let input = "-3 + 5";
    let expr = parse_expression(input).expect("Failed to parse unary minus expression");
    // -3 + 5 = 2.
    assert_eq!(evaluate(&expr, &[]).unwrap(), 2);
}
