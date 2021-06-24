extern crate libc;
use libc::{SIGTERM, SIGUSR1};

use std::time;
use std::thread::sleep;

// diff between static and const:
// 1 static live in the static part of the binary (near bottom of addr space) - and can't be copied over. They are mutable.
// 2 const live in normal addr space and can be copied over by the compiler as many times as necessary for best perf. They are immutable.
static mut SHUT_DOWN: bool = false;

pub fn register_signal_handlers() {
    unsafe {
        //note how we're casting function pointers as word-sized integers
        libc::signal(SIGTERM, handle_sigterm as usize);
        libc::signal(SIGUSR1, handle_sigusr1 as usize);
    }
}

// must be fast and simple - so only sets the flag
// this means somewhere else in the app we're constantly checking the flag
pub fn handle_sigterm() {

    //todo not too sure why we need to register signal handlers again here?
    register_signal_handlers();

    println!("handling SIGTERM");
    unsafe {
        SHUT_DOWN = true
    }
}

pub fn handle_sigusr1() {
    register_signal_handlers();
    println!("handling SIGUSR1 - doing nothing really");
}

fn main() {
    // register handlers so that when our app receives a certain kind of signal it knows what to do
    register_signal_handlers();

    let delay = time::Duration::from_secs(1);

    for i in 1_usize.. {
        println!("{}", i);
        unsafe {
            // note that static variables need to be handled inside of unsafe blocks - you shouldn't use them unless you know what you're doing
            if SHUT_DOWN {
                println!("killing the process");
                return;
            }
        }

        sleep(delay);

        let signal = if i > 5 {
            SIGTERM
        } else {
            SIGUSR1
        };

        unsafe {
            libc::raise(signal);
        }
    }

    //loop above will go on forever so this part is unreachable
    unreachable!();
}