use crate::machine::instruction::Instruction;
use crate::machine::arithmetic::parse_expression;
use crate::machine::term::Term;

/// Parse an input string (from a file) containing one LAM instruction per line.
/// Lines starting with ';' or '#' are treated as comments and ignored.
/// Returns a Vec of Instructions or an error string.
pub fn parse_instructions(input: &str) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();
    for (line_num, line) in input.lines().enumerate() {
        let line = line.trim();
        // Skip empty lines or comment lines.
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        // Split the line into tokens by commas.
        let tokens = split_tokens(line).map_err(|e| format!("Line {}: {}", line_num + 1, e))?;
        if tokens.is_empty() {
            continue;
        }
        // The first token may contain the mnemonic and some parameters.
        let mut parts = tokens[0].split_whitespace();
        let mnemonic = parts
            .next()
            .ok_or_else(|| format!("Line {}: missing mnemonic", line_num + 1))?
            .to_uppercase();
        // Collect any extra parts from the first token along with remaining tokens.
        let mut params: Vec<&str> = parts.collect();
        params.extend_from_slice(&tokens[1..]);

        // Match on the mnemonic and parse parameters accordingly.
        let instr = match mnemonic.as_str() {
            "PUT_CONST" => {
                if params.len() != 2 {
                    return Err(format!(
                        "Line {}: PUT_CONST expects 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let register = parse_register(params[0])?;
                let value = params[1]
                    .parse::<i32>()
                    .map_err(|e| format!("Line {}: failed to parse integer in PUT_CONST: {}", line_num + 1, e))?;
                Instruction::PutConst { register, value }
            }
            "PUT_VAR" => {
                if params.len() != 3 {
                    return Err(format!(
                        "Line {}: PUT_VAR expects 3 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let register = parse_register(params[0])?;
                let var_id = params[1]
                    .parse::<usize>()
                    .map_err(|e| format!("Line {}: failed to parse variable id in PUT_VAR: {}", line_num + 1, e))?;
                let name = parse_string(params[2])?;
                Instruction::PutVar { register, var_id, name }
            }
            "GET_CONST" => {
                if params.len() != 2 {
                    return Err(format!(
                        "Line {}: GET_CONST expects 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let register = parse_register(params[0])?;
                let value = params[1]
                    .parse::<i32>()
                    .map_err(|e| format!("Line {}: failed to parse integer in GET_CONST: {}", line_num + 1, e))?;
                Instruction::GetConst { register, value }
            }
            "GET_VAR" => {
                if params.len() != 3 {
                    return Err(format!(
                        "Line {}: GET_VAR expects 3 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let register = parse_register(params[0])?;
                let var_id = params[1]
                    .parse::<usize>()
                    .map_err(|e| format!("Line {}: failed to parse variable id in GET_VAR: {}", line_num + 1, e))?;
                let name = parse_string(params[2])?;
                Instruction::GetVar { register, var_id, name }
            }
            "CALL" => {
                if params.len() != 1 {
                    return Err(format!(
                        "Line {}: CALL expects 1 parameter, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let predicate = parse_string_or_ident(params[0]);
                Instruction::Call { predicate }
            }
            "PROCEED" => {
                if !params.is_empty() {
                    return Err(format!(
                        "Line {}: PROCEED expects no parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                Instruction::Proceed
            }
            "CHOICE" => {
                if params.len() != 1 {
                    return Err(format!(
                        "Line {}: CHOICE expects 1 parameter, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let alternative = params[0]
                    .parse::<usize>()
                    .map_err(|e| format!("Line {}: failed to parse alternative in CHOICE: {}", line_num + 1, e))?;
                Instruction::Choice { alternative }
            }
            "ALLOCATE" => {
                if params.len() != 1 {
                    return Err(format!(
                        "Line {}: ALLOCATE expects 1 parameter, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let n = params[0]
                    .parse::<usize>()
                    .map_err(|e| format!("Line {}: failed to parse integer in ALLOCATE: {}", line_num + 1, e))?;
                Instruction::Allocate { n }
            }
            "DEALLOCATE" => {
                if !params.is_empty() {
                    return Err(format!(
                        "Line {}: DEALLOCATE expects no parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                Instruction::Deallocate
            }
            "ARITHMETIC_IS" => {
                if params.len() != 2 {
                    return Err(format!(
                        "Line {}: ARITHMETIC_IS expects 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let target = parse_register(params[0])?;
                let expr = parse_expression(params[1])
                    .map_err(|e| format!("Line {}: failed to parse expression in ARITHMETIC_IS: {}", line_num + 1, e))?;
                Instruction::ArithmeticIs { target, expression: expr }
            }
            "SET_LOCAL" => {
                if params.len() != 2 {
                    return Err(format!(
                        "Line {}: SET_LOCAL expects 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let index = params[0]
                    .parse::<usize>()
                    .map_err(|e| format!("Line {}: failed to parse index in SET_LOCAL: {}", line_num + 1, e))?;
                let value = parse_term_literal(params[1])?;
                Instruction::SetLocal { index, value }
            }
            "GET_LOCAL" => {
                if params.len() != 2 {
                    return Err(format!(
                        "Line {}: GET_LOCAL expects 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let index = params[0]
                    .parse::<usize>()
                    .map_err(|e| format!("Line {}: failed to parse index in GET_LOCAL: {}", line_num + 1, e))?;
                let register = parse_register(params[1])?;
                Instruction::GetLocal { index, register }
            }
            "FAIL" => {
                if !params.is_empty() {
                    return Err(format!(
                        "Line {}: FAIL expects no parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                Instruction::Fail
            }
            "GET_STRUCTURE" => {
                if params.len() != 3 {
                    return Err(format!(
                        "Line {}: GET_STRUCTURE expects 3 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let register = parse_register(params[0])?;
                let functor = parse_string_or_ident(params[1]);
                let arity = params[2]
                    .parse::<usize>()
                    .map_err(|e| format!("Line {}: failed to parse arity in GET_STRUCTURE: {}", line_num + 1, e))?;
                Instruction::GetStructure { register, functor, arity }
            }
            "INDEXED_CALL" => {
                if params.len() != 2 {
                    return Err(format!(
                        "Line {}: INDEXED_CALL expects 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let predicate = parse_string_or_ident(params[0]);
                let index_register = parse_register(params[1])?;
                Instruction::IndexedCall { predicate, index_register }
            }
            "MULTI_INDEXED_CALL" => {
                if params.len() < 2 {
                    return Err(format!(
                        "Line {}: MULTI_INDEXED_CALL expects at least 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let predicate = parse_string_or_ident(params[0]);
                let mut index_registers = Vec::new();
                for token in &params[1..] {
                    let reg = parse_register(token)?;
                    index_registers.push(reg);
                }
                Instruction::MultiIndexedCall { predicate, index_registers }
            }
            "TAIL_CALL" => {
                if params.len() != 1 {
                    return Err(format!(
                        "Line {}: TAIL_CALL expects 1 parameter, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let predicate = parse_string_or_ident(params[0]);
                Instruction::TailCall { predicate }
            }
            "ASSERT_CLAUSE" => {
                if params.len() != 2 {
                    return Err(format!(
                        "Line {}: ASSERT_CLAUSE expects 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let predicate = parse_string_or_ident(params[0]);
                let address = params[1]
                    .parse::<usize>()
                    .map_err(|e| format!("Line {}: failed to parse address in ASSERT_CLAUSE: {}", line_num + 1, e))?;
                Instruction::AssertClause { predicate, address }
            }
            "RETRACT_CLAUSE" => {
                if params.len() != 2 {
                    return Err(format!(
                        "Line {}: RETRACT_CLAUSE expects 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let predicate = parse_string_or_ident(params[0]);
                let address = params[1]
                    .parse::<usize>()
                    .map_err(|e| format!("Line {}: failed to parse address in RETRACT_CLAUSE: {}", line_num + 1, e))?;
                Instruction::RetractClause { predicate, address }
            }
            "CUT" => {
                if !params.is_empty() {
                    return Err(format!(
                        "Line {}: CUT expects no parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                Instruction::Cut
            }
            "BUILD_COMPOUND" => {
                if params.len() < 3 {
                    return Err(format!(
                        "Line {}: BUILD_COMPOUND expects at least 3 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let target = parse_register(params[0])?;
                let functor = parse_string_or_ident(params[1]);
                let mut arg_registers = Vec::new();
                for token in &params[2..] {
                    let reg = parse_register(token)?;
                    arg_registers.push(reg);
                }
                Instruction::BuildCompound { target, functor, arg_registers }
            }
            "PUT_STR" => {
                if params.len() != 2 {
                    return Err(format!(
                        "Line {}: PUT_STR expects 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let register = parse_register(params[0])?;
                let value = parse_string(params[1])?;
                Instruction::PutStr { register, value }
            }
            "GET_STR" => {
                if params.len() != 2 {
                    return Err(format!(
                        "Line {}: GET_STR expects 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let register = parse_register(params[0])?;
                let value = parse_string(params[1])?;
                Instruction::GetStr { register, value }
            }
            "MOVE" => {
                if params.len() != 2 {
                    return Err(format!(
                        "Line {}: MOVE expects 2 parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                let src = parse_register(params[0])?;
                let dst = parse_register(params[1])?;
                Instruction::Move { src, dst }
            }
            "HALT" => {
                if !params.is_empty() {
                    return Err(format!(
                        "Line {}: HALT expects no parameters, got {}",
                        line_num + 1,
                        params.len()
                    ));
                }
                Instruction::Halt
            }
            _ => {
                return Err(format!(
                    "Line {}: unknown instruction mnemonic '{}'",
                    line_num + 1,
                    mnemonic
                ))
            }
        };
        instructions.push(instr);
    }
    Ok(instructions)
}

/// Splits a line by commas and trims whitespace.
/// If any token (after splitting) is empty (e.g. due to a trailing comma), returns an error.
fn split_tokens(line: &str) -> Result<Vec<&str>, String> {
    let tokens: Vec<&str> = line.split(',').map(|token| token.trim()).collect();
    // If the trimmed line is not empty and any token is empty, report an error.
    if !line.trim().is_empty() && tokens.iter().any(|token| token.is_empty()) {
        return Err("Empty token detected: possible trailing comma or missing parameter".to_string());
    }
    Ok(tokens)
}

/// Parse a register token. For example, "R0" or "r1" returns 0 or 1.
/// Now, if the token is too short or does not start with 'R' or 'r', it produces an error message containing
/// the expected phrase.
fn parse_register(token: &str) -> Result<usize, String> {
    let token = token.trim();
    if token.len() < 2 || !(token.starts_with('R') || token.starts_with('r')) {
        return Err(format!("Register token '{}' must start with 'R' or 'r'", token));
    }
    let num_str: String = token.chars().skip(1).collect();
    num_str
        .parse::<usize>()
        .map_err(|e| format!("Failed to parse register number in '{}': {}", token, e))
}

/// Parse a string literal. If the token is enclosed in double quotes, remove them.
fn parse_string(token: &str) -> Result<String, String> {
    let token = token.trim();
    if token.starts_with('"') && token.ends_with('"') && token.len() >= 2 {
        Ok(token[1..token.len() - 1].to_string())
    } else {
        // Allow bare strings if they contain no spaces.
        if token.contains(' ') {
            Err(format!("Expected a quoted string if spaces are present: {}", token))
        } else {
            Ok(token.to_string())
        }
    }
}

/// Parse a token that can be either a quoted string literal or an identifier.
fn parse_string_or_ident(token: &str) -> String {
    let token = token.trim();
    if token.starts_with('"') && token.ends_with('"') && token.len() >= 2 {
        token[1..token.len() - 1].to_string()
    } else {
        token.to_string()
    }
}

/// Parse a term literal for instructions such as SET_LOCAL.
/// Supports integer constants, quoted string literals, or register references (which become variables).
fn parse_term_literal(token: &str) -> Result<Term, String> {
    let token = token.trim();
    if token.starts_with('R') || token.starts_with('r') {
        let reg = parse_register(token)?;
        Ok(Term::Var(reg))
    } else if token.starts_with('"') && token.ends_with('"') && token.len() >= 2 {
        Ok(Term::Str(token[1..token.len() - 1].to_string()))
    } else if let Ok(n) = token.parse::<i32>() {
        Ok(Term::Const(n))
    } else {
        // If the token is unquoted and contains spaces, produce an error.
        if token.contains(' ') {
            Err(format!("Expected a quoted string if spaces are present: {}", token))
        } else {
            // Otherwise, treat as a bare identifier string.
            Ok(Term::Str(token.to_string()))
        }
    }
}
