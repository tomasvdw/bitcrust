//! SCRIPTING interface
//!
//! CURRENTLY ONLY USED FOR DISPLAY
//!
//! libbitcoinconsensus is used for script verification


pub mod context;

pub mod opcode;

mod opcode_pushdata;

pub mod stack;

#[derive(Debug, PartialEq, Eq)]
pub enum ScriptError {
    StackUnderflow,
    NumericOverflow,

    UnexpectedEndOfScript,

    InvalidOpcode,

    PushdataTooLarge
}
