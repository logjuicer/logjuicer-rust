// Copyright (C) 2022 Red Hat
// SPDX-License-Identifier: Apache-2.0
#![allow(unused_imports)]
use logreduce_allocator::Tracing;

#[global_allocator]
pub static ALLOCATOR: Tracing = Tracing::new();

fn test_bytes_lines(r: &mut dyn std::io::Read) {
    // let mut v = Vec::new();
    for bytes in logreduce_iterator::BytesLines::new(r) {
        let (bytes, nr) = bytes.unwrap();
        let s: &str = std::str::from_utf8(&bytes[..]).unwrap();
        println!("{} | {}", nr, s);
        //        v.push(line)
        // println!("{}", String::from_utf8(line.unwrap().to_vec()).unwrap());
    }
}

pub fn main() {
    ALLOCATOR
        .run(|| {
            vec![false, false, true];
        })
        .print("vector of three bool");
    let mut s = String::with_capacity(65536);
    let mut line_count = 0;
    let mut count = 1;
    for _ in 0..10 {
        if line_count == 3 {
            for _ in 0..308 {
                s.push('A');
            }
            s.push('\n');
            line_count += 1;
        }
        if line_count == 5 {
            s.push_str("a line\\nanother\\nline\n");
            line_count += 1;
        }
        for _ in 0..128 {
            s.push(std::char::from_u32(33 + count % 90).unwrap());
            count += 1;
        }
        line_count += 1;
        s.push('\n');
    }
    for _ in 0..308 {
        s.push('A');
    }
    s.push('\n');
    println!("s size: {} (lines {})", s.len(), line_count);
    ALLOCATOR
        .run(|| test_bytes_lines(&mut std::io::Cursor::new(&s)))
        .print("bytes_lines");
}
