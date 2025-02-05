use crate::machine::instruction::Instruction;
use crate::term::Term;
use crate::arithmetic::parse_expression;

#[derive(Debug)]
pub enum AssemblerError {
    Message(String),
}

// A structure representing one predicate definition.
#[derive(Debug)]
struct PredicateDefinition {
    name: String, // e.g. "edge/2" or "path/2"
    instructions: Vec<Instruction>,
}

/// Assembles a LAM program written in a logical, sectioned format into a final vector
/// of LAM instructions. The source file supports two sections:
///   - .predicates : Contains predicate definitions. Each definition starts with a line:
///         pred <predicate_name>:
///     followed by one instruction per line.
///   - .query      : Contains the query instructions (the “main” program).
///
/// Returns the final instruction vector with a header that registers each predicate.
pub fn assemble_program(input: &str) -> Result<Vec<Instruction>, AssemblerError> {
    let mut predicates: Vec<PredicateDefinition> = Vec::new();
    let mut query_instructions: Vec<Instruction> = Vec::new();

    let mut current_section = "";
    let mut current_pred_name: Option<String> = None;
    let mut current_pred_instr: Vec<Instruction> = Vec::new();

    // Process the file line by line.
    for (line_no, orig_line) in input.lines().enumerate() {
        let line = orig_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Section directives start with a dot.
        if line.starts_with('.') {
            // If we were in a predicate definition, finish it.
            if !current_section.is_empty() && current_section == ".predicates" {
                if let Some(name) = current_pred_name.take() {
                    predicates.push(PredicateDefinition {
                        name,
                        instructions: current_pred_instr.clone(),
                    });
                    current_pred_instr.clear();
                }
            }
            current_section = line;
            continue;
        }

        // In the .predicates section, look for predicate headers or instructions.
        if current_section == ".predicates" {
            if line.starts_with("pred ") {
                // Line should be: pred <predicate_name>:
                if let Some(colon_pos) = line.find(':') {
                    let raw_header = line[5..colon_pos].trim();
                    // If the header contains a '/', then only take the portion before it.
                    let header = if let Some(slash_pos) = raw_header.find('/') {
                        &raw_header[..slash_pos]
                    } else {
                        raw_header
                    };
                    // If we were in a previous predicate definition, finish it.
                    if let Some(name) = current_pred_name.take() {
                        predicates.push(PredicateDefinition {
                            name,
                            instructions: current_pred_instr.clone(),
                        });
                        current_pred_instr.clear();
                    }
                    current_pred_name = Some(header.to_string());
                } else {
                    return Err(AssemblerError::Message(format!(
                        "Line {}: Missing ':' in predicate header",
                        line_no + 1
                    )));
                }
          } else {
              let instr = parse_logical_instruction(line)
                  .map_err(|e| AssemblerError::Message(format!("Line {}: {}", line_no + 1, e)))?;
              current_pred_instr.push(instr);
          }
        } else if current_section == ".query" {
            let instr = parse_logical_instruction(line)
                .map_err(|e| AssemblerError::Message(format!("Line {}: {}", line_no + 1, e)))?;
            query_instructions.push(instr);
        } else {
            return Err(AssemblerError::Message(format!(
                "Line {}: Unknown section directive",
                line_no + 1
            )));
        }
    }
    // If a predicate is still being defined at the end of the file, finish it.
    if let Some(name) = current_pred_name.take() {
        predicates.push(PredicateDefinition {
            name,
            instructions: current_pred_instr,
        });
    }

    // Assemble final instructions.
    // First, compute header AssertClause instructions.
    let mut header: Vec<Instruction> = Vec::new();
    let mut predicate_addresses: Vec<(String, usize)> = Vec::new();

    // We will eventually place the predicate definitions after the header.
    // Reserve header space: the header will be the same length as the number of predicates.
    // (We compute each predicate's starting address as header length + current length.)
    let header_len = predicates.len();
    let mut final_instructions: Vec<Instruction> = Vec::new();

    // Append each predicate's instructions, recording its starting address.
    for pred in &predicates {
        let start_addr = header_len + final_instructions.len();
        predicate_addresses.push((pred.name.clone(), start_addr));
        final_instructions.extend_from_slice(&pred.instructions);
    }

    // Append the query instructions after predicate definitions.
    final_instructions.extend(query_instructions);

    // Now, create header AssertClause instructions.
    for (name, addr) in predicate_addresses {
        header.push(Instruction::AssertClause { predicate: name, address: addr });
    }

    // Final assembled program: header followed by predicate definitions and then query.
    let mut assembled = header;
    assembled.extend(final_instructions);
    Ok(assembled)
}

// A helper function to parse a single logical instruction line.
// This is a simplified version of our previous parser.
fn parse_logical_instruction(line: &str) -> Result<Instruction, String> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.is_empty() {
        return Err("Empty line".to_string());
    }
    match tokens[0] {
        "PutConst" => {
            if tokens.len() != 3 {
                return Err("PutConst expects 2 arguments".to_string());
            }
            let register = tokens[1]
                .parse::<usize>()
                .map_err(|_| "Invalid register".to_string())?;
            let value = tokens[2]
                .parse::<i32>()
                .map_err(|_| "Invalid value".to_string())?;
            Ok(Instruction::PutConst { register, value })
        }
        "PutVar" => {
            if tokens.len() != 4 {
                return Err("PutVar expects 3 arguments".to_string());
            }
            let register = tokens[1]
                .parse::<usize>()
                .map_err(|_| "Invalid register".to_string())?;
            let var_id = tokens[2]
                .parse::<usize>()
                .map_err(|_| "Invalid var_id".to_string())?;
            let name = tokens[3].trim_matches('"').to_string();
            Ok(Instruction::PutVar { register, var_id, name })
        }
        "Call" => {
            if tokens.len() != 2 {
                return Err("Call expects 1 argument".to_string());
            }
            let predicate = tokens[1].to_string();
            Ok(Instruction::Call { predicate })
        }
        "Proceed" => Ok(Instruction::Proceed),
        "AssertClause" => {
            if tokens.len() != 3 {
                return Err("AssertClause expects 2 arguments".to_string());
            }
            let predicate = tokens[1].to_string();
            let address = tokens[2]
                .parse::<usize>()
                .map_err(|_| "Invalid address".to_string())?;
            Ok(Instruction::AssertClause { predicate, address })
        }
        "Fail" => Ok(Instruction::Fail),
        "ArithmeticIs" => {
            if tokens.len() < 3 {
                return Err("ArithmeticIs expects at least 2 arguments".to_string());
            }
            let target = tokens[1]
                .parse::<usize>()
                .map_err(|_| "Invalid target".to_string())?;
            let expr_str = tokens[2..].join(" ");
            let expression = parse_expression(&expr_str)
                .map_err(|e| format!("ArithmeticIs error: {}", e))?;
            Ok(Instruction::ArithmeticIs { target, expression })
        }
        "BuildCompound" => {
            if tokens.len() < 3 {
                return Err("BuildCompound expects at least 2 arguments".to_string());
            }
            let target = tokens[1]
                .parse::<usize>()
                .map_err(|_| "Invalid target".to_string())?;
            let functor = tokens[2].to_string();
            let mut arg_registers = Vec::new();
            for token in &tokens[3..] {
                let reg = token
                    .parse::<usize>()
                    .map_err(|_| "Invalid register".to_string())?;
                arg_registers.push(reg);
            }
            Ok(Instruction::BuildCompound { target, functor, arg_registers })
        }
        "Choice" => {
            if tokens.len() != 2 {
                return Err("Choice expects 1 argument".to_string());
            }
            let alternative = tokens[1]
                .parse::<usize>()
                .map_err(|_| "Invalid alternative".to_string())?;
            Ok(Instruction::Choice { alternative })
        }
        "Deallocate" => Ok(Instruction::Deallocate),
        "Allocate" => {
            if tokens.len() != 2 {
                return Err("Allocate expects 1 argument".to_string());
            }
            let n = tokens[1]
                .parse::<usize>()
                .map_err(|_| "Invalid size".to_string())?;
            Ok(Instruction::Allocate { n })
        }
        "SetLocal" => {
            if tokens.len() != 3 {
                return Err("SetLocal expects 2 arguments".to_string());
            }
            let index = tokens[1]
                .parse::<usize>()
                .map_err(|_| "Invalid index".to_string())?;
            let value = tokens[2]
                .parse::<i32>()
                .map_err(|_| "Invalid value".to_string())?;
            Ok(Instruction::SetLocal { index, value: Term::Const(value) })
        }
        "GetLocal" => {
            if tokens.len() != 3 {
                return Err("GetLocal expects 2 arguments".to_string());
            }
            let index = tokens[1]
                .parse::<usize>()
                .map_err(|_| "Invalid index".to_string())?;
            let register = tokens[2]
                .parse::<usize>()
                .map_err(|_| "Invalid register".to_string())?;
            Ok(Instruction::GetLocal { index, register })
        }
        other => Err(format!("Unknown instruction '{}'", other)),
    }
}
