/// A lock-free single-producer single-consumer (SPSC) bounded ring buffer.

/// This implementation uses atomic operations to manage the head and tail indices
/// of the buffer, ensuring that the producer and consumer can operate concurrently
/// without the need for locks. The buffer has a fixed capacity, and attempts to
/// push to a full buffer or pop from an empty buffer will fail gracefully.
/// Example
///  

/// Remember this is SPSC (One producer running in some sort of loop,
/// and same for the consumer). So the producer and consumer
/// only need to sync with each other.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SPSCRingBufferError {
    #[error("Error while pushing the value: {0}")]
    PushError(usize),
    #[error("Error while popping at head: {0}")]
    PopError(usize)
}

pub struct SpscRingBuffer<T> {
  buffer: Vec<UnsafeCell<T>>,
  capacity: usize,
  write: AtomicUsize,
  read: AtomicUsize,
}

unsafe impl<T: Send> Sync for SpscRingBuffer<T> {}

impl<T> SpscRingBuffer<T> {
  pub fn new(capacity: usize) -> Self {
    let mut buffer = Vec::with_capacity(capacity);
    for _ in 0..capacity {
      buffer.push(UnsafeCell::new(unsafe { std::mem::zeroed() }));
    }
    SpscRingBuffer {
      buffer,
      capacity,
      write: AtomicUsize::new(0),
      read: AtomicUsize::new(0),
    }
  }

  pub fn push(&self, value: T) -> Result<usize, SPSCRingBufferError> {
    // TODO: Implement caching for writer index.
    let write = self.write.load(Ordering::Relaxed);
    let next_write = (write + 1) % self.capacity;

    if next_write == self.read.load(Ordering::Acquire) {
      return Err(SPSCRingBufferError::PushError(write)); // Buffer is full
    }

    unsafe {
      *self.buffer[write].get() = value;
    }
    self.write.store(next_write, Ordering::Release);
    Ok(write)
  }

  pub fn pop(&self) -> Option<(usize, T)> {
    // TODO: Implement caching for reader index.
    let read = self.read.load(Ordering::Relaxed);
    let write = self.write.load(Ordering::Acquire);

    if empty(read, write) {
      return None;
    }

    let value = unsafe { std::ptr::read(self.buffer[read].get()) };
    // Remove the use of `%` operator by using a mask.
    self.read.store((read + 1) % self.capacity, Ordering::Release);
    Some((read, value))
  }

  pub fn empty(&self) -> bool {
    self.read.load(Ordering::Relaxed) == self.write.load(Ordering::Relaxed)
  }
}

pub fn empty(read_idx : usize, write_idx : usize) -> bool {
  read_idx == write_idx
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_spsc_ring_buffer() {
    let buffer: SpscRingBuffer<u64> = SpscRingBuffer::<u64>::new(3);

    assert!(buffer.push(1).is_ok());
    assert!(buffer.push(2).is_ok());
    assert!(buffer.push(3).is_err()); // Buffer should be full

    assert_eq!(buffer.pop(), Some((0, 1)));
    assert_eq!(buffer.pop(), Some((1, 2)));
    assert_eq!(buffer.pop(), None); // Buffer should be empty
  }
  #[test]
  fn spsc_ring_buffer() {
      const COUNT: u64 = 50;
      let t = AtomicUsize::new(1);
      let q: SpscRingBuffer<u64> = SpscRingBuffer::<u64>::new(3);
      let v: Vec<AtomicUsize> = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

      std::thread::scope(|scope| {
          scope.spawn(|| loop {
              match t.load(Ordering::SeqCst) {
                  0 if q.empty() => break,
                  _ => {
                      while let Some((idx, val)) = q.pop() {
                          v[idx].fetch_add(1, Ordering::SeqCst);
                      }
                  }
              }
          });

          scope.spawn(|| {
              for i in 0..COUNT {
                  match q.push(i) {
                      Ok(n) => {
                        v[n].fetch_add(1, Ordering::SeqCst);
                      },
                      Err(_) => {
                          println!("Error while pushing the value");
                          std::thread::sleep(std::time::Duration::from_millis(1));
                          continue;
                      }
                  }
              }

              t.fetch_sub(1, Ordering::SeqCst);
          });
      });

      for c in v {
          assert_eq!(c.load(Ordering::SeqCst), 1);
      }
  }
}