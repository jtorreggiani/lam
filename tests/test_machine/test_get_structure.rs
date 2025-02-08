#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;

    #[test]
    fn test_get_structure() {
        let code = vec![
            Instruction::PutConst { register: 0, value: 1 },
            Instruction::PutConst { register: 1, value: 2 },
            Instruction::BuildCompound { target: 2, functor: "f".to_string(), arg_registers: vec![0, 1] },
            Instruction::GetStructure { register: 2, functor: "f".to_string(), arity: 2 },
        ];
        
        let mut machine = Machine::new(3, code);
        machine.run().expect("Machine run should succeed");
        
        let expected = Term::Compound("f".to_string(), vec![Term::Const(1), Term::Const(2)]);
        assert_eq!(machine.registers[2], Some(expected));
    }
}
