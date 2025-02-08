// tests/test_instruction_parser_extra.rs

use lam::machine::instruction_parser::parse_instructions;

#[test]
fn test_put_const_extra_parameters() {
    let input = "PUT_CONST R0, 42, 100";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("PUT_CONST expects 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_put_const_missing_parameters() {
    let input = "PUT_CONST R0";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("PUT_CONST expects 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_put_var_missing_parameters() {
    let input = "PUT_VAR R1, 0";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("PUT_VAR expects 3 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_put_var_non_numeric_var_id() {
    let input = "PUT_VAR R1, not_a_number, \"X\"";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("failed to parse variable id"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_get_var_non_numeric_var_id() {
    let input = "GET_VAR R0, not_a_number, \"X\"";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("failed to parse variable id"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_call_extra_parameters() {
    let input = "CALL \"halt\", extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("CALL expects 1 parameter"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_choice_missing_parameters() {
    let input = "CHOICE";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("CHOICE expects 1 parameter"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_choice_extra_parameters() {
    let input = "CHOICE 100, 200";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("CHOICE expects 1 parameter"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_allocate_non_numeric() {
    let input = "ALLOCATE abc";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("failed to parse integer in ALLOCATE"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_arithmetic_is_extra_parameters() {
    let input = "ARITHMETIC_IS R0, 3+4, extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("ARITHMETIC_IS expects 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_set_local_extra_parameters() {
    let input = "SET_LOCAL 0, 42, extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("SET_LOCAL expects 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_get_local_extra_parameters() {
    let input = "GET_LOCAL 0, R1, extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("GET_LOCAL expects 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_fail_extra_parameters() {
    let input = "FAIL extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("FAIL expects no parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_get_structure_extra_parameters() {
    let input = "GET_STRUCTURE R0, \"f\", 2, extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("GET_STRUCTURE expects 3 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_indexed_call_extra_parameters() {
    let input = "INDEXED_CALL \"p\", R0, extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("INDEXED_CALL expects 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_multi_indexed_call_insufficient_parameters() {
    let input = "MULTI_INDEXED_CALL \"p\"";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("MULTI_INDEXED_CALL expects at least 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_tail_call_extra_parameters() {
    let input = "TAIL_CALL \"dummy\", extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("TAIL_CALL expects 1 parameter"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_assert_clause_extra_parameters() {
    let input = "ASSERT_CLAUSE \"p\", 123, extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("ASSERT_CLAUSE expects 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_retract_clause_extra_parameters() {
    let input = "RETRACT_CLAUSE \"p\", 456, extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("RETRACT_CLAUSE expects 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_build_compound_insufficient_parameters() {
    let input = "BUILD_COMPOUND R2, \"f\"";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("BUILD_COMPOUND expects at least 3 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_put_str_extra_parameters() {
    let input = "PUT_STR R0, \"hello\", extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("PUT_STR expects 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_get_str_extra_parameters() {
    let input = "GET_STR R0, \"world\", extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("GET_STR expects 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_move_extra_parameters() {
    let input = "MOVE R1, R2, extra";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("MOVE expects 2 parameters"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_get_str_unquoted_string_with_space() {
    // When a parameter is unquoted and contains spaces, parse_string should error.
    let input = "GET_STR R0, hello world";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("Expected a quoted string if spaces are present"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_set_local_unquoted_string_with_space() {
    let input = "SET_LOCAL 0, hello world";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("Expected a quoted string if spaces are present"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_get_local_invalid_register_format() {
    let input = "GET_LOCAL 0, X1";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("must start with 'R' or 'r'"),
        "Unexpected error: {}",
        err
    );
}

#[test]
fn test_parse_register_too_short() {
    // A register token that is too short (e.g., "R") should trigger an error.
    let input = "PUT_CONST R, 42";
    let err = parse_instructions(input).unwrap_err();
    assert!(
        err.contains("Register token 'R' must start with 'R' or 'r'"),
        "Unexpected error: {}",
        err
    );
}
