/// The ring buffer implementation that supports Single Producer and Single Consumer.
/// The ring buffer is a FIFO data structure that uses a single,
/// fixed-size buffer as if it were connected end-to-end.

/// NOTE:
/// 1. It 'relies' on the fact that the buffer is power of 2.
/// https://www.snellman.net/blog/archive/2016-12-13-ring-buffers/
/// https://lkml.iu.edu/hypermail/linux/kernel/0409.1/2709.html
/// 2. The implementation is not thread-safe.

use std::{fmt, ops::Add};
use thiserror::Error;
use log::debug;

#[derive(Error, Debug)]
pub enum SPSCRingBufferError {
    #[error("Error while pushing the value: {0}")]
    PushError(u64),
    #[error("Error while popping at head: {0}")]
    PopError(u64)
}

/// FIFO ring buffer with Single Producer and Single Consumer.
pub struct SPSCRingBuffer {
    head: u64, // From where we will **pop** the next value.
    tail: u64, // To where we will **push** the next value.
    buffer: Vec<u64>,
}

impl SPSCRingBuffer {
    pub fn new(cap: usize) -> Self {
        let head = 0;
        let tail = 0;
        let buffer = vec!(0; cap);
        if (cap & (cap - 1)) != 0 {
            debug!("Capacity not a power of 2, may not work correctly.");
        }
        Self {
            head,
            tail,
            buffer
        }
    }
    pub fn print_status(&self, op: String) {
        println!("Inside print_status: {:?}", self);
        println!("`{0}` at head:{1}, tail:{2}", op, self.head, self.tail);
    }
    pub fn push(&mut self, v: u64) -> bool {
        dbg!(self.print_status(format!("Push Before: {v}")));
        if !self.full() {
            let idx = self.tail as usize % self.buffer.capacity();
            self.buffer[idx] = v;
            self.tail = self.fold(self.tail + 1);
            dbg!(self.print_status(format!("Push After: {v}")));
            true
        } else {
            false
        }
    }
    /// Forcefully pushes a value into the ring buffer.
    /// If the buffer is full, it will overwrite the oldest value.
    pub fn force_push(&mut self, v: u64) {
        dbg!(self.print_status(format!("Force Push Before: {v}")));
        let idx = self.tail as usize % self.buffer.capacity();
        self.buffer[idx] = v;
        self.tail = self.fold(self.tail + 1);
        dbg!(self.print_status(format!("Force Push After: {v}")));
    }
    /// Pops a value from the ring buffer.
    /// Returns an error if the buffer is empty.
    pub fn pop(&mut self) -> Result<u64, SPSCRingBufferError> {
        let idx = self.head as usize % self.buffer.capacity();
        let v = self.buffer[idx];
        dbg!(self.print_status(format!("Pop {v}")));
        if self.empty() {
            Err(SPSCRingBufferError::PopError(self.tail))
        } else {
            // For debugging purpose.
            dbg!(self.buffer[idx] = 25);
            self.head = self.fold(self.head + 1);
            Ok(v)
        }
    }
    pub fn full(&self) -> bool {
        self.free() == 0
    }
    pub fn empty(&self) -> bool {
        self.tail == self.head
    }
    /// Returns the capacity (maximum number of elements that
    /// can be allocated) of the ring buffer.
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
    pub fn distance(&self) -> u64 {
        if self.tail >= self.head {
            self.tail - self.head
        } else {
            self.tail + 2*self.buffer.capacity() as u64 - self.head
        }
    }
    /// Returns the number of elements in the ring buffer.
    pub fn size(&self) -> usize {
        self.fold(self.distance()) as usize
    }
    /// Returns the number of free slots in the ring buffer.
    pub fn free(&self) -> usize {
        self.buffer.capacity() - self.size()
    }
    fn fold(&self, val: u64) -> u64 {
        // See dizzy57's answer on https://www.snellman.net/blog/archive/2016-12-13-ring-buffers/
        val % (2*self.buffer.capacity()) as u64
    }
}

impl fmt::Debug for SPSCRingBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.buffer[..].fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn create() {
        let rb : SPSCRingBuffer = SPSCRingBuffer::new(10);
        assert_eq!(rb.capacity(), 10);
    }
    #[test]
    fn push() {
        let mut rb : SPSCRingBuffer = SPSCRingBuffer::new(10);
        for i in 0..10 {
            assert!(rb.push(i));
        }
    }
    #[test]
    fn force_push() {
        let mut rb : SPSCRingBuffer = SPSCRingBuffer::new(8);
        for i in 0..97 {
            rb.force_push(i);
        }
        assert_eq!(rb.pop().unwrap(), 96);
    }
    #[test]
    fn force_push_and_pop() {
        let mut rb = SPSCRingBuffer::new(16);
        for i in 0..10 {
            rb.force_push(i+2);
        }
        for _ in 0..10 {
            assert!(rb.pop().is_ok());
        }
        assert_eq!(rb.free(), 16);
    }
    #[test]
    fn push_and_pop() {
        let mut rb = SPSCRingBuffer::new(16);
        for i in 0..10 {
            assert!(rb.push(i));
        }
        for _ in 0..10 {
            assert!(rb.pop().is_ok());
        }
    }
    #[test]
    fn push_and_pop_at_random() {
        let mut rb = SPSCRingBuffer::new(16);
        rb.push(0);
        rb.push(1);
        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let y: f64 = rng.gen();
            if y < 0.5 {
                if !rb.full() {
                    rb.push(1);
                }
            } else {
                if !rb.empty() {
                    assert!(rb.pop().is_ok());
                }
            }
        }
    }
    #[test]
    fn check_size() {
        let mut rb : SPSCRingBuffer = SPSCRingBuffer::new(16);
        for i in 0..11 {
            rb.force_push(i);
        }
        assert_eq!(rb.size(), 11);
        assert_eq!(rb.free(), rb.capacity() - rb.size());
    }
}
