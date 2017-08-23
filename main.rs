use std::env;
use std::fs::File;
use std::io;
use std::io::{stdin, Read, Write};

use std::fmt;
use std::str::{self, FromStr};
use std::borrow::Cow;

use std::thread::{self, JoinHandle};
use std::sync::mpsc::{channel, Receiver};

macro_rules! prompt {
  () => {{
    print!("> ");
    std::io::stdout().flush().unwrap();
  }};
  ($fmt:expr) => {{
    print!(concat!($fmt, "> "));
    std::io::stdout().flush().unwrap();
  }};
  ($fmt:expr, $($arg:tt)*) => {{
    print!(concat!($fmt, "> "), $($arg)*);
    std::io::stdout().flush().unwrap();
  }};
}

fn main() {
    let filename = env::args().nth(1).unwrap();
    match exec_program(filename) {
        Err(err) => println!("{:?}", err),
        _ => {}
    }
}

fn exec_program(filename: String) -> Result<(), io::Error> {
    let mut program = Brainfuck::from_filename(filename)?;
    program.run();
    Ok(())
}

const RAM_LENGTH: usize = 0xf000;

#[derive(PartialEq, Eq)]
enum Mode {
    Running,
    Debugging,
}

#[derive(Clone, Copy)]
enum OpCode {
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

struct Brainfuck {
    code: Box<[OpCode]>,
    ram: Box<[u8]>,

    data: usize,
    pgrm: usize,

    mode: Mode,

    stdin_rx: Receiver<String>,
    _stdin_thread: JoinHandle<()>,
}

impl Brainfuck {
    fn new(code: Box<[OpCode]>) -> Brainfuck {
        let (stdin_sx, stdin_rx) = channel();
        let stdin_thread = thread::spawn(move || loop {
                                             stdin_sx.send(read_stdin()).unwrap();
                                         });

        Brainfuck {
            code: code,
            ram: vec![0; RAM_LENGTH].into_boxed_slice(),
            data: 0,
            pgrm: 0,

            mode: Mode::Running,
            stdin_rx: stdin_rx,
            _stdin_thread: stdin_thread,
        }
    }

    fn from_filename(filename: String) -> Result<Brainfuck, io::Error> {
        let source_code = load_program_file(filename)?;
        if let Some(code) = Self::compile(source_code.as_slice()) {
            Ok(Self::new(code))
        } else {
            panic!("Could not compile");
        }
    }

    fn compile(code: &[u8]) -> Option<Box<[OpCode]>> {
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

    fn run_instruction(&mut self, pgrm: usize, opcode: OpCode) {
        match opcode {
            OpCode::IncrPtr => self.data += 1,
            OpCode::DecrPtr => self.data -= 1,
            OpCode::Incr => self.ram[self.data] = self.ram[self.data].wrapping_add(1),
            OpCode::Decr => self.ram[self.data] = self.ram[self.data].wrapping_sub(1),
            OpCode::Print => print!("{}", self.ram[self.data] as char),
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
            OpCode::Breakpoint => {
                self.mode = Mode::Debugging
            },
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

    fn run(&mut self) {
        while let Some((pgrm, opcode)) = self.next() {
            if self.mode == Mode::Debugging {
                self.wait_console_commands(pgrm, opcode);
            }
            self.run_instruction(pgrm, opcode)
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
                    const MAX_ROWS: usize =  8;
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
                },
                Ok(Command::PrintMemory) => {
                    const NUM_ROWS: usize = 8;
                    const NUM_COLS: usize = 16;
                    let mut data = self.data / NUM_COLS;
                    for _ in 0..NUM_ROWS {
                        print!("0x{:08x}  ", data);
                        for x in 0..NUM_COLS {
                            let byte = self.ram[data];
                            data = data.wrapping_add(1);
                            print!("{:02x}", byte);
                            if x < NUM_COLS - 1 {
                                print!(" ");
                            }
                        }
                        println!();
                    }
                }
                Ok(Command::Next) => {
                    return
                }
                Ok(Command::Exit) => {
                    self.mode = Mode::Running;
                    return
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

fn load_program_file(filename: String) -> Result<Vec<u8>, io::Error> {
    let mut buf = Vec::new();
    File::open(filename)?.read_to_end(&mut buf)?;
    Ok(buf)
}
