use bounded_vec::BoundedVec;

use core::cell::RefCell;
use std::rc::Rc;

use allocator_api2::alloc;

#[derive(Clone)]
struct DropCounter {
    count: Rc<RefCell<usize>>,
}

impl DropCounter {
    #[inline]
    const fn new(count: Rc<RefCell<usize>>) -> Self {
        Self { count }
    }

    fn increment(&mut self) {
        *self.count.borrow_mut() += 1;
    }

    fn drop_count(&self) -> usize {
        self.count.borrow().clone()
    }
}

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.increment();
    }
}

fn create_drop_counter_vec_in<A>(len: usize, alloc: A) -> (DropCounter, BoundedVec<DropCounter, A>)
where
    A: alloc::Allocator + Clone,
{
    let drop_counter = DropCounter::new(Rc::new(RefCell::new(0)));
    let mut vec = BoundedVec::with_capacity_in(len, alloc);
    for _ in 0..len {
        vec.push(drop_counter.clone());
    }

    (drop_counter, vec)
}

fn run_test_bounded_vec_clear<A>(length: usize, alloc: A)
where
    A: alloc::Allocator + Clone,
{
    let (drop_counter, mut vec) = create_drop_counter_vec_in(length, alloc);
    let expected = vec.len();
    vec.clear();
    let result = drop_counter.drop_count();

    assert_eq!(result, expected);
}

#[test]
fn test_bounded_vec_clear_range() {
    let max_length = 128;
    let alloc = alloc::Global;
    for length in 0..max_length {
        run_test_bounded_vec_clear(length, alloc.clone());
    }
}
