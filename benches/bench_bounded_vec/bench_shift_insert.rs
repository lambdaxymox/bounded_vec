use bounded_vec::BoundedVec;

use criterion::{
    Criterion,
    criterion_group,
};

fn bench_bounded_vec_shift_insert_last(c: &mut Criterion) {
    let dummy_data = 0_i32;

    c.bench_function("bounded_vec_shift_insert_last", |b| {
        b.iter(|| {
            let mut vec = BoundedVec::with_capacity(1024);
            for i in 0..1024 {
                vec.shift_insert(i, core::hint::black_box(dummy_data));
            }

            vec
        });
    });
}

criterion_group!(bench_shift_insert, bench_bounded_vec_shift_insert_last);
