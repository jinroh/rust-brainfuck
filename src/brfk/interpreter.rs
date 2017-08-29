use std::io::{self, BufRead, Write};

use std::str::{self, FromStr};
use std::borrow::Cow;

use super::opcodes::OpCode;

const RAM_LENGTH: usize = 0xf000;

#[derive(PartialEq, Eq)]
enum Mode {
    Running,
    Debugging,
}

#[derive(Debug)]
enum Command {
    Next,
    PrintCode,
    PrintMemory,
    Exit,
}

impl FromStr for Command {
    type Err = Cow<'static, str>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "next" | "n" => Ok(Command::Next),
            "code" | "c" => Ok(Command::PrintCode),
            "mem" | "m" => Ok(Command::PrintMemory),
            "exit" | "quit" | "q" => Ok(Command::Exit),
            _ => Err(format!("Unable to parse command: {}", s).into()),
        }
    }
}

pub struct Interpreter<'a> {
    code: &'a [OpCode],
    ram: Box<[u8]>,
    data: usize,

    input: Box<BufRead>,
    output: Box<Write>,

    mode: Mode,
}

impl<'a> Interpreter<'a> {
    pub fn new(input: Box<BufRead>, output: Box<Write>, code: &'a [OpCode]) -> Interpreter<'a> {
        Interpreter {
            code: code,
            ram: vec![0; RAM_LENGTH].into_boxed_slice(),
            data: 0,

            input: input,
            output: output,

            mode: Mode::Running,
        }
    }

    pub fn run(&mut self) -> Result<(), io::Error> {
        self.run_recur(self.code, 0)
    }

    pub fn run_recur(&mut self, code: &[OpCode], offset: usize) -> Result<(), io::Error> {
        for (pc, opcode) in code.iter().enumerate() {
            let pc_real = pc + offset;
            if self.mode == Mode::Debugging {
                self.wait_console_commands(pc_real, opcode);
            }
            match *opcode {
                OpCode::IncrPtr => self.data += 1,
                OpCode::DecrPtr => self.data -= 1,
                OpCode::Incr => self.ram[self.data] = self.deref().wrapping_add(1),
                OpCode::Decr => self.ram[self.data] = self.deref().wrapping_sub(1),
                OpCode::Print => {
                    self.output.write_all(&self.ram[self.data..self.data + 1])?;
                    if self.mode == Mode::Debugging {
                        println!();
                    }
                }
                OpCode::Load => {
                    self.input
                        .read_exact(&mut self.ram[self.data..self.data + 1])?
                }
                OpCode::Breakpoint => self.mode = Mode::Debugging,
                OpCode::While(ref code) => {
                    while self.deref() != 0 {
                        self.run_recur(&code, pc_real)?
                    }
                }
            }
        }
        Ok(())
    }

    fn deref(&self) -> u8 {
        self.ram[self.data]
    }

    fn wait_console_commands(&mut self, pc: usize, opcode: &OpCode) {
        self.prompt_console(pc, opcode);
        self.output.flush().unwrap();
        let mut command_string = String::new();
        while {
                  command_string.clear();
                  self.input.read_line(&mut command_string).is_ok()
              } {
            command_string.pop();
            command_string.trim();
            if command_string.len() == 0 {
                self.prompt_console(pc, opcode);
                continue;
            }
            let command = command_string.parse::<Command>();
            match command {
                Ok(Command::PrintCode) => {
                    const NUM_COLS: usize = 64;
                    const MAX_ROWS: usize = 8;
                    for r in 0..MAX_ROWS {
                        let skip = pc + r * NUM_COLS;
                        if self.code.len() <= skip {
                            break;
                        }
                        let iter = self.code.iter().skip(skip).take(NUM_COLS);
                        for c in iter {
                            print!("{:?}", c);
                        }
                        println!();
                    }
                }
                Ok(Command::PrintMemory) => {
                    const NUM_ROWS: usize = 8;
                    const NUM_COLS: usize = 16;
                    let data = self.data / NUM_COLS;
                    for r in 0..NUM_ROWS {
                        let mut data_local;
                        data_local = data + r * NUM_ROWS;
                        print!("0x{:08x}  ", data_local);
                        for x in 0..NUM_COLS {
                            let byte = self.ram[data_local];
                            data_local = data_local.wrapping_add(1);
                            print!("{:02x}", byte);
                            if x < NUM_COLS - 1 {
                                print!(" ");
                            }
                        }
                        print!("  ");
                        data_local = data + r * NUM_ROWS;
                        for _ in 0..NUM_COLS {
                            let byte = self.ram[data_local];
                            data_local = data_local.wrapping_add(1);
                            match byte {
                                0x20...0x7e => print!("{}", byte as char),
                                _ => print!("."),
                            }
                        }
                        println!();
                    }
                }
                Ok(Command::Next) => return,
                Ok(Command::Exit) => {
                    self.mode = Mode::Running;
                    return;
                }
                Err(ref e) => println!("{}", e),
            }
            self.prompt_console(pc, opcode);
        }
    }

    fn prompt_console(&self, pc: usize, opcode: &OpCode) {
        print!("(brainfuck 0x{:08x}:0x{:08x}:{:?}) ", pc, self.data, opcode);
        ::std::io::stdout().flush().unwrap();
    }
}
