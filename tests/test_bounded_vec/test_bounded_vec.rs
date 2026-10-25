use bounded_vec::BoundedVec;

#[cfg(feature = "nightly")]
use std::alloc;

#[cfg(not(feature = "nightly"))]
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
fn test_bounded_vec_extend1() {
    let mut vec = BoundedVec::with_capacity(10);

    assert_eq!(vec.as_slice(), &[]);

    vec.extend([1_i32, 2_i32, 3_i32]);

    assert_eq!(vec.as_slice(), [1_i32, 2_i32, 3_i32]);
}

#[test]
fn test_bounded_vec_extend2() {
    let mut vec1 = BoundedVec::with_capacity(16);

    for i in 0..16 {
        vec1.push(i);
    }

    let mut vec2 = BoundedVec::with_capacity(16);
    vec2.extend(0..16);

    assert_eq!(vec1, vec2);
}

#[test]
fn test_bounded_vec_extend3() {
    let mut vec1 = BoundedVec::with_capacity(16);
    let mut vec2 = BoundedVec::with_capacity(16);

    for i in 0..3 {
        vec1.push(i);
    }

    vec1.extend(3..16);
    vec2.extend(0..16);

    assert_eq!(vec1, vec2);
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
    let mut vec: BoundedVec<i32, alloc::Global> = BoundedVec::from([444_i32, 127_i32, 780_i32, 59_i32, 920_i32, 496_i32]);

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
fn test_bounded_vec_dedup() {
    fn test_case(this: BoundedVec<i32>, that: BoundedVec<i32>) {
        let mut vec = this;
        vec.dedup();
        assert_eq!(vec, that);
    }

    test_case(BoundedVec::with_capacity(3), BoundedVec::with_capacity(4));
    test_case(BoundedVec::from(&[1_i32]), BoundedVec::from(&[1_i32]));
    test_case(BoundedVec::from(&[1_i32, 1_i32]), BoundedVec::from(&[1_i32]));
    test_case(
        BoundedVec::from(&[1_i32, 2_i32, 3_i32]),
        BoundedVec::from(&[1_i32, 2_i32, 3_i32]),
    );
    test_case(
        BoundedVec::from(&[1_i32, 1_i32, 2_i32, 3_i32]),
        BoundedVec::from(&[1_i32, 2_i32, 3_i32]),
    );
    test_case(
        BoundedVec::from(&[1_i32, 2_i32, 2_i32, 3_i32]),
        BoundedVec::from(&[1_i32, 2_i32, 3_i32]),
    );
    test_case(
        BoundedVec::from(&[1_i32, 2_i32, 3_i32, 3_i32]),
        BoundedVec::from(&[1_i32, 2_i32, 3_i32]),
    );
    test_case(
        BoundedVec::from(&[1_i32, 1_i32, 2_i32, 2_i32, 2_i32, 3_i32, 3_i32]),
        BoundedVec::from(&[1_i32, 2_i32, 3_i32]),
    );
    test_case(
        BoundedVec::from(&[1_i32, 1_i32, 2_i32, 2_i32, 2_i32, 3_i32, 3_i32, 4_i32, 5_i32, 5_i32, 5_i32, 6_i32, 7_i32, 7_i32]),
        BoundedVec::from(&[1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32, 7_i32]),
    );
}

#[test]
fn test_bounded_vec_dedup_by_key() {
    fn test_case(this: BoundedVec<i32>, that: BoundedVec<i32>) {
        let mut vec = this;
        vec.dedup_by_key(|i| *i / 10);
        assert_eq!(vec, that);
    }

    test_case(BoundedVec::with_capacity(3), BoundedVec::with_capacity(4));
    test_case(BoundedVec::from(&[10_i32]), BoundedVec::from(&[10_i32]));
    test_case(BoundedVec::from(&[10_i32, 11_i32]), BoundedVec::from(&[10_i32]));
    test_case(
        BoundedVec::from(&[10_i32, 20_i32, 30_i32]),
        BoundedVec::from(&[10_i32, 20_i32, 30_i32]),
    );
    test_case(
        BoundedVec::from(&[10_i32, 11_i32, 20_i32, 30_i32]),
        BoundedVec::from(&[10_i32, 20_i32, 30_i32]),
    );
    test_case(
        BoundedVec::from(&[10_i32, 20_i32, 21_i32, 30_i32]),
        BoundedVec::from(&[10_i32, 20_i32, 30_i32]),
    );
    test_case(
        BoundedVec::from(&[10_i32, 20_i32, 30_i32, 31_i32]),
        BoundedVec::from(&[10_i32, 20_i32, 30_i32]),
    );
    test_case(
        BoundedVec::from(&[10_i32, 11_i32, 20_i32, 21_i32, 22_i32, 30_i32, 31_i32]),
        BoundedVec::from(&[10_i32, 20_i32, 30_i32]),
    );
}

#[test]
#[should_panic]
fn test_bounded_vec_swap_remove_out_of_bounds1() {
    let mut vec = BoundedVec::from(&[0_i32]);
    vec.swap_remove(1);
}

#[test]
#[should_panic]
fn test_bounded_vec_swap_remove_out_of_bounds2() {
    let mut vec = BoundedVec::from(&[0_i32]);
    vec.swap_remove(usize::MAX);
}

#[test]
fn test_bounded_vec_truncate1() {
    let mut vec = BoundedVec::from([899_i32, 615_i32, 623_i32, 487_i32, 935_i32, 806_i32, 381_i32, 967_i32]);
    let expected = BoundedVec::from([899_i32, 615_i32, 623_i32, 487_i32]);
    vec.truncate(4);

    assert_eq!(vec, expected);
}

#[test]
fn test_bounded_vec_truncate2() {
    let mut vec = BoundedVec::from([899_i32, 615_i32, 623_i32, 487_i32, 935_i32, 806_i32, 381_i32, 967_i32]);
    let expected = BoundedVec::from([899_i32, 615_i32, 623_i32, 487_i32, 935_i32, 806_i32, 381_i32, 967_i32]);
    vec.truncate(vec.len());

    assert_eq!(vec, expected);
}

#[test]
fn test_bounded_vec_truncate3() {
    let mut vec = BoundedVec::from([899_i32, 615_i32, 623_i32, 487_i32, 935_i32, 806_i32, 381_i32, 967_i32]);
    let expected = BoundedVec::with_capacity(0);
    vec.truncate(0);

    assert_eq!(vec, expected);
}

#[test]
fn test_bounded_vec_truncate_len1() {
    let mut vec = BoundedVec::from([1_i32]);

    vec.truncate(1);
    assert_eq!(vec.len(), 1);
    vec.truncate(0);
    assert_eq!(vec.len(), 0);
}

#[test]
fn test_bounded_vec_truncate_len2() {
    let mut vec = BoundedVec::from([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);

    vec.truncate(6);
    assert_eq!(vec.len(), 6);
    vec.truncate(5);
    assert_eq!(vec.len(), 5);
    vec.truncate(3);
    assert_eq!(vec.len(), 3);
    vec.truncate(0);
    assert_eq!(vec.len(), 0);
}

#[test]
fn test_bounded_vec_truncate_drop1() {
    static mut DROP_COUNT: usize = 0;

    fn get_drop_count() -> usize {
        unsafe { DROP_COUNT }
    }

    struct Value {
        _data: i32,
    }

    impl Value {
        fn new(data: i32) -> Self {
            Self { _data: data }
        }
    }

    impl Drop for Value {
        fn drop(&mut self) {
            unsafe {
                DROP_COUNT += 1;
            }
        }
    }

    let mut vec = BoundedVec::from([Value::new(1_i32)]);

    vec.truncate(1);
    assert_eq!(get_drop_count(), 0);
    vec.truncate(0);
    assert_eq!(get_drop_count(), 1);
}

#[test]
fn test_bounded_vec_truncate_drop2() {
    static mut DROP_COUNT: usize = 0;

    fn get_drop_count() -> usize {
        unsafe { DROP_COUNT }
    }

    struct Value {
        _data: i32,
    }

    impl Value {
        fn new(data: i32) -> Self {
            Self { _data: data }
        }
    }

    impl Drop for Value {
        fn drop(&mut self) {
            unsafe {
                DROP_COUNT += 1;
            }
        }
    }

    let mut vec = BoundedVec::from([
        Value::new(1_i32),
        Value::new(2_i32),
        Value::new(3_i32),
        Value::new(4_i32),
        Value::new(5_i32),
        Value::new(6_i32),
    ]);

    vec.truncate(6);
    assert_eq!(get_drop_count(), 0);
    vec.truncate(5);
    assert_eq!(get_drop_count(), 1);
    vec.truncate(3);
    assert_eq!(get_drop_count(), 3);
    vec.truncate(0);
    assert_eq!(get_drop_count(), 6);
}

#[test]
#[should_panic]
fn test_bounded_vec_truncate_fail() {
    struct BadValue {
        data: usize,
    }

    impl BadValue {
        fn new(data: usize) -> Self {
            BadValue { data }
        }
    }

    impl Drop for BadValue {
        fn drop(&mut self) {
            let BadValue { ref mut data } = *self;
            if *data == 0xbadbeef_usize {
                panic!("BadElem panic: 0xbadbeef")
            }
        }
    }

    let mut vec = BoundedVec::from([
        BadValue::new(1_usize),
        BadValue::new(2_usize),
        BadValue::new(0xbadbeef_usize),
        BadValue::new(4_usize),
    ]);

    vec.truncate(0);
}

#[test]
fn test_bounded_vec_into_iter_clone_empty() {
    let vec: BoundedVec<i32> = BoundedVec::with_capacity(10);
    let mut cloned_vec = BoundedVec::with_capacity(10);
    for value in vec.into_iter() {
        cloned_vec.push(value);
    }

    assert!(cloned_vec.is_empty());
}

#[test]
fn test_bounded_vec_into_iter_clone1() {
    let vec: BoundedVec<i32> = BoundedVec::from([1_i32]);
    let mut cloned_vec = BoundedVec::with_capacity(3);
    for value in vec.clone().into_iter() {
        cloned_vec.push(value);
    }

    assert_eq!(cloned_vec, vec);
}

#[test]
fn test_bounded_vec_into_iter_clone2() {
    let vec: BoundedVec<i32> = BoundedVec::from([1_i32, 2_i32]);
    let mut cloned_vec = BoundedVec::with_capacity(3);
    for value in vec.clone().into_iter() {
        cloned_vec.push(value);
    }

    assert_eq!(cloned_vec, vec);
}

#[test]
fn test_bounded_vec_into_iter_clone3() {
    let vec: BoundedVec<i32> = BoundedVec::from([1_i32, 2_i32, 3_i32, 4_i32]);
    let mut cloned_vec = BoundedVec::with_capacity(4);
    for value in vec.clone().into_iter() {
        cloned_vec.push(value);
    }

    assert_eq!(cloned_vec, vec);
}

#[test]
fn test_bounded_vec_into_iter_clone4() {
    let vec = BoundedVec::from([String::from("foo")]);
    let mut cloned_vec = BoundedVec::with_capacity(3);
    for value in vec.clone().into_iter() {
        cloned_vec.push(value);
    }

    assert_eq!(cloned_vec, vec);
}

#[test]
fn test_bounded_vec_into_iter_clone5() {
    let vec = BoundedVec::from([String::from("foo"), String::from("bar")]);
    let mut cloned_vec = BoundedVec::with_capacity(3);
    for value in vec.clone().into_iter() {
        cloned_vec.push(value);
    }

    assert_eq!(cloned_vec, vec);
}

#[test]
fn test_bounded_vec_into_iter_clone6() {
    let vec = BoundedVec::from([String::from("foo"), String::from("bar"), String::from("baz"), String::from("quux")]);
    let mut cloned_vec = BoundedVec::with_capacity(4);
    for value in vec.clone().into_iter() {
        cloned_vec.push(value);
    }

    assert_eq!(cloned_vec, vec);
}

#[test]
fn test_bounded_vec_into_iter_partial0() {
    let vec = BoundedVec::from([
        String::from("foo"),
        String::from("bar"),
        String::from("baz"),
        String::from("quux"),
        String::from("quuz"),
        String::from("garply"),
    ]);
    let expected: BoundedVec<String> = BoundedVec::from([]);
    let mut result = BoundedVec::with_capacity(1);
    for value in vec.clone().into_iter().take(0) {
        result.push(value);
    }

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_into_iter_partial1() {
    let vec = BoundedVec::from([
        String::from("foo"),
        String::from("bar"),
        String::from("baz"),
        String::from("quux"),
        String::from("quuz"),
        String::from("garply"),
    ]);
    let expected = BoundedVec::from([String::from("foo")]);
    let mut result = BoundedVec::with_capacity(1);
    for value in vec.clone().into_iter().take(1) {
        result.push(value);
    }

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_into_iter_partial2() {
    let vec = BoundedVec::from([
        String::from("foo"),
        String::from("bar"),
        String::from("baz"),
        String::from("quux"),
        String::from("quuz"),
        String::from("garply"),
    ]);
    let expected = BoundedVec::from([String::from("foo"), String::from("bar")]);
    let mut result = BoundedVec::with_capacity(2);
    for value in vec.clone().into_iter().take(2) {
        result.push(value);
    }

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_into_iter_partial3() {
    let vec = BoundedVec::from([
        String::from("foo"),
        String::from("bar"),
        String::from("baz"),
        String::from("quux"),
        String::from("quuz"),
        String::from("garply"),
    ]);
    let expected = BoundedVec::from([String::from("foo"), String::from("bar"), String::from("baz")]);
    let mut result = BoundedVec::with_capacity(3);
    for value in vec.clone().into_iter().take(3) {
        result.push(value);
    }

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_into_iter_partial4() {
    let vec = BoundedVec::from([
        String::from("foo"),
        String::from("bar"),
        String::from("baz"),
        String::from("quux"),
        String::from("quuz"),
        String::from("garply"),
    ]);
    let expected = BoundedVec::from([String::from("foo"), String::from("bar"), String::from("baz"), String::from("quux")]);
    let mut result = BoundedVec::with_capacity(4);
    for value in vec.clone().into_iter().take(4) {
        result.push(value);
    }

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_into_iter_partial5() {
    let vec = BoundedVec::from([
        String::from("foo"),
        String::from("bar"),
        String::from("baz"),
        String::from("quux"),
        String::from("quuz"),
        String::from("garply"),
    ]);
    let expected = BoundedVec::from([
        String::from("foo"),
        String::from("bar"),
        String::from("baz"),
        String::from("quux"),
        String::from("quuz"),
    ]);
    let mut result = BoundedVec::with_capacity(5);
    for value in vec.clone().into_iter().take(5) {
        result.push(value);
    }

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_into_iter_as_slice1() {
    let vec = BoundedVec::from(["foo", "bar", "baz", "quux"]);
    let mut iter = vec.into_iter();
    assert_eq!(iter.as_slice(), ["foo", "bar", "baz", "quux"]);
    let _ = iter.next();
    assert_eq!(iter.as_slice(), ["bar", "baz", "quux"]);
    let _ = iter.next();
    assert_eq!(iter.as_slice(), ["baz", "quux"]);
    let _ = iter.next();
    assert_eq!(iter.as_slice(), ["quux"]);
    let _ = iter.next();
    assert!(iter.as_slice().is_empty());
}

#[test]
fn test_bounded_vec_into_iter_as_mut_slice1() {
    let vec = BoundedVec::from(["foo", "bar", "baz", "quux"]);
    let mut iter = vec.into_iter();
    assert_eq!(iter.as_mut_slice(), ["foo", "bar", "baz", "quux"]);
    let _ = iter.next();
    assert_eq!(iter.as_mut_slice(), ["bar", "baz", "quux"]);
    let _ = iter.next();
    assert_eq!(iter.as_mut_slice(), ["baz", "quux"]);
    let _ = iter.next();
    assert_eq!(iter.as_mut_slice(), ["quux"]);
    let _ = iter.next();
    assert!(iter.as_mut_slice().is_empty());
}

#[test]
fn test_bounded_vec_into_iter_as_mut_slice2() {
    let vec = BoundedVec::from(["foo", "bar", "baz", "quux"]);
    let mut iter = vec.into_iter();
    assert_eq!(iter.as_mut_slice(), ["foo", "bar", "baz", "quux"]);
    iter.as_mut_slice()[0] = "FOO";
    assert_eq!(iter.next(), Some("FOO"));
    assert_eq!(iter.as_mut_slice(), ["bar", "baz", "quux"]);
    iter.as_mut_slice()[0] = "BAR";
    assert_eq!(iter.next(), Some("BAR"));
    assert_eq!(iter.as_mut_slice(), ["baz", "quux"]);
    iter.as_mut_slice()[0] = "BAZ";
    assert_eq!(iter.next(), Some("BAZ"));
    assert_eq!(iter.as_mut_slice(), ["quux"]);
    iter.as_mut_slice()[0] = "QUUX";
    assert_eq!(iter.next(), Some("QUUX"));
    assert!(iter.as_mut_slice().is_empty());
    assert_eq!(iter.next(), None);
}

#[test]
fn test_bounded_vec_debug_fmt_empty_unit() {
    let vec: BoundedVec<()> = BoundedVec::with_capacity(0);
    let expected = "[]";
    let result = format!("{:?}", vec.as_slice());

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_debug_fmt_empty_u8() {
    let vec: BoundedVec<u8> = BoundedVec::with_capacity(0);
    let expected = "[]";
    let result = format!("{:?}", vec.as_slice());

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_debug_fmt_empty_u16() {
    let vec: BoundedVec<u16> = BoundedVec::with_capacity(0);
    let expected = "[]";
    let result = format!("{:?}", vec.as_slice());

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_debug_fmt_empty_u32() {
    let vec: BoundedVec<u32> = BoundedVec::with_capacity(0);
    let expected = "[]";
    let result = format!("{:?}", vec.as_slice());

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_debug_fmt_empty_u64() {
    let vec: BoundedVec<u64> = BoundedVec::with_capacity(0);
    let expected = "[]";
    let result = format!("{:?}", vec.as_slice());

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_debug_fmt_empty_usize() {
    let vec: BoundedVec<usize> = BoundedVec::with_capacity(0);
    let expected = "[]";
    let result = format!("{:?}", vec.as_slice());

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_debug_fmt_empty_string() {
    let vec: BoundedVec<u8> = BoundedVec::with_capacity(0);
    let expected = "[]";
    let result = format!("{:?}", vec.as_slice());

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_indexing() {
    let vec = BoundedVec::from([10_i32, 20_i32, 30_i32]);

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
    let vec: BoundedVec<i32, alloc::Global> = BoundedVec::from([10_i32]);
    let _ = vec[1];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_indexing_out_of_bounds3() {
    let vec: BoundedVec<i32, alloc::Global> = BoundedVec::from([10_i32, 20_i32]);
    let _ = vec[2];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_indexing_out_of_bounds4() {
    let vec: BoundedVec<i32, alloc::Global> = BoundedVec::from([10_i32, 20_i32, 30_i32]);
    let _ = vec[3];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_slice_out_of_bounds1() {
    let vec = BoundedVec::from([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    let _ = &vec[!0..];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_slice_out_of_bounds2() {
    let vec = BoundedVec::from([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    let _ = &vec[..7];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_slice_out_of_bounds3() {
    let vec = BoundedVec::from([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    let _ = &vec[!0..5];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_slice_out_of_bounds4() {
    let vec = BoundedVec::from([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    let _ = &vec[1..7];

    assert!(true);
}

#[test]
#[should_panic]
fn test_bounded_vec_slice_out_of_bounds5() {
    let vec = BoundedVec::from([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    let _ = &vec[3..2];

    assert!(true);
}

#[test]
fn test_bounded_vec_extract_if_empty_true() {
    let mut vec: BoundedVec<i32> = BoundedVec::with_capacity(10);
    assert_eq!(vec.len(), 0);
    {
        let mut iter = vec.extract_if(.., |_| true);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.as_slice(), []);
}

#[test]
fn test_bounded_vec_extract_if_empty_false() {
    let mut vec: BoundedVec<i32> = BoundedVec::with_capacity(10);
    assert_eq!(vec.len(), 0);
    {
        let mut iter = vec.extract_if(.., |_| false);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    assert_eq!(vec.len(), 0);
    assert_eq!(vec.as_slice(), []);
}

#[test]
fn test_bounded_vec_extract_if_total_true() {
    let mut vec = BoundedVec::from([0_i32, 1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32, 7_i32, 8_i32, 9_i32, 10_i32]);
    let old_length = vec.len();
    let mut count = 0;
    {
        let mut iter = vec.extract_if(.., |_| true);
        while let Some(_) = iter.next() {
            count += 1;
            assert_eq!(iter.size_hint(), (0, Some(old_length - count)));
        }

        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    assert_eq!(count, 11);
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.as_slice(), []);
}

#[test]
fn test_bounded_vec_extract_if_total_false() {
    let mut vec = BoundedVec::from([0_i32, 1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32, 7_i32, 8_i32, 9_i32, 10_i32]);
    let old_length = vec.len();
    let mut count = 0;
    {
        let mut iter = vec.extract_if(.., |_| false);
        while let Some(_) = iter.next() {
            count += 1;
            assert_eq!(iter.size_hint(), (0, Some(old_length - count)));
        }

        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    assert_eq!(count, 0);
    assert_eq!(vec.len(), old_length);
    assert_eq!(
        vec.as_slice(),
        [0_i32, 1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32, 7_i32, 8_i32, 9_i32, 10_i32]
    );
}

#[test]
fn test_bounded_vec_extract_if_partial_true() {
    let mut vec = BoundedVec::from([0_i32, 1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32, 7_i32, 8_i32, 9_i32, 10_i32]);
    let old_length = vec.len();
    let mut count = 0;
    {
        let mut iter = vec.extract_if(2..8, |_| true);
        while let Some(_) = iter.next() {
            count += 1;
        }

        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    assert_eq!(count, 6);
    assert_eq!(vec.len(), old_length - count);
    assert_eq!(vec.as_slice(), [0_i32, 1_i32, 8_i32, 9_i32, 10_i32]);
}

#[test]
fn test_bounded_vec_extract_if_partial_false() {
    let mut vec = BoundedVec::from([0_i32, 1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32, 7_i32, 8_i32, 9_i32, 10_i32]);
    let old_length = vec.len();
    let mut count = 0;
    {
        let mut iter = vec.extract_if(2..8, |_| false);
        while let Some(_) = iter.next() {
            count += 1;
        }

        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    assert_eq!(count, 0);
    assert_eq!(vec.len(), old_length);
    assert_eq!(
        vec.as_slice(),
        [0_i32, 1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32, 7_i32, 8_i32, 9_i32, 10_i32]
    );
}

#[test]
#[should_panic]
fn test_bounded_vec_extract_if_out_of_bounds() {
    let mut vec = BoundedVec::from([1_i32, 2_i32, 3_i32]);
    let _ = vec.extract_if(10.., |_| true).for_each(drop);

    assert!(true);
}

#[test]
fn test_bounded_vec_extract_if_retains_unvisited_elements() {
    let mut vec = BoundedVec::from([0_i32, 1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32, 7_i32, 8_i32, 9_i32, 10_i32]);
    let mut count = 0;
    {
        let mut iter = vec.extract_if(.., |_| true);
        while count < 3 {
            let _ = iter.next();
            count += 1;
        }
    }

    assert_eq!(vec.as_slice(), [3_i32, 4_i32, 5_i32, 6_i32, 7_i32, 8_i32, 9_i32, 10_i32]);
}

#[rustfmt::skip]
#[test]
fn test_bounded_vec_extract_if_many1() {
    let mut vec = BoundedVec::from([
        0_i32,    1_i32,    2_i32,    3_i32,    4_i32,    5_i32,    6_i32,    7_i32,    8_i32,
        9_i32,    10_i32,   i32::MAX, i32::MAX, i32::MAX, i32::MAX, 11_i32,   12_i32,   13_i32,
        14_i32,   15_i32,   16_i32,   17_i32,   18_i32,   19_i32,   20_i32,   21_i32,   22_i32,
        23_i32,   24_i32,   25_i32,   26_i32,   27_i32,   28_i32,   29_i32,   30_i32,   31_i32,
        i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, 32_i32,
    ]);
    let mut extracted: BoundedVec<i32> = BoundedVec::with_capacity(vec.capacity());
    extracted.extend(vec.extract_if(.., |v| *v == i32::MAX));

    let mut expected_vec = BoundedVec::with_capacity(33);
    expected_vec.extend(0_i32..=32_i32);

    let expected_extracted = BoundedVec::from([i32::MAX; 12]);

    assert_eq!(vec, expected_vec);
    assert_eq!(extracted, expected_extracted);
}

#[rustfmt::skip]
#[test]
fn test_bounded_vec_extract_if_many2() {
    let mut vec = BoundedVec::from([
        i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, 0_i32,
        1_i32,    2_i32,    3_i32,    4_i32,    5_i32,    6_i32,    7_i32,    8_i32,
        9_i32,    10_i32,   11_i32,   12_i32,   13_i32,   14_i32,   15_i32,   16_i32,
        17_i32,   18_i32,   19_i32,   20_i32,   21_i32,   22_i32,   23_i32,   24_i32,
        25_i32,   26_i32,   27_i32,   28_i32,   29_i32,   30_i32,   31_i32,   32_i32,
        i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX,
    ]);
    let mut extracted: BoundedVec<i32> = BoundedVec::with_capacity(vec.capacity());
    extracted.extend(vec.extract_if(.., |v| *v == i32::MAX));

    let mut expected_vec = BoundedVec::with_capacity(33);
    expected_vec.extend(0_i32..=32_i32);

    let expected_extracted = BoundedVec::from([i32::MAX; 15]);

    assert_eq!(vec, expected_vec);
    assert_eq!(extracted, expected_extracted);
}

#[rustfmt::skip]
#[test]
fn test_bounded_vec_extract_if_many3() {
    let mut vec = BoundedVec::from([
        i32::MAX, 0_i32,    i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, 1_i32,
        2_i32,    3_i32,    4_i32,    5_i32,    6_i32,    7_i32,    8_i32,    9_i32,
        10_i32,   i32::MAX, i32::MAX, i32::MAX, i32::MAX, 11_i32,   12_i32,   13_i32,
        14_i32,   15_i32,   16_i32,   17_i32,   18_i32,   19_i32,   20_i32,   21_i32,
        22_i32,   23_i32,   24_i32,   25_i32,   26_i32,   27_i32,   28_i32,   29_i32,
        30_i32,   31_i32,   32_i32,   i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX,
    ]);
    let mut extracted: BoundedVec<i32> = BoundedVec::with_capacity(vec.capacity());
    extracted.extend(vec.extract_if(.., |v| *v == i32::MAX));

    let mut expected_vec = BoundedVec::with_capacity(33);
    expected_vec.extend(0_i32..=32_i32);

    let expected_extracted = BoundedVec::from([i32::MAX; 15]);

    assert_eq!(vec, expected_vec);
    assert_eq!(extracted, expected_extracted);
}

#[rustfmt::skip]
#[test]
fn test_bounded_vec_extract_if_many4() {
    let mut vec = BoundedVec::from([
        i32::MAX, 0_i32,  i32::MAX, 1_i32,  i32::MAX, 2_i32,  i32::MAX, 3_i32,
        i32::MAX, 4_i32,  i32::MAX, 5_i32,  i32::MAX, 6_i32,  i32::MAX, 7_i32,
        i32::MAX, 8_i32,  i32::MAX, 9_i32,  i32::MAX, 10_i32, i32::MAX, 11_i32,
        i32::MAX, 12_i32, i32::MAX, 13_i32, i32::MAX, 14_i32, i32::MAX, 15_i32,
        i32::MAX, 16_i32, i32::MAX, 17_i32, i32::MAX, 18_i32, i32::MAX, 19_i32,
        i32::MAX, 20_i32, i32::MAX, 21_i32, i32::MAX, 22_i32, i32::MAX, 23_i32,
        i32::MAX, 24_i32, i32::MAX, 25_i32, i32::MAX, 26_i32, i32::MAX, 27_i32,
        i32::MAX, 28_i32, i32::MAX, 29_i32, i32::MAX, 30_i32, i32::MAX, 31_i32,
        i32::MAX, 32_i32,
    ]);
    let mut extracted: BoundedVec<i32> = BoundedVec::with_capacity(vec.capacity());
    extracted.extend(vec.extract_if(.., |v| *v == i32::MAX));

    let mut expected_vec = BoundedVec::with_capacity(33);
    expected_vec.extend(0_i32..=32_i32);

    let expected_extracted = BoundedVec::from([i32::MAX; 33]);

    assert_eq!(vec, expected_vec);
    assert_eq!(extracted, expected_extracted);
}

#[test]
fn test_bounded_vec_retain1() {
    let mut vec = BoundedVec::from([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    vec.retain(|&x| x % 2_i32 == 0_i32);

    assert_eq!(vec.as_slice(), [2_i32, 4_i32, 6_i32]);
}

#[test]
fn test_bounded_vec_retain2() {
    let mut vec = BoundedVec::from([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    vec.retain(|_| true);

    assert_eq!(vec.as_slice(), [1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
}

#[test]
fn test_bounded_vec_retain3() {
    let mut vec = BoundedVec::from([1_i32, 2_i32, 3_i32, 4_i32, 5_i32, 6_i32]);
    vec.retain(|_| false);

    assert_eq!(vec.as_slice(), []);
}

#[test]
fn test_bounded_vec_shift_insert_first() {
    let mut vec = BoundedVec::with_capacity(3);
    assert!(vec.is_empty());
    vec.shift_insert(0, 1_i32);
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.as_slice(), &[1_i32]);
    vec.shift_insert(0, 2_i32);
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.as_slice(), &[2_i32, 1_i32]);
    vec.shift_insert(0, 3_i32);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec.as_slice(), &[3_i32, 2_i32, 1_i32]);
}

#[test]
fn test_bounded_vec_shift_insert_last() {
    let mut vec = BoundedVec::with_capacity(3);
    assert!(vec.is_empty());
    vec.shift_insert(0, 1_i32);
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.as_slice(), &[1_i32]);
    vec.shift_insert(1, 2_i32);
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.as_slice(), &[1_i32, 2_i32]);
    vec.shift_insert(2, 3_i32);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec.as_slice(), &[1_i32, 2_i32, 3_i32]);
}

#[test]
fn test_bounded_vec_shift_insert_middle() {
    let mut vec = BoundedVec::with_capacity(5);
    vec.extend([i32::MAX, i32::MAX]);
    assert_eq!(vec.len(), 2);
    vec.shift_insert(1, 1_i32);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec.as_slice(), &[i32::MAX, 1_i32, i32::MAX]);
    vec.shift_insert(1, 2_i32);
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.as_slice(), &[i32::MAX, 2_i32, 1_i32, i32::MAX]);
    vec.shift_insert(1, 3_i32);
    assert_eq!(vec.len(), 5);
    assert_eq!(vec.as_slice(), &[i32::MAX, 3_i32, 2_i32, 1_i32, i32::MAX]);
}

#[test]
#[should_panic]
fn test_bounded_vec_shift_insert_out_of_bounds() {
    let mut vec = BoundedVec::with_capacity(1);
    vec.shift_insert(2, 1_i32);

    assert!(true);
}

#[test]
fn test_bounded_vec_swap_remove1() {
    let mut vec = BoundedVec::from(['a', 'b', 'c']);
    let result = vec.swap_remove(0);

    assert_eq!(result, 'a');
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.as_slice(), &['c', 'b']);
}

#[test]
fn test_bounded_vec_swap_remove2() {
    let mut vec = BoundedVec::from(['a', 'b', 'c']);
    let result = vec.swap_remove(1);

    assert_eq!(result, 'b');
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.as_slice(), &['a', 'c']);
}

#[test]
fn test_bounded_vec_swap_remove3() {
    let mut vec = BoundedVec::from(['a', 'b', 'c']);
    let result = vec.swap_remove(2);

    assert_eq!(result, 'c');
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.as_slice(), &['a', 'b']);
}

#[test]
#[should_panic]
fn test_bounded_vec_swap_remove_out_of_bounds() {
    let mut vec: BoundedVec<i32> = BoundedVec::with_capacity(2);
    vec.swap_remove(2);

    assert!(true);
}

#[test]
fn test_bounded_vec_shift_remove1() {
    let mut vec = BoundedVec::from(['a', 'b', 'c', 'd', 'e']);
    let result = vec.shift_remove(0);

    assert_eq!(result, 'a');
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.as_slice(), &['b', 'c', 'd', 'e']);
}

#[test]
fn test_bounded_vec_shift_remove2() {
    let mut vec = BoundedVec::from(['a', 'b', 'c', 'd', 'e']);
    let result = vec.shift_remove(1);

    assert_eq!(result, 'b');
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.as_slice(), &['a', 'c', 'd', 'e']);
}

#[test]
fn test_bounded_vec_shift_remove3() {
    let mut vec = BoundedVec::from(['a', 'b', 'c', 'd', 'e']);
    let result = vec.shift_remove(2);

    assert_eq!(result, 'c');
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.as_slice(), &['a', 'b', 'd', 'e']);
}

#[test]
fn test_bounded_vec_shift_remove4() {
    let mut vec = BoundedVec::from(['a', 'b', 'c', 'd', 'e']);
    let result = vec.shift_remove(3);

    assert_eq!(result, 'd');
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.as_slice(), &['a', 'b', 'c', 'e']);
}

#[test]
fn test_bounded_vec_shift_remove5() {
    let mut vec = BoundedVec::from(['a', 'b', 'c', 'd', 'e']);
    let result = vec.shift_remove(4);

    assert_eq!(result, 'e');
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.as_slice(), &['a', 'b', 'c', 'd']);
}

#[test]
fn test_bounded_vec_shift_remove6() {
    let mut vec = BoundedVec::from(['a', 'b', 'c', 'd', 'e']);
    assert_eq!(vec.shift_remove(0), 'a');
    assert_eq!(vec.len(), 4);
    assert_eq!(vec.as_slice(), &['b', 'c', 'd', 'e']);
    assert_eq!(vec.shift_remove(0), 'b');
    assert_eq!(vec.len(), 3);
    assert_eq!(vec.as_slice(), &['c', 'd', 'e']);
    assert_eq!(vec.shift_remove(0), 'c');
    assert_eq!(vec.len(), 2);
    assert_eq!(vec.as_slice(), &['d', 'e']);
    assert_eq!(vec.shift_remove(0), 'd');
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.as_slice(), &['e']);
    assert_eq!(vec.shift_remove(0), 'e');
    assert_eq!(vec.len(), 0);
    assert!(vec.is_empty());
}

#[test]
#[should_panic]
fn test_bounded_vec_shift_remove_out_of_bounds() {
    let mut vec: BoundedVec<i32> = BoundedVec::with_capacity(2);
    vec.shift_remove(2);

    assert!(true);
}

#[test]
fn test_bounded_vec_pop_if_empty_true() {
    let mut vec: BoundedVec<i32> = BoundedVec::with_capacity(10);

    assert_eq!(vec.pop_if(|_| true), None);
    assert!(vec.is_empty());
}

#[test]
fn test_bounded_vec_pop_if_empty_false() {
    let mut vec: BoundedVec<i32> = BoundedVec::with_capacity(10);

    assert_eq!(vec.pop_if(|_| false), None);
    assert!(vec.is_empty());
}

#[test]
fn test_bounded_vec_pop_if_true() {
    let mut vec = BoundedVec::from([1_i32, 2_i32, 3_i32]);

    assert_eq!(vec.pop_if(|_| true), Some(3_i32));
    assert_eq!(vec.as_slice(), &[1_i32, 2_i32]);
}

#[test]
fn test_bounded_vec_pop_if_false() {
    let mut vec: BoundedVec<i32> = BoundedVec::from([1_i32, 2_i32, 3_i32]);

    assert_eq!(vec.pop_if(|_| false), None);
    assert_eq!(vec.as_slice(), &[1_i32, 2_i32, 3_i32]);
}
