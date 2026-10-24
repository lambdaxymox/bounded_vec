#![deny(unsafe_op_in_unsafe_fn)]
#![deny(private_interfaces)]
#![cfg_attr(feature = "nightly", feature(allocator_api))]
#![no_std]
extern crate alloc as alloc_crate;

mod bench_bounded_vec;

use criterion::criterion_main;

criterion_main!(
    bench_bounded_vec::bench_as_slice_index,
    bench_bounded_vec::bench_pop,
    bench_bounded_vec::bench_push,
    bench_bounded_vec::bench_replace_insert,
    bench_bounded_vec::bench_shift_insert,
    bench_bounded_vec::bench_shift_remove,
    bench_bounded_vec::bench_swap_remove,
);
