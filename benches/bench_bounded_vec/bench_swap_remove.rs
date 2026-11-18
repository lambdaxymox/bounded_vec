use bounded_vec::BoundedVec;

use criterion::{
    Criterion,
    criterion_group,
};

fn bench_bounded_vec_swap_remove(c: &mut Criterion) {
    let dummy_data = 0_i32;

    c.bench_function("bounded_vec_swap_remove", |b| {
        b.iter_batched(
            || {
                let mut vec = BoundedVec::with_capacity(1000);
                vec.extend((0..1000).map(|_| dummy_data));
                vec
            },
            |mut vec| {
                for _ in 0..vec.len() {
                    let _ = core::hint::black_box(vec.swap_remove(0));
                }
            },
            criterion::BatchSize::NumIterations(1000),
        );
    });
}

criterion_group!(bench_swap_remove, bench_bounded_vec_swap_remove);
