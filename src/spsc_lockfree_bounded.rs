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
use std::fmt;

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

  pub fn print_status(&self, op: String) {
    let read = self.read.load(Ordering::SeqCst);
    let write = self.write.load(Ordering::SeqCst);
    //println!("Inside print_status: {:?}", self); TODO: impl fmt::Debug
    println!("`{0}` at read:{1}, write:{2}", op, read, write);
  }

  pub fn push(&self, value: T) -> Result<usize, SPSCRingBufferError> {
    // TODO: Implement caching for writer index.
    let write = self.write.load(Ordering::Relaxed);
    let next_write = (write + 1) % self.capacity;

    if next_write == self.read.load(Ordering::Acquire) {
      return Err(SPSCRingBufferError::PushError(write)); // Buffer is full
    }

    self.print_status(format!("Push:"));
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

    self.print_status(format!("Pop:"));
    let value = unsafe { std::ptr::read(self.buffer[read].get()) };
    // Remove the use of `%` operator by using a mask.
    self.read.store((read + 1) % self.capacity, Ordering::Release);
    Some((read, value))
  }

  pub fn empty(&self) -> bool {
    self.read.load(Ordering::Relaxed) == self.write.load(Ordering::Relaxed)
  }
}

impl<T> fmt::Debug for SpscRingBuffer<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      self.buffer[..].fmt(f)
  }
}

pub fn empty(read_idx : usize, write_idx : usize) -> bool {
  read_idx == write_idx
}

#[cfg(test)]
mod tests {
  use super::*;
  use rand::Rng;
  use SPSCRingBufferError;

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
  fn push_and_pop() {
      let rb = SpscRingBuffer::new(8);
      for i in 0..7u64 {
          assert!(rb.push(i).is_ok());
      }
      assert_eq!(rb.read.load(Ordering::SeqCst), 0);
      assert_eq!(rb.write.load(Ordering::SeqCst), 7);
      assert!(rb.push(0).is_err());
      for _ in 0..7u64 {
        assert!(rb.pop().is_some());
      }
      assert!(rb.pop().is_none());
  }
  #[test]
  fn push_and_pop_at_random() {
      let rb = SpscRingBuffer::new(16);
      assert!(rb.push(0).is_ok());
      assert!(rb.push(1).is_ok());
      let mut rng = rand::thread_rng();

      for _ in 0..100 {
          let y: f64 = rng.gen();
          if y < 0.5 {
              rb.push(1);
          } else {
              if !rb.empty() {
                  assert!(rb.pop().is_some());
              }
          }
      }
  }
  #[test]
  fn spsc_ring_buffer() {
      const COUNT: u64 = 50;
      let t = AtomicUsize::new(1);
      let q: SpscRingBuffer<u64> = SpscRingBuffer::<u64>::new(16);
      let tracker: Vec<AtomicUsize> = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

      std::thread::scope(|scope| {
          // 
          scope.spawn(|| loop {
              match t.load(Ordering::SeqCst) {
                  0 if q.empty() => break,
                  _ => {
                      while let Some((idx, val)) = q.pop() {
                          tracker[idx].fetch_add(1, Ordering::SeqCst);
                      }
                  }
              }
          });

          scope.spawn(|| {
              // Keep pushing until the buffer is full
              for i in 0..COUNT {
                  match q.push(i) {
                      Ok(n) => {
                        tracker[n].fetch_add(1, Ordering::SeqCst);
                      },
                      Err(SPSCRingBufferError::PushError(x)) => {
                          println!("Error while pushing the value {i} at idx {x}");
                          std::thread::sleep(std::time::Duration::from_millis(1));
                          continue;
                      }
                      _ => {
                        panic!("Unexpected error");
                      }
                  }
              }

              // Signal the consumer to stop after COUNT iterations.
              t.fetch_sub(1, Ordering::SeqCst);
          });
      });

      for c in tracker {
          assert_eq!(c.load(Ordering::SeqCst), 1);
      }
  }
}