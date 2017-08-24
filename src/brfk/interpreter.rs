use std::io::{stdin, Write};

use std::fmt;
use std::str::{self, FromStr};
use std::borrow::Cow;

use std::thread::{self, JoinHandle};
use std::sync::mpsc::{sync_channel, Receiver};

use super::opcodes::OpCode;

const RAM_LENGTH: usize = 0xf000;

#[derive(PartialEq, Eq)]
enum Mode {
    Running,
    Debugging,
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
            OpCode::Jmp(_) => write!(f, "["),
            OpCode::JmpClose(_) => write!(f, "]"),
        }
    }
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

pub struct Interpreter {
    code: Box<[OpCode]>,
    ram: Box<[u8]>,

    data: usize,
    pgrm: usize,

    mode: Mode,

    stdin_rx: Receiver<String>,
    _stdin_thread: JoinHandle<()>,
}

impl Interpreter {
    pub fn new(code: Box<[OpCode]>) -> Interpreter {
        let (stdin_sx, stdin_rx) = sync_channel(0);
        let stdin_thread = thread::spawn(move || loop {
            stdin_sx.send(read_stdin()).unwrap();
         });

        Interpreter {
            code: code,
            ram: vec![0; RAM_LENGTH].into_boxed_slice(),
            data: 0,
            pgrm: 0,

            mode: Mode::Running,

            stdin_rx: stdin_rx,
            _stdin_thread: stdin_thread,
        }
    }

    pub fn run(&mut self) {
        while let Some((pgrm, opcode)) = self.next() {
            if self.mode == Mode::Debugging {
                self.wait_console_commands(pgrm, opcode);
            }
            self.run_instruction(pgrm, opcode)
        }
    }

    fn run_instruction(&mut self, pgrm: usize, opcode: OpCode) {
        match opcode {
            OpCode::IncrPtr => self.data += 1,
            OpCode::DecrPtr => self.data -= 1,
            OpCode::Incr => self.ram[self.data] = self.ram[self.data].wrapping_add(1),
            OpCode::Decr => self.ram[self.data] = self.ram[self.data].wrapping_sub(1),
            OpCode::Print => {
              print!("{}", self.ram[self.data] as char);
              if self.mode == Mode::Debugging {
                println!();
              }
            },
            OpCode::Load => {
                prompt!();
                while let Ok(s) = self.stdin_rx.recv() {
                    let result = s.as_bytes();
                    if result.len() > 0 {
                        self.ram[self.data] = result[0];
                        break;
                    }
                    prompt!();
                }
            }
            OpCode::Breakpoint => self.mode = Mode::Debugging,
            OpCode::Jmp(offset) => {
                if self.ram[self.data] == 0 {
                    self.pgrm = pgrm + offset + 1
                }
            }
            OpCode::JmpClose(offset) => {
                if self.ram[self.data] != 0 {
                    self.pgrm = pgrm - offset + 1
                }
            }
        }
    }

    fn next(&mut self) -> Option<(usize, OpCode)> {
        let pgrm = self.pgrm;
        if pgrm < self.code.len() {
            let opcode = self.code[pgrm];
            self.pgrm += 1;
            Some((pgrm, opcode))
        } else {
            None
        }
    }

    fn wait_console_commands(&mut self, pgrm: usize, opcode: OpCode) {
        self.prompt_console(pgrm, opcode);
        while let Ok(command_string) = self.stdin_rx.recv() {
            if command_string == "" {
                self.prompt_console(pgrm, opcode);
                continue;
            }
            let command = command_string.parse::<Command>();
            match command {
                Ok(Command::PrintCode) => {
                    const NUM_COLS: usize = 64;
                    const MAX_ROWS: usize = 8;
                    for r in 0..MAX_ROWS {
                        let skip = pgrm + r * NUM_COLS;
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
            self.prompt_console(pgrm, opcode);
        }
    }

    fn prompt_console(&self, pgrm: usize, opcode: OpCode) {
        prompt!("(brainfuck 0x{:08x}:0x{:08x}:{:?}) ",
                pgrm,
                self.data,
                opcode);
    }
}

fn read_stdin() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().into()
}
