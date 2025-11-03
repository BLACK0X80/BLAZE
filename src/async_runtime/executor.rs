use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread;

pub struct Executor {
    task_queue: Arc<Mutex<VecDeque<Arc<Task>>>>,
    num_threads: usize,
}

struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

struct ExecutorWaker {
    task: Arc<Task>,
    task_queue: Arc<Mutex<VecDeque<Arc<Task>>>>,
}

impl Executor {
    pub fn new(num_threads: usize) -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            num_threads: num_threads.max(1),
        }
    }
    
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
        });
        
        self.task_queue.lock().unwrap().push_back(task);
    }
    
    pub fn run(&self) {
        let mut handles = Vec::new();
        
        for _ in 0..self.num_threads {
            let queue = Arc::clone(&self.task_queue);
            
            let handle = thread::spawn(move || {
                loop {
                    let task = {
                        let mut q = queue.lock().unwrap();
                        q.pop_front()
                    };
                    
                    if let Some(task) = task {
                        let waker = Arc::new(ExecutorWaker {
                            task: Arc::clone(&task),
                            task_queue: Arc::clone(&queue),
                        }).into();
                        
                        let mut context = Context::from_waker(&waker);
                        let mut future = task.future.lock().unwrap();
                        
                        if future.as_mut().poll(&mut context).is_pending() {
                        }
                    } else {
                        thread::sleep(std::time::Duration::from_millis(1));
                        
                        if queue.lock().unwrap().is_empty() {
                            break;
                        }
                    }
                }
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
    
    pub fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let result = Arc::new(Mutex::new(None));
        let result_clone = Arc::clone(&result);
        
        self.spawn(async move {
            let output = future.await;
            *result_clone.lock().unwrap() = Some(output);
        });
        
        self.run();
        
        Arc::try_unwrap(result)
            .unwrap()
            .into_inner()
            .unwrap()
            .unwrap()
    }
}

impl Wake for ExecutorWaker {
    fn wake(self: Arc<Self>) {
        self.task_queue.lock().unwrap().push_back(Arc::clone(&self.task));
    }
    
    fn wake_by_ref(self: &Arc<Self>) {
        self.task_queue.lock().unwrap().push_back(Arc::clone(&self.task));
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new(num_cpus::get())
    }
}

pub struct JoinHandle<T> {
    result: Arc<Mutex<Option<T>>>,
}

impl<T> JoinHandle<T> {
    pub fn new(result: Arc<Mutex<Option<T>>>) -> Self {
        Self { result }
    }
    
    pub async fn await_result(self) -> Option<T> {
        loop {
            if let Some(result) = self.result.lock().unwrap().take() {
                return Some(result);
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
    }
}
