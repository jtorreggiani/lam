use lam::machine::{Instruction, Machine};
use lam::term::Term;
use lam::arithmetic::{Expression, evaluate};

// Test for the ArithmeticIs instruction.
// This test simulates evaluating the expression (3 + 4) * 2 and storing the result
// in register 0. The expected result is 14.
#[test]
fn test_arithmetic_is() {
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
    // Pass an empty slice because the expression does not contain any variables.
    assert_eq!(evaluate(&expr, &[]), 42);
}

#[test]
fn test_evaluate_add() {
    let expr = Expression::Add(
        Box::new(Expression::Const(3)), 
        Box::new(Expression::Const(4))
    );
    // No variables here, so we pass an empty slice.
    assert_eq!(evaluate(&expr, &[]), 7);
}

#[test]
fn test_evaluate_complex() {
    // (10 - 2) * (1 + 3) = 8 * 4 = 32.
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
    // Pass an empty slice since the expression has no variable references.
    assert_eq!(evaluate(&expr, &[]), 32);
}
