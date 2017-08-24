
#[macro_use]
mod macros;

pub mod opcodes;
pub mod interpreter;

pub use self::opcodes::compile;
pub use self::interpreter::Interpreter;
