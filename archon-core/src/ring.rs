#![allow(dead_code)]
#![allow(unused_variables)]

use core::sync::atomic::AtomicBool;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

pub const LOAD_ORDER: Ordering = Ordering::Acquire;
pub const STORE_ORDER: Ordering = Ordering::Release;
pub const FETCH_ORDER: Ordering = Ordering::Release;

pub struct RingBuffer<T, const SIZE: usize> {
    buffer: [Option<T>; SIZE],
    head: usize,
    tail: usize,
    is_full: bool,
}

impl<T, const SIZE: usize> RingBuffer<T, SIZE> {
    pub const fn new() -> Self {
        assert!(SIZE > 0, "RingBuffer size must be greater than 0");
        RingBuffer {
            buffer: [const { None }; SIZE],
            head: 0,
            tail: 0,
            is_full: false,
        }
    }

    pub fn add(&mut self, item: T) {
        self.buffer[self.head] = Some(item);
        self.head = (self.head + 1) % SIZE;

        if self.is_full {
            self.tail = (self.tail + 1) % SIZE;
        }

        self.is_full = self.head == self.tail;
    }

    pub fn take(&mut self) -> Option<T> {
        if self.head == self.tail && !self.is_full {
            return None;
        }

        let item: Option<T> = self.buffer[self.tail].take();
        self.tail = (self.tail + 1) % SIZE;
        self.is_full = false;

        item
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.is_full = false;

        for i in 0..SIZE {
            self.buffer[i] = None;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail && !self.is_full
    }

    pub fn is_full(&self) -> bool {
        self.is_full
    }
}

pub struct AtomicRingBuffer<T, const SIZE: usize> {
    buffer: [Option<T>; SIZE],
    head: AtomicUsize,
    tail: AtomicUsize,
    is_full: AtomicBool,
}

impl<T, const SIZE: usize> AtomicRingBuffer<T, SIZE> {
    fn head(&self) -> usize {
        self.head.load(LOAD_ORDER)
    }

    fn tail(&self) -> usize {
        self.tail.load(LOAD_ORDER)
    }

    fn set_head(&self, val: usize) {
        self.head.store(val, STORE_ORDER);
    }

    fn set_tail(&self, val: usize) {
        self.tail.store(val, STORE_ORDER);
    }

    fn set_is_full(&self, val: bool) {
        self.is_full.store(val, STORE_ORDER);
    }

    fn head_add(&self) -> usize {
        let head: usize = self.head() + 1;
        self.set_head(head);
        head
    }

    fn tail_add(&self) -> usize {
        let tail: usize = self.tail() + 1;
        self.set_tail(tail);
        tail
    }
}

impl<T, const SIZE: usize> AtomicRingBuffer<T, SIZE> {
    pub const fn new() -> Self {
        assert!(SIZE > 0, "AtomicRingBuffer size must be greater than 0");
        AtomicRingBuffer {
            buffer: [const { None }; SIZE],
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            is_full: AtomicBool::new(false),
        }
    }

    pub fn add(&mut self, item: T) {
        self.buffer[self.head()] = Some(item);
        self.set_head(self.head_add() % SIZE);

        if self.is_full() {
            self.set_tail(self.tail_add() % SIZE);
        }

        self.set_is_full(self.head() == self.tail());
    }

    pub fn take(&mut self) -> Option<T> {
        if self.head() == self.tail() && !self.is_full() {
            return None;
        }

        let item: Option<T> = self.buffer[self.tail()].take();
        self.set_tail(self.tail_add() % SIZE);
        self.set_is_full(false);

        item
    }

    pub fn clear(&mut self) {
        self.set_head(0);
        self.set_tail(0);
        self.set_is_full(false);

        for i in 0..SIZE {
            self.buffer[i] = None;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head() == self.tail() && !self.is_full()
    }

    pub fn is_full(&self) -> bool {
        self.is_full.load(LOAD_ORDER)
    }
}

impl<T, const SIZE: usize> AtomicRingBuffer<T, SIZE>
where
    T: Clone,
{
    pub fn take_clone(&self) -> Option<T> {
        if self.head() == self.tail() && !self.is_full() {
            return None;
        }

        let item: Option<T> = self.buffer[self.tail()].clone();
        self.set_tail(self.tail_add() % SIZE);
        self.set_is_full(false);

        item
    }
}
