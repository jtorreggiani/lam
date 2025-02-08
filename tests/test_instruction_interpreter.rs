use lam::machine::instruction_parser::parse_instructions;
use lam::machine::instruction::Instruction;
use lam::machine::term::Term;
use lam::machine::arithmetic::Expression;

#[test]
fn test_put_const() {
    let input = "PUT_CONST R0, 42";
    let instructions = parse_instructions(input).expect("Failed to parse PUT_CONST");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::PutConst { register, value } => {
            assert_eq!(*register, 0);
            assert_eq!(*value, 42);
        }
        _ => panic!("Expected PutConst instruction"),
    }
}

#[test]
fn test_put_var() {
    let input = "PUT_VAR R1, 0, \"X\"";
    let instructions = parse_instructions(input).expect("Failed to parse PUT_VAR");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::PutVar { register, var_id, name } => {
            assert_eq!(*register, 1);
            assert_eq!(*var_id, 0);
            assert_eq!(name, "X");
        }
        _ => panic!("Expected PutVar instruction"),
    }
}

#[test]
fn test_get_const() {
    let input = "GET_CONST R0, 42";
    let instructions = parse_instructions(input).expect("Failed to parse GET_CONST");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::GetConst { register, value } => {
            assert_eq!(*register, 0);
            assert_eq!(*value, 42);
        }
        _ => panic!("Expected GetConst instruction"),
    }
}

#[test]
fn test_get_var() {
    let input = "GET_VAR R0, 0, \"X\"";
    let instructions = parse_instructions(input).expect("Failed to parse GET_VAR");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::GetVar { register, var_id, name } => {
            assert_eq!(*register, 0);
            assert_eq!(*var_id, 0);
            assert_eq!(name, "X");
        }
        _ => panic!("Expected GetVar instruction"),
    }
}

#[test]
fn test_call() {
    let input = "CALL \"halt\"";
    let instructions = parse_instructions(input).expect("Failed to parse CALL");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::Call { predicate } => {
            assert_eq!(predicate, "halt");
        }
        _ => panic!("Expected Call instruction"),
    }
}

#[test]
fn test_proceed() {
    let input = "PROCEED";
    let instructions = parse_instructions(input).expect("Failed to parse PROCEED");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::Proceed => {},
        _ => panic!("Expected Proceed instruction"),
    }
}

#[test]
fn test_choice() {
    let input = "CHOICE 123";
    let instructions = parse_instructions(input).expect("Failed to parse CHOICE");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::Choice { alternative } => {
            assert_eq!(*alternative, 123);
        }
        _ => panic!("Expected Choice instruction"),
    }
}

#[test]
fn test_allocate_and_deallocate() {
    let input = "ALLOCATE 3\nDEALLOCATE";
    let instructions = parse_instructions(input).expect("Failed to parse ALLOCATE/DEALLOCATE");
    assert_eq!(instructions.len(), 2);
    match &instructions[0] {
        Instruction::Allocate { n } => assert_eq!(*n, 3),
        _ => panic!("Expected Allocate instruction"),
    }
    match &instructions[1] {
        Instruction::Deallocate => {},
        _ => panic!("Expected Deallocate instruction"),
    }
}

#[test]
fn test_arithmetic_is() {
    let input = "ARITHMETIC_IS R0, 3+4*2";
    let instructions = parse_instructions(input).expect("Failed to parse ARITHMETIC_IS");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::ArithmeticIs { target, expression } => {
            assert_eq!(*target, 0);
            // We expect an addition at the top level since 3+4*2 is parsed as 3+(4*2)
            match expression {
                Expression::Add(_, _) => {},
                _ => panic!("Expected an addition expression in ARITHMETIC_IS"),
            }
        }
        _ => panic!("Expected ArithmeticIs instruction"),
    }
}

#[test]
fn test_set_local() {
    let input = "SET_LOCAL 0, 42";
    let instructions = parse_instructions(input).expect("Failed to parse SET_LOCAL");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::SetLocal { index, value } => {
            assert_eq!(*index, 0);
            assert_eq!(value, &Term::Const(42));
        }
        _ => panic!("Expected SetLocal instruction"),
    }
}

#[test]
fn test_get_local() {
    let input = "GET_LOCAL 0, R1";
    let instructions = parse_instructions(input).expect("Failed to parse GET_LOCAL");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::GetLocal { index, register } => {
            assert_eq!(*index, 0);
            assert_eq!(*register, 1);
        }
        _ => panic!("Expected GetLocal instruction"),
    }
}

#[test]
fn test_fail() {
    let input = "FAIL";
    let instructions = parse_instructions(input).expect("Failed to parse FAIL");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::Fail => {},
        _ => panic!("Expected Fail instruction"),
    }
}

#[test]
fn test_get_structure() {
    let input = "GET_STRUCTURE R0, \"f\", 2";
    let instructions = parse_instructions(input).expect("Failed to parse GET_STRUCTURE");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::GetStructure { register, functor, arity } => {
            assert_eq!(*register, 0);
            assert_eq!(functor, "f");
            assert_eq!(*arity, 2);
        }
        _ => panic!("Expected GetStructure instruction"),
    }
}

#[test]
fn test_indexed_call() {
    let input = "INDEXED_CALL \"p\", R0";
    let instructions = parse_instructions(input).expect("Failed to parse INDEXED_CALL");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::IndexedCall { predicate, index_register } => {
            assert_eq!(predicate, "p");
            assert_eq!(*index_register, 0);
        }
        _ => panic!("Expected IndexedCall instruction"),
    }
}

#[test]
fn test_multi_indexed_call() {
    let input = "MULTI_INDEXED_CALL \"p\", R1, R2";
    let instructions = parse_instructions(input).expect("Failed to parse MULTI_INDEXED_CALL");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::MultiIndexedCall { predicate, index_registers } => {
            assert_eq!(predicate, "p");
            assert_eq!(index_registers, &vec![1, 2]);
        }
        _ => panic!("Expected MultiIndexedCall instruction"),
    }
}

#[test]
fn test_tail_call() {
    let input = "TAIL_CALL \"dummy\"";
    let instructions = parse_instructions(input).expect("Failed to parse TAIL_CALL");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::TailCall { predicate } => {
            assert_eq!(predicate, "dummy");
        }
        _ => panic!("Expected TailCall instruction"),
    }
}

#[test]
fn test_assert_clause() {
    let input = "ASSERT_CLAUSE \"p\", 123";
    let instructions = parse_instructions(input).expect("Failed to parse ASSERT_CLAUSE");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::AssertClause { predicate, address } => {
            assert_eq!(predicate, "p");
            assert_eq!(*address, 123);
        }
        _ => panic!("Expected AssertClause instruction"),
    }
}

#[test]
fn test_retract_clause() {
    let input = "RETRACT_CLAUSE \"p\", 456";
    let instructions = parse_instructions(input).expect("Failed to parse RETRACT_CLAUSE");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::RetractClause { predicate, address } => {
            assert_eq!(predicate, "p");
            assert_eq!(*address, 456);
        }
        _ => panic!("Expected RetractClause instruction"),
    }
}

#[test]
fn test_cut() {
    let input = "CUT";
    let instructions = parse_instructions(input).expect("Failed to parse CUT");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::Cut => {},
        _ => panic!("Expected Cut instruction"),
    }
}

#[test]
fn test_build_compound() {
    let input = "BUILD_COMPOUND R2, \"f\", R0, R1";
    let instructions = parse_instructions(input).expect("Failed to parse BUILD_COMPOUND");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::BuildCompound { target, functor, arg_registers } => {
            assert_eq!(*target, 2);
            assert_eq!(functor, "f");
            assert_eq!(arg_registers, &vec![0, 1]);
        }
        _ => panic!("Expected BuildCompound instruction"),
    }
}

#[test]
fn test_put_str() {
    let input = "PUT_STR R0, \"hello\"";
    let instructions = parse_instructions(input).expect("Failed to parse PUT_STR");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::PutStr { register, value } => {
            assert_eq!(*register, 0);
            assert_eq!(value, "hello");
        }
        _ => panic!("Expected PutStr instruction"),
    }
}

#[test]
fn test_get_str() {
    let input = "GET_STR R0, \"world\"";
    let instructions = parse_instructions(input).expect("Failed to parse GET_STR");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::GetStr { register, value } => {
            assert_eq!(*register, 0);
            assert_eq!(value, "world");
        }
        _ => panic!("Expected GetStr instruction"),
    }
}

#[test]
fn test_move() {
    let input = "MOVE R1, R2";
    let instructions = parse_instructions(input).expect("Failed to parse MOVE");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::Move { src, dst } => {
            assert_eq!(*src, 1);
            assert_eq!(*dst, 2);
        }
        _ => panic!("Expected Move instruction"),
    }
}

#[test]
fn test_halt() {
    let input = "HALT";
    let instructions = parse_instructions(input).expect("Failed to parse HALT");
    assert_eq!(instructions.len(), 1);
    match &instructions[0] {
        Instruction::Halt => {},
        _ => panic!("Expected Halt instruction"),
    }
}

#[test]
fn test_comments_and_empty_lines() {
    let input = r#"
        ; This is a comment line
        # Another comment
        PUT_CONST R0, 100
        
        HALT
    "#;
    let instructions = parse_instructions(input).expect("Failed to parse input with comments");
    assert_eq!(instructions.len(), 2);
    match &instructions[0] {
        Instruction::PutConst { register, value } => {
            assert_eq!(*register, 0);
            assert_eq!(*value, 100);
        }
        _ => panic!("Expected PutConst instruction"),
    }
    match &instructions[1] {
        Instruction::Halt => {},
        _ => panic!("Expected Halt instruction"),
    }
}
