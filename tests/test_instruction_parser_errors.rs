use lam::machine::instruction_parser::parse_instructions;
use lam::machine::instruction::Instruction;

#[test]
fn test_empty_input() {
    let input = "";
    let instructions = parse_instructions(input).expect("Empty input should return empty vector");
    assert!(instructions.is_empty());
}

#[test]
fn test_whitespace_input() {
    let input = "    \n  \t\n";
    let instructions = parse_instructions(input).expect("Whitespace input should return empty vector");
    assert!(instructions.is_empty());
}

#[test]
fn test_only_comments() {
    let input = "; this is a comment\n# another comment\n   ; one more comment";
    let instructions = parse_instructions(input).expect("Input with only comments should return empty vector");
    assert!(instructions.is_empty());
}

#[test]
fn test_invalid_mnemonic() {
    let input = "FOO R0, 42";
    let result = parse_instructions(input);
    assert!(result.is_err(), "An unknown mnemonic should yield an error");
    let err = result.unwrap_err();
    assert!(
        err.to_lowercase().contains("unknown"),
        "Error message should mention the unknown mnemonic, got: {}",
        err
    );
}

#[test]
fn test_missing_parameter_put_const() {
    let input = "PUT_CONST R0";
    let result = parse_instructions(input);
    assert!(result.is_err(), "Missing parameter should yield an error");
    let err = result.unwrap_err();
    assert!(
        err.contains("expects 2 parameters"),
        "Expected error about missing parameters, got: {}",
        err
    );
}

#[test]
fn test_extra_parameter_proceed() {
    let input = "PROCEED extra";
    let result = parse_instructions(input);
    assert!(result.is_err(), "Extra parameter for PROCEED should yield an error");
    let err = result.unwrap_err();
    assert!(
        err.contains("expects no parameters"),
        "Error message should mention no parameters allowed, got: {}",
        err
    );
}

#[test]
fn test_invalid_register_token() {
    let input = "PUT_CONST X0, 42";
    let result = parse_instructions(input);
    assert!(result.is_err(), "Invalid register token should yield an error");
    let err = result.unwrap_err();
    assert!(
        err.contains("must start with 'R' or 'r'"),
        "Error message should mention invalid register token, got: {}",
        err
    );
}

#[test]
fn test_invalid_integer_in_put_const() {
    let input = "PUT_CONST R0, abc";
    let result = parse_instructions(input);
    assert!(result.is_err(), "Non-numeric integer parameter should yield an error");
    let err = result.unwrap_err();
    assert!(
        err.contains("failed to parse integer"),
        "Error message should mention failed parsing of integer, got: {}",
        err
    );
}

#[test]
fn test_invalid_arithmetic_expression() {
    let input = "ARITHMETIC_IS R0, 3+*4";
    let result = parse_instructions(input);
    assert!(result.is_err(), "Malformed arithmetic expression should yield an error");
    let err = result.unwrap_err();
    assert!(
        err.contains("failed to parse expression"),
        "Error message should mention failure to parse arithmetic expression, got: {}",
        err
    );
}

#[test]
fn test_set_local_invalid_index() {
    let input = "SET_LOCAL not_a_number, 42";
    let result = parse_instructions(input);
    assert!(result.is_err(), "Non-numeric index in SET_LOCAL should yield an error");
    let err = result.unwrap_err();
    assert!(
        err.contains("failed to parse index"),
        "Error message should mention failed parsing of index, got: {}",
        err
    );
}

#[test]
fn test_get_local_invalid_register() {
    let input = "GET_LOCAL 0, X";
    let result = parse_instructions(input);
    assert!(result.is_err(), "Invalid register token in GET_LOCAL should yield an error");
    let err = result.unwrap_err();
    assert!(
        err.contains("must start with 'R' or 'r'"),
        "Error message should mention invalid register token, got: {}",
        err
    );
}

#[test]
fn test_move_invalid_register() {
    let input = "MOVE R1, A2";
    let result = parse_instructions(input);
    assert!(result.is_err(), "Invalid register token in MOVE should yield an error");
    let err = result.unwrap_err();
    assert!(
        err.contains("must start with 'R' or 'r'"),
        "Error message should mention invalid register token, got: {}",
        err
    );
}

#[test]
fn test_trailing_comma() {
    let input = "PUT_CONST R0, 42,";
    let result = parse_instructions(input);
    assert!(result.is_err(), "Trailing comma should cause a parameter count error");
    let err = result.unwrap_err();
    assert!(
        err.contains("Empty token detected"),
        "Error message should mention empty token, got: {}",
        err
    );
}

#[test]
fn test_extra_whitespace_and_commas() {
    let input = "   PUT_CONST   R0   ,   42   \n   HALT   ";
    let instructions = parse_instructions(input).expect("Should correctly parse with extra whitespace");
    assert_eq!(instructions.len(), 2);
    if let Instruction::PutConst { register, value } = &instructions[0] {
        assert_eq!(*register, 0);
        assert_eq!(*value, 42);
    } else {
        panic!("Expected a PutConst instruction");
    }
    match &instructions[1] {
        Instruction::Halt => {},
        _ => panic!("Expected a Halt instruction"),
    }
}

#[test]
fn test_complex_arithmetic_expression() {
    let input = "ARITHMETIC_IS R0, (3+4)*2 - 1";
    let instructions = parse_instructions(input).expect("Should parse a complex arithmetic expression");
    assert_eq!(instructions.len(), 1);
    if let Instruction::ArithmeticIs { target, expression } = &instructions[0] {
        assert_eq!(*target, 0);
        let expr_str = format!("{:?}", expression);
        assert!(!expr_str.is_empty(), "Parsed expression should not be empty");
    } else {
        panic!("Expected an ArithmeticIs instruction");
    }
}

#[test]
fn test_multi_indexed_call_invalid_register() {
    let input = "MULTI_INDEXED_CALL \"p\", R1, BAD";
    let result = parse_instructions(input);
    assert!(result.is_err(), "Invalid register token in MULTI_INDEXED_CALL should yield an error");
    let err = result.unwrap_err();
    assert!(
        err.contains("must start with 'R' or 'r'"),
        "Error message should mention invalid register token, got: {}",
        err
    );
}
