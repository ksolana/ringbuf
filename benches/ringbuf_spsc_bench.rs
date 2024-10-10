use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ringbuf::spsc_array::SPSCRingBuffer;

fn create(n: u64) {
    for _ in 0..n {
        let rb : SPSCRingBuffer = SPSCRingBuffer::new(10);
        assert_eq!(rb.capacity(), 10);
        assert_eq!(rb.size(), 0);
        assert_eq!(rb.free(), rb.capacity() - rb.size() -1);
    }
}

fn push(n: u64) {
    for _ in 0..n {
        let mut rb : SPSCRingBuffer = SPSCRingBuffer::new(8);
        for i in 0..7 {
            assert!(rb.push(i));
        }
        assert_eq!(rb.size(), 7);
        assert_eq!(rb.free(), rb.capacity() - rb.size() -1);
    }
}

fn spsc_create(c: &mut Criterion) {
    c.bench_function("create 20", |b| b.iter(|| create(black_box(20))));
}

fn spsc_push(c: &mut Criterion) {
    c.bench_function("push 20", |b| b.iter(|| push(black_box(20))));
}

criterion_group!(benches, spsc_create, spsc_push);
criterion_main!(benches);