use bounded_vec::BoundedVec;

use criterion::{
    Criterion,
    criterion_group,
};

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
