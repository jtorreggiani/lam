// tests/test_machine/test_dynamic_clause_indexing.rs

use lam::machine::core::Machine;
use lam::machine::instruction::Instruction;
use lam::term::Term;
use std::collections::HashMap;

#[test]
fn test_dynamic_clause_assert_and_index() {
    // Prepare a simple program that does nothing
    let code = vec![
         Instruction::AssertClause { predicate: "p".to_string(), address: 42 },
         Instruction::AssertClause { predicate: "p".to_string(), address: 43 },
    ];
    let mut machine = Machine::new(1, code);
    
    // For testing, pre-populate the index_table for predicate "p" with a dummy key.
    // (In a more complete system, keys would be computed from the clause head.)
    let mut dummy_index: HashMap<Vec<Term>, Vec<usize>> = HashMap::new();
    // We use a fixed key (for example, [Const(1)]) for testing.
    dummy_index.insert(vec![Term::Const(1)], Vec::new());
    machine.index_table.insert("p".to_string(), dummy_index);
    
    // Run the assertions; these will update both the predicate table and the index table.
    let _ = machine.run();
    
    // Check that in the index_table for "p", the dummy key now has both clause addresses.
    let index_map = machine.index_table.get("p").expect("Index table for 'p' exists");
    let clause_list = index_map.get(&vec![Term::Const(1)]).expect("Key [Const(1)] exists");
    assert!(clause_list.contains(&42), "Clause address 42 should be indexed");
    assert!(clause_list.contains(&43), "Clause address 43 should be indexed");
}
