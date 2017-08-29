use std;

extern crate libc;

const PAGE_SIZE: usize = 4096;

pub struct Jit<'a> {
    mem: &'a mut [u8],
}

impl<'a> Jit<'a> {
    pub fn new() -> Jit<'a> {
        let mem : &mut [u8];
        let size = 1 * PAGE_SIZE;

        unsafe {
            let mut raw: *mut libc::c_void = std::mem::uninitialized();
            libc::posix_memalign(&mut raw, PAGE_SIZE, size);
            libc::mprotect(raw, size, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);
            mem = std::slice::from_raw_parts_mut(std::mem::transmute(raw), size);
        }

        Jit {
            mem: mem,

        }
    }
}
