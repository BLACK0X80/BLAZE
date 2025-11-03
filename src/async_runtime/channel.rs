use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub struct Channel<T> {
    inner: Arc<ChannelInner<T>>,
}

struct ChannelInner<T> {
    queue: Mutex<VecDeque<T>>,
    condvar: Condvar,
    capacity: usize,
    closed: Mutex<bool>,
}

pub struct Sender<T> {
    inner: Arc<ChannelInner<T>>,
}

pub struct Receiver<T> {
    inner: Arc<ChannelInner<T>>,
}

impl<T> Channel<T> {
    pub fn bounded(capacity: usize) -> (Sender<T>, Receiver<T>) {
        let inner = Arc::new(ChannelInner {
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            condvar: Condvar::new(),
            capacity,
            closed: Mutex::new(false),
        });
        
        (
            Sender { inner: Arc::clone(&inner) },
            Receiver { inner },
        )
    }
    
    pub fn unbounded() -> (Sender<T>, Receiver<T>) {
        Self::bounded(usize::MAX)
    }
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), T> {
        let mut queue = self.inner.queue.lock().unwrap();
        
        if *self.inner.closed.lock().unwrap() {
            return Err(value);
        }
        
        while queue.len() >= self.inner.capacity {
            queue = self.inner.condvar.wait(queue).unwrap();
            
            if *self.inner.closed.lock().unwrap() {
                return Err(value);
            }
        }
        
        queue.push_back(value);
        self.inner.condvar.notify_one();
        Ok(())
    }
    
    pub fn try_send(&self, value: T) -> Result<(), T> {
        let mut queue = self.inner.queue.lock().unwrap();
        
        if *self.inner.closed.lock().unwrap() || queue.len() >= self.inner.capacity {
            return Err(value);
        }
        
        queue.push_back(value);
        self.inner.condvar.notify_one();
        Ok(())
    }
    
    pub fn close(&self) {
        *self.inner.closed.lock().unwrap() = true;
        self.inner.condvar.notify_all();
    }
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        let mut queue = self.inner.queue.lock().unwrap();
        
        loop {
            if let Some(value) = queue.pop_front() {
                self.inner.condvar.notify_one();
                return Some(value);
            }
            
            if *self.inner.closed.lock().unwrap() {
                return None;
            }
            
            queue = self.inner.condvar.wait(queue).unwrap();
        }
    }
    
    pub fn try_recv(&self) -> Option<T> {
        let mut queue = self.inner.queue.lock().unwrap();
        let value = queue.pop_front();
        
        if value.is_some() {
            self.inner.condvar.notify_one();
        }
        
        value
    }
    
    pub fn recv_timeout(&self, duration: std::time::Duration) -> Option<T> {
        let mut queue = self.inner.queue.lock().unwrap();
        let deadline = std::time::Instant::now() + duration;
        
        loop {
            if let Some(value) = queue.pop_front() {
                self.inner.condvar.notify_one();
                return Some(value);
            }
            
            if *self.inner.closed.lock().unwrap() {
                return None;
            }
            
            let now = std::time::Instant::now();
            if now >= deadline {
                return None;
            }
            
            let timeout = deadline - now;
            let (q, timeout_result) = self.inner.condvar.wait_timeout(queue, timeout).unwrap();
            queue = q;
            
            if timeout_result.timed_out() {
                return None;
            }
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
