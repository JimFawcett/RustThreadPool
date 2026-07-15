//! Demonstration application for the `rust_thread_pool` library.
//!
//! This application showcases all public functions of the `ThreadPool<M>` struct:
//!   - `ThreadPool::new`       — construct a pool with N worker threads
//!   - `ThreadPool::post_message` — send work items into the pool's queue
//!   - `ThreadPool::get_message`  — retrieve a result (stub / default value)
//!   - `ThreadPool::wait`      — block until all workers finish
//!
//! Run with:
//!   cargo run

use rust_thread_pool::ThreadPool;
use rust_blocking_queue::BlockingQueue;

use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ---------------------------------------------------------------------------
// A simple work-item type that the thread pool will process.
// It must implement: Send + 'static (pool requirement),
//                    Debug + Clone  (post_message / get_message bounds),
//                    Default        (get_message bound).
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
struct WorkItem {
    id: u32,
    payload: String,
}

impl WorkItem {
    fn new(id: u32, payload: impl Into<String>) -> Self {
        WorkItem {
            id,
            payload: payload.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: build a shared results collector that worker closures can write to.
// We wrap it in Arc<Mutex<Vec<...>>> so multiple threads can append results.
// ---------------------------------------------------------------------------
type ResultsVec = Arc<Mutex<Vec<String>>>;

// ---------------------------------------------------------------------------
// Demo 1 — Basic construction, post_message, and wait
//
// Demonstrates:
//   • ThreadPool::new        (create a pool with 4 worker threads)
//   • ThreadPool::post_message (push 10 work items onto the queue)
//   • ThreadPool::wait       (block until all threads drain the queue and stop)
// ---------------------------------------------------------------------------
fn demo_basic_pool() {
    println!("=================================================================");
    println!("Demo 1: Basic construction, post_message, and wait");
    println!("=================================================================");

    // Shared results collector — workers will record processed item IDs here.
    let results: ResultsVec = Arc::new(Mutex::new(Vec::new()));
    let results_clone = Arc::clone(&results);

    // -----------------------------------------------------------------------
    // ThreadPool::new
    //   • nt = 4  → spawn 4 worker threads
    //   • The closure is the per-thread processing loop.
    //     It loops, dequeues a WorkItem, processes it, and stores the result.
    //     A WorkItem with id == 0 acts as a "poison-pill" sentinel that tells
    //     a thread to stop.
    // -----------------------------------------------------------------------
    println!("\n[Demo 1] Creating a ThreadPool with 4 worker threads …");

    let results_for_closure = Arc::clone(&results_clone);

    let mut pool: ThreadPool<WorkItem> = ThreadPool::new(4, move |queue: &BlockingQueue<WorkItem>| {
        loop {
            // Dequeue the next work item (blocks until one is available).
            let item = queue.deQ();

            // Poison-pill: id == 0 means "shut down this thread".
            if item.id == 0 {
                println!(
                    "  [Worker {:?}] received shutdown signal — exiting.",
                    std::thread::current().id()
                );
                break;
            }

            // Simulate some processing work.
            std::thread::sleep(Duration::from_millis(20));

            let msg = format!(
                "Thread {:?} processed WorkItem {{ id: {}, payload: \"{}\" }}",
                std::thread::current().id(),
                item.id,
                item.payload
            );
            println!("  [Worker] {}", msg);

            // Store result in the shared collector.
            results_for_closure
                .lock()
                .expect("mutex poisoned")
                .push(msg);
        }
    });

    // -----------------------------------------------------------------------
    // ThreadPool::post_message
    //   Send 10 real work items, then 4 poison-pills (one per worker thread).
    // -----------------------------------------------------------------------
    println!("\n[Demo 1] Posting 10 work items …");
    for i in 1..=10u32 {
        let item = WorkItem::new(i, format!("task-{}", i));
        println!("  [Main] post_message → {:?}", item);
        pool.post_message(item);
    }

    // Send one poison-pill per worker thread so they all terminate cleanly.
    println!("\n[Demo 1] Posting 4 shutdown (poison-pill) messages …");
    for _ in 0..4 {
        pool.post_message(WorkItem::default()); // id == 0
    }

    // -----------------------------------------------------------------------
    // ThreadPool::wait
    //   Block until every worker thread has joined.
    // -----------------------------------------------------------------------
    println!("\n[Demo 1] Calling pool.wait() — blocking until workers finish …");
    pool.wait();

    println!("\n[Demo 1] All workers finished. Results collected:");
    let locked = results.lock().expect("mutex poisoned");
    for (idx, r) in locked.iter().enumerate() {
        println!("  Result[{}]: {}", idx, r);
    }

    println!("\n[Demo 1] ✓ Demonstrated: new, post_message, wait\n");
}

// ---------------------------------------------------------------------------
// Demo 2 — get_message (stub / Default return)
//
// The library documents `get_message` as an incomplete stub that currently
// returns `M::default()`.  We demonstrate calling it and show the returned
// value so the function is exercised in a visible way.
// ---------------------------------------------------------------------------
fn demo_get_message() {
    println!("=================================================================");
    println!("Demo 2: get_message (stub — returns M::default())");
    println!("=================================================================");

    // We need a running pool (workers must not terminate immediately so the
    // pool object stays alive long enough for us to call get_message).
    // A single worker that waits for a poison-pill is sufficient.
    println!("\n[Demo 2] Creating a 1-thread pool to keep the pool alive …");

    let mut pool: ThreadPool<WorkItem> = ThreadPool::new(1, |queue: &BlockingQueue<WorkItem>| {
        // Wait for the single poison-pill.
        loop {
            let item = queue.deQ();
            if item.id == 0 {
                break;
            }
        }
    });

    // -----------------------------------------------------------------------
    // ThreadPool::get_message
    //   As documented, this is a stub that returns M::default().
    //   WorkItem::default() → WorkItem { id: 0, payload: "" }
    // -----------------------------------------------------------------------
    println!("\n[Demo 2] Calling pool.get_message() …");
    let msg: WorkItem = pool.get_message();
    println!(
        "  [Main] get_message returned: {:?}  (expected default → id=0, payload=\"\")",
        msg
    );

    // Shut down the worker.
    pool.post_message(WorkItem::default());
    pool.wait();

    println!("\n[Demo 2] ✓ Demonstrated: get_message\n");
}

// ---------------------------------------------------------------------------
// Demo 3 — Multiple pools running concurrently with different message types
//
// Shows that ThreadPool<M> is generic: here we run a second pool whose
// message type is a plain String, proving the API works with any M that
// satisfies the required bounds.
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Default)]
struct StringMsg(String);

fn demo_string_message_pool() {
    println!("=================================================================");
    println!("Demo 3: ThreadPool with a different message type (StringMsg)");
    println!("=================================================================");

    let counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);

    println!("\n[Demo 3] Creating a 3-thread StringMsg pool …");
    let mut pool: ThreadPool<StringMsg> =
        ThreadPool::new(3, move |queue: &BlockingQueue<StringMsg>| {
            loop {
                let item = queue.deQ();
                if item.0 == "STOP" {
                    println!(
                        "  [Worker {:?}] STOP received — exiting.",
                        std::thread::current().id()
                    );
                    break;
                }
                println!(
                    "  [Worker {:?}] processing: \"{}\"",
                    std::thread::current().id(),
                    item.0
                );
                *counter_clone.lock().expect("mutex poisoned") += 1;
                std::thread::sleep(Duration::from_millis(15));
            }
        });

    // Post several string messages.
    let messages = [
        "Hello from Demo 3",
        "Rust thread pools are great",
        "Concurrent processing with BlockingQueue",
        "Each message handled by one worker",
        "Workers share the queue automatically",
        "No manual locking required by the caller",
    ];

    println!("\n[Demo 3] Posting {} string messages …", messages.len());
    for msg_str in &messages {
        let msg = StringMsg(msg_str.to_string());
        println!("  [Main] post_message → {:?}", msg);
        pool.post_message(msg);
    }

    // Three poison-pills for three workers.
    println!("\n[Demo 3] Posting 3 STOP messages …");
    for _ in 0..3 {
        pool.post_message(StringMsg("STOP".to_string()));
    }

    pool.wait();

    let processed = *counter.lock().expect("mutex poisoned");
    println!(
        "\n[Demo 3] Workers processed {} real messages (expected {}).",
        processed,
        messages.len()
    );
    println!("\n[Demo 3] ✓ Demonstrated: new, post_message, wait with StringMsg type\n");
}

// ---------------------------------------------------------------------------
// Demo 4 — Stress test: high message volume with many threads
//
// Demonstrates that post_message and wait are safe under load.
// ---------------------------------------------------------------------------
fn demo_stress_test() {
    println!("=================================================================");
    println!("Demo 4: Stress test — 8 threads, 200 messages");
    println!("=================================================================");

    const NUM_THREADS: u8 = 8;
    const NUM_MESSAGES: u32 = 200;

    let total_processed: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let tp_clone = Arc::clone(&total_processed);

    println!(
        "\n[Demo 4] Spawning {} worker threads …",
        NUM_THREADS
    );

    let mut pool: ThreadPool<WorkItem> =
        ThreadPool::new(NUM_THREADS, move |queue: &BlockingQueue<WorkItem>| {
            loop {
                let item = queue.deQ();
                if item.id == 0 {
                    break; // poison-pill
                }
                // Minimal work to keep the demo fast.
                *tp_clone.lock().expect("mutex poisoned") += 1;
            }
        });

    println!(
        "[Demo 4] Posting {} work items …",
        NUM_MESSAGES
    );
    for i in 1..=NUM_MESSAGES {
        pool.post_message(WorkItem::new(i, format!("stress-{}", i)));
    }

    // Send one poison-pill per thread.
    println!(
        "[Demo 4] Posting {} shutdown signals …",
        NUM_THREADS
    );
    for _ in 0..NUM_THREADS {
        pool.post_message(WorkItem::default());
    }

    println!("[Demo 4] Waiting for all threads to complete …");
    pool.wait();

    let processed = *total_processed.lock().expect("mutex poisoned");
    println!(
        "\n[Demo 4] Stress test complete. Processed: {}/{} messages.",
        processed, NUM_MESSAGES
    );
    assert_eq!(
        processed, NUM_MESSAGES,
        "Not all messages were processed!"
    );
    println!("[Demo 4] ✓ All messages accounted for.\n");
}

// ---------------------------------------------------------------------------
// main — run every demo in sequence
// ---------------------------------------------------------------------------
fn main() {
    println!();
    println!("*****************************************************************");
    println!("  rust_thread_pool — Public API Demonstration");
    println!("*****************************************************************");
    println!();

    // --- Demo 1: core API (new, post_message, wait) ---
    demo_basic_pool();

    // --- Demo 2: get_message stub ---
    demo_get_message();

    // --- Demo 3: generic type parameter (StringMsg pool) ---
    demo_string_message_pool();

    // --- Demo 4: stress test ---
    demo_stress_test();

    println!("*****************************************************************");
    println!("  All demonstrations completed successfully.");
    println!("*****************************************************************");
    println!();
}