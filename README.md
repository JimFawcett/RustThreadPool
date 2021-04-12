# RustThreadPool

https://JimFawcett.github.io/RustThreadPool.html

Rust threadpool that accepts number of threads and function object in constructor.  Uses RustBlockingQueue

# Rust ThreadPool
  
<table>
  <tr>
    <td>
<pre>/////////////////////////////////////////////////////////////
// rust_thread_pool::lib.rs - threadpool wit BlockingQueue // 
//                                                         //
// Jim Fawcett, https://JimFawcett.github.com, 29 Jun 2020 //
/////////////////////////////////////////////////////////////
/*
   There are two undefined methods for ThreadPool<M>
   that need to be implemented before this design is
   complete, e.g.:
   - post_work_item posts a function object to input queue
   - get_result retrieves results from an output queue
*/
#![allow(dead_code)]
use std::fmt::*;
use rust_blocking_queue::*;
use std::thread::*;
use std::sync::*;
use std::default::{Default};

#[derive(Debug)]
pub struct ThreadPool<M> 
{
    sbq: Arc<BlockingQueue<M>>,
    thrd: Vec<Option<JoinHandle<()>>>
    /* see note below about Option */
}
impl<M> ThreadPool<M> 
where M: Send + 'static
{
    /*-----------------------------------------------------
      construct threadpool, starting nt threads,
      provide threadpool processing as f:F in new 
    */
    pub fn new<F>(nt:u8, f:F) -> ThreadPool<M> 
    where F: FnOnce(&BlockingQueue<M>) -> () + Send + 'static + Copy
    {
        /* safely share BlockingQueue with Arc */
        let sqm = Arc::new(BlockingQueue::<M>::new());
        let mut vt = Vec::<Option<JoinHandle<()>>>::new();
        /* start nt threads */
        for _i in 0..nt {
            /*----------------------------------------------- 
              ref sq to master shared queue (sqm) is captured
              by thread proc closure 
            */
            let sq = Arc::clone(&sqm);
            let handle = std::thread::spawn( move || { 
                f(&sq);  // thread_pool_processing
            });
            vt.push(Some(handle));
        }
        Self { // return newly created threadpool
            sbq: sqm,
            thrd: vt, 
        }
    }
    /*-- wait for threads to finish --*/
    pub fn wait(&mut self) {
        
        for handle in &mut self.thrd {
            let _ = handle.take().unwrap().join();
            /*
              This is a hack!
              Without the Option, wrapping threadhandle, can't move threadhandle
              out of Vec<JoinHandle<()>>, so error in line above. 
              
              Can move out of the option as long as we replace
              the moved value (take swaps None for Some in option).

              I was stumpted until I saw this link.  Apparently a well known hack.
              https://users.rust-lang.org/t/spawn-threads-and-join-in-destructor/1613
            */
        }
    }
    /*-- post to ThreadPool queue --*/
    pub fn post_message(&mut self, _msg:M) 
    where M:Debug + Clone {
        self.sbq.en_q(_msg);
    }
    /*-- return results to caller --*/
    pub fn get_message(&mut self) -> M 
    where M:Debug + Clone + Default {
        /* to be defined */
        let m:M = M::default();
        m
    }
}</pre>
    </td>
    <td>
<pre>/////////////////////////////////////////////////////////////
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

/*-- test queued string messages in pool --*/
pub fn test_message_in_pool(tp: &BlockingQueue<String>) {
    let q = String::from("quit");
    let id = thread::current().id();
    loop {
        let msg = tp.de_q();
        print!("\n  deQed {:<12} : {:?}", msg, id);
        if msg == q {
            tp.en_q(msg);
            break;
        }
        thread::yield_now();
    }
}
/*-- post message to pool --*/
pub fn post_message_to_pool() {
    let mut tp = ThreadPool::<String>::new(4, test_message_in_pool);
    let msg = String::from("message #");
    
    let _millis = time::Duration::from_millis(10);
    
    for i in 0..20 {
        let mut msg = msg.clone();
        msg.push_str(&i.to_string());
        tp.post_message(msg);
        // thread::sleep(_millis);
    }
    tp.post_message("quit".to_string());
    tp.wait();
}
/*-----------------------------------------------------------
  Define WorkItem type to execute in ThreadPool<WorkItem>
*/
#[derive(Debug, Clone)]
pub struct WorkItem {
    stop: bool,
}
impl WorkItem {
    pub fn new() -> WorkItem {
        WorkItem {
            stop: false,
        }
    }
    pub fn execute(&self) -> bool {
        let _id = thread::current().id();
        print!("\n  executing work item: {:?}", _id);
        self.stop
    }
    pub fn quit(&mut self) {
        self.stop = true;
    }
}

/*-- test queued WorkItems in pool --*/
pub fn test_workitem_in_pool(tp: &BlockingQueue<WorkItem>) {
    loop {
        let wi: WorkItem = tp.de_q();
        if wi.execute() {
            tp.en_q(wi);
            break;
        }
        thread::yield_now();
    }
    print!("\n  thread terminating");
}

/*-- post to pool --*/
pub fn post_workitem_to_pool() {

    let mut tp = ThreadPool::<WorkItem>::new(4, test_workitem_in_pool);
    let mut wkitm = WorkItem::new();
    
    let _millis = time::Duration::from_millis(10);
    
    for _i in 0..20 {
        tp.post_message(wkitm.clone());
        thread::sleep(_millis);
    }
    wkitm.quit();
    tp.post_message(wkitm);
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
    print!("\n ==========================================");

    post_workitem_to_pool();
    // post_message_to_pool();
    // test0();  // test BlockingQueue

    print!("\n\n  That's all Folks!\n");
}</pre>
    </td>
  <tr>
</table>
