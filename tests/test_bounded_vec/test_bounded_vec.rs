use bounded_vec::BoundedVec;

use allocator_api2::alloc;

#[test]
fn test_bounded_vec_empty() {
    let vec: BoundedVec<i32, alloc::Global> = BoundedVec::with_capacity_in(3, alloc::Global);

    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 3);
}

#[test]
fn test_bounded_vec_capacity_zst() {
    let vec: BoundedVec<i32, alloc::Global> = BoundedVec::with_capacity_in(3, alloc::Global);

    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 3);
}

#[test]
fn test_bounded_vec_push_index1() {
    let capacity = 3;
    let mut vec = BoundedVec::with_capacity_in(capacity, alloc::Global);

    vec.push(1_i32);
    vec.push(2_i32);
    vec.push(3_i32);

    assert_eq!(vec[0], 1_i32);
    assert_eq!(vec[1], 2_i32);
    assert_eq!(vec[2], 3_i32);

    assert_eq!(vec.len(), capacity);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_push_index2() {
    let capacity = 64;
    let mut vec = BoundedVec::with_capacity_in(capacity, alloc::Global);
    let len = vec.capacity();

    for i in 0..len {
        vec.push(i);
    }

    for i in 0..len {
        assert_eq!(vec[i], i);
    }

    assert_eq!(vec.len(), capacity);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_push_index3() {
    let capacity = 32;
    let mut vec = BoundedVec::with_capacity_in(capacity, alloc::Global);
    let len = vec.capacity();

    for _ in 0..len {
        vec.push(usize::MAX);
    }

    for i in 0..len {
        assert_eq!(vec[i], usize::MAX);
    }

    assert_eq!(vec.len(), capacity);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_push_len1() {
    let capacity = 3;
    let mut vec = BoundedVec::with_capacity_in(capacity, alloc::Global);

    vec.push(1_i32);
    vec.push(2_i32);
    vec.push(3_i32);

    assert_eq!(vec.len(), capacity);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_push_len2() {
    let capacity = 64;
    let mut vec = BoundedVec::with_capacity_in(capacity, alloc::Global);

    for i in 0..capacity {
        vec.push(i);
    }

    assert_eq!(vec.len(), capacity);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_push_len3() {
    let capacity = 32;
    let mut vec = BoundedVec::with_capacity_in(capacity, alloc::Global);
    let len = vec.capacity();

    for _ in 0..len {
        vec.push(usize::MAX);
    }

    assert_eq!(vec.len(), capacity);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_push_zst_capacity() {
    for capacity in [0, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024] {
        let vec: BoundedVec<(), alloc::Global> = BoundedVec::with_capacity_in(capacity, alloc::Global);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), capacity);
    }
}

#[test]
fn test_bounded_vec_pop_empty() {
    let capacity = 3;
    let mut vec: BoundedVec<i32, alloc::Global> = BoundedVec::with_capacity_in(capacity, alloc::Global);

    assert_eq!(vec.pop(), None);

    for _ in 0..32 {
        assert_eq!(vec.pop(), None);
    }

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_pop1() {
    let capacity = 3;
    let mut vec: BoundedVec<i32, alloc::Global> = BoundedVec::with_capacity_in(capacity, alloc::Global);

    vec.push(1_i32);
    vec.push(2_i32);
    vec.push(3_i32);

    assert_eq!(vec.len(), capacity);

    assert_eq!(vec.pop(), Some(3_i32));
    assert_eq!(vec.pop(), Some(2_i32));
    assert_eq!(vec.pop(), Some(1_i32));
    assert_eq!(vec.pop(), None);

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_pop2() {
    let capacity = 64;
    let mut vec = BoundedVec::with_capacity_in(capacity, alloc::Global);
    let len = vec.capacity();

    for i in 0..len {
        vec.push(i);
    }

    for i in 0..len {
        assert_eq!(vec.pop(), Some(len - (i + 1)));
    }

    assert_eq!(vec.pop(), None);

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_pop3() {
    let capacity = 32;
    let mut vec = BoundedVec::with_capacity_in(capacity, alloc::Global);
    let len = vec.capacity();

    for _ in 0..len {
        vec.push(usize::MAX);
    }

    for _ in 0..len {
        assert_eq!(vec.pop(), Some(usize::MAX));
    }

    assert_eq!(vec.pop(), None);

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_pop_len1() {
    let mut vec = BoundedVec::with_capacity_in(3, alloc::Global);

    vec.push(1_i32);
    vec.push(2_i32);
    vec.push(3_i32);

    assert_eq!(vec.len(), 3);
    assert_eq!(vec.capacity(), 3);

    let _ = vec.pop();

    assert_eq!(vec.len(), 2);
    assert_eq!(vec.capacity(), 3);

    let _ = vec.pop();

    assert_eq!(vec.len(), 1);
    assert_eq!(vec.capacity(), 3);

    let _ = vec.pop();

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 3);
}

#[test]
fn test_bounded_vec_pop_len2() {
    let capacity = 64;
    let mut vec = BoundedVec::with_capacity_in(capacity, alloc::Global);
    let len = vec.capacity();

    for i in 0..len {
        vec.push(i);
    }

    for i in 0..len {
        let _ = vec.pop();

        assert_eq!(vec.len(), len - (i + 1));
    }

    let _ = vec.pop();

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_pop_len3() {
    let capacity = 32;
    let mut vec = BoundedVec::with_capacity_in(capacity, alloc::Global);
    let len = vec.capacity();

    for i in 0..len {
        vec.push(i);
    }

    for i in 0..len {
        let _ = vec.pop();

        assert_eq!(vec.len(), len - (i + 1));
    }

    let _ = vec.pop();

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), capacity);
}

#[test]
fn test_bounded_vec_clear1() {
    let mut vec: BoundedVec<i32, alloc::Global> = BoundedVec::with_capacity_in(3, alloc::Global);

    assert_eq!(vec.len(), 0);
    assert!(vec.is_empty());
    assert_eq!(vec.capacity(), 3);

    vec.clear();

    assert_eq!(vec.len(), 0);
    assert!(vec.is_empty());
    assert_eq!(vec.capacity(), 3);
}

#[test]
fn test_bounded_vec_clear2() {
    let mut vec: BoundedVec<i32, alloc::Global> = BoundedVec::from_iter([444_i32, 127_i32, 780_i32, 59_i32, 920_i32, 496_i32]);

    assert_eq!(vec.len(), 6);
    assert!(!vec.is_empty());
    assert_eq!(vec.capacity(), 6);

    for value in vec.iter() {
        assert!(vec.contains(value));
    }

    let vec_before_clear = vec.clone();
    vec.clear();

    assert_eq!(vec.len(), 0);
    assert!(vec.is_empty());
    assert_eq!(vec.capacity(), 6);

    for value in vec_before_clear.iter() {
        assert!(!vec.contains(value));
    }
}

#[test]
fn test_bounded_vec_clone1() {
    let vec: BoundedVec<isize, alloc::Global> = BoundedVec::with_capacity_in(3, alloc::Global);
    let cloned_vec = vec.clone();

    assert_eq!(vec.len(), cloned_vec.len());
    assert_eq!(vec.capacity(), cloned_vec.capacity());
    assert_eq!(vec, cloned_vec);
}

#[test]
fn test_bounded_vec_clone2() {
    let mut vec: BoundedVec<isize, alloc::Global> = BoundedVec::with_capacity_in(16, alloc::Global);
    for i in 0..8 {
        vec.push_within_capacity(i as isize).unwrap();
    }

    let cloned_vec = vec.clone();

    assert_eq!(vec.len(), cloned_vec.len());
    assert_eq!(vec.capacity(), cloned_vec.capacity());
    assert_eq!(vec, cloned_vec);
}

#[test]
fn test_bounded_vec_clone3() {
    let mut vec: BoundedVec<isize, alloc::Global> = BoundedVec::with_capacity_in(15, alloc::Global);
    for i in 0..3 {
        vec.push(i);
    }

    for i in 0..10 {
        vec.push_within_capacity(i as isize).unwrap();
    }

    let cloned_vec = vec.clone();

    assert_eq!(vec.len(), cloned_vec.len());
    assert_eq!(vec.capacity(), cloned_vec.capacity());
    assert_eq!(vec, cloned_vec);
}

#[test]
fn test_bounded_vec_slice_from_ref() {
    let vec = BoundedVec::from(&[1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);

    assert_eq!(&vec[0..1], &[1_i32]);
    assert_eq!(&vec[1..2], &[2_i32]);
    assert_eq!(&vec[2..3], &[3_i32]);
    assert_eq!(&vec[3..4], &[4_i32]);
    assert_eq!(&vec[4..5], &[5_i32]);
    assert_eq!(&vec[5..6], &[6_i32]);

    assert_eq!(&vec[0..2], &[1_i32, 2_i32]);
    assert_eq!(&vec[1..3], &[2_i32, 3_i32]);
    assert_eq!(&vec[2..4], &[3_i32, 4_i32]);
    assert_eq!(&vec[3..5], &[4_i32, 5_i32]);
    assert_eq!(&vec[4..6], &[5_i32, 6_i32]);

    assert_eq!(&vec[0..3], &[1_i32, 2_i32, 3_i32]);
    assert_eq!(&vec[1..4], &[2_i32, 3_i32, 4_i32]);
    assert_eq!(&vec[2..5], &[3_i32, 4_i32, 5_i32]);
    assert_eq!(&vec[3..6], &[4_i32, 5_i32, 6_i32]);

    assert_eq!(&vec[0..4], &[1_i32, 2_i32, 3_i32, 4_i32]);
    assert_eq!(&vec[1..5], &[2_i32, 3_i32, 4_i32, 5_i32]);
    assert_eq!(&vec[2..6], &[3_i32, 4_i32, 5_i32, 6_i32]);

    assert_eq!(&vec[0..5], &[1_i32, 2_i32, 3_i32, 4_i32, 5_i32]);
    assert_eq!(&vec[1..6], &[2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);

    assert_eq!(&vec[0..6], &[1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    assert_eq!(&vec[..], &[1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    assert_eq!(vec.as_slice(), &[1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
}

#[test]
fn test_bounded_vec_slice_from_mut1() {
    let mut vec = BoundedVec::from(&[1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    {
        let slice = &mut vec[..2];

        assert_eq!(slice, &[1_i32, 2_i32]);

        for i in 0..slice.len() {
            slice[i] += 20_i32;
        }
    }

    assert_eq!(vec.as_slice(), &[21_i32, 22_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
}

#[test]
fn test_bounded_vec_slice_from_mut2() {
    let mut vec = BoundedVec::from(&[1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    {
        let slice = &mut vec[2..];

        assert_eq!(slice, &[3_i32, 4_i32, 5_i32, 6_i32]);

        for i in 0..slice.len() {
            slice[i] += 20_i32;
        }
    }

    assert_eq!(vec.as_slice(), &[1_i32, 2_i32, 23_i32, 24_i32, 25_i32, 26_i32]);
}

#[test]
fn test_bounded_vec_indexing() {
    let vec = BoundedVec::from_iter([10_i32, 20_i32, 30_i32]);

    assert_eq!(vec[0], 10_i32);
    assert_eq!(vec[1], 20_i32);
    assert_eq!(vec[2], 30_i32);

    let mut idx = 0;

    assert_eq!(vec[idx], 10_i32);
    idx += 1;
    assert_eq!(vec[idx], 20_i32);
    idx += 1;
    assert_eq!(vec[idx], 30_i32);
}

#[test]
#[should_panic]
fn test_bounded_vec_indexing_out_of_bounds1() {
    let vec: BoundedVec<i32, alloc::Global> = BoundedVec::with_capacity_in(3, alloc::Global);
    let _ = vec[0];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_indexing_out_of_bounds2() {
    let vec: BoundedVec<i32, alloc::Global> = BoundedVec::from_iter([10_i32]);
    let _ = vec[1];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_indexing_out_of_bounds3() {
    let vec: BoundedVec<i32, alloc::Global> = BoundedVec::from_iter([10_i32, 20_i32]);
    let _ = vec[2];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_indexing_out_of_bounds4() {
    let vec: BoundedVec<i32, alloc::Global> = BoundedVec::from_iter([10_i32, 20_i32, 30_i32]);
    let _ = vec[3];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_slice_out_of_bounds1() {
    let vec = BoundedVec::from_iter([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    let _ = &vec[!0..];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_slice_out_of_bounds2() {
    let vec = BoundedVec::from_iter([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    let _ = &vec[..7];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_slice_out_of_bounds3() {
    let vec = BoundedVec::from_iter([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    let _ = &vec[!0..5];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_slice_out_of_bounds4() {
    let vec = BoundedVec::from_iter([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    let _ = &vec[1..7];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_slice_out_of_bounds5() {
    let vec = BoundedVec::from_iter([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    let _ = &vec[3..2];

    assert!(true);
}
