//! Parses textual LAM programs into a vector of instructions.

use crate::machine::instruction::Instruction;

pub fn parse_program(input: &str) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();
    for (line_no, line) in input.lines().enumerate() {
        let line = line.trim();
        // Skip blank lines and comments.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }
        let instr = match tokens[0] {
            "PutStr" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: PutStr expects 2 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let value = tokens[2].trim_matches('"').to_string();
                Instruction::PutStr { register, value }
            }
            "GetStr" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: GetStr expects 2 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let value = tokens[2].trim_matches('"').to_string();
                Instruction::GetStr { register, value }
            }
            "PutConst" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: PutConst expects 2 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let value = tokens[2].parse::<i32>()
                    .map_err(|_| format!("Line {}: invalid value", line_no + 1))?;
                Instruction::PutConst { register, value }
            }
            "PutVar" => {
                if tokens.len() != 4 {
                    return Err(format!("Line {}: PutVar expects 3 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let var_id = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid var_id", line_no + 1))?;
                let name = tokens[3].trim_matches('"').to_string();
                Instruction::PutVar { register, var_id, name }
            }
            "GetConst" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: GetConst expects 2 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let value = tokens[2].parse::<i32>()
                    .map_err(|_| format!("Line {}: invalid value", line_no + 1))?;
                Instruction::GetConst { register, value }
            }
            "GetVar" => {
                if tokens.len() != 4 {
                    return Err(format!("Line {}: GetVar expects 3 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let var_id = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid var_id", line_no + 1))?;
                let name = tokens[3].trim_matches('"').to_string();
                Instruction::GetVar { register, var_id, name }
            }
            "Call" => {
                if tokens.len() != 2 {
                    return Err(format!("Line {}: Call expects 1 argument", line_no + 1));
                }
                let predicate = tokens[1].to_string();
                Instruction::Call { predicate }
            }
            "Proceed" => Instruction::Proceed,
            "Choice" => {
                if tokens.len() != 2 {
                    return Err(format!("Line {}: Choice expects 1 argument", line_no + 1));
                }
                let alternative = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid alternative", line_no + 1))?;
                Instruction::Choice { alternative }
            }
            "Allocate" => {
                if tokens.len() != 2 {
                    return Err(format!("Line {}: Allocate expects 1 argument", line_no + 1));
                }
                let n = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid size", line_no + 1))?;
                Instruction::Allocate { n }
            }
            "Deallocate" => Instruction::Deallocate,
            "Fail" => Instruction::Fail,
            "ArithmeticIs" => {
                if tokens.len() < 3 {
                    return Err(format!("Line {}: ArithmeticIs expects at least 2 arguments", line_no + 1));
                }
                let target = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid target", line_no + 1))?;
                let expr_str = tokens[2..].join(" ");
                let expression = crate::arithmetic::parse_expression(&expr_str)
                    .map_err(|e| format!("Line {}: ArithmeticIs expression error: {}", line_no + 1, e))?;
                Instruction::ArithmeticIs { target, expression }
            }
            "SetLocal" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: SetLocal expects 2 arguments", line_no + 1));
                }
                let index = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid index", line_no + 1))?;
                let value = tokens[2].parse::<i32>()
                    .map_err(|_| format!("Line {}: invalid value", line_no + 1))?;
                Instruction::SetLocal { index, value: crate::term::Term::Const(value) }
            }
            "GetLocal" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: GetLocal expects 2 arguments", line_no + 1));
                }
                let index = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid index", line_no + 1))?;
                let register = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                Instruction::GetLocal { index, register }
            }
            "GetStructure" => {
                if tokens.len() != 4 {
                    return Err(format!("Line {}: GetStructure expects 3 arguments", line_no + 1));
                }
                let register = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                let functor = tokens[2].to_string();
                let arity = tokens[3].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid arity", line_no + 1))?;
                Instruction::GetStructure { register, functor, arity }
            }
            "IndexedCall" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: IndexedCall expects 2 arguments", line_no + 1));
                }
                let predicate = tokens[1].to_string();
                let index_register = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid index register", line_no + 1))?;
                Instruction::IndexedCall { predicate, index_register }
            }
            "MultiIndexedCall" => {
                if tokens.len() < 3 {
                    return Err(format!("Line {}: MultiIndexedCall expects at least 2 arguments", line_no + 1));
                }
                let predicate = tokens[1].to_string();
                let mut index_registers = Vec::new();
                for token in &tokens[2..] {
                    let reg = token.parse::<usize>()
                        .map_err(|_| format!("Line {}: invalid index register", line_no + 1))?;
                    index_registers.push(reg);
                }
                Instruction::MultiIndexedCall { predicate, index_registers }
            }
            "TailCall" => {
                if tokens.len() != 2 {
                    return Err(format!("Line {}: TailCall expects 1 argument", line_no + 1));
                }
                let predicate = tokens[1].to_string();
                Instruction::TailCall { predicate }
            }
            "AssertClause" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: AssertClause expects 2 arguments", line_no + 1));
                }
                let predicate = tokens[1].to_string();
                let address = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid address", line_no + 1))?;
                Instruction::AssertClause { predicate, address }
            }
            "RetractClause" => {
                if tokens.len() != 3 {
                    return Err(format!("Line {}: RetractClause expects 2 arguments", line_no + 1));
                }
                let predicate = tokens[1].to_string();
                let address = tokens[2].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid address", line_no + 1))?;
                Instruction::RetractClause { predicate, address }
            }
            "Cut" => Instruction::Cut,
            "BuildCompound" => {
                if tokens.len() < 3 {
                    return Err(format!("Line {}: BuildCompound expects at least 2 arguments", line_no + 1));
                }
                let target = tokens[1].parse::<usize>()
                    .map_err(|_| format!("Line {}: invalid target", line_no + 1))?;
                let functor = tokens[2].to_string();
                let mut arg_registers = Vec::new();
                for token in &tokens[3..] {
                    let reg = token.parse::<usize>()
                        .map_err(|_| format!("Line {}: invalid register", line_no + 1))?;
                    arg_registers.push(reg);
                }
                Instruction::BuildCompound { target, functor, arg_registers }
            }
            "Halt" => Instruction::Halt,
            other => return Err(format!("Line {}: Unknown instruction '{}'", line_no + 1, other)),
        };
        instructions.push(instr);
    }
    Ok(instructions)
}
