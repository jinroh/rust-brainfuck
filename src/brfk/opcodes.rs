use std::slice::Iter;
use std::fmt;

pub enum OpCode {
    IncrPtr,
    DecrPtr,
    Incr,
    Decr,
    Print,
    Load,
    Breakpoint,
    While(Vec<OpCode>),
}

impl fmt::Debug for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OpCode::IncrPtr => write!(f, ">"),
            OpCode::DecrPtr => write!(f, "<"),
            OpCode::Incr => write!(f, "+"),
            OpCode::Decr => write!(f, "-"),
            OpCode::Print => write!(f, "."),
            OpCode::Load => write!(f, ","),
            OpCode::Breakpoint => write!(f, "!"),
            OpCode::While(_) => write!(f, "["),
        }
    }
}

#[derive(Debug)]
pub enum CompileError {
    UnclosedWhile,
    TooClosedWhile,
}

pub fn compile(code: &[u8]) -> Result<Vec<OpCode>, CompileError> {
    compile_recur(&mut code.iter(), 0)
}

fn compile_recur(code: &mut Iter<u8>, indent: usize) -> Result<Vec<OpCode>, CompileError> {
    let mut opcodes: Vec<OpCode> = Vec::with_capacity(code.len());

    while let Some(b) = code.next() {
        match *b as char {
            '>' => opcodes.push(OpCode::IncrPtr),
            '<' => opcodes.push(OpCode::DecrPtr),
            '+' => opcodes.push(OpCode::Incr),
            '-' => opcodes.push(OpCode::Decr),
            '.' => opcodes.push(OpCode::Print),
            ',' => opcodes.push(OpCode::Load),
            '!' => opcodes.push(OpCode::Breakpoint),
            '[' => opcodes.push(OpCode::While(compile_recur(code, indent + 1)?)),
            ']' => {
                return if indent > 0 {
                    Ok(opcodes)
                } else {
                    Err(CompileError::TooClosedWhile)
                }
            }
            _ => {}
        }
    }

    return if indent > 0 {
        Err(CompileError::UnclosedWhile)
    } else {
        Ok(opcodes)
    }
}
