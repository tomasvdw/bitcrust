
pub mod context;

pub mod opcode;

pub mod stack;

#[derive(Debug)]
pub enum ScriptError {
    Pop_Empty_Stack,
    Numeric_Overflow,
}