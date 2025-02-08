#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;

    #[test]
    fn test_build_compound() {
        // Program:
        // 0: PutConst reg0, 42
        // 1: PutConst reg1, 99
        // 2: BuildCompound into reg2 using functor "f" and registers [0, 1]
        let code = vec![
            Instruction::PutConst { register: 0, value: 42 },
            Instruction::PutConst { register: 1, value: 99 },
            Instruction::BuildCompound { target: 2, functor: "f".to_string(), arg_registers: vec![0, 1] },
        ];
        
        let mut machine = Machine::new(3, code);
        machine.run().expect("Machine run should succeed");
        
        let expected = Term::Compound("f".to_string(), vec![Term::Const(42), Term::Const(99)]);
        assert_eq!(machine.registers[2], Some(expected));
    }
}
