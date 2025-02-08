#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use lam::machine::core::Machine;
    use lam::machine::term::Term;

    #[test]
    fn test_update_index_table_on_retract() {
        // Create a machine with one register and no instructions.
        let mut machine = Machine::new(1, vec![]);

        // Prepopulate the index table for predicate "p" with two keys:
        // key1: [Const(1)] → [42, 43]
        // key2: [Const(2)] → [42]
        let key1 = vec![Term::Const(1)];
        let key2 = vec![Term::Const(2)];
        let mut index_map: HashMap<Vec<Term>, Vec<usize>> = HashMap::new();
        index_map.insert(key1.clone(), vec![42, 43]);
        index_map.insert(key2.clone(), vec![42]);
        machine.index_table.insert("p".to_string(), index_map);

        // Call update_index_table_on_retract to remove clause address 42
        machine.update_index_table_on_retract("p", 42);

        // Retrieve and check the updated index table.
        let updated_index = machine.index_table.get("p").unwrap();
        // For key1, only 43 should remain.
        assert_eq!(updated_index.get(&key1).unwrap(), &vec![43]);
        // For key2, the list should now be empty.
        assert!(updated_index.get(&key2).unwrap().is_empty());
    }
}