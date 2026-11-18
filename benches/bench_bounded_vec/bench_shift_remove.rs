use bounded_vec::BoundedVec;

use criterion::{
    Criterion,
    criterion_group,
};

fn bench_bounded_vec_shift_remove_last(c: &mut Criterion) {
    let dummy_data = 0_i32;

    c.bench_function("bounded_vec_shift_remove_last", |b| {
        b.iter_batched(
            || {
                let mut vec = BoundedVec::with_capacity(1000);
                vec.extend((0..1000).map(|_| dummy_data));
                vec
            },
            |mut vec| {
                for _ in 0..vec.len() {
                    let last_index = vec.len() - 1;
                    let _ = core::hint::black_box(vec.shift_remove(last_index));
                }
            },
            criterion::BatchSize::NumIterations(1000),
        );
    });
}

criterion_group!(bench_shift_remove, bench_bounded_vec_shift_remove_last);
