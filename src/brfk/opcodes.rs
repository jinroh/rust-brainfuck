#[derive(Clone, Copy)]
pub enum OpCode {
    IncrPtr,
    DecrPtr,
    Incr,
    Decr,
    Print,
    Load,
    Breakpoint,
    Jmp(usize),
    JmpClose(usize),
}

pub fn compile(code: &[u8]) -> Option<Box<[OpCode]>> {
    let mut codes: Vec<OpCode> = Vec::with_capacity(code.len());

    const N: usize = 512;
    let mut jmp_index: [usize; N] = [0; N];
    let mut jmp_count: usize = 0;

    for &b in code.iter() {
        match b as char {
            '>' => codes.push(OpCode::IncrPtr),
            '<' => codes.push(OpCode::DecrPtr),
            '+' => codes.push(OpCode::Incr),
            '-' => codes.push(OpCode::Decr),
            '.' => codes.push(OpCode::Print),
            ',' => codes.push(OpCode::Load),
            '!' => codes.push(OpCode::Breakpoint),
            '[' => {
                if jmp_count >= N {
                    return None;
                }
                jmp_index[jmp_count] = codes.len();
                jmp_count += 1;
                codes.push(OpCode::Jmp(0)); // placeholder
            }
            ']' => {
                if jmp_count == 0 {
                    return None;
                }
                let idx = jmp_index[jmp_count - 1];
                let off = codes.len() - idx;
                codes[idx] = OpCode::Jmp(off);
                codes.push(OpCode::JmpClose(off));
                jmp_count -= 1;
            }
            _ => {}
        }
    }

    if jmp_count != 0 {
        return None;
    }

    Some(codes.into_boxed_slice())
}
