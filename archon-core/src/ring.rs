#![allow(dead_code)]
#![allow(unused_variables)]

pub struct RingBuffer<T, const SIZE: usize> {
    buffer: [Option<T>; SIZE],
    head: usize,
    tail: usize,
    is_full: bool,
}

impl<T, const SIZE: usize> RingBuffer<T, SIZE> {
    pub fn new() -> Self {
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
