//! The machine module defines the core components of the LAM abstract machine.
//! It utilizes several design patterns:
//! - **Command Pattern:** Each instruction is a command that can execute itself.
//! - **Strategy Pattern:** Built-in predicates are implemented as strategies.
//! - **Memento Pattern:** Backtracking is implemented using saved machine states.

pub mod instruction;
pub mod frame;
pub mod choice_point;
pub mod machine;

pub use instruction::Instruction;
pub use frame::Frame;
pub use choice_point::ChoicePoint;
pub use machine::Machine;
pub use machine::MachineError;
