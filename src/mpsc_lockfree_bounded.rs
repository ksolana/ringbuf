use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct RingBuffer<T> {
  buffer: Vec<UnsafeCell<T>>,
  capacity: usize,
  write: AtomicUsize,
  read: AtomicUsize,
}

unsafe impl<T: Send> Sync for RingBuffer<T> {}

impl<T> RingBuffer<T> {
  pub fn new(capacity: usize) -> Arc<Self> {
    let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(UnsafeCell::new(unsafe { std::mem::zeroed() }));
    }
    Arc::new(Self {
      buffer,
      capacity,
      write: AtomicUsize::new(0),
      read: AtomicUsize::new(0),
    })
  }

  pub fn try_push(&self, item: T) -> Result<(), T> {
    let head = self.write.load(Ordering::Relaxed);
    let next_head = (head + 1) % self.capacity;

    if next_head == self.read.load(Ordering::Acquire) {
      return Err(item); // Buffer is full
    }

    unsafe {
      *self.buffer[head].get() = item;
    }
    //self.buffer[head] = item;
    self.write.store(next_head, Ordering::Release);
    Ok(())
  }

  pub fn pop(&self) -> Option<T> {
    let tail = self.read.load(Ordering::Relaxed);

    if tail == self.write.load(Ordering::Acquire) {
      return None; // Buffer is empty
    }

    let item = unsafe { std::ptr::read(self.buffer[tail].get()) };

    self.read.store((tail + 1) % self.capacity, Ordering::Release);
    Some(item)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::thread;

  #[test]
  fn test_ring_buffer() {
    let buffer = RingBuffer::new(3);
    let buffer_clone = Arc::clone(&buffer);

    let producer = thread::spawn(move || {
      for i in 0..3 {
        buffer_clone.try_push(i).unwrap();
      }
    });

    let consumer = thread::spawn(move || {
      for _ in 0..3 {
        while let Some(item) = buffer.pop() {
          println!("Consumed: {}", item);
        }
      }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
  }
}