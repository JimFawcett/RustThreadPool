# RustThreadPool

https://JimFawcett.github.io/RustThreadPool.html

Rust threadpool accepts number of threads and function object in constructor.  Uses RustBlockingQueue

<img src="https://JimFawcett.github.io/Pictures/ThreadPoolDiagram.jpg" width="500" />                                   

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

/*---------------------------------------------
  pull message from threadpool, called by 
  threadpool threads 
----------------------------------------------*/
pub fn get_message(&mut self) -> M 
where M:Debug + Clone + Default {

