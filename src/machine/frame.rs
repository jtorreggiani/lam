/// A frame to store return information for a predicate call.
#[derive(Debug, Clone)]
pub struct Frame {
    pub return_pc: usize,
}