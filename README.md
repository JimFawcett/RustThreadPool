# RustThreadPool

https://JimFawcett.github.io/RustThreadPool.html

## Concept:
RustThreadPool is a facility for processing a function object concurrently on a specified number of threads, using a thread-safe blocking queue. Rust threadpool accepts number of threads and function object in constructor.  Uses RustBlockingQueue.

<img src="https://JimFawcett.github.io/Pictures/ThreadPoolDiagram.jpg" width="500" />                                   

## Design:
There is one struct, ThreadPool<M>, with an associated function and five methods in this design:

```rust
#[derive(Debug)]
pub struct ThreadPool<M> 
{
    sbq: Arc<BlockingQueue<M>>,
    thrd: Vec<Option<JoinHandle<()>>>
    /* see note below about Option */
}
/*-----------------------------------------------------
  construct threadpool, starting nt threads,
  provide threadpool processing as f:F in new 
*/
pub fn new<F>(nt:u8, f:F) -> ThreadPool<M> 
where F: FnOnce(&BlockingQueue<M>) -> () + Send + 'static + Copy

/*-- wait for threads to finish --*/
pub fn wait(&mut self)

/*-- post to ThreadPool queue --*/
pub fn post_message(&mut self, _msg:M) 
where M:Debug + Clone 


