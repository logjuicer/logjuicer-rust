// Copyright (C) 2022 Red Hat
// SPDX-License-Identifier: Apache-2.0

//! A tracing allocator to measure the maximum resident memory used.
//!
//! Code based on <https://fasterthanli.me/articles/small-strings-in-rust>

use std::alloc::{GlobalAlloc, System};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Counters {
    pub max: u64,
    pub count: u64,
    pub elapsed: std::time::Duration,
}

impl Counters {
    pub fn print(&self, msg: &str) {
        println!("{}:", msg);
        println!("          elapsed | {} µs", self.elapsed.as_micros());
        println!("     total events | {}", self.count);
        println!("      peak bytes  | {}", bytesize::ByteSize(self.max as _));
    }
}

pub struct Tracing {
    inner: System,
    max: AtomicU64,
    count: AtomicU64,
    current: AtomicU64,
}

impl Tracing {
    pub const fn new() -> Self {
        Self {
            inner: System,
            current: AtomicU64::new(0),
            max: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    pub fn run<F: FnOnce()>(&self, run_while_counting: F) -> Counters {
        let now = std::time::SystemTime::now();

        self.current.store(0, Ordering::SeqCst);
        self.count.store(0, Ordering::SeqCst);
        self.max.store(0, Ordering::SeqCst);

        run_while_counting();

        Counters {
            count: self.count.load(Ordering::SeqCst),
            max: self.max.load(Ordering::SeqCst),
            elapsed: now.elapsed().unwrap(),
        }
    }
}

unsafe impl GlobalAlloc for Tracing {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let size = layout.size() as u64;
        let current = self
            .current
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |prev| Some(prev + size))
            .map(|prev| prev + size)
            .unwrap();
        self.max.fetch_max(current, Ordering::SeqCst);
        self.count.fetch_add(1, Ordering::SeqCst);
        self.inner.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        self.current
            .fetch_sub(layout.size() as u64, Ordering::SeqCst);
        self.count.fetch_add(1, Ordering::SeqCst);
        self.inner.dealloc(ptr, layout)
    }
}
