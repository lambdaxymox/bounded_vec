#![deny(unsafe_op_in_unsafe_fn)]
#![deny(private_interfaces)]
#![deny(clippy::missing_safety_doc)]
#![cfg_attr(not(any(test)), no_std)]
extern crate core;
extern crate allocator_api2;

use core::fmt;
use core::mem::MaybeUninit;
use core::ops;

use allocator_api2::alloc;
use allocator_api2::boxed::Box;

pub struct BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    inner: Box<[MaybeUninit<T>], A>,
    length: usize,
}

impl<T, A> BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    pub fn with_capacity_in(capacity: usize, alloc: A) -> Self {
        if capacity == 0 {
            let slice_ptr = core::ptr::slice_from_raw_parts_mut(core::ptr::dangling_mut(), 0);
            let inner = unsafe { Box::from_raw_in(slice_ptr, alloc) };

            return Self { inner, length: 0 };
        }

        let layout = core::alloc::Layout::array::<T>(capacity).expect("invalid layout");
        let raw_ptr = alloc.allocate(layout).expect("`BoundedVec` allocation failed");

        let slice_ptr = core::ptr::slice_from_raw_parts_mut(
            raw_ptr.as_ptr().cast::<MaybeUninit<T>>(),
            capacity,
        );

        let inner = unsafe { Box::from_raw_in(slice_ptr, alloc) };

        Self { inner, length: 0 }
    }
}

impl<T, A> BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.length
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.inner.len()
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub const fn allocator(&self) -> &A {
        Box::allocator(&self.inner)
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: The first `self.length` elements have been initialized.
        unsafe {
            core::slice::from_raw_parts(self.inner.as_ptr().cast::<T>(), self.length)
        }
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: The first `self.length` elements have been initialized.
        unsafe {
            core::slice::from_raw_parts_mut(self.inner.as_mut_ptr().cast::<T>(), self.length)
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn as_ptr(&self) -> *const T {
        self.inner.as_ptr().cast::<T>()
    }

    #[inline(always)]
    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr().cast::<T>()
    }
}

impl<T, A> BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[inline(always)]
    pub fn push(&mut self, value: T) {
        let length = self.len();

        assert!(length < self.capacity());

        self.inner[length] = MaybeUninit::new(value);
        self.length += 1;
    }

    #[inline(always)]
    pub fn push_within_capacity(&mut self, value: T) -> Result<(), T> {
        let length = self.len();
        if length >= self.capacity() {
            return Err(value);
        }

        self.inner[length] = MaybeUninit::new(value);
        self.length += 1;

        Ok(())
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            None
        } else {
            unsafe {
                self.length -= 1;
                Some(core::ptr::read(self.as_ptr().add(self.len())))
            }
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        let values: *mut [T] = self.as_mut_slice();
        unsafe {
            self.length = 0;
            core::ptr::drop_in_place(values);
        }
    }

    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }
}

impl<T, A> BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[inline]
    unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());

        self.length = new_len;
    }

    #[inline]
    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe {
            let ptr = self.as_mut_ptr().add(self.len()) as *mut MaybeUninit<T>;
            let len = self.capacity() - self.len();

            core::slice::from_raw_parts_mut(ptr, len)
        }
    }
}

impl<T, A> fmt::Debug for BoundedVec<T, A>
where
    T: fmt::Debug,
    A: alloc::Allocator + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.as_slice(), formatter)
    }
}

impl<T, A> AsRef<[T]> for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, A> AsMut<[T]> for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, A> ops::Deref for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, A> ops::DerefMut for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, A> Default for BoundedVec<T, A>
where
    A: alloc::Allocator + Default,
{
    fn default() -> Self {
        Self {
            inner: Box::default(),
            length: 0,
        }
    }
}

impl<T, I, A> ops::Index<I> for BoundedVec<T, A>
where
    I: core::slice::SliceIndex<[T]>,
    A: alloc::Allocator,
{
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &Self::Output {
        ops::Index::index(&**self, index)
    }
}

impl<T, I, A> ops::IndexMut<I> for BoundedVec<T, A>
where
    I: core::slice::SliceIndex<[T]>,
    A: alloc::Allocator,
{
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        ops::IndexMut::index_mut(&mut **self, index)
    }
}

impl<T, A> BoundedVec<T, A>
where
    T: Clone,
    A: alloc::Allocator + Clone,
{
    #[inline]
    fn from_slice_in(slice: &[T], alloc: A) -> BoundedVec<T, A> {
        struct DropGuard<'a, T, A>
        where
            A: alloc::Allocator + Clone,
        {
            vec: &'a mut BoundedVec<T, A>,
            initialized_count: usize,
        }

        impl<'a, T, A> Drop for DropGuard<'a, T, A>
        where
            A: alloc::Allocator + Clone,
        {
            #[inline]
            fn drop(&mut self) {
                // SAFETY: The elements of the bounded vector were initialized before calling `drop`.
                unsafe { self.vec.set_len(self.initialized_count); }
            }
        }

        let mut vec: BoundedVec<T, A> = BoundedVec::with_capacity_in(slice.len(), alloc);
        let mut guard = DropGuard {
            vec: &mut vec,
            initialized_count: 0,
        };
        let slots = guard.vec.spare_capacity_mut();
        for (i, b) in slice.iter().enumerate().take(slots.len()) {
            guard.initialized_count = i;
            slots[i].write(b.clone());
        }

        core::mem::forget(guard);

        // SAFETY: The bounded vector was allocated and initialized with the same capacity as the
        // original bounded vector, and with the same length as the original bound vector.
        unsafe { vec.set_len(slice.len()); }

        vec
    }
}

impl<T, A> Clone for BoundedVec<T, A>
where
    T: Clone,
    A: alloc::Allocator + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        struct DropGuard<'a, T, A>
        where
            A: alloc::Allocator + Clone,
        {
            vec: &'a mut BoundedVec<T, A>,
            initialized_count: usize,
        }

        impl<'a, T, A> Drop for DropGuard<'a, T, A>
        where
            A: alloc::Allocator + Clone,
        {
            #[inline]
            fn drop(&mut self) {
                // SAFETY: The elements of the bounded vector were initialized before calling `drop`.
                unsafe {
                    self.vec.set_len(self.initialized_count);
                }
            }
        }

        let cloned_alloc = self.allocator().clone();
        let mut cloned_vec: BoundedVec<T, A> = BoundedVec::with_capacity_in(self.capacity(), cloned_alloc);
        let mut guard = DropGuard {
            vec: &mut cloned_vec,
            initialized_count: 0,
        };
        let slots = guard.vec.spare_capacity_mut();
        for (i, b) in self.as_slice().iter().enumerate().take(slots.len()) {
            guard.initialized_count = i;
            slots[i].write(b.clone());
        }

        core::mem::forget(guard);

        // SAFETY: The bounded vector was allocated and initialized with the same capacity as the
        // original bounded vector, and with the same length as the original bound vector.
        unsafe { cloned_vec.set_len(self.len()); }

        cloned_vec
    }
}

impl<T, A> Drop for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    fn drop(&mut self) {
        unsafe {
            let initialized_slice = core::ptr::slice_from_raw_parts_mut(
                self.inner.as_mut_ptr().cast::<T>(),
                self.length,
            );

            core::ptr::drop_in_place(initialized_slice);
        }
    }
}

impl<T> From<&[T]> for BoundedVec<T, alloc::Global>
where
    T: Clone,
{
    fn from(slice: &[T]) -> Self {
        Self::from_slice_in(slice, alloc::Global::default())
    }
}

impl<T> From<&mut [T]> for BoundedVec<T, alloc::Global>
where
    T: Clone,
{
    fn from(slice: &mut [T]) -> Self {
        Self::from_slice_in(slice, alloc::Global::default())
    }
}

impl<const N: usize, T> From<&[T; N]> for BoundedVec<T, alloc::Global>
where
    T: Clone,
{
    fn from(array: &[T; N]) -> Self {
        Self::from(array.as_slice())
    }
}

impl<const N: usize, T> From<&mut [T; N]> for BoundedVec<T, alloc::Global>
where
    T: Clone,
{
    fn from(array: &mut [T; N]) -> Self {
        Self::from(array.as_mut_slice())
    }
}

impl<T> FromIterator<T> for BoundedVec<T, alloc::Global> {
    #[inline]
    #[track_caller]
    fn from_iter<I>(iterable: I) -> BoundedVec<T, alloc::Global>
    where
        I: IntoIterator<Item = T>,
    {
        let iterator = iterable.into_iter();
        let (lower, _) = iterator.size_hint();

        let mut vec = BoundedVec::with_capacity_in(lower, alloc::Global);

        for item in iterator {
            vec.push(item);
        }

        vec
    }
}

impl<T, A1, A2> PartialEq<BoundedVec<T, A2>> for BoundedVec<T, A1>
where
    T: PartialEq,
    A1: alloc::Allocator,
    A2: alloc::Allocator,
{
    fn eq(&self, other: &BoundedVec<T, A2>) -> bool {
        PartialEq::eq(self.as_slice(), other.as_slice())
    }
}

impl<T, A1, A2> PartialOrd<BoundedVec<T, A2>> for BoundedVec<T, A1>
where
    T: PartialOrd,
    A1: alloc::Allocator,
    A2: alloc::Allocator,
{
    #[inline]
    fn partial_cmp(&self, other: &BoundedVec<T, A2>) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_slice(), other.as_slice())
    }
}

impl<T, A> Eq for BoundedVec<T, A>
where
    T: Eq,
    A: alloc::Allocator,
{
}

impl<T, A> Ord for BoundedVec<T, A>
where
    T: Ord,
    A: alloc::Allocator,
{
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(self.as_slice(), other.as_slice())
    }
}
