use lam::machine::{Instruction, Machine};
use lam::term::Term;
use lam::arithmetic::{Expr, evaluate};

// Test for the ArithmeticIs instruction.
//
// This test simulates evaluating the expression (3 + 4) * 2 and storing the result
// in register 0. The expected result is 14.
#[test]
fn test_arithmetic_is() {
    let code = vec![
        // Evaluate the expression (3 + 4) * 2.
        Instruction::ArithmeticIs { 
            target: 0, 
            expression: Expr::Mul(
                Box::new(Expr::Add(Box::new(Expr::Const(3)), Box::new(Expr::Const(4)))),
                Box::new(Expr::Const(2))
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
    let expr = Expr::Const(42);
    assert_eq!(evaluate(&expr), 42);
}

#[test]
fn test_evaluate_add() {
    let expr = Expr::Add(Box::new(Expr::Const(3)), Box::new(Expr::Const(4)));
    assert_eq!(evaluate(&expr), 7);
}

#[test]
fn test_evaluate_complex() {
    // (10 - 2) * (1 + 3) = 8 * 4 = 32.
    let expr = Expr::Mul(
        Box::new(Expr::Sub(Box::new(Expr::Const(10)), Box::new(Expr::Const(2)))),
        Box::new(Expr::Add(Box::new(Expr::Const(1)), Box::new(Expr::Const(3))))
    );
    assert_eq!(evaluate(&expr), 32);
}
