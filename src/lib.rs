/////////////////////////////////////////////////////////////
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let test = |bq:&BlockingQueue<String>| { 
            let msg = bq.de_q();
            print!("\n  {:?}", msg);
        };
        let mut tp = ThreadPool::<String>::new(2, test);
        let msg = "test message".to_string();
        tp.post_message(msg);
        tp.post_message("quit".to_string());
        tp.wait();
    }
}
