//! this doc is an example of "nonlocal control transfer"
//! - when we want to break out of a bunch of stack frames all at once
//! instead of waiting for each one to unwind as it normally would
//!
//! to do that we're using "intrinsics" - setjmp and longjmp
//! that allow us to teleport the program out of the inner-most stack frame and right to the end
//!
//! rust lang itself doesn't have setjmp / longjmp - those need to be pulled out of the compiler toolchain
//! more specifically, they're available through LLVM, a sub-compiler behind rustc
//!
//! rust itself is agnostic to the platform it runs on
//! but the compiler knows about its "target" platform and hence can provide target-specific functions
//! "intrinsics" are part of that for x86 platforms

#![feature(link_llvm_intrinsics)] //this line allows the use of "intrinsics".
#![allow(non_camel_case_types)]
#![cfg(not(windows))]

extern crate libc;

use libc::{SIGUSR1, SIGALRM, SIGHUP, SIGQUIT, SIGTERM};
use std::mem;

const JMP_BUF_WIDTH:usize = mem::size_of::<usize>() * 8; //will be 32 bit or 64bit depending on platform
type jmp_buf = [i8; JMP_BUF_WIDTH]; //we need enough storage space to store 8 integers - will be enough to save the state of the program

static mut SHUT_DOWN:bool = false; //this is the flag we're manipulating with SIGTERM
static mut RETURN_HERE:jmp_buf = [0; JMP_BUF_WIDTH]; //initialize the buffer with 0s

const MOCK_SIGNAL_AT: usize = 3; //after 3 loop iterations we'll send a fake signal

/// here we pull out the intrinsics out of llvm and define them as fns we'll be able to use in our code
/// we have to go through C to do that
extern "C" {
    #[link_name = "llvm.eh.sjlj.setjmp"]
    pub fn setjmp(_: *mut i8) -> i32;

    #[link_name = "llvm.eh.sjljlongjmp"]
    pub fn longjmp(_: *mut i8) -> ();
}

/// casts the pointer from the type that rust understands to the type that C understands
#[inline]
fn ptr_to_jmp_buf() -> *mut i8 {
    unsafe {&RETURN_HERE as *const i8 as *mut i8}
}

/// performs longjump to wherever the pointer is pointing
#[inline]
fn return_early() {
    let franken_pointer = ptr_to_jmp_buf();
    unsafe {longjmp(franken_pointer)};
}

fn register_signal_handler() {
    unsafe {
        libc::signal(SIGUSR1, handle_signals as usize);
        libc::signal(SIGTERM, handle_signals as usize);
    }
}

fn handle_signals(_signal: u32) {
    register_signal_handler();
    unsafe {
        SHUT_DOWN = true;
    }
    return_early();
}

fn print_depth(depth: usize) {
    for _ in 0..depth {
        print!("#");
    }
    println!("");
}

fn dive(depth: usize, max_depth: usize) {
    unsafe {
        if SHUT_DOWN {
            println!("shutting down!");
            return;
        }
    }

    print_depth(depth);
    if depth >= max_depth {
        return
    } else if depth == MOCK_SIGNAL_AT { //after 3 iterations we're faking a signal
        unsafe {
            libc::raise(SIGUSR1);
        }
    }
    dive(depth+1, max_depth);
    print_depth(depth);
}

fn main() {
    const JUMP_SET: i32 = 0;

    register_signal_handler();

    // setjmp to return_point, which is all 0s
    let return_point = ptr_to_jmp_buf();
    let rc = unsafe{ setjmp(return_point) };

    // todo not sure. Why do we even need JUMP_SET? Seems only used here?
    // my best guess is that at some point return_point stops being all 0s, and so we go into the else clause
    // that probably happens when we call longjmp()
    if rc == JUMP_SET {
       dive(0, 10);
    } else {
        println!("early return!");
    }

    println!("finishing!");
}