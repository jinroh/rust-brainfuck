use std::slice::Iter;

pub enum OpCode {
    IncrPtr,
    DecrPtr,
    Incr,
    Decr,
    Print,
    Load,
    Breakpoint,
    Loop(Vec<OpCode>),
}

#[derive(Debug)]
pub enum CompileError {
    UnclosedLoop,
    TooClosedLoop,
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
            '[' => opcodes.push(OpCode::Loop(compile_recur(code, indent + 1)?)),
            ']' => {
                return if indent > 0 {
                    Ok(opcodes)
                } else {
                    Err(CompileError::TooClosedLoop)
                }
            }
            _ => {}
        }
    }

    return if indent > 0 {
        Err(CompileError::UnclosedLoop)
    } else {
        Ok(opcodes)
    }
}
