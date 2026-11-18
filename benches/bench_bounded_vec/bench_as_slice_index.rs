use bounded_vec::BoundedVec;

use criterion::{
    Criterion,
    criterion_group,
};

fn bench_bounded_vec_as_slice_index(c: &mut Criterion) {
    let dummy_data = 0_i32;
    let mut vec = BoundedVec::with_capacity(10000);
    vec.extend((0..10000).map(|_| dummy_data));

    c.bench_function("bounded_vec_as_slice_index", |b| {
        b.iter(|| {
            let slice = vec.as_slice();
            for i in 0..slice.len() {
                let _ = core::hint::black_box(slice[i]);
            }
        });
    });
}

criterion_group!(bench_as_slice_index, bench_bounded_vec_as_slice_index);
