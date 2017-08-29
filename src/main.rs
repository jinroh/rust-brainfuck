mod brfk;

use std::fmt;
use std::env;
use std::error;
use std::fs::File;
use std::io::{self, stdin, stdout, Read, Write, BufReader, BufWriter};
use std::result::Result;

use brfk::{Interpreter, CompileError};

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Compile(CompileError),
    Static(&'static str),
}

pub fn run_interpreter(filename: String) -> Result<(), Box<error::Error>> {
    let source_code = load_program_file(filename).map_err(CliError::Io)?;
    let code = brfk::compile(&source_code).map_err(CliError::Compile)?;

    Interpreter::new(Box::new(BufReader::new(stdin())),
                     Box::new(BufWriter::new(stdout())),
                     &code)
            .run()?;
    Ok(())

}

pub fn load_program_file(filename: String) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    File::open(filename)?.read_to_end(&mut buf)?;
    Ok(buf)
}

fn main() {
    ::std::process::exit(match env::args()
        .nth(1)
        .ok_or_else(|| From::from(CliError::Static("Missing argument")))
        .and_then(|filename| run_interpreter(filename)) {
            Ok(_) => 0,
            Err(err) => {
                writeln!(io::stderr(), "{}", err.description()).unwrap();
                1
            }
        }
    )
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            CliError::Io(ref err) => write!(f, "{}", err),
            CliError::Compile(ref err) => write!(f, "{:?}", err),
            CliError::Static(s) => write!(f, "{}", s),
        }
    }
}

impl error::Error for CliError {
    fn description(&self) -> &str {
        match *self {
            CliError::Io(ref err) => err.description(),
            CliError::Compile(_) => "Compile error",
            CliError::Static(s) => s,
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}

impl From<CompileError> for CliError {
    fn from(err: CompileError) -> CliError {
        CliError::Compile(err)
    }
}
