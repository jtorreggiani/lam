// src/unification.rs
use crate::term::Term;
use crate::machine::MachineError;
use crate::union_find::UnionFind;

/// This function performs unification of two terms.
/// It assumes that `uf` is the current union-find structure and will update it.
pub fn unify_terms(uf: &mut UnionFind, t1: &Term, t2: &Term) -> Result<(), MachineError> {
    let resolved1 = uf.resolve(t1);
    let resolved2 = uf.resolve(t2);

    match (&resolved1, &resolved2) {
        (Term::Const(a), Term::Const(b)) => {
            if a == b { Ok(()) }
            else {
                Err(MachineError::UnificationFailed(format!(
                    "Constants do not match: {} vs {}", a, b
                )))
            }
        },
        (Term::Var(v), other) => {
            uf.bind(*v, other)
        },
        (other, Term::Var(v)) => {
            uf.bind(*v, other)
        },
        (Term::Compound(f1, args1), Term::Compound(f2, args2)) => {
            if f1 != f2 || args1.len() != args2.len() {
                return Err(MachineError::UnificationFailed(format!(
                    "Compound term mismatch: {} vs {}", f1, f2
                )));
            }
            for (a, b) in args1.iter().zip(args2.iter()) {
                unify_terms(uf, a, b)?;
            }
            Ok(())
        },
        (t1, t2) => Err(MachineError::UnificationFailed(format!(
            "Failed to unify {:?} with {:?}", t1, t2
        ))),
    }
}
