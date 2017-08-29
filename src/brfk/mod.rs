pub mod opcodes;
pub mod interpreter;

pub use self::opcodes::compile;
pub use self::opcodes::CompileError;
pub use self::interpreter::Interpreter;

// pub mod jit;
// pub use self::jit::Jit;
