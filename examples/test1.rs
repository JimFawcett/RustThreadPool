/////////////////////////////////////////////////////////////
// rust_thread_pool::test1.rs - demo thread pool           //
//                                                         //
// Jim Fawcett, https://JimFawcett.github.io, 29 Jun 2020  //
/////////////////////////////////////////////////////////////
/*
   This is a good demonstration of the way BlockingQueue
   will be used in an application.
*/
#![allow(dead_code)]
#![allow(unused_imports)]

use std::io::*;
use std::sync::*;
use std::thread;
use std::time;
use rust_blocking_queue::{BlockingQueue};
use rust_thread_pool::{ThreadPool};

/*-- test function run by thread pool threads --*/
fn test_fun() { 
    let millis = time::Duration::from_millis(1);
    let id = thread::current().id();
    for _ in 0..5 {
        print!("\n  this is a test from thread {:?}", id);
        thread::sleep(millis);
    }
 }

/*-- use test function for thread pool processing --*/
fn test1() {
    print!("\n  -- demonstrate threadpool using function --\n");
    let mut tp = ThreadPool::<String>::new(5, test_fun);
    tp.wait();
}
/*-- using closure for thread pool processing --*/
fn test2() {
    print!("\n  -- demonstrate threadpool using closure --\n");
    /* define closure cl */
    let millis = time::Duration::from_millis(1);
    /* capture data */
    let msg = "test message";
    print!("\n  capture data is: {:?}\n", msg);
    let cl = move || { 
        let id = thread::current().id();
        for _ in 0..5 {
            print!("\n  {} from {:?}", msg, id);
            thread::sleep(millis);
        }
    };
    /* run closure cl in thread pool with 5 threads */
    let mut tp = ThreadPool::<String>::new(5, cl);
    tp.wait();
}
/*-- simple test of BlockingQueue --*/
fn test0() {

    let share = Arc::new(BlockingQueue::<String>::new());
    let share1 = Arc::clone(&share);
    let share2 = Arc::clone(&share);

    let flush = || { let _ = std::io::stdout().flush(); };

    /*-- child thread dequeues messages --*/
    let handle = thread::spawn(move || {
        print!("\n  child thread started");
        flush();
        loop {
            let t = share1.de_q();
            print!("\n  dequeued {} on child thread", t);
            flush();
            if &t == "quit" {
                break;
            }
        }
        print!("\n  thread shutting down");
        flush();
    });

    /*-- main thread enqueues messages --*/
    for i in 0..5 {
        let msg = format!("msg #{}", i.to_string());
        print!("\n  enqueued {:?} on main thread", msg);
        flush();
        share2.en_q(msg);
    }
    /*-- shut down child thread --*/
    print!("\n  enqueued {:?} on main thread", "quit");
    flush();
    share2.en_q("quit".to_string());

    /*-- child thread must complete before exiting --*/
    print!("\n  waiting for child thread to stop");
    flush();
    let _ = handle.join();

    print!("\n  queue length = {}", share2.len());
}
/* test BlockingQueue and ThreadPool */
fn main() {

    print!("\n  Demonstrate queue shared between threads");
    print!("\n ==========================================\n");

    // test0();  // test BlockingQueue
    // test1();  // test threadpool with function
    test2();  // thest threadpool with closure

    print!("\n\n  That's all Folks!\n");
}
