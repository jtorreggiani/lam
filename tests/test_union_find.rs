#[cfg(test)]
mod tests {
  use lam::machine::unification::UnionFind;
  use lam::machine::term::Term;

  /// Test that binding a variable to a constant works and that the variable is resolved.
  #[test]
  fn test_bind_and_resolve() {
      let mut uf = UnionFind::new();
      // Bind variable 1 to constant 42.
      uf.bind(1, &Term::Const(42)).expect("Binding should succeed");
      assert_eq!(uf.resolve(&Term::Var(1)), Term::Const(42));
  }

  /// Test binding a variable to another variable and then binding that variable.
  #[test]
  fn test_bind_variable_to_variable() {
      let mut uf = UnionFind::new();
      // Bind variable 1 to variable 2.
      uf.bind(1, &Term::Var(2)).expect("Binding should succeed");
      // Then bind variable 2 to constant 10.
      uf.bind(2, &Term::Const(10)).expect("Binding should succeed");
      // The resolution of Var(1) should now be constant 10.
      assert_eq!(uf.resolve(&Term::Var(1)), Term::Const(10));
  }

  /// Test that after binding and then undoing the trail the variable becomes unbound.
  #[test]
  fn test_undo_trail() {
      let mut uf = UnionFind::new();
      let initial_trail_len = uf.trail.len();
      // Bind variable 1 to constant 42.
      uf.bind(1, &Term::Const(42)).expect("Binding should succeed");
      let after_bind = uf.resolve(&Term::Var(1));
      assert_eq!(after_bind, Term::Const(42));
      // Undo the binding.
      uf.undo_trail(initial_trail_len);
      // Now, Var(1) should be unbound.
      assert_eq!(uf.resolve(&Term::Var(1)), Term::Var(1));
  }

  /// Test that binding a variable to itself does nothing (no extra binding is added).
  #[test]
  fn test_bind_variable_to_itself() {
      let mut uf = UnionFind::new();
      // Binding a variable to itself should succeed but not change the state.
      uf.bind(1, &Term::Var(1)).expect("Self-binding should succeed");
      // Resolution should yield the same unbound variable.
      assert_eq!(uf.resolve(&Term::Var(1)), Term::Var(1));
  }
}