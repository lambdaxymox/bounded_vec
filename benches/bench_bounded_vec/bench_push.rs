use bounded_vec::BoundedVec;

use criterion::{
    Criterion,
    criterion_group,
};

#[cfg(feature = "nightly")]
use alloc_crate::alloc;

#[cfg(not(feature = "nightly"))]
use allocator_api2::alloc;

fn bench_bounded_vec_push(c: &mut Criterion) {
    let dummy_data = 0_i32;

    c.bench_function("type_bounded_vec_push", |b| {
        b.iter(|| {
            let mut vec = BoundedVec::with_capacity(1024);
            for _ in 0..1024 {
                vec.push(core::hint::black_box(dummy_data));
            }

            vec
        });
    });
}

criterion_group!(bench_push, bench_bounded_vec_push);
