#[cfg(test)]
mod tests {
    use lam::machine::core::Machine;
    use lam::machine::instruction::Instruction;
    use lam::machine::term::Term;
    use std::collections::HashMap;

    #[test]
    fn test_dynamic_clause_indexing() {
        // Program that asserts two clauses and then performs an IndexedCall.
        let code = vec![
            // Initialize register 0 to 1 (the key for indexing).
            Instruction::PutConst { register: 0, value: 1 },
            Instruction::AssertClause { predicate: "p".to_string(), address: 42 },
            Instruction::AssertClause { predicate: "p".to_string(), address: 43 },
            // IndexedCall will use the key in register 0.
            Instruction::IndexedCall { predicate: "p".to_string(), index_register: 0 },
            Instruction::Proceed,
        ];
        let mut machine = Machine::new(1, code);
        
        // Pre-populate the index table for predicate "p" with a dummy key.
        let mut dummy_index: HashMap<Vec<Term>, Vec<usize>> = HashMap::new();
        dummy_index.insert(vec![Term::Const(1)], Vec::new());
        machine.index_table.insert("p".to_string(), dummy_index);
        
        machine.run().expect("Machine run should succeed");
        
        // Check that for key [Const(1)] the clause addresses (42 or 43) are present.
        if let Some(index_map) = machine.index_table.get("p") {
            if let Some(clause_list) = index_map.get(&vec![Term::Const(1)]) {
                assert!(clause_list.contains(&42) || clause_list.contains(&43),
                        "Clause addresses 42 or 43 should be indexed");
            } else {
                panic!("Key [Const(1)] does not exist in index map");
            }
        } else {
            panic!("Index table for 'p' does not exist");
        }
    }
}
