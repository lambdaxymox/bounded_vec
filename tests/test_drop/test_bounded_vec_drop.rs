use bounded_vec::BoundedVec;

use std::cell::RefCell;
use std::rc::Rc;

use allocator_api2::alloc;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DropCounter {
    count: Rc<RefCell<u32>>,
}

impl DropCounter {
    #[inline]
    const fn new(count: Rc<RefCell<u32>>) -> Self {
        Self { count }
    }
}

impl Drop for DropCounter {
    fn drop(&mut self) {
        *self.count.borrow_mut() += 1;
    }
}

#[test]
fn test_bounded_vec_double_drop() {
    struct TwoBoundedVec {
        x: BoundedVec<DropCounter, alloc::Global>,
        y: BoundedVec<DropCounter, alloc::Global>,
    }

    let (ref_count_x, ref_count_y) = (Rc::new(RefCell::new(0)), Rc::new(RefCell::new(0)));
    {
        let mut tv = TwoBoundedVec {
            x: BoundedVec::with_capacity_in(1, alloc::Global),
            y: BoundedVec::with_capacity_in(1, alloc::Global),
        };

        tv.x.push(DropCounter::new(ref_count_x.clone()));
        tv.y.push(DropCounter::new(ref_count_y.clone()));

        drop(tv.x);
    }

    let count_x = *ref_count_x.borrow();
    let count_y = *ref_count_y.borrow();

    assert_eq!(count_x, 1);
    assert_eq!(count_y, 1);
}

#[test]
fn test_bounded_vec_drop_all_items1() {
    let count = 1024;
    let ref_count = Rc::new(RefCell::new(0));
    let counter = DropCounter::new(ref_count.clone());
    {
        let mut vec = BoundedVec::with_capacity_in(count, alloc::Global);
        for _ in 0..count {
            vec.push(counter.clone());
        }
    }

    let expected = count as u32;
    let result = *ref_count.borrow();

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_push_should_not_drop_value() {
    let mut vec = BoundedVec::with_capacity_in(1, alloc::Global);

    let ref_count = Rc::new(RefCell::new(0));
    let counter = DropCounter::new(ref_count.clone());

    vec.push(counter);

    let count = *ref_count.borrow();

    assert_eq!(count, 0);
}

#[test]
fn test_bounded_vec_pop_should_not_drop_return_value() {
    let mut vec = BoundedVec::with_capacity_in(1, alloc::Global);

    let ref_count = Rc::new(RefCell::new(0));
    let counter = DropCounter::new(ref_count.clone());

    vec.push(counter);

    let _counter = vec.pop();

    let count = *ref_count.borrow();

    assert_eq!(count, 0);
}

#[test]
fn test_bounded_vec_clear_should_drop() {
    let mut vec = BoundedVec::with_capacity_in(1, alloc::Global);

    let ref_count = Rc::new(RefCell::new(0));
    let counter = DropCounter::new(ref_count.clone());

    vec.push(counter);

    vec.clear();

    let count = *ref_count.borrow();

    assert_eq!(count, 1);
}

#[test]
fn test_bounded_vec_drop_should_drop_elements() {
    let ref_count = Rc::new(RefCell::new(0));
    {
        let mut vec = BoundedVec::with_capacity_in(1, alloc::Global);

        let counter = DropCounter::new(ref_count.clone());

        vec.push(counter);
        // `vec` drops here.
    }

    let count = *ref_count.borrow();

    assert_eq!(count, 1);
}

#[test]
fn test_bounded_vec_drop_should_not_drop_popped_elements() {
    let ref_count = Rc::new(RefCell::new(0));
    let _counter = {
        let mut vec = BoundedVec::with_capacity_in(1, alloc::Global);

        vec.push(DropCounter::new(ref_count.clone()));

        let counter = vec.pop();

        counter
        // `vec` drops here.
    };

    let count = *ref_count.borrow();

    assert_eq!(count, 0);
}
