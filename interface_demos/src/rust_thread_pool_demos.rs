use rust_thread_pool::ThreadPool;
use rust_blocking_queue::BlockingQueue;

use std::time::Duration;

#[derive(Debug, Clone, Default)]
struct WorkItem {
    id: u32,
    payload: String,
}

impl WorkItem {
    fn new(id: u32, payload: impl Into<String>) -> Self {
        WorkItem { id, payload: payload.into() }
    }
}

// Demo 1 - ThreadPool::new, post_message, wait
//
// Closure must be Copy so it can be cloned into each thread.
// Non-capturing closures satisfy this; Arc captures do not.
fn demo_basic_pool() {
    println!("=================================================================");
    println!("Demo 1: new, post_message, wait");
    println!("=================================================================");

    let worker = |queue: &BlockingQueue<WorkItem>| {
        loop {
            let item = queue.de_q();
            if item.id == 0 {
                println!("  [Worker {:?}] shutdown", std::thread::current().id());
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
            println!(
                "  [Worker {:?}] id={} payload=\"{}\"",
                std::thread::current().id(), item.id, item.payload
            );
        }
    };

    let mut pool: ThreadPool<WorkItem> = ThreadPool::new(4, worker);

    println!("\n[Demo 1] posting 10 work items");
    for i in 1..=10u32 {
        pool.post_message(WorkItem::new(i, format!("task-{}", i)));
    }

    println!("[Demo 1] posting 4 shutdown signals");
    for _ in 0..4 {
        pool.post_message(WorkItem::default());
    }

    println!("[Demo 1] waiting ...");
    pool.wait();
    println!("[Demo 1] done\n");
}

// Demo 2 - get_message (stub returning M::default())
fn demo_get_message() {
    println!("=================================================================");
    println!("Demo 2: get_message stub");
    println!("=================================================================");

    let worker = |queue: &BlockingQueue<WorkItem>| {
        let item = queue.de_q(); // wait for poison-pill
        let _ = item;
    };

    let mut pool: ThreadPool<WorkItem> = ThreadPool::new(1, worker);

    let result: WorkItem = pool.get_message();
    println!("\n[Demo 2] get_message returned: {:?}", result);

    pool.post_message(WorkItem::default());
    pool.wait();
    println!("[Demo 2] done\n");
}

// Demo 3 - ThreadPool generic over a different message type
#[derive(Debug, Clone, Default)]
struct StringMsg(String);

fn demo_string_pool() {
    println!("=================================================================");
    println!("Demo 3: ThreadPool<StringMsg>");
    println!("=================================================================");

    let worker = |queue: &BlockingQueue<StringMsg>| {
        loop {
            let item = queue.de_q();
            if item.0 == "STOP" {
                println!("  [Worker {:?}] STOP", std::thread::current().id());
                break;
            }
            println!("  [Worker {:?}] \"{}\"", std::thread::current().id(), item.0);
            std::thread::sleep(Duration::from_millis(10));
        }
    };

    let mut pool: ThreadPool<StringMsg> = ThreadPool::new(3, worker);

    let messages = [
        "Hello from Demo 3",
        "Rust thread pools are generic",
        "BlockingQueue handles synchronization",
        "Workers share the queue automatically",
        "No manual locking required by the caller",
        "Each message handled by one worker",
    ];

    println!("\n[Demo 3] posting {} messages", messages.len());
    for s in &messages {
        pool.post_message(StringMsg(s.to_string()));
    }

    println!("[Demo 3] posting 3 STOP signals");
    for _ in 0..3 {
        pool.post_message(StringMsg("STOP".to_string()));
    }

    pool.wait();
    println!("[Demo 3] done\n");
}

// Demo 4 - stress test: high message volume
fn demo_stress_test() {
    println!("=================================================================");
    println!("Demo 4: stress test - 8 threads, 200 messages");
    println!("=================================================================");

    const NUM_THREADS: u8 = 8;
    const NUM_MESSAGES: u32 = 200;

    let worker = |queue: &BlockingQueue<WorkItem>| {
        loop {
            let item = queue.de_q();
            if item.id == 0 {
                break;
            }
        }
    };

    let mut pool: ThreadPool<WorkItem> = ThreadPool::new(NUM_THREADS, worker);

    println!("\n[Demo 4] posting {} items", NUM_MESSAGES);
    for i in 1..=NUM_MESSAGES {
        pool.post_message(WorkItem::new(i, format!("stress-{}", i)));
    }

    println!("[Demo 4] posting {} shutdown signals", NUM_THREADS);
    for _ in 0..NUM_THREADS {
        pool.post_message(WorkItem::default());
    }

    println!("[Demo 4] waiting ...");
    pool.wait();
    println!("[Demo 4] done\n");
}

fn main() {
    println!();
    println!("*****************************************************************");
    println!("  rust_thread_pool - Public API Demonstration");
    println!("*****************************************************************");
    println!();

    demo_basic_pool();
    demo_get_message();
    demo_string_pool();
    demo_stress_test();

    println!("*****************************************************************");
    println!("  All demonstrations completed.");
    println!("*****************************************************************");
    println!();
}
