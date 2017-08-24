use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::result::Result;

mod brfk;

pub fn run_interpreter(filename: String) -> Result<(), io::Error> {
    let source_code = load_program_file(filename)?;
    match brfk::compile(source_code.as_slice()) {
        Ok(code) => {
            let mut program = brfk::Interpreter::new(&code);
            program.run();
        }
        Err(err) => panic!("Could not compile: {:?}", err)
    }
    Ok(())
}

pub fn load_program_file(filename: String) -> Result<Vec<u8>, io::Error> {
    let mut buf = Vec::new();
    File::open(filename)?.read_to_end(&mut buf)?;
    Ok(buf)
}

fn main() {
    let filename = env::args().nth(1).unwrap();
    match run_interpreter(filename) {
        Err(err) => println!("{:?}", err),
        _ => {}
    }
}
