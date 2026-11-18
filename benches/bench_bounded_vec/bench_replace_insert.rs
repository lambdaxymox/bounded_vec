use bounded_vec::BoundedVec;

use criterion::{
    Criterion,
    criterion_group,
};

fn bench_bounded_vec_replace_insert_last(c: &mut Criterion) {
    let dummy_data = 0_i32;

    c.bench_function("bounded_vec_replace_insert_last", |b| {
        b.iter(|| {
            let mut vec = BoundedVec::with_capacity(1024);
            for _ in 0..1024 {
                let last_index = if vec.len() == 0 { 0 } else { vec.len() - 1 };
                vec.replace_insert(last_index, core::hint::black_box(dummy_data));
            }

            vec
        });
    });
}

criterion_group!(bench_replace_insert, bench_bounded_vec_replace_insert_last);
