//!


pub mod context;

pub mod opcode;

pub mod stack;

#[derive(Debug, PartialEq, Eq)]
pub enum ScriptError {
    StackUnderflow,
    NumericOverflow,
}