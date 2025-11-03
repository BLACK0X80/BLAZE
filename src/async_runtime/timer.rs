use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

pub struct Timer {
    deadline: Instant,
}

pub struct Sleep {
    deadline: Instant,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
        }
    }
    
    pub fn sleep(duration: Duration) -> Sleep {
        Sleep {
            deadline: Instant::now() + duration,
        }
    }
    
    pub fn is_ready(&self) -> bool {
        Instant::now() >= self.deadline
    }
    
    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }
}

impl Future for Sleep {
    type Output = ();
    
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.deadline {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub fn sleep(duration: Duration) -> Sleep {
    Timer::sleep(duration)
}

pub fn timeout<F>(duration: Duration, future: F) -> Timeout<F>
where
    F: Future,
{
    Timeout {
        future,
        delay: sleep(duration),
    }
}

pub struct Timeout<F> {
    future: F,
    delay: Sleep,
}

impl<F> Future for Timeout<F>
where
    F: Future,
{
    type Output = Result<F::Output, ()>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let this = self.get_unchecked_mut();
            
            if let Poll::Ready(output) = Pin::new_unchecked(&mut this.future).poll(cx) {
                return Poll::Ready(Ok(output));
            }
            
            if let Poll::Ready(()) = Pin::new_unchecked(&mut this.delay).poll(cx) {
                return Poll::Ready(Err(()));
            }
        }
        
        Poll::Pending
    }
}
