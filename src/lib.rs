use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SPSCRingBufferError {
    #[error("Error while pushing the value: {0}")]
    PushError(u64),
    #[error("Error while popping at head: {0}")]
    PopError(u64)
}

pub struct SPSCRingBuffer {
    head: u64,
    tail: u64,
    buffer: Vec<u64>,
}

impl SPSCRingBuffer {
    pub fn new(cap: usize) -> Self {
        let head = 0;
        let tail = 0;
        let buffer = Vec::with_capacity(cap);
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
        dbg!(self.print_status(format!("Push {v}")));
        if !self.full() {
            self.buffer.push(v);
            self.tail = (self.tail + 1) % self.buffer.capacity() as u64;
            true
        } else {
            false
        }
    }
    pub fn pop(&mut self) -> Result<u64, SPSCRingBufferError> {
        let v = self.buffer[self.head as usize];
        dbg!(self.print_status(format!("Pop {v}")));
        if self.empty() {
            Err(SPSCRingBufferError::PopError(self.tail))
        } else {
            self.head = (self.head + 1) % self.buffer.capacity() as u64;
            Ok(v)
        }
    }
    pub fn full(&self) -> bool {
        (self.tail + 1) % self.buffer.capacity() as u64 == self.head
    }
    pub fn empty(&self) -> bool {
        self.head == self.tail
    }
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
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
    fn push_and_pop() {
        let mut rb = SPSCRingBuffer::new(10);
        for i in 0..10 {
            assert!(rb.push(i));
        }
        for _ in 0..10 {
            assert!(rb.pop().is_ok());
        }
    }
    #[test]
    fn push_and_pop_at_random() {
        let mut rb = SPSCRingBuffer::new(10);
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
}
