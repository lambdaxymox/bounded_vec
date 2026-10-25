// Portions of this file are derived from `rust`,
// Copyright (c) <year> The Rust Project Contributors
// Licensed under either of
//   * Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
//   * MIT license (http://opensource.org/licenses/MIT)
// at your option.
use crate::extract_if::ExtractIf;
use crate::into_iter::IntoIter;

#[cfg(feature = "nightly")]
use alloc_crate::alloc;

#[cfg(feature = "nightly")]
use alloc_crate::boxed::Box;

#[cfg(not(feature = "nightly"))]
use allocator_api2::alloc;

#[cfg(not(feature = "nightly"))]
use allocator_api2::boxed::Box;

use core::fmt;
use core::hash;
use core::mem::{
    ManuallyDrop,
    MaybeUninit,
};
use core::ops;
use core::ptr::NonNull;
use core::slice;

/// A bounded growable array type.
///
/// This type is similar to [`std::Vec`], but it has a fixed capacity.
///
/// A bounded vector is parameterized by the following parameters:
///
/// * a pointer to a memory allocation.
/// * capacity --- the number of elements the vector can store, or equivalently, the size of the
///   memory allocation in units of elements.
/// * length --- the number of elements currently stored in the vector.
///
/// # Capacity And Length
///
/// The **capacity** of a vector is the number of elements that can be stored in the vector inside
/// the same allocation. That is, it is the number of elements the vector can store. This should
/// not be confused with the **length** of the vector, which is the number of elements currently
/// stored in the vector. The length of a vector is always less than or equal to its capacity.
///
/// # Examples
///
/// ```
/// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
/// # use bounded_vec::BoundedVec;
/// #
/// # #[cfg(feature = "nightly")]
/// # use std::alloc::Global;
/// #
/// # #[cfg(not(feature = "nightly"))]
/// # use allocator_api2::alloc::Global;
/// #
/// let capacity = 3;
/// let mut vec: BoundedVec<i32, Global> = BoundedVec::with_capacity(capacity);
///
/// assert_eq!(vec.len(), 0);
/// assert_eq!(vec.capacity(), capacity);
///
/// vec.push(42);
///
/// assert_eq!(vec.get(0), Some(&42));
/// assert_eq!(vec.get(1), None);
/// assert_eq!(vec.get(2), None);
///
/// assert_eq!(vec.len(), 1);
/// assert_eq!(vec.capacity(), capacity);
/// ```
pub struct BoundedVec<T, A = alloc::Global>
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
    /// Constructs a new empty bounded vector using a specific memory allocator and a specific
    /// capacity.
    ///
    /// The vector will be able to hold at most `capacity` elements. The method will allocate
    /// space for **exactly** `capacity` elements. If `capacity` is zero, the constructor does not
    /// allocate memory.
    ///
    /// Note that while the returned vector will have a **capacity** of at least `capacity`, it
    /// will have a **length** of zero, because no elements have been pushed to the vector yet.
    ///
    /// # Panics
    ///
    /// This method panics if the new capacity exceeds `isize::MAX` _bytes_.
    ///
    /// # Examples
    ///
    /// Creating a bounded vector with capacity `capacity > 0`.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let capacity = 32;
    /// let vec: BoundedVec<i32, Global> = BoundedVec::with_capacity_in(capacity, Global);
    ///
    /// assert!(vec.capacity() >= capacity);
    /// assert!(vec.is_empty());
    /// ```
    ///
    /// Creating a bounded vector with capacity `capacity == 0`.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let vec: BoundedVec<i32, Global> = BoundedVec::with_capacity_in(0, Global);
    ///
    /// assert_eq!(vec.capacity(), 0);
    /// assert!(vec.is_empty());
    /// ```
    pub fn with_capacity_in(capacity: usize, alloc: A) -> Self {
        if capacity == 0 {
            let slice_ptr = core::ptr::slice_from_raw_parts_mut(core::ptr::dangling_mut(), 0);
            let inner = unsafe { Box::from_raw_in(slice_ptr, alloc) };

            return Self { inner, length: 0 };
        }

        let layout = alloc::Layout::array::<T>(capacity).expect("invalid layout");
        let raw_ptr = alloc.allocate(layout).expect("`BoundedVec` allocation failed");

        let slice_ptr = core::ptr::slice_from_raw_parts_mut(raw_ptr.as_ptr().cast::<MaybeUninit<T>>(), capacity);

        let inner = unsafe { Box::from_raw_in(slice_ptr, alloc) };

        Self { inner, length: 0 }
    }

    /// Constructs a bounded vector directly from a pointer, a length, a capacity, and a
    /// memory allocator.
    ///
    /// # Safety
    ///
    /// This method is highly unsafe. A safe use of it must satisfy the following invariants:
    ///
    /// * The pointer `ptr` must be non-null.
    /// * The allocation referred to by `ptr` must have been allocated using the supplied
    ///   allocator.
    /// * The element type `T` must have the same alignment that `ptr` was allocated with.
    ///   The type `T` cannot have a less strict alignment is not sufficient; the alignment really
    ///   must be equal to satisfy the [`dealloc`] requirement that memory must be allocated and
    ///   deallocated with the same layout.
    /// * The allocation size in bytes (`mem::size_of::<T>() * capacity`) must
    ///   be the same size as the pointer was allocated with. Similar to alignment, [`dealloc`]
    ///   must be called with the same layout `size`.
    /// * The length `length` of the elements inside the allocation must be less than or equal to
    ///   the capacity `capacity`.
    /// * The first `length` values must be properly initialized values of type `T`.
    /// * `capacity` must be the capacity that the pointer was allocated with.
    /// * The allocated size in bytes must be no larger than `isize::MAX`.
    ///   See the safety documentation of [`pointer::offset`].
    ///
    /// These requirements always hold for any `ptr` that has been allocated via a [`BoundedVec`].
    ///
    /// The ownership of `ptr` is effectively transferred to the bounded vector which may then
    /// change the contents of memory pointed to by the pointer at will. The caller must ensure
    /// that nothing else uses the pointer `ptr` after calling this method.
    ///
    /// # Examples
    ///
    /// Using memory that was allocated by a bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// # use core::ptr;
    /// # use core::mem;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let original_capacity = 16;
    /// let mut original_vec = BoundedVec::with_capacity(original_capacity);
    /// original_vec.push(1);
    /// original_vec.push(2);
    /// original_vec.push(3);
    /// #
    /// # assert_eq!(original_vec.len(), 3);
    /// # assert_eq!(original_vec.capacity(), original_capacity);
    /// #
    ///
    /// // Prevent running `vec`'s destructor to completely control the allocation.
    /// let mut vec = mem::ManuallyDrop::new(original_vec);
    ///
    /// // Destructure `vec` into its constituent parts.
    /// let ptr: *mut i32 = vec.as_mut_ptr();
    /// let length = vec.len();
    /// let capacity = vec.capacity();
    /// let alloc = unsafe { ptr::read(vec.allocator()) };
    ///
    /// let expected = BoundedVec::from([4, 5, 6]);
    /// let result = unsafe {
    ///     // Mutate the values directly in memory.
    ///     for i in 0..length {
    ///         ptr::write(ptr.add(i), 4 + i as i32);
    ///     }
    ///
    ///     // Rebuild the vector.
    ///     BoundedVec::from_raw_parts_in(ptr, length, capacity, alloc)
    /// };
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), expected.len());
    /// assert_eq!(result.capacity(), original_capacity);
    /// # assert!(!result.is_empty());
    /// # assert_eq!(result.len(), length);
    /// # assert_eq!(result.capacity(), capacity);
    /// # assert!(result.len() <= result.capacity());
    /// # assert!(!expected.is_empty());
    /// # assert_eq!(expected.len(), length);
    /// # assert_eq!(expected.capacity(), 3);
    /// # assert!(expected.len() <= expected.capacity());
    ///
    /// let mut result = result;
    /// let new_capacity = capacity;
    ///
    /// // It is safe to work further with the vector since it satisfies the required invariants.
    /// for _ in 0..(new_capacity - length) {
    ///     result.push(i32::MAX);
    /// }
    ///
    /// let expected = BoundedVec::from([
    ///     4,        5,        6,        i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX,
    ///     i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX,
    /// ]);
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), expected.len());
    /// assert!(result.len() <= result.capacity());
    /// # assert_eq!(result.len(), new_capacity);
    /// # assert_eq!(result.capacity(), new_capacity);
    /// ```
    ///
    /// Using memory that was allocated outside a bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// # use core::ptr::NonNull;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::{Allocator, Layout, Global};
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::{Allocator, Layout, Global};
    /// #
    /// let value = 1_000_000;
    /// let layout = Layout::array::<u32>(16).unwrap();
    /// let alloc = Global;
    /// let length = 1;
    /// let capacity = 16;
    /// let vec = unsafe {
    ///     let mut memory: NonNull<u32> = alloc.allocate(layout).unwrap().cast::<u32>();
    ///     memory.write(value);
    ///
    ///     BoundedVec::from_raw_parts_in(memory.as_mut() as *mut u32, length, capacity, alloc)
    /// };
    ///
    /// assert_eq!(vec.as_slice(), &[value]);
    /// assert_eq!(vec.len(), length);
    /// assert_eq!(vec.capacity(), capacity);
    /// # assert!(!vec.is_empty());
    ///
    /// // It is safe to work further with the vector since it satisfies the required invariants.
    /// let mut result = vec;
    /// for _ in 0..(capacity - length) {
    ///     result.push(u32::MAX);
    /// }
    ///
    /// let expected = BoundedVec::from([
    ///     value,     u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX,
    ///     u32::MAX,  u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX,
    /// ]);
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), result.capacity());
    /// # assert_eq!(result.len(), capacity);
    /// # assert_eq!(result.capacity(), capacity);
    /// ```
    ///
    /// [`dealloc`]: std::alloc::Allocator::dealloc
    #[inline]
    pub unsafe fn from_raw_parts_in(ptr: *mut T, length: usize, capacity: usize, alloc: A) -> Self {
        unsafe {
            let data = slice::from_raw_parts_mut(ptr, capacity) as *mut [T] as *mut [MaybeUninit<T>];

            Self {
                inner: Box::from_raw_in(data, alloc),
                length,
            }
        }
    }

    /// Constructs a bounded vector directly from a non-null pointer, a length, a capacity,
    /// and a memory allocator.
    ///
    /// # Safety
    ///
    /// This method is highly unsafe. A safe use of it must satisfy the following invariants:
    ///
    /// * The pointer `ptr` must be non-null.
    /// * The allocation referred to by `ptr` must have been allocated using the supplied
    ///   allocator.
    /// * The element type `T` must have the same alignment that `ptr` was allocated with.
    ///   The type `T` cannot have a less strict alignment is not sufficient; the alignment really
    ///   must be equal to satisfy the [`dealloc`] requirement that memory must be allocated and
    ///   deallocated with the same layout.
    /// * The allocation size in bytes (`mem::size_of::<T>() * capacity`) must
    ///   be the same size as the pointer was allocated with. Similar to alignment, [`dealloc`]
    ///   must be called with the same layout size.
    /// * The length `length` of the elements inside the allocation must be less than or equal to
    ///   the capacity `capacity`.
    /// * The first `length` values must be properly initialized values of type `T`.
    /// * `capacity` must be the capacity that the pointer was allocated with.
    /// * The allocated size in bytes must be no larger than `isize::MAX`.
    ///   See the safety documentation of [`pointer::offset`].
    ///
    /// These requirements always hold for any `ptr` that has been allocated via a [`BoundedVec`].
    ///
    /// The ownership of `ptr` is effectively transferred to the bounded vector which may then
    /// change the contents of memory pointed to by the pointer at will. The caller must ensure
    /// that nothing else uses the pointer `ptr` after calling this method.
    ///
    /// # Examples
    ///
    /// Using memory that was allocated by a bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// # use core::ptr::NonNull;
    /// # use core::ptr;
    /// # use core::mem;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let original_capacity = 16;
    /// let mut original_vec = BoundedVec::with_capacity(original_capacity);
    /// original_vec.push(1);
    /// original_vec.push(2);
    /// original_vec.push(3);
    /// #
    /// # assert_eq!(original_vec.len(), 3);
    /// # assert_eq!(original_vec.capacity(), original_capacity);
    /// #
    ///
    /// // Prevent running `original_vec`'s destructor to completely control the allocation.
    /// let mut vec = mem::ManuallyDrop::new(original_vec);
    ///
    /// // Destructure `original_vec` into its constituent parts.
    /// let ptr: NonNull<i32> = vec.as_non_null();
    /// let length = vec.len();
    /// let capacity = vec.capacity();
    /// let alloc = unsafe { ptr::read(vec.allocator()) };
    ///
    /// let expected = BoundedVec::from([4, 5, 6]);
    /// let result = unsafe {
    ///     // Mutate the values directly in memory.
    ///     for i in 0..length {
    ///         ptr::write(ptr.as_ptr().add(i), 4 + i as i32);
    ///     }
    ///
    ///     // Rebuild the vector.
    ///     BoundedVec::from_parts_in(ptr, length, capacity, alloc)
    /// };
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), expected.len());
    /// assert_eq!(result.capacity(), original_capacity);
    /// # assert!(!result.is_empty());
    /// # assert_eq!(result.len(), length);
    /// # assert_eq!(result.capacity(), capacity);
    /// # assert!(result.len() <= result.capacity());
    /// # assert!(!expected.is_empty());
    /// # assert_eq!(expected.len(), length);
    /// # assert_eq!(expected.capacity(), 3);
    /// # assert!(expected.len() <= expected.capacity());
    ///
    /// let mut result = result;
    /// let new_capacity = 16;
    ///
    /// // It is safe to work further with the vector since it satisfies the required invariants.
    /// for _ in 0..(new_capacity - length) {
    ///     result.push(i32::MAX);
    /// }
    ///
    /// let expected = BoundedVec::from([
    ///     4,        5,        6,        i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX,
    ///     i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX,
    /// ]);
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), expected.len());
    /// assert!(result.len() <= result.capacity());
    /// # assert_eq!(result.len(), new_capacity);
    /// # assert_eq!(result.capacity(), new_capacity);
    /// ```
    ///
    /// Using memory that was allocated outside a bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// # use core::ptr::NonNull;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::{Allocator, Layout, Global};
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::{Allocator, Layout, Global};
    /// #
    /// let value = 1_000_000;
    /// let layout = Layout::array::<u32>(16).unwrap();
    /// let alloc = Global;
    /// let length = 1;
    /// let capacity = 16;
    /// let vec = unsafe {
    ///     let mut memory: NonNull<u32> = alloc.allocate(layout).unwrap().cast::<u32>();
    ///     memory.write(value);
    ///
    ///     BoundedVec::from_parts_in(memory, length, capacity, alloc)
    /// };
    ///
    /// assert_eq!(vec.as_slice(), &[value]);
    /// assert_eq!(vec.len(), length);
    /// assert_eq!(vec.capacity(), capacity);
    /// # assert!(!vec.is_empty());
    ///
    /// // It is safe to work further with the vector since it satisfies the required invariants.
    /// let mut result = vec;
    /// for _ in 0..(capacity - length) {
    ///     result.push(u32::MAX);
    /// }
    ///
    /// let expected = BoundedVec::from([
    ///     value,     u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX,
    ///     u32::MAX,  u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX,
    /// ]);
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), result.capacity());
    /// # assert_eq!(result.len(), capacity);
    /// # assert_eq!(result.capacity(), capacity);
    /// ```
    ///
    /// [`dealloc`]: std::alloc::Allocator::dealloc
    #[inline]
    pub unsafe fn from_parts_in(ptr: NonNull<T>, length: usize, capacity: usize, alloc: A) -> Self {
        unsafe {
            let data = slice::from_raw_parts_mut(ptr.as_ptr(), capacity) as *mut [T] as *mut [MaybeUninit<T>];

            Self {
                inner: Box::from_raw_in(data, alloc),
                length,
            }
        }
    }
}

impl<T> BoundedVec<T, alloc::Global> {
    /// Constructs a new empty bounded vector using a specific capacity.
    ///
    /// The vector will be able to hold at most `capacity` elements. The method will allocate
    /// space for **exactly** `capacity` elements. If `capacity` is zero, the constructor does not
    /// allocate memory.
    ///
    /// Note that while the returned vector will have a **capacity** of at least `capacity`, it
    /// will have a **length** of zero, because no elements have been pushed to the vector yet.
    ///
    /// # Panics
    ///
    /// This method panics if the new capacity exceeds `isize::MAX` _bytes_.
    ///
    /// # Examples
    ///
    /// Creating a bounded vector with capacity `capacity > 0`.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let capacity = 32;
    /// let vec: BoundedVec<i32> = BoundedVec::with_capacity(capacity);
    ///
    /// assert_eq!(vec.capacity(), capacity);
    /// assert!(vec.is_empty());
    /// ```
    ///
    /// Creating a bounded vector with capacity `capacity == 0`.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let vec: BoundedVec<i32> = BoundedVec::with_capacity(0);
    ///
    /// assert_eq!(vec.capacity(), 0);
    /// assert!(vec.is_empty());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_in(capacity, alloc::Global)
    }

    /// Constructs a bounded vector directly from a pointer, a length, and a capacity.
    ///
    /// # Safety
    ///
    /// This method is highly unsafe. A safe use of it must satisfy the following invariants:
    ///
    /// * The pointer `ptr` must be non-null.
    /// * The allocation referred to by `ptr` must have been allocated using the global allocator.
    /// * The element type `T` must have the same alignment that `ptr` was allocated with.
    ///   The type `T` cannot have a less strict alignment is not sufficient; the alignment really
    ///   must be equal to satisfy the [`dealloc`] requirement that memory must be allocated and
    ///   deallocated with the same layout.
    /// * The allocation size in bytes (`mem::size_of::<T>() * capacity`) must
    ///   be the same size as the pointer was allocated with. Similar to alignment, [`dealloc`]
    ///   must be called with the same layout `size`.
    /// * The length `length` of the elements inside the allocation must be less than or equal to
    ///   the capacity `capacity`.
    /// * The first `length` values must be properly initialized values of type `T`.
    /// * `capacity` must be the capacity that the pointer was allocated with.
    /// * The allocated size in bytes must be no larger than `isize::MAX`.
    ///   See the safety documentation of [`pointer::offset`].
    ///
    /// These requirements always hold for any `ptr` that has been allocated via a [`BoundedVec`].
    ///
    /// The ownership of `ptr` is effectively transferred to the bounded vector which may then
    /// change the contents of memory pointed to by the pointer at will. The caller must ensure
    /// that nothing else uses the pointer `ptr` after calling this method.
    ///
    /// # Examples
    ///
    /// Using memory that was allocated by a bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// # use core::ptr::NonNull;
    /// # use core::ptr;
    /// # use core::mem;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let original_capacity = 16;
    /// let mut original_vec = BoundedVec::with_capacity(original_capacity);
    /// original_vec.push(1);
    /// original_vec.push(2);
    /// original_vec.push(3);
    /// #
    /// # assert_eq!(original_vec.len(), 3);
    /// # assert_eq!(original_vec.capacity(), original_capacity);
    /// #
    ///
    /// // Prevent running `original_vec`'s destructor to completely control the allocation.
    /// let mut vec = mem::ManuallyDrop::new(original_vec);
    ///
    /// // Destructure `original_vec` into its constituent parts.
    /// let ptr: NonNull<i32> = vec.as_non_null();
    /// let length = vec.len();
    /// let capacity = vec.capacity();
    ///
    /// let expected = BoundedVec::from([4, 5, 6]);
    /// let result = unsafe {
    ///     // Mutate the values directly in memory.
    ///     for i in 0..length {
    ///         ptr::write(ptr.as_ptr().add(i), 4 + i as i32);
    ///     }
    ///
    ///     // Rebuild the vector.
    ///     BoundedVec::from_raw_parts(ptr.as_ptr(), length, capacity)
    /// };
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), expected.len());
    /// assert_eq!(result.capacity(), original_capacity);
    /// # assert!(!result.is_empty());
    /// # assert_eq!(result.len(), length);
    /// # assert_eq!(result.capacity(), capacity);
    /// # assert!(result.len() <= result.capacity());
    /// # assert!(!expected.is_empty());
    /// # assert_eq!(expected.len(), length);
    /// # assert_eq!(expected.capacity(), 3);
    /// # assert!(expected.len() <= expected.capacity());
    ///
    /// let mut result = result;
    /// let new_capacity = capacity;
    ///
    /// // It is safe to work further with the vector since it satisfies the required invariants.
    /// for _ in 0..(new_capacity - length) {
    ///     result.push(i32::MAX);
    /// }
    ///
    /// let expected = BoundedVec::from([
    ///     4,        5,        6,        i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX,
    ///     i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX,
    /// ]);
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), expected.len());
    /// assert!(result.len() <= result.capacity());
    /// # assert_eq!(result.len(), new_capacity);
    /// # assert_eq!(result.capacity(), new_capacity);
    /// ```
    ///
    /// Using memory that was allocated outside a bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// # use core::ptr::NonNull;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::{Allocator, Layout, Global};
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::{Allocator, Layout, Global};
    /// #
    /// let value = 1_000_000;
    /// let layout = Layout::array::<u32>(16).unwrap();
    /// let length = 1;
    /// let capacity = 16;
    /// let vec = unsafe {
    ///     let mut memory: NonNull<u32> = Global.allocate(layout).unwrap().cast::<u32>();
    ///     memory.write(value);
    ///
    ///     BoundedVec::from_raw_parts(memory.as_ptr(), length, capacity)
    /// };
    ///
    /// assert_eq!(vec.as_slice(), &[value]);
    /// assert_eq!(vec.len(), length);
    /// assert_eq!(vec.capacity(), capacity);
    /// # assert!(!vec.is_empty());
    ///
    /// // It is safe to work further with the vector since it satisfies the required invariants.
    /// let mut result = vec;
    /// for _ in 0..(capacity - length) {
    ///     result.push(u32::MAX);
    /// }
    ///
    /// let expected = BoundedVec::from([
    ///     value,     u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX,
    ///     u32::MAX,  u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX,
    /// ]);
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), result.capacity());
    /// # assert_eq!(result.len(), capacity);
    /// # assert_eq!(result.capacity(), capacity);
    /// ```
    ///
    /// [`dealloc`]: std::alloc::Allocator::dealloc
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut T, length: usize, capacity: usize) -> Self {
        unsafe { Self::from_raw_parts_in(ptr, length, capacity, alloc::Global) }
    }

    /// Constructs a bounded vector directly from a pointer, a length, and a capacity.
    ///
    /// # Safety
    ///
    /// This method is highly unsafe. A safe use of it must satisfy the following invariants:
    ///
    /// * The pointer `ptr` must be non-null.
    /// * The allocation referred to by `ptr` must have been allocated using the global allocator.
    /// * The element type `T` must have the same alignment that `ptr` was allocated with.
    ///   The type `T` cannot have a less strict alignment is not sufficient; the alignment really
    ///   must be equal to satisfy the [`dealloc`] requirement that memory must be allocated and
    ///   deallocated with the same layout.
    /// * The allocation size in bytes (`mem::size_of::<T>() * capacity`) must
    ///   be the same size as the pointer was allocated with. Similar to alignment, [`dealloc`]
    ///   must be called with the same layout `size`.
    /// * The length `length` of the elements inside the allocation must be less than or equal to
    ///   the capacity `capacity`.
    /// * The first `length` values must be properly initialized values of type `T`.
    /// * `capacity` must be the capacity that the pointer was allocated with.
    /// * The allocated size in bytes must be no larger than `isize::MAX`.
    ///   See the safety documentation of [`pointer::offset`].
    ///
    /// These requirements always hold for any `ptr` that has been allocated via a [`BoundedVec`].
    ///
    /// The ownership of `ptr` is effectively transferred to the bounded vector which may then
    /// change the contents of memory pointed to by the pointer at will. The caller must ensure
    /// that nothing else uses the pointer `ptr` after calling this method.
    ///
    /// # Examples
    ///
    /// Using memory that was allocated by a bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// # use core::ptr::NonNull;
    /// # use core::ptr;
    /// # use core::mem;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let original_capacity = 16;
    /// let mut original_vec = BoundedVec::with_capacity(original_capacity);
    /// original_vec.push(1);
    /// original_vec.push(2);
    /// original_vec.push(3);
    /// #
    /// # assert_eq!(original_vec.len(), 3);
    /// # assert_eq!(original_vec.capacity(), original_capacity);
    /// #
    ///
    /// // Prevent running `original_vec`'s destructor to completely control the allocation.
    /// let mut vec = mem::ManuallyDrop::new(original_vec);
    ///
    /// // Destructure `original_vec` into its constituent parts.
    /// let ptr: NonNull<i32> = vec.as_non_null();
    /// let length = vec.len();
    /// let capacity = vec.capacity();
    ///
    /// let expected = BoundedVec::from([4, 5, 6]);
    /// let result = unsafe {
    ///     // Mutate the values directly in memory.
    ///     for i in 0..length {
    ///         ptr::write(ptr.as_ptr().add(i), 4 + i as i32);
    ///     }
    ///
    ///     // Rebuild the vector.
    ///     BoundedVec::from_parts(ptr, length, capacity)
    /// };
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), expected.len());
    /// assert_eq!(result.capacity(), original_capacity);
    /// # assert!(!result.is_empty());
    /// # assert_eq!(result.len(), length);
    /// # assert_eq!(result.capacity(), capacity);
    /// # assert!(result.len() <= result.capacity());
    /// # assert!(!expected.is_empty());
    /// # assert_eq!(expected.len(), length);
    /// # assert_eq!(expected.capacity(), 3);
    /// # assert!(expected.len() <= expected.capacity());
    ///
    /// let mut result = result;
    /// let new_capacity = 16;
    ///
    /// // It is safe to work further with the vector since it satisfies the required invariants.
    /// for _ in 0..(new_capacity - length) {
    ///     result.push(i32::MAX);
    /// }
    ///
    /// let expected = BoundedVec::from([
    ///     4,        5,        6,        i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX,
    ///     i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX, i32::MAX,
    /// ]);
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), expected.len());
    /// assert!(result.len() <= result.capacity());
    /// # assert_eq!(result.len(), new_capacity);
    /// # assert_eq!(result.capacity(), new_capacity);
    /// ```
    ///
    /// Using memory that was allocated outside a bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// # use core::ptr::NonNull;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::{Allocator, Layout, Global};
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::{Allocator, Layout, Global};
    /// #
    /// let value = 1_000_000;
    /// let layout = Layout::array::<u32>(16).unwrap();
    /// let length = 1;
    /// let capacity = 16;
    /// let vec = unsafe {
    ///     let mut memory: NonNull<u32> = Global.allocate(layout).unwrap().cast::<u32>();
    ///     memory.write(value);
    ///
    ///     BoundedVec::from_parts(memory, length, capacity)
    /// };
    ///
    /// assert_eq!(vec.as_slice(), &[value]);
    /// assert_eq!(vec.len(), length);
    /// assert_eq!(vec.capacity(), capacity);
    /// # assert!(!vec.is_empty());
    ///
    /// // It is safe to work further with the vector since it satisfies the required invariants.
    /// let mut result = vec;
    /// for _ in 0..(capacity - length) {
    ///     result.push(u32::MAX);
    /// }
    ///
    /// let expected = BoundedVec::from([
    ///     value,     u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX,
    ///     u32::MAX,  u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX, u32::MAX,
    /// ]);
    ///
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// assert_eq!(result.len(), result.capacity());
    /// # assert_eq!(result.len(), capacity);
    /// # assert_eq!(result.capacity(), capacity);
    /// ```
    ///
    /// [`dealloc`]: std::alloc::Allocator::dealloc
    #[inline]
    pub unsafe fn from_parts(ptr: NonNull<T>, length: usize, capacity: usize) -> Self {
        unsafe { Self::from_parts_in(ptr, length, capacity, alloc::Global) }
    }
}

impl<T, A> BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    /// Returns the capacity of a bounded vector.
    ///
    /// The **capacity** of a bounded vector is the number of elements the vector can hold.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let capacity = 32;
    /// let mut vec: BoundedVec<i32, Global> = BoundedVec::with_capacity_in(capacity, Global);
    ///
    /// assert_eq!(vec.capacity(), capacity);
    /// assert_eq!(vec.len(), 0);
    ///
    /// for i in 0..capacity {
    ///     vec.push(i as i32);
    /// }
    ///
    /// assert_eq!(vec.capacity(), capacity);
    /// assert_eq!(vec.len(), capacity);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.len()
    }

    /// Returns the length of a bounded vector.
    ///
    /// The **length** of a bounded vector is the number of elements stored inside it.
    /// The length satisfies the following. Given a bounded vector `vec`
    ///
    /// ```text
    /// vec.len() ≤ vec.capacity().
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let len = 32;
    /// let mut vec: BoundedVec<i32, Global> = BoundedVec::with_capacity_in(len, Global);
    ///
    /// assert_eq!(vec.len(), 0);
    /// # assert_eq!(vec.capacity(), len);
    ///
    /// for i in 0..len {
    ///     vec.push(i as i32);
    /// }
    ///
    /// assert_eq!(vec.len(), len);
    /// # assert_eq!(vec.capacity(), len);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.length
    }

    /// Determines whether a bounded vector is empty or not.
    ///
    /// A bounded vector is **empty** if it contains no elements, i.e. its length is zero.
    /// This method satisfies the following. Given a bounded vector `vec`
    ///
    /// ```text
    /// vec.is_empty() ⇔ vec.len() = 0.
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec: BoundedVec<i32, Global> = BoundedVec::with_capacity_in(1, Global);
    ///
    /// assert!(vec.is_empty());
    ///
    /// vec.push(1);
    ///
    /// assert!(!vec.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the memory allocator from the bounded vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let vec: BoundedVec<i32> = BoundedVec::with_capacity(11);
    ///
    /// assert!(vec.is_empty());
    ///
    /// let alloc: &Global = vec.allocator();
    /// ```
    #[inline]
    pub fn allocator(&self) -> &A {
        Box::allocator(&self.inner)
    }
}

impl<T, A> BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    /// Returns an immutable slice of the elements of a bounded vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let array: [i32; 3] = [9, 28, 37];
    /// let vec = BoundedVec::from(array);
    ///
    /// let expected = array.as_slice();
    /// let result = vec.as_slice();
    ///
    /// assert_eq!(result, expected);
    /// assert_eq!(result.len(), vec.len());
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: The first `self.length` elements have been initialized.
        unsafe { slice::from_raw_parts(self.inner.as_ptr().cast::<T>(), self.length) }
    }

    /// Returns a mutable slice of the elements of a bounded vector.
    ///
    /// # Examples
    ///
    /// Getting a mutable slice of a bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut array: [i32; 3] = [9, 28, 37];
    /// let mut vec = BoundedVec::from(array);
    ///
    /// let expected = array.as_mut_slice();
    /// let result = vec.as_mut_slice();
    ///
    /// assert_eq!(result, expected);
    /// assert_eq!(result.len(), vec.len());
    /// ```
    ///
    /// Getting and mutating a mutable slice of a bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut array: [i32; 3] = [9, 28, 37];
    /// let mut vec = BoundedVec::from(array);
    /// {
    ///     let slice = vec.as_mut_slice();
    ///     for i in 0..slice.len() {
    ///         slice[i] = 2 * slice[i];
    ///     }
    /// }
    ///
    /// let expected_array = [18, 56, 74];
    /// let expected = expected_array.as_slice();
    /// let result = vec.as_slice();
    ///
    /// assert_eq!(result, expected);
    /// assert_eq!(result.len(), vec.len());
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: The first `self.length` elements have been initialized.
        unsafe { slice::from_raw_parts_mut(self.inner.as_mut_ptr().cast::<T>(), self.length) }
    }

    /// Returns a raw pointer to the vector's buffer, or a dangling raw pointer valid for zero
    /// sized reads if the vector didn't allocate.
    ///
    /// The caller must ensure that the vector outlives the pointer this function returns, or else
    /// it will end up dangling. Modifying the vector may cause its underlying buffer to be
    /// reallocated, which would also invalidate any existing pointers to its elements.
    ///
    /// The caller must also ensure that the memory the pointer (non-transitively) points to
    /// is never written to (except inside an `UnsafeCell`) using this pointer or any pointer
    /// derived from it. If you need to mutate the contents of the slice, use
    /// [`as_mut_ptr`].
    ///
    /// This method guarantees that for the purpose of the aliasing model, this method
    /// does not materialize a reference to the underlying slice, and thus the returned pointer
    /// will remain valid when mixed with other calls to [`as_ptr`], [`as_mut_ptr`],
    /// and [`as_non_null`].
    ///
    /// Note that calling other methods that materialize mutable references to the slice,
    /// or mutable references to specific elements you are planning on accessing through this
    /// pointer, as well as writing to those elements, may still invalidate this pointer.
    /// See the second example below for how this guarantee can be used.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let vec = BoundedVec::from([1, 2, 4, 8]);
    /// let ptr = vec.as_ptr();
    ///
    /// unsafe {
    ///     for i in 0..vec.len() {
    ///         assert_eq!(*ptr.add(i), 1 << i);
    ///     }
    /// }
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 4, 8]);
    /// ```
    ///
    /// Due to the aliasing guarantee, the following code is legal:
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([0, 1, 2]);
    ///
    /// unsafe {
    ///     let ptr1 = vec.as_ptr();
    ///     let _ = ptr1.read();
    ///     let ptr2 = vec.as_mut_ptr().offset(2);
    ///     ptr2.write(2);
    ///     // Notably, writing to `ptr2` did **not** invalidate `ptr1`
    ///     // because it mutated a different element:
    ///     let _ = ptr1.read();
    /// }
    /// ```
    ///
    /// [`as_mut_ptr`]: BoundedVec::as_mut_ptr
    /// [`as_ptr`]: BoundedVec::as_ptr
    /// [`as_non_null`]: BoundedVec::as_non_null
    #[inline]
    #[must_use]
    pub fn as_ptr(&self) -> *const T {
        self.inner.as_ptr().cast::<T>()
    }

    /// Returns a raw mutable pointer to the vector's buffer, or a dangling raw pointer valid for
    /// zero sized reads if the vector didn't allocate.
    ///
    /// The caller must ensure that the vector outlives the pointer this function returns, or else
    /// it will end up dangling. Modifying the vector may cause its underlying buffer to be
    /// reallocated, which would also invalidate any existing pointers to its elements.
    ///
    /// This method guarantees that for the purpose of the aliasing model, this method
    /// does not materialize a reference to the underlying slice, and thus the returned pointer
    /// will remain valid when mixed with other calls to [`as_ptr`], [`as_mut_ptr`],
    /// and [`as_non_null`].
    ///
    /// Note that calling other methods that materialize references to the slice,
    /// or references to specific elements you are planning on accessing through this pointer,
    /// may still invalidate this pointer.
    /// See the second example below for how this guarantee can be used.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// // Allocate vector big enough for 4 elements.
    /// let length = 4;
    /// let mut vec: BoundedVec<i32> = BoundedVec::with_capacity(length);
    /// let ptr = vec.as_mut_ptr();
    ///
    /// // Initialize elements via raw pointer writes, then set the length.
    /// unsafe {
    ///     for i in 0..length {
    ///         *ptr.add(i) = (i + 1) as i32;
    ///     }
    ///     vec.set_len(length);
    /// }
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 3, 4]);
    /// ```
    ///
    /// Due to the aliasing guarantee, the following code is legal:
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec: BoundedVec<i32> = BoundedVec::with_capacity(4);
    /// vec.push(0);
    ///
    /// unsafe {
    ///     let ptr1 = vec.as_mut_ptr();
    ///     ptr1.write(1);
    ///     let ptr2 = vec.as_mut_ptr();
    ///     ptr2.write(2);
    ///     // Notably, writing to `ptr2` did **not** invalidate `ptr1`:
    ///     ptr1.write(3);
    /// }
    /// ```
    ///
    /// [`as_mut_ptr`]: BoundedVec::as_mut_ptr
    /// [`as_ptr`]: BoundedVec::as_ptr
    /// [`as_non_null`]: BoundedVec::as_non_null
    #[inline]
    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr().cast::<T>()
    }

    /// Returns a [`NonNull`] pointer to the vector's buffer, or a dangling [`NonNull`] pointer
    /// valid for zero sized reads if the vector didn't allocate.
    ///
    /// The caller must ensure that the vector outlives the pointer this function returns, or else
    /// it will end up dangling. Modifying the vector may cause its underlying buffer to be
    /// reallocated, which would also invalidate any existing pointers to its elements.
    ///
    /// This method guarantees that for the purpose of the aliasing model, this method
    /// does not materialize a reference to the underlying slice, and thus the returned pointer
    /// will remain valid when mixed with other calls to [`as_ptr`], [`as_mut_ptr`],
    /// and [`as_non_null`].
    ///
    /// Note that calling other methods that materialize references to the slice,
    /// or references to specific elements you are planning on accessing through this pointer,
    /// may still invalidate this pointer.
    /// See the second example below for how this guarantee can be used.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// // Allocate vector big enough for 4 elements.
    /// let length = 4;
    /// let mut vec = BoundedVec::with_capacity(length);
    /// let ptr = vec.as_non_null();
    ///
    /// // Initialize elements via raw pointer writes, then set length.
    /// unsafe {
    ///     for i in 0..length {
    ///         ptr.add(i).write((i + 1) as i32);
    ///     }
    ///     vec.set_len(length);
    /// }
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 3, 4]);
    /// ```
    ///
    /// Due to the aliasing guarantee, the following code is legal:
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::with_capacity(4);
    ///
    /// unsafe {
    ///     let ptr1 = vec.as_non_null();
    ///     ptr1.write(1);
    ///     let ptr2 = vec.as_non_null();
    ///     ptr2.write(2);
    ///     // Notably, writing to `ptr2` did **not** invalidate `ptr1`:
    ///     ptr1.write(3);
    /// }
    /// ```
    ///
    /// [`as_mut_ptr`]: BoundedVec::as_mut_ptr
    /// [`as_ptr`]: BoundedVec::as_ptr
    /// [`as_non_null`]: BoundedVec::as_non_null
    #[inline]
    #[must_use]
    pub fn as_non_null(&mut self) -> NonNull<T> {
        // SAFETY: A bounded vector always holds a non-null pointer.
        unsafe {
            let ptr = NonNull::new_unchecked(Box::as_mut_ptr(&mut self.inner));

            ptr.cast::<T>()
        }
    }

    /// Consumes and leaks a bounded vector, returning a mutable reference to its contents as a
    /// slice.
    ///
    /// Note that the type `T` must outlive the chosen lifetime `'a`. If the type has only static
    /// references, or no references at all, the lifetime can be chosen to be `'static`. This
    /// method preserves the size of the underlying allocation for the vector, so the leaked
    /// allocation may include unused capacity that is not part of the returned slice.
    ///
    /// The primary use case for this function is for data that lives for the remainder of the
    /// run time of the program. Dropping this reference will leak memory.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let vec = BoundedVec::from([1, 2, 3, 4, 5]);
    /// #
    /// # assert_eq!(vec.len(), 5);
    /// # assert_eq!(vec.capacity(), 5);
    /// #
    /// let static_ref: &'static mut [usize] = vec.leak();
    ///
    /// assert_eq!(static_ref.len(), 5);
    /// assert_eq!(static_ref, &[1, 2, 3, 4, 5]);
    ///
    /// static_ref[3] = usize::MAX;
    ///
    /// assert_eq!(static_ref.len(), 5);
    /// assert_eq!(static_ref, &[1, 2, 3, usize::MAX, 5]);
    /// ```
    #[inline]
    pub fn leak<'a>(self) -> &'a mut [T]
    where
        A: 'a,
    {
        let mut me = ManuallyDrop::new(self);

        unsafe { slice::from_raw_parts_mut(me.as_mut_ptr(), me.length) }
    }

    /// Decomposes a bounded vector with any memory allocator into its constituent parts:
    /// `(pointer, length, capacity, allocator)`.
    ///
    /// This method returns a pointer to the memory allocation containing the vector, the
    /// length of the vector inside the allocation, the capacity of the vector (the
    /// length in elements of the memory allocation), and the underlying memory allocator that
    /// manages the memory allocation. These are the same arguments in the same order as the
    /// arguments to [`from_raw_parts_in`].
    ///
    /// After decomposing the vector, the user must ensure that they properly manage the
    /// memory allocation pointed to by the raw pointer. The primary way to do this is to convert
    /// the pointer into another data structure such as a [`Vec`] or [`BoundedVec`].
    ///
    /// [`from_raw_parts_in`]: BoundedVec::from_raw_parts_in
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let array: [i32; 3] = [-1, 0, 1];
    /// let vec = BoundedVec::from(array);
    ///
    /// assert_eq!(vec.as_slice(), &[-1, 0, 1]);
    ///
    /// let (ptr, length, capacity, alloc) = vec.into_raw_parts_with_alloc();
    /// let reinterpreted = unsafe {
    ///     let ptr = ptr as *mut u32;
    ///     BoundedVec::from_raw_parts_in(ptr, length, capacity, alloc)
    /// };
    ///
    /// assert_eq!(reinterpreted.as_slice(), &[4294967295, 0, 1]);
    /// ```
    #[must_use]
    pub fn into_raw_parts_with_alloc(self) -> (*mut T, usize, usize, A) {
        let mut me = ManuallyDrop::new(self);
        let length = me.len();
        let capacity = me.capacity();
        let ptr = me.as_mut_ptr();
        let alloc = unsafe { core::ptr::read(me.allocator()) };

        (ptr, length, capacity, alloc)
    }

    /// Decomposes a bounded vector with the global allocator into its constituent parts:
    /// `(non-null pointer, length, capacity)`.
    ///
    /// This method returns a [`NonNull`] pointer to the memory allocation containing the vector,
    /// the length of the vector inside the allocation, and the capacity of the vector (the
    /// length in elements of the memory allocation). These are the same arguments in the same
    /// order as the arguments to [`from_parts_in`].
    ///
    /// After decomposing the vector, the user must ensure that they properly manage the
    /// memory allocation pointed to by the raw pointer. The primary way to do this is to convert
    /// the pointer into another data structure such as a [`Vec`] or [`BoundedVec`].
    ///
    /// [`from_parts_in`]: BoundedVec::from_parts_in
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let array: [i32; 3] = [-1, 0, 1];
    /// let vec = BoundedVec::from(array);
    ///
    /// assert_eq!(vec.as_slice(), &[-1, 0, 1]);
    ///
    /// let (ptr, length, capacity, alloc) = vec.into_parts_with_alloc();
    /// let reinterpreted = unsafe {
    ///     let ptr = ptr.cast::<u32>();
    ///     BoundedVec::from_parts_in(ptr, length, capacity, alloc)
    /// };
    ///
    /// assert_eq!(reinterpreted.as_slice(), &[4294967295, 0, 1]);
    /// ```
    #[must_use]
    pub fn into_parts_with_alloc(self) -> (NonNull<T>, usize, usize, A) {
        let mut me = ManuallyDrop::new(self);

        // SAFETY: A bounded vector always has a non-null pointer.
        let ptr = unsafe { NonNull::new_unchecked(me.as_mut_ptr()) };
        let len = me.len();
        let capacity = me.capacity();
        let alloc = unsafe { core::ptr::read(me.allocator()) };

        (ptr, len, capacity, alloc)
    }
}

impl<T> BoundedVec<T, alloc::Global> {
    /// Decomposes a bounded vector with the global allocator into its constituent parts:
    /// `(pointer, length, capacity)`.
    ///
    /// This method returns a pointer to the memory allocation containing the vector, the
    /// length of the vector inside the allocation, and the capacity of the vector (the
    /// length in elements of the memory allocation). These are the same arguments in the same
    /// order as the arguments to [`from_raw_parts`].
    ///
    /// After decomposing the vector, the user must ensure that they properly manage the
    /// memory allocation pointed to by the raw pointer. The primary way to do this is to convert
    /// the pointer into another data structure such as a [`Vec`] or [`BoundedVec`].
    ///
    /// [`from_raw_parts`]: BoundedVec::from_raw_parts
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let array: [i32; 3] = [-1, 0, 1];
    /// let vec = BoundedVec::from(array);
    ///
    /// assert_eq!(vec.as_slice(), &[-1, 0, 1]);
    ///
    /// let (ptr, length, capacity) = vec.into_raw_parts();
    /// let reinterpreted = unsafe {
    ///     let ptr = ptr as *mut u32;
    ///     BoundedVec::from_raw_parts(ptr, length, capacity)
    /// };
    ///
    /// assert_eq!(reinterpreted.as_slice(), &[4294967295, 0, 1]);
    /// ```
    #[must_use]
    pub fn into_raw_parts(self) -> (*mut T, usize, usize) {
        let mut me = ManuallyDrop::new(self);
        let length = me.len();
        let capacity = me.capacity();
        let ptr = me.as_mut_ptr();

        (ptr, length, capacity)
    }

    /// Decomposes a bounded vector with the global allocator into its constituent parts:
    /// `(non-null pointer, length, capacity)`.
    ///
    /// This method returns a [`NonNull`] pointer to the memory allocation containing the vector,
    /// the length of the vector inside the allocation, and the capacity of the vector (the
    /// length in elements of the memory allocation). These are the same arguments in the same
    /// order as the arguments to [`from_parts`].
    ///
    /// After decomposing the vector, the user must ensure that they properly manage the
    /// memory allocation pointed to by the raw pointer. The primary way to do this is to convert
    /// the pointer into another data structure such as a [`Vec`] or [`BoundedVec`].
    ///
    /// [`from_parts`]: BoundedVec::from_parts
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let array: [i32; 3] = [-1, 0, 1];
    /// let vec = BoundedVec::from(array);
    ///
    /// assert_eq!(vec.as_slice(), &[-1, 0, 1]);
    ///
    /// let (ptr, length, capacity) = vec.into_parts();
    /// let reinterpreted = unsafe {
    ///     let ptr = ptr.cast::<u32>();
    ///     BoundedVec::from_parts(ptr, length, capacity)
    /// };
    ///
    /// assert_eq!(reinterpreted.as_slice(), &[4294967295, 0, 1]);
    /// ```
    #[must_use]
    pub fn into_parts(self) -> (NonNull<T>, usize, usize) {
        let mut me = ManuallyDrop::new(self);

        // SAFETY: A bounded vector always has a non-null pointer.
        let ptr = unsafe { NonNull::new_unchecked(me.as_mut_ptr()) };
        let len = me.len();
        let capacity = me.capacity();

        (ptr, len, capacity)
    }
}

impl<T, A> BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    /// Appends a new element to the end of a bounded vector.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector. Let `vec_before` be the state of `vec` before this method is called,
    /// and let `vec_after` be the state of `vec` after this method is completed.
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { vec_before.len() < vec_before.capacity() }
    /// vec.push(value)
    /// {
    ///     vec_after.len() = vec_before.len() + 1
    ///     ∧ (∀ i ∈ [0, vec_before.len()). vec_after[i] = vec_before[i])
    /// }
    ///
    /// { vec_before.len() = vec_before.capacity() }
    /// vec.push(value)
    /// { ⊥ }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Complexity Characteristics
    ///
    /// This method runs in **O(1)** time.
    ///
    /// # Panics
    ///
    /// This method panics if `self.len() == self.capacity()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::with_capacity(4);
    ///
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), 4);
    ///
    /// vec.push(1);
    /// vec.push(2);
    ///
    /// assert_eq!(vec.len(), 2);
    /// assert_eq!(vec.capacity(), 4);
    ///
    /// vec.push(3);
    ///
    /// assert_eq!(vec.len(), 3);
    /// assert_eq!(vec.capacity(), 4);
    /// assert_eq!(vec.as_slice(), &[1, 2, 3]);
    /// ```
    #[inline]
    #[track_caller]
    pub fn push(&mut self, value: T) {
        let length = self.len();

        assert!(length < self.capacity());

        self.inner[length] = MaybeUninit::new(value);
        self.length += 1;
    }

    /// Appends an element to a bounded vector if there is sufficient spare capacity.
    /// Otherwise, an error is returned with the element.
    ///
    /// Unlike [`push`], this method will not panic when there's insufficient capacity.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector. Let `vec_before` be the state of `vec` before this method is called,
    /// let `vec_after` be the state of `vec` after this method completes. Let `result` be the
    /// value that this method returns after completing.
    ///
    /// We say that `vec_after` is **equal to** `vec_before` if and only if
    ///
    /// ```text
    /// vec_after = vec_before ⇔
    ///     (vec_before.len() = vec_after.len())
    ///     ∧ (∀ i ∈ [0, vec_before.len()). vec_after[i] = vec_before[i]).
    /// ```
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { vec_before.len() < vec_before.capacity() }
    /// vec.push_within_capacity(value)
    /// {
    ///     result = Ok(())
    ///     ∧ vec_after.len() = vec_before.len() + 1
    ///     ∧ vec_after[vec_before.len()] = value
    ///     ∧ (∀ i ∈ [0, vec_before.len()). vec_after[i] = vec_before[i])
    /// }
    ///
    /// { vec_before.len() = vec_before.capacity() }
    /// vec.push_within_capacity(value)
    /// { result = Err(value) ∧ vec_after = vec_before }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Complexity Characteristics
    ///
    /// This method runs in **O(1)** time.
    ///
    /// # Examples
    ///
    /// Pushing elements to the vector within the capacity of the vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let min_capacity = 4;
    /// let mut vec = BoundedVec::with_capacity(min_capacity);
    ///
    /// for i in 0..min_capacity {
    ///     let result = vec.push_within_capacity((i + 1) as i32);
    ///     assert!(result.is_ok());
    /// }
    /// assert!(vec.capacity() >= min_capacity);
    /// assert_eq!(vec.len(), min_capacity);
    /// assert_eq!(vec.as_slice(), &[1, 2, 3, 4]);
    /// ```
    ///
    /// Trying to push elements past the capacity of the vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let min_capacity = 4;
    /// let mut vec = BoundedVec::with_capacity(min_capacity);
    ///
    /// assert!(vec.capacity() >= min_capacity);
    /// let actual_capacity = vec.capacity();
    /// for i in 0..actual_capacity {
    ///     let result = vec.push_within_capacity((i + 1) as i32);
    ///     assert!(result.is_ok());
    ///     assert_eq!(vec.capacity(), actual_capacity);
    /// }
    ///
    /// let result = vec.push_within_capacity(i32::MAX);
    /// assert!(result.is_err());
    /// assert_eq!(vec.capacity(), actual_capacity);
    /// ```
    ///
    /// [`push`]: BoundedVec::push
    #[inline]
    pub fn push_within_capacity(&mut self, value: T) -> Result<(), T> {
        let length = self.len();
        if length >= self.capacity() {
            return Err(value);
        }

        self.inner[length] = MaybeUninit::new(value);
        self.length += 1;

        Ok(())
    }

    /// Removes and returns the last element in a bounded vector if the vector is non-empty,
    /// and returns `None` if the collection is empty.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector. Let `vec_before` be the state of `vec` before this method is called,
    /// let `vec_after` be the state of `vec` after this method completes. Let `result` be the
    /// value that this method returns after completing.
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { vec_before.len() = 0 }
    /// vec.pop()
    /// { (result = None) ∧ (vec_after.len() = 0) }
    ///
    /// { vec_before.len() > 0 }
    /// vec.pop()
    /// {
    ///     result = Some(vec_before[vec_before.len() - 1])
    ///     ∧ (vec_after.len() = vec_before.len() - 1)
    ///     ∧ (∀ i ∈ [0, vec_after.len()). vec_after[i] = vec_before[i]).
    /// }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Complexity Characteristics
    ///
    /// This method runs in **O(1)** time.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([1, 2, 3]);
    ///
    /// assert!(!vec.is_empty());
    ///
    /// assert_eq!(vec.pop(), Some(3));
    /// assert_eq!(vec.pop(), Some(2));
    /// assert_eq!(vec.pop(), Some(1));
    ///
    /// assert!(vec.is_empty());
    ///
    /// assert_eq!(vec.pop(), None);
    /// ```
    #[inline]
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

    /// Removes and returns the last element from a vector depending on whether it satisfies the
    /// provided predicate.
    ///
    /// This method returns behaves as follows:
    /// * If the vector is nonempty, let `value` be the last element in the vector. If
    ///   `predicate(value) == true`, this method returns `Some(value)`. If
    ///   `predicate(value) == false`, this method returns `None`.
    /// * If the vector is empty, this method returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([
    ///     "foo",
    ///     "bar",
    ///     "baz",
    ///     "quux",
    /// ]);
    /// let predicate = |st: &mut &str| { st.len() % 2 == 0 };
    ///
    /// assert_eq!(vec.pop_if(predicate), Some("quux"));
    /// assert_eq!(vec.as_slice(), &["foo", "bar", "baz"]);
    /// assert_eq!(vec.pop_if(predicate), None);
    /// assert_eq!(vec.as_slice(), &["foo", "bar", "baz"]);
    /// ```
    pub fn pop_if<F>(&mut self, predicate: F) -> Option<T>
    where
        F: FnOnce(&mut T) -> bool,
    {
        let last = self.last_mut()?;
        if predicate(last) { self.pop() } else { None }
    }

    /// Removes all values from the bounded vector.
    ///
    /// After calling this method, the collection will be empty. This method does not change the
    /// allocated capacity of the bounded vector.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector, `vec_before` be the state of `vec` before this method is called,
    /// and `vec_after` be the state of `vec` after this method completes.
    ///
    /// We say that `vec` **contains** a value `e :: T`, or that `e` is an **element of** `vec` if
    /// the following holds:
    ///
    /// ```text
    /// ∀ e :: T. (e ∈ vec) ⇔ (∃ i ∈ [0, vec.len()). vec[i] = e).
    /// ```
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { true }
    /// vec.clear()
    /// { (vec_after.len() = 0) ∧ (∀ e ∈ vec_before. e ∉ vec_after) }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Complexity Characteristics
    ///
    /// This method runs in **O(n)** time, where `n` is an affine function of the length of the
    /// vector.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::with_capacity(10);
    ///
    /// vec.extend([1, 2, 3]);
    ///
    /// assert_eq!(vec.len(), 3);
    ///
    /// let old_capacity = vec.capacity();
    /// vec.clear();
    ///
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), old_capacity);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        let values: *mut [T] = self.as_mut_slice();
        unsafe {
            self.length = 0;
            core::ptr::drop_in_place(values);
        }
    }

    /// Inserts a new value into a bounded vector, replacing the old value.
    ///
    /// This method behaves with respect to `index` as follows:
    ///
    /// * If `index < self.len()`, it replaces the existing value at `index`.
    /// * If `index == self.len()` and `self.len() < self.capacity()`, it pushes `value` to the
    ///   end of the collection.
    /// * If `index == self.len()` and `self.len() == self.capacity()`, it panics.
    /// * If `index > self.len()`, it panics.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector, `vec_before` be the state of `vec` before this method is called,
    /// and let `vec_after` be the state of `vec` after this method completes.
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { index < vec_before.len() }
    /// vec.replace_insert(index, value)
    /// {
    ///     vec_after.len() = vec_before.len()
    ///     ∧ vec_after[index] = value
    ///     ∧ (∀ i ∈ [0, vec_before.len()). i ≠ index ⇒ vec_after[i] = vec_before[i])
    /// }
    ///
    /// { index = vec_before.len() ∧ vec_before.len() < vec_before.capacity() }
    /// vec.replace_insert(index, value)
    /// {
    ///     vec_after.len() = vec_before.len() + 1
    ///     ∧ vec_after[index] = value
    ///     ∧ (∀ i ∈ [0, vec_before.len()). vec_after[i] = vec_before[i])
    /// }
    ///
    /// { index = vec_before.len() ∧ vec_before.len() = vec_before.capacity() }
    /// vec.replace_insert(index, value)
    /// { ⊥ }
    ///
    /// { index > vec_before.len() }
    /// vec.replace_insert(index, value)
    /// { ⊥ }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Complexity Characteristics
    ///
    /// This method runs in **O(1)** time.
    ///
    /// # Panics
    ///
    /// This method panics if the index `index` is larger than the length of the collection, or
    /// if the collection is full, i.e. `self.len() == self.capacity()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::with_capacity(3);
    ///
    /// assert!(vec.is_empty());
    ///
    /// vec.replace_insert(0, 1);
    ///
    /// assert_eq!(vec.len(), 1);
    /// assert_eq!(vec.as_slice(), &[1]);
    ///
    /// vec.replace_insert(0, 2);
    ///
    /// assert_eq!(vec.len(), 1);
    /// assert_eq!(vec.as_slice(), &[2]);
    /// ```
    #[inline]
    pub fn replace_insert(&mut self, index: usize, value: T) {
        #[cold]
        #[track_caller]
        #[cfg_attr(feature = "nightly", optimize(size))]
        fn index_out_of_bounds_failure(index: usize, length: usize) -> ! {
            panic!(
                "replace_insert index out of bounds: Got index `{index}`. Need index `{index}` <= len, where len is `{length}`."
            );
        }

        #[cold]
        #[track_caller]
        #[cfg_attr(feature = "nightly", optimize(size))]
        fn capacity_failure(length: usize) -> ! {
            panic!("replace_insert vector full: The insertion index is `{length}`, and the vector is full");
        }

        let length = self.len();
        if index > length {
            index_out_of_bounds_failure(index, length);
        }

        if (index == length) && (length == self.capacity()) {
            capacity_failure(length);
        }

        unsafe {
            if index < length {
                let value_ptr = self.as_mut_ptr().add(index);

                let _old_value = core::ptr::read(value_ptr);

                core::ptr::write(value_ptr, value);
            } else {
                let value_ptr = self.as_mut_ptr().add(index);

                // SAFETY: We are pushing to the end of the vector, so no dropping is needed.
                core::ptr::write(value_ptr, value);

                // We pushed to the vector instead of replacing a value inside the vector.
                self.set_len(length + 1);
            }
        }
    }

    /// Inserts a new value into a bounded vector, shifting the old value and all values
    /// after it up in the collection.
    ///
    /// This method behaves with respect to `index` as follows:
    ///
    /// * If `index < self.len()` and `self.len() < self.capacity()`, it shifts the current value
    ///   at `index` and all successive values in the collection up one index. This method inserts
    ///   the value `value` at the position with index `index`.
    /// * If `index == self.len()` and `self.len() < self.capacity()`, it pushes `value` to the
    ///   end of the collection.
    /// * If `index <= self.len()` and `self.len() == self.capacity()`, it panics.
    /// * If `index > self.len()`, it panics.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector, `vec_before` be the state of `vec` before this method is called,
    /// and let `vec_after` be the state of `vec` after this method completes.
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { index < vec_before.len() ∧ vec_before.len() < vec_before.capacity() }
    /// vec.shift_insert(index, value)
    /// {
    ///     vec_after.len() = vec_before.len() + 1
    ///     ∧ vec_after[index] = value
    ///     ∧ (∀ i ∈ [0, index). vec_after[i] = vec_before[i])
    ///     ∧ (∀ i ∈ [index, vec_before.len()). vec_after[i + 1] = vec_before[i])
    /// }
    ///
    /// { index = vec_before.len() ∧ vec_before.len() < vec_before.capacity() }
    /// vec.shift_insert(index, value)
    /// {
    ///     vec_after.len() = vec_before.len() + 1
    ///     ∧ vec_after[index] = value
    ///     ∧ (∀ i ∈ [0, vec_before.len()). vec_after[i] = vec_before[i])
    /// }
    ///
    /// { index ≤ vec_before.len() ∧ vec_before.len() = vec_before.capacity() }
    /// vec.shift_index(index, value)
    /// { ⊥ }
    ///
    /// { index > vec_before.len() }
    /// vec.shift_insert(index, value)
    /// { ⊥ }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Complexity Characteristics
    ///
    /// This method runs in **O(n)** time, where `n` is an affine function of the length of the
    /// vector. Every value after the insertion index must be shifted up. The worst case
    /// input is when the input index is `index == 0`. In the worst case, every value in the vector
    /// must be shifted up.
    ///
    /// # Panics
    ///
    /// This method panics if the index `index` is larger than the length of the collection, or
    /// if the collection is full.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::with_capacity(10);
    ///
    /// assert!(vec.is_empty());
    ///
    /// vec.shift_insert(0, 1);
    ///
    /// assert_eq!(vec.len(), 1);
    /// assert_eq!(vec.as_slice(), &[1]);
    ///
    /// vec.shift_insert(0, 2);
    ///
    /// assert_eq!(vec.len(), 2);
    /// assert_eq!(vec.as_slice(), &[2, 1]);
    /// ```
    #[inline]
    #[track_caller]
    pub fn shift_insert(&mut self, index: usize, value: T) {
        #[cold]
        #[track_caller]
        #[cfg_attr(feature = "nightly", optimize(size))]
        fn index_out_of_bounds_failure(index: usize, length: usize) -> ! {
            panic!(
                "shift_insert index out of bounds: Got index `{index}`. Need index `{index}` <= len, where len is `{length}`."
            );
        }

        #[cold]
        #[track_caller]
        #[cfg_attr(feature = "nightly", optimize(size))]
        fn capacity_failure(length: usize) -> ! {
            panic!(
                "shift_insert vector full: The vector has length `{length}` equal to its capacity, so there is no more room to insert elements."
            );
        }

        let length = self.len();
        if index > length {
            index_out_of_bounds_failure(index, length);
        }

        if length == self.capacity() {
            capacity_failure(length);
        }

        unsafe {
            {
                let slot_ptr = self.as_mut_ptr().add(index);
                if index < length {
                    core::ptr::copy(slot_ptr, slot_ptr.add(1), length - index);
                }

                core::ptr::write(slot_ptr, value);
            }

            self.set_len(length + 1);
        }
    }

    /// Removes a value from a bounded vector, moving the last value in the collection to
    /// the index where the removed value occupies the collection.
    ///
    /// This method behaves with respect to `index` as follows:
    ///
    /// * If `index < self.len() - 1`, it moves the last value in the collection to the slot at
    ///   `index`, leaving the rest of the values in place.
    /// * If `index == self.len() - 1`, it removes the value from end of the collection with no
    ///   reordering of the remaining values in the collection.
    /// * If `index >= self.len()`, it panics.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector, `vec_before` be the state of `vec` before this method is called,
    /// and let `vec_after` be the state of `vec` after this method completes. Let `result` be the
    /// value that this method returns after completing.
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { index < vec_before.len() - 1 }
    /// vec.swap_remove(index)
    /// {
    ///     result = vec_before[vec_before.len() - 1]
    ///     ∧ vec_after.len() = vec_before.len() - 1
    ///     ∧ vec_after[index] = vec_before[vec_before.len() - 1]
    ///     ∧ (∀ i ∈ [0, vec_before.len() - 1). i ≠ index ⇒ vec_after[i] = vec_before[i])
    /// }
    ///
    /// { index = vec_before.len() - 1 }
    /// vec.swap_remove(index)
    /// {
    ///     result = vec_before[vec_before.len() - 1]
    ///     ∧ vec_after.len() = vec_before.len() - 1
    ///     ∧ (∀ i ∈ [0, vec_before.len() - 1). vec_after[i] = vec_before[i])
    /// }
    ///
    /// { index ≥ vec_before.len() }
    /// vec.swap_remove(index, value)
    /// { ⊥ }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Complexity Characteristics
    ///
    /// This method runs in **O(1)** time.
    ///
    /// # Panics
    ///
    /// This method panics if the index `index` is larger than the length of the collection. In
    /// particular, this method panics when the bounded vector is empty.
    ///
    /// # Examples
    ///
    /// Showing how swap removal happens.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let vec = BoundedVec::from([1, 2, 3, i32::MAX]);
    /// {
    ///     let mut cloned = vec.clone();
    ///     cloned.swap_remove(3);
    ///     assert_eq!(cloned.as_slice(), &[1, 2, 3]);
    /// }
    /// {
    ///     let mut cloned = vec.clone();
    ///     cloned.swap_remove(2);
    ///     assert_eq!(cloned.as_slice(), &[1, 2, i32::MAX]);
    /// }
    /// {
    ///     let mut cloned = vec.clone();
    ///     cloned.swap_remove(1);
    ///     assert_eq!(cloned.as_slice(), &[1, i32::MAX, 3]);
    /// }
    /// {
    ///     let mut cloned = vec.clone();
    ///     cloned.swap_remove(0);
    ///     assert_eq!(cloned.as_slice(), &[i32::MAX, 2, 3]);
    /// }
    /// ```
    #[inline]
    #[track_caller]
    pub fn swap_remove(&mut self, index: usize) -> T {
        #[cold]
        #[track_caller]
        #[cfg_attr(feature = "nightly", optimize(size))]
        fn index_out_of_bounds_failure(index: usize, length: usize) -> ! {
            panic!("swap_remove index out of bounds: Got `{index}`, length is `{length}`.");
        }

        let length = self.len();
        if index >= length {
            index_out_of_bounds_failure(index, length);
        }

        let value = unsafe {
            let _value = core::ptr::read(self.as_ptr().add(index));
            let base_ptr = self.as_mut_ptr();
            core::ptr::copy(base_ptr.add(length - 1), base_ptr.add(index), 1);
            self.set_len(length - 1);

            _value
        };

        value
    }

    /// Removes a value from a bounded vector, shifting every successive value in the
    /// collection down one index to fill where the removed value occupies the collection.
    ///
    /// This method behaves with respect to `index` as follows:
    ///
    /// * If `index < self.len()`, it moves the every successive value in the collection to
    ///   the slot at `index` down one unit. Every value preceding the slot at `index` remains
    ///   in the same location.
    /// * If `index >= self.len()`, it panics.
    ///
    /// In particular, the method acts like a [`pop`] when the last value in the collection is
    /// shift-removed, because the sub-collection of successor values is empty.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector, `vec_before` be the state of `vec` before this method is called,
    /// let `vec_after` be the state of `vec` after this method completes, and let `result` be the
    /// value that this method returns after completing.
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { index < vec_before.len() }
    /// vec.shift_remove(index)
    /// {
    ///     result = vec_before[index]
    ///     ∧ vec_after.len() = vec_before.len() - 1
    ///     ∧ (∀ i ∈ [0, index). vec_after[i] = vec_before[i])
    ///     ∧ (∀ i ∈ [index, vec_after.len()). vec_after[i] = vec_before[i + 1])
    /// }
    ///
    /// { index ≥ vec_before.len() }
    /// vec.shift_remove(index)
    /// { ⊥ }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Complexity Characteristics
    ///
    /// This method runs in average **O(n)** time, where `n` is an affine function of the length of
    /// the vector. The worst case input is when `index == 0`. In the worst case, every remaining
    /// element of the vector is shifted down one index.
    ///
    /// # Panics
    ///
    /// This method panics if the index `index` is larger than the length of the collection. In
    /// particular, the method panics when `self` is empty.
    ///
    /// # Examples
    ///
    /// Showing how shift removal happens.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let vec = BoundedVec::from([1, 2, 3, i32::MAX]);
    /// {
    ///     let mut cloned = vec.clone();
    ///     cloned.swap_remove(3);
    ///     assert_eq!(cloned.as_slice(), &[1, 2, 3]);
    /// }
    /// {
    ///     let mut cloned = vec.clone();
    ///     cloned.swap_remove(2);
    ///     assert_eq!(cloned.as_slice(), &[1, 2, i32::MAX]);
    /// }
    /// {
    ///     let mut cloned = vec.clone();
    ///     cloned.swap_remove(1);
    ///     assert_eq!(cloned.as_slice(), &[1, i32::MAX, 3]);
    /// }
    /// {
    ///     let mut cloned = vec.clone();
    ///     cloned.swap_remove(0);
    ///     assert_eq!(cloned.as_slice(), &[i32::MAX, 2, 3]);
    /// }
    /// ```
    ///
    /// [`pop`]: BoundedVec::pop
    #[inline]
    pub fn shift_remove(&mut self, index: usize) -> T {
        #[cold]
        #[track_caller]
        #[cfg_attr(feature = "nightly", optimize(size))]
        fn index_out_of_bounds_failure(index: usize, length: usize) -> ! {
            panic!("shift_remove index out of bounds: Got `{index}`, length is `{length}`.");
        }

        let length = self.len();
        if index >= length {
            index_out_of_bounds_failure(index, length);
        }

        let value = unsafe {
            let _value = {
                let ptr = self.as_mut_ptr().add(index);
                let __value = core::ptr::read(ptr);

                core::ptr::copy(ptr.add(1), ptr, length - index - 1);

                __value
            };

            self.set_len(length - 1);

            _value
        };

        value
    }
}

impl<T, A> BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    /// Forces the length of and bounded vector to be set to `new_len`.
    ///
    /// This is a low-level operation that does not maintain the invariants of the bounded
    /// vector. Normally one changes the length of the collection using operations such as
    /// [`clear`].
    ///
    /// Note that reducing the length of a bounded vector using this method will not drop
    /// the truncated elements. If those elements own heap-allocated memory or other resources,
    /// this will result in a memory leak.
    ///
    /// # Safety
    ///
    /// This method is safe to call if the following conditions hold:
    ///
    /// * The length `new_len` is less than or equal to `self.capacity()`.
    /// * The elements in the subslice `[self.len(), new_len()` must be initialized.
    ///
    /// # Examples
    ///
    /// Safely reducing the length of a bounded vector with this method.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// # use core::ptr;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// struct DropCounter {}
    ///
    /// static mut DROP_COUNT: u32 = 0;
    ///
    /// impl Drop for DropCounter {
    ///     fn drop(&mut self) {
    ///         unsafe { DROP_COUNT += 1; }
    ///     }
    /// }
    ///
    /// let capacity = 4;
    /// let mut vec = BoundedVec::with_capacity(capacity);
    ///
    /// vec.push(Box::new(DropCounter {}));
    /// vec.push(Box::new(DropCounter {}));
    /// vec.push(Box::new(DropCounter {}));
    ///
    /// assert_eq!(vec.len(), 3);
    /// assert_eq!(vec.capacity(), capacity);
    /// unsafe {
    ///     let ptr = vec.as_mut_ptr();
    ///     // Read, then drop the last two elements.
    ///     let _: Box<DropCounter> = ptr::read(ptr.add(2));
    ///     let _: Box<DropCounter> = ptr::read(ptr.add(1));
    ///     vec.set_len(1);
    /// }
    ///
    /// assert_eq!(vec.len(), 1);
    /// assert_eq!(vec.capacity(), capacity);
    ///
    /// // No data leaks because we dropped then shrank the length.
    /// assert_eq!(unsafe { DROP_COUNT }, 2);
    /// ```
    ///
    /// Safely extending the length of a bounded vector with this method without leaking
    /// memory.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// # use core::ptr;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// struct DropCounter {}
    ///
    /// static mut DROP_COUNT: u32 = 0;
    ///
    /// impl Drop for DropCounter {
    ///     fn drop(&mut self) {
    ///         unsafe { DROP_COUNT += 1; }
    ///     }
    /// }
    ///
    /// let capacity = 4;
    /// let mut vec = BoundedVec::with_capacity(capacity);
    ///
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), capacity);
    /// unsafe {
    ///     let ptr: *mut Box<DropCounter> = vec.as_mut_ptr();
    ///     // Write the elements into the allocation directly.
    ///     ptr::write(ptr.add(0), Box::new(DropCounter {}));
    ///     ptr::write(ptr.add(1), Box::new(DropCounter {}));
    ///     ptr::write(ptr.add(2), Box::new(DropCounter {}));
    ///     vec.set_len(3);
    /// }
    ///
    /// assert_eq!(vec.len(), 3);
    /// assert_eq!(vec.capacity(), capacity);
    ///
    /// // No data leaks after writing directly into the allocation.
    /// assert_eq!(unsafe { DROP_COUNT }, 0);
    /// ```
    ///
    /// Safely extending the length of a bounded vector with this method.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// # use core::ptr;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let capacity = 4;
    /// let mut vec = BoundedVec::with_capacity(capacity);
    ///
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.capacity(), capacity);
    /// unsafe {
    ///     let ptr: *mut i32 = vec.as_mut_ptr();
    ///     // Write the elements into the allocation directly.
    ///     ptr::write(ptr.add(0), 1);
    ///     ptr::write(ptr.add(1), 2);
    ///     ptr::write(ptr.add(2), 3);
    ///     vec.set_len(3);
    /// }
    ///
    /// assert_eq!(vec.len(), 3);
    /// assert_eq!(vec.capacity(), capacity);
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 3]);
    /// ```
    ///
    /// [`clear`]: BoundedVec::clear
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());

        self.length = new_len;
    }

    /// Returns the remaining spare capacity of the bounded vector as a slice of
    /// [`MaybeUninit<T>`].
    ///
    /// The returned slice can be used to fill the bounded vector with data before marking
    /// the data as initialized using the [`set_len`] method.
    ///
    /// [`set_len`]: BoundedVec::set_len
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::with_capacity(10);
    ///
    /// // Fill in the first 3 elements.
    /// let uninit = vec.spare_capacity_mut();
    /// uninit[0].write(1);
    /// uninit[1].write(2);
    /// uninit[2].write(3);
    ///
    /// // Mark the first 3 elements of the vector as being initialized.
    /// unsafe {
    ///     vec.set_len(3);
    /// }
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 3]);
    /// ```
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe {
            let data = self.as_mut_ptr().add(self.len()) as *mut MaybeUninit<T>;
            let len = self.capacity() - self.len();

            slice::from_raw_parts_mut(data, len)
        }
    }
}

impl<T, A> BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[inline]
    fn extend_with(&mut self, count: usize, value: T)
    where
        T: Clone,
    {
        struct SetLenOnDrop<'a> {
            len: &'a mut usize,
            local_len: usize,
        }

        impl<'a> SetLenOnDrop<'a> {
            #[inline]
            fn new(len: &'a mut usize) -> Self {
                SetLenOnDrop { local_len: *len, len }
            }

            #[inline]
            fn increment(&mut self, increment: usize) {
                self.local_len += increment;
            }

            #[inline]
            fn current(&self) -> usize {
                self.local_len
            }
        }

        impl Drop for SetLenOnDrop<'_> {
            #[inline]
            fn drop(&mut self) {
                *self.len = self.local_len;
            }
        }

        let length = self.len();
        let capacity = self.capacity();

        if length + count > capacity {
            panic!("The supplied value count overflows the vector: length=`{length}` capacity=`{capacity}`, count=`{count}`");
        }

        unsafe {
            let mut ptr = self.as_mut_ptr().add(self.len());
            // Use SetLenOnDrop to work around bug where compiler
            // might not realize the store through `ptr` through self.set_len()
            // don't alias.
            let mut local_length = SetLenOnDrop::new(&mut self.length);

            // Write all elements except the last one
            for _ in 1..count {
                core::ptr::write(ptr, value.clone());
                ptr = ptr.add(1);
                // Increment the length in every step in case clone() panics
                local_length.increment(1);
            }

            if count > 0 {
                // We can write the last element directly without cloning needlessly
                core::ptr::write(ptr, value);
                local_length.increment(1);
            }

            debug_assert_eq!(local_length.current(), length + count);
            // len set by scope guard
        }
    }

    #[inline]
    fn extend_from_iter<I>(&mut self, iterator: I)
    where
        I: Iterator<Item = T>,
    {
        for item in iterator {
            self.push(item);
        }
    }
}

impl<T, A> BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    /// Creates an iterator which uses a closure to determine if an element in the range should be
    /// removed.
    ///
    /// If the closure returns `true`, the element is removed from the vector
    /// and yielded. If the closure returns `false`, or panics, the element
    /// remains in the vector and will not be yielded.
    ///
    /// Only elements that fall in the provided range are considered for extraction, but any
    /// elements after the range will still have to be moved if any element has been extracted.
    ///
    /// If the returned [`ExtractIf`] is not exhausted, e.g. because it is dropped without
    /// iterating or the iteration short-circuits, then the remaining elements will be retained.
    /// Use [`retain_mut`] with a negated predicate if you do not need the returned iterator.
    ///
    /// [`retain_mut`]: BoundedVec::retain_mut
    ///
    /// Using this method is equivalent to the following code:
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// # let some_predicate = |x: &mut i32| { *x % 2 == 1 };
    /// # let mut vec = BoundedVec::from([0, 1, 2, 3, 4, 5, 6]);
    /// # let mut vec2 = vec.clone();
    ///
    /// # let range = 1..5;
    /// let mut i = range.start;
    /// let end_items = vec.len() - range.end;
    /// # let mut extracted = BoundedVec::with_capacity(10);
    ///
    /// while i < vec.len() - end_items {
    ///     if some_predicate(vec.get_mut(i).unwrap()) {
    ///         let val = vec.shift_remove(i);
    /// #         extracted.push(val);
    ///         // your code here
    ///     } else {
    ///         i += 1;
    ///     }
    /// }
    ///
    /// # let mut extracted2 = BoundedVec::with_capacity(range.len());
    /// # extracted2.extend(vec2.extract_if(range, some_predicate));
    /// #
    /// # assert_eq!(vec.as_slice(), vec2.as_slice());
    /// # assert_eq!(extracted.as_slice(), extracted2.as_slice());
    /// ```
    ///
    /// But `extract_if` is easier to use. `extract_if` is also more efficient,
    /// because it can backshift the elements of the array in bulk.
    ///
    /// The iterator also lets you mutate the value of each element in the
    /// closure, regardless of whether you choose to keep or remove it.
    ///
    /// # Panics
    ///
    /// This method panics if `range` is out of bounds.
    ///
    /// # Examples
    ///
    /// Splitting a vector into even and odd values, reusing the original vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::{IntoIter, BoundedVec};
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut numbers = BoundedVec::from([1, 2, 3, 4, 5, 6, 8, 9, 11, 13, 14, 15]);
    /// let mut evens = BoundedVec::with_capacity(numbers.len());
    /// evens.extend(numbers.extract_if(.., |x| *x % 2 == 0));
    /// let odds = numbers;
    ///
    /// assert_eq!(evens.as_slice(), &[2, 4, 6, 8, 14]);
    /// assert_eq!(odds.as_slice(), &[1, 3, 5, 9, 11, 13, 15]);
    /// ```
    ///
    /// Using the range argument to only process a part of the vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::{IntoIter, BoundedVec};
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut items = BoundedVec::from([0, 0, 0, 0, 0, 0, 0, 1, 2, 1, 2, 1, 2]);
    /// let mut ones = BoundedVec::with_capacity(items.len());
    /// ones.extend(items.extract_if(7.., |x| *x == 1));
    ///
    /// assert_eq!(items.as_slice(), &[0, 0, 0, 0, 0, 0, 0, 2, 2, 2]);
    /// assert_eq!(ones.len(), 3);
    /// ```
    pub fn extract_if<F, R>(&mut self, range: R, filter: F) -> ExtractIf<'_, T, F, A>
    where
        F: FnMut(&mut T) -> bool,
        R: ops::RangeBounds<usize>,
    {
        ExtractIf::new(self, filter, range)
    }

    /// Appends all elements from a slice to the bounded vector.
    ///
    /// # Panics
    ///
    /// This method panics if `self.len() + slice.len() > self.capacity()`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let array: [i32; 6] = [1, 2, 3, 4, 5, 6];
    /// let extension: [i32; 4] = [7, 8, 9, 10];
    /// let combined: [i32; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    /// let expected = BoundedVec::from(combined);
    ///
    /// let mut result = BoundedVec::with_capacity(16);
    /// for item in array.iter().copied() {
    ///     result.push(item);
    /// }
    /// #
    /// # assert_eq!(result.len(), array.len());
    /// # assert_eq!(result.capacity(), 16);
    /// # assert_eq!(result.as_slice(), array.as_slice());
    /// #
    ///
    /// result.extend_from_slice(&extension);
    ///
    /// assert_eq!(result.len(), array.len() + extension.len());
    /// assert_eq!(result.as_slice(), expected.as_slice());
    /// ```
    #[track_caller]
    pub fn extend_from_slice(&mut self, other: &[T])
    where
        T: Clone,
    {
        self.extend_from_iter::<_>(other.iter().cloned());
    }

    /// Resizes the bounded vector in place so that `len` is equal to `new_len`.
    ///
    /// This method behaves as follows:
    ///
    /// * If `new_len > len`, the vector is extended by the difference, with each additional slot
    ///   filled with `value`.
    /// * If `new_len < len`, the vector is truncated. Each entry in `[new_len, len()` is dropped by
    ///   this method.
    ///
    /// If you need more flexibility (or want to rely on [`Default`] instead of
    /// [`Clone`]), use [`BoundedVec::resize_with`].
    /// If you only need to resize to a smaller size, use [`BoundedVec::truncate`].
    ///
    /// # Panics
    ///
    /// This method panics if the new length exceeds the capacity of the bounded vector.
    ///
    /// # Examples
    ///
    /// Extending a bounded vector with a default value.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::with_capacity(14);
    /// vec.extend([
    ///     "spam",
    ///     "eggs",
    ///     "sausage",
    ///     "spam",
    ///     "baked beans",
    ///     "spam",
    ///     "Lobster Thermidor aux Crevettes with a Mornay sauce, garnished with truffle pâté, brandy, with a fried egg on top, and spam",
    ///     "bacon",
    /// ]);
    /// vec.resize(14, "spam");
    ///
    /// assert_eq!(vec.len(), 14);
    ///
    /// let expected = BoundedVec::from([
    ///     "spam",
    ///     "eggs",
    ///     "sausage",
    ///     "spam",
    ///     "baked beans",
    ///     "spam",
    ///     "Lobster Thermidor aux Crevettes with a Mornay sauce, garnished with truffle pâté, brandy, with a fried egg on top, and spam",
    ///     "bacon",
    ///     "spam",
    ///     "spam",
    ///     "spam",
    ///     "spam",
    ///     "spam",
    ///     "spam",
    /// ]);
    ///
    /// assert_eq!(vec.as_slice(), expected.as_slice());
    /// ```
    ///
    /// Shrinking a bounded vector with a default value.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([
    ///     "spam",
    ///     "eggs",
    ///     "sausage",
    ///     "spam",
    ///     "baked beans",
    ///     "spam",
    ///     "Lobster Thermidor aux Crevettes with a Mornay sauce, garnished with truffle pâté, brandy, with a fried egg on top, and spam",
    ///     "bacon",
    ///     "spam",
    ///     "spam",
    ///     "spam",
    ///     "spam",
    ///     "spam",
    ///     "spam",
    /// ]);
    /// let expected = BoundedVec::from([
    ///     "spam",
    ///     "eggs",
    ///     "sausage",
    ///     "spam",
    ///     "baked beans",
    ///     "spam",
    ///     "Lobster Thermidor aux Crevettes with a Mornay sauce, garnished with truffle pâté, brandy, with a fried egg on top, and spam",
    ///     "bacon",
    /// ]);
    ///
    /// vec.resize(8, "I DON'T WANT SPAM!");
    ///
    /// assert_eq!(vec.len(), 8);
    /// assert_eq!(vec.as_slice(), expected.as_slice());
    /// ```
    #[track_caller]
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        let len = self.len();

        if new_len > len {
            self.extend_with(new_len - len, value)
        } else {
            self.truncate(new_len);
        }
    }

    /// Resizes the bounded vector in place so that is length equals `new_len`.
    ///
    /// If the length `new_len` is greater than the length `len`, the bounded vector is
    /// extended by the difference, with each additional slot filled with the result of calling
    /// the closure `f`. The return values from `f` will end up in the `Vec` in the order
    /// they have been generated.
    ///
    /// If `new_len` is less than `len`, the bounded vector is truncated, so the result is
    /// similar to calling [`truncate`].
    ///
    /// This method uses a closure to create new values on every push. To clone a given value,
    /// use [`resize`]. To use a data type's default value to generate values, use the
    /// [`Default::default`] method.
    ///
    /// # Panics
    ///
    /// This method panics if the new length exceeds the capacity of the bounded vector.
    ///
    /// # Examples
    ///
    /// Resizing to the same size does not change the collection.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let length = 3;
    /// let mut vec = {
    ///     let mut _vec = BoundedVec::with_capacity(10);
    ///     for i in 1..(length + 1) {
    ///         _vec.push(i);
    ///     }
    ///     _vec.push(0);
    ///     _vec.push(0);
    ///     _vec
    /// };
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 3, 0, 0]);
    ///
    /// vec.resize_with(5, Default::default);
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 3, 0, 0]);
    /// ```
    ///
    /// Resizing a collection to a larger collection.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::with_capacity(10);
    ///
    /// let mut p = 1;
    /// vec.resize_with(4, || { p *= 2; p });
    ///
    /// assert_eq!(vec.as_slice(), &[2, 4, 8, 16]);
    /// ```
    ///
    /// [`truncate`]: BoundedVec::truncate
    /// [`resize`]: BoundedVec::resize
    #[track_caller]
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> T,
    {
        let len = self.len();
        if new_len > len {
            self.extend::<_>(core::iter::repeat_with(f).take(new_len - len));
        } else {
            self.truncate(new_len);
        }
    }

    /// Shortens a bounded vector to the supplied length, dropping the remaining elements.
    ///
    /// This method keeps the first `len` elements, and drops the rest of the elements, so that
    /// the length after calling this method is at most `len`. This method does nothing if
    /// `self.len() <= len`.
    ///
    /// # Examples
    ///
    /// Truncating a bounded vector when `len < self.len()`.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([1, 2, 3, 4, 5, 6]);
    /// vec.truncate(2);
    ///
    /// assert_eq!(vec.len(), 2);
    /// assert_eq!(vec.as_slice(), &[1, 2]);
    /// ```
    ///
    /// No truncation occurs when `len == self.len()`.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let array: [i32; 6] = [1, 2, 3, 4, 5, 6];
    /// let mut vec = BoundedVec::from(array);
    /// vec.truncate(6);
    ///
    /// assert_eq!(vec.len(), 6);
    /// assert_eq!(vec.as_slice(), &array);
    /// ```
    ///
    /// No truncation occurs when `len > self.len()`.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let array: [i32; 6] = [1, 2, 3, 4, 5, 6];
    /// let mut vec = BoundedVec::from(array);
    /// vec.truncate(7);
    ///
    /// assert_eq!(vec.len(), 6);
    /// assert_eq!(vec.as_slice(), &array);
    ///
    /// vec.truncate(10000);
    ///
    /// assert_eq!(vec.len(), 6);
    /// assert_eq!(vec.as_slice(), &array);
    /// ```
    ///
    /// Truncating when `len == 0` is equivalent to calling the [`clear`] method.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([1, 2, 3, 4, 5, 6]);
    /// vec.truncate(0);
    ///
    /// assert_eq!(vec.len(), 0);
    /// assert_eq!(vec.as_slice(), &[]);
    /// ```
    ///
    /// [`clear`]: BoundedVec::clear
    /// [`drain`]: BoundedVec::drain
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len > self.len() {
            return;
        }

        let remaining_len = self.len() - len;
        unsafe {
            let slice = core::ptr::slice_from_raw_parts_mut(self.as_mut_ptr().add(len), remaining_len);
            self.set_len(len);
            core::ptr::drop_in_place(slice);
        }
    }
}

impl<T, A> BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    /// Retains only the elements in the bounded vector that satisfy the supplied predicate.
    ///
    /// This method removes all elements from the collection for which the predicate returns
    /// `false`. In particular, for each element `e` in the collection, it removes `e` provided
    /// that `keep(&e) == false`. This method operates in place, and preserves the order of the
    /// retained elements.
    ///
    /// In other words, after calling this method, the vector contains only elements for which
    /// `keep(e)` is true, in the same order as they appeared originally.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector, `vec_before` be the state of `vec` before this method is called,
    /// `vec_after` be the state of `vec` after this method completes, and `keep: T → bool` be the
    /// predicate function passed to this method.
    ///
    /// We say that the vector `vec` **contains** a value `e :: T`, or that `e` is an **element**
    /// of `vec` if and only if
    ///
    /// ```text
    /// ∀ e :: T. (e ∈ vec) ⇔ (∃ i ∈ [0, vec.len()). vec[i] = e).
    /// ```
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { true }
    /// vec.retain(keep)
    /// {
    ///     ∀ e ∈ vec_after. keep(e)
    ///     ∧ (∀ i ∈ [0, vec_after.len()). ∃ k ∈ [0, vec_before.len()).
    ///         (vec_after[i] = vec_before[k])
    ///         ∧ keep(vec_before[k])
    ///         ∧ (∀ j < k. vec_before[j] = vec_after[i] ⇒ ¬keep(vec_before[j])
    ///     )
    ///     ∧ (∀ i < j < vec_after.len(). ∃ k < l < vec_before.len().
    ///         (vec_after[i] = vec_before[k])
    ///         ∧ (vec_after[j] = vec_before[l])
    ///         ∧ keep(vec_before[k])
    ///         ∧ keep(vec_before[l])
    ///     )
    /// }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([1, 2, 3, 4, 5, 6]);
    /// vec.retain(|&x| x % 2 == 0);
    ///
    /// assert_eq!(vec.as_slice(), &[2, 4, 6]);
    /// ```
    pub fn retain<F>(&mut self, mut keep: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.retain_mut::<_>(|elem| keep(elem));
    }

    /// Retains only the elements in the bounded vector that satisfy the supplied predicate
    /// passing a mutable reference to it.
    ///
    /// This method removes all elements from the collection for which the predicate returns
    /// `false`. In particular, for each element `e` in the collection, it removes `e` provided
    /// that `keep(&e) == false`. This method operates in place, and preserves the order of the
    /// retained elements.
    ///
    /// In other words, after calling this method, the vector contains only elements for which
    /// `keep(e)` is true, in the same order as they appeared originally.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector, `vec_before` be the state of `vec` before this method is called,
    /// `vec_after` be the state of `vec` after this method completes, and `keep: T → bool` be the
    /// predicate function passed to this method.
    ///
    /// We say that the vector `vec` **contains** a value `e :: T`, or that `e` is an **element**
    /// of `vec` if and only if
    ///
    /// ```text
    /// ∀ e :: T. (e ∈ vec) ⇔ (∃ i ∈ [0, vec.len()). vec[i] = e).
    /// ```
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { true }
    /// vec.retain(keep)
    /// {
    ///     ∀ e ∈ vec_after. keep(e)
    ///     ∧ (∀ i ∈ [0, vec_after.len()). ∃ k ∈ [0, vec_before.len()).
    ///         (vec_after[i] = vec_before[k])
    ///         ∧ keep(vec_before[k])
    ///         ∧ (∀ j < k. vec_before[j] = vec_after[i] ⇒ ¬keep(vec_before[j])
    ///     )
    ///     ∧ (∀ i < j < vec_after.len(). ∃ k < l < vec_before.len().
    ///         (vec_after[i] = vec_before[k])
    ///         ∧ (vec_after[j] = vec_before[l])
    ///         ∧ keep(vec_before[k])
    ///         ∧ keep(vec_before[l])
    ///     )
    /// }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([1, 2, 3, 4, 5, 6]);
    /// vec.retain_mut(|x| if *x <= 3 {
    ///     *x += 1;
    ///     true
    /// } else {
    ///     false
    /// });
    ///
    /// assert_eq!(vec.as_slice(), &[2, 3, 4]);
    /// ```
    pub fn retain_mut<F>(&mut self, mut keep: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let original_len = self.len();

        if original_len == 0 {
            // Empty case: explicit return allows better optimization, vs letting compiler infer it
            return;
        }

        // Avoid double drop if the drop guard is not executed,
        // since we may make some holes during the process.
        unsafe { self.set_len(0) };

        // Vec: [Kept, Kept, Hole, Hole, Hole, Hole, Unchecked, Unchecked]
        //      |<-              processed len   ->| ^- next to check
        //                  |<-  deleted cnt     ->|
        //      |<-              original_len                          ->|
        // Kept: Elements which predicate returns true on.
        // Hole: Moved or dropped element slot.
        // Unchecked: Unchecked valid elements.
        //
        // This drop guard will be invoked when predicate or `drop` of element panicked.
        // It shifts unchecked elements to cover holes and `set_len` to the correct length.
        // In cases when predicate and `drop` never panic, it will be optimized out.
        struct BackshiftOnDrop<'a, T, A>
        where
            A: alloc::Allocator,
        {
            v: &'a mut BoundedVec<T, A>,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<T, A> Drop for BackshiftOnDrop<'_, T, A>
        where
            A: alloc::Allocator,
        {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    // SAFETY: Trailing unchecked items must be valid since we never touch them.
                    unsafe {
                        core::ptr::copy(
                            self.v.as_ptr().add(self.processed_len),
                            self.v.as_mut_ptr().add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                    }
                }
                // SAFETY: After filling holes, all items are in contiguous memory.
                unsafe {
                    self.v.set_len(self.original_len - self.deleted_cnt);
                }
            }
        }

        let mut g = BackshiftOnDrop {
            v: self,
            processed_len: 0,
            deleted_cnt: 0,
            original_len,
        };

        fn process_loop<F, T, A, const DELETED: bool>(original_len: usize, f: &mut F, g: &mut BackshiftOnDrop<'_, T, A>)
        where
            A: alloc::Allocator,
            F: FnMut(&mut T) -> bool,
        {
            while g.processed_len != original_len {
                // SAFETY: Unchecked element must be valid.
                let cur = unsafe { &mut *g.v.as_mut_ptr().add(g.processed_len) };
                if !f(cur) {
                    // Advance early to avoid double drop if `drop_in_place` panicked.
                    g.processed_len += 1;
                    g.deleted_cnt += 1;
                    // SAFETY: We never touch this element again after dropped.
                    unsafe { core::ptr::drop_in_place(cur) };
                    // We already advanced the counter.
                    if DELETED {
                        continue;
                    } else {
                        break;
                    }
                }
                if DELETED {
                    // SAFETY: `deleted_cnt` > 0, so the hole slot must not overlap with current element.
                    // We use copy for move, and never touch this element again.
                    unsafe {
                        let hole_slot = g.v.as_mut_ptr().add(g.processed_len - g.deleted_cnt);
                        core::ptr::copy_nonoverlapping(cur, hole_slot, 1);
                    }
                }
                g.processed_len += 1;
            }
        }

        // Stage 1: Nothing was deleted.
        process_loop::<F, T, A, false>(original_len, &mut keep, &mut g);

        // Stage 2: Some elements were deleted.
        process_loop::<F, T, A, true>(original_len, &mut keep, &mut g);

        // All item are processed. This can be optimized to `set_len` by LLVM.
        drop(g);
    }

    /// Removes consecutive repeated elements in the bounded vector according to the
    /// [`PartialEq`] trait implementation.
    ///
    /// This method removes all duplicates if the collection is sorted.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector, `vec_before` be the state of `vec` before this method is called,
    /// and `vec_after` be the state of `vec` after this method completes. A function `g: A → B` is
    /// called **strictly increasing** if and only if
    ///
    /// ```text
    /// strictly_increasing(g) := ∀ i ∈ A. ∀ j ∈ A. i < j ⇒ g(i) < g(j).
    /// ```
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { true }
    /// vec.dedup()
    /// {
    ///     vec_after.len() ≤ vec_before.len()
    ///     ∧ (∃ g: [0, vec_after.len()) → [0, vec_before.len()).
    ///         strictly_increasing(g) ∧ ∀ i ∈ [0, vec_after.len()). vec_after[i] = vec_before[g(i)]
    ///       )
    ///     ∧ (∀ i ∈ [0, vec_after.len() - 1). vec_after[i] ≠ vec_after[i + 1])
    ///     ∧ (∀ i ∈ [0, vec_after.len()). ∃ j ∈ [0, vec_before.len()). vec_after[i] = vec_before[j]
    ///         ∧ (∀ k < j. vec_before[k] = vec_after[i] ⇒ (∃ m < j. (vec_before[m] = vec_after[i]) ∧ (m < k)))
    ///         ∨ (∀ k < j. vec_before[k] ≠ vec_after[i])
    ///       )
    ///     )
    /// }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Examples
    ///
    /// Deduplicating an unsorted bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([1, 2, 3, 2, 2, 2, 6, 4, 4]);
    /// vec.dedup();
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 3, 2, 6, 4]);
    /// ```
    ///
    /// Deduplicating a sorted bounded vector with duplicate values.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([1, 2, 3, 3, 3, 3, 4, 4, 4, 5]);
    /// vec.dedup();
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 3, 4, 5]);
    /// ```
    ///
    /// Deduplicating a sorted bounded vector with no duplicate values does nothing.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from( [1, 2, 3, 4, 5]);
    /// vec.dedup();
    ///
    /// assert_eq!(vec.as_slice(), &[1, 2, 3, 4, 5]);
    /// ```
    #[inline]
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        self.dedup_by(|a, b| a == b)
    }

    /// Removes all but the first of consecutive elements in the bounded vector that resolve
    /// to the same key.
    ///
    /// This removes all duplicates if the collection is sorted (since each duplicate value
    /// trivially resolves to the same key).
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector, `vec_before` be the state of `vec` before this method is called,
    /// `vec_after` be the state of `vec` after this method completes, and `key: T → K` be the
    /// key function. A function `g: A → B` is called **strictly increasing** if and only if
    ///
    /// ```text
    /// strictly_increasing(g) := ∀ i ∈ A. ∀ j ∈ A. i < j ⇒ g(i) < g(j).
    /// ```
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { true }
    /// vec.dedup_by_key(key)
    /// {
    ///     vec_after.len() ≤ vec_before.len()
    ///     ∧ (∃ g: [0, vec_after.len()) → [0, vec_before.len()).
    ///         strictly_increasing(g) ∧ (∀ i ∈ [0, vec_after.len()). vec_after[i] = vec_before[g(i)])
    ///     )
    ///     ∧ (∀ i ∈ [0, vec_after.len() - 1). key(vec_after[i]) ≠ key(vec_after[i + 1]))
    /// }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Examples
    ///
    /// Deduplicating an unsorted bounded vector by key.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([10, 20, 21, 30, 20]);
    /// vec.dedup_by_key(|i| *i / 10);
    ///
    /// assert_eq!(vec.as_slice(), &[10, 20, 30, 20]);
    /// ```
    ///
    /// Deduplicating a sorted bounded vector by key with duplicate values.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([10, 20, 20, 21, 30, 30, 30, 40]);
    /// vec.dedup_by_key(|i| *i / 10);
    ///
    /// assert_eq!(vec.as_slice(), &[10, 20, 30, 40]);
    /// ```
    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.dedup_by::<_>(|a, b| key(a) == key(b))
    }

    /// Removes all but the first of consecutive elements in the vector satisfying a given equality
    /// relation.
    ///
    /// The `same_bucket` function is passed references to two elements from the collection and
    /// must determine if the elements compare equal. The elements are passed in opposite order
    /// from their order in the slice, so if `same_bucket(a, b)` returns `true`, `a` is removed.
    ///
    /// This method removes all duplicates if the collection is sorted.
    ///
    /// # Formal Properties (Optional Section)
    ///
    /// ***Note: This section is optional for most users and contains advanced material.
    /// It explains the precise axiomatic (formal, logic-based) semantics of these operations for
    /// those seeking a thorough understanding.***
    ///
    /// Let `vec` be a vector, `vec_before` be the state of `vec` before this method is called,
    /// `vec_after` be the state of `vec` after this method completes, and
    /// `same_bucket: (T, T) → bool` be the binary predicate. A function `g: A → B` is called
    /// **strictly increasing** if and only if
    ///
    /// ```text
    /// strictly_increasing(g) := ∀ i ∈ A. ∀ j ∈ A. i < j ⇒ g(i) < g(j).
    /// ```
    ///
    /// This method satisfies:
    ///
    /// ```text
    /// { true }
    /// vec.dedup_by(same_bucket)
    /// {
    ///     vec_after.len() ≤ vec_before.len()
    ///     ∧ (∃ g: [0, vec_after.len()) → [0, vec_before.len()).
    ///         strictly_increasing(g) ∧ (∀ i ∈ [0, vec_after.len()). vec_after[i] = vec_before[g(i)])
    ///     )
    ///     ∧ (∀ i ∈ [0, vec_after.len() - 1). ¬same_bucket(vec_after[i], vec_after[i + 1]))
    /// }
    /// ```
    ///
    /// where `{P} S {Q}` is the Hoare triple indicating how this method acts on `vec`.
    ///
    /// # Examples
    ///
    /// Deduplicating an unsorted bounded vector.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([
    ///     "foo",
    ///     "bar", "Bar",
    ///     "baz",
    ///     "bar",
    ///     "quux", "Quux", "QuuX"
    /// ]);
    /// vec.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    ///
    /// assert_eq!(vec.as_slice(), &["foo", "bar", "baz", "bar", "quux"]);
    /// ```
    ///
    /// Deduplicating a sorted bounded vector with duplicate values.
    ///
    /// ```
    /// # #![cfg_attr(feature = "nightly", feature(allocator_api))]
    /// # use bounded_vec::BoundedVec;
    /// #
    /// # #[cfg(feature = "nightly")]
    /// # use std::alloc::Global;
    /// #
    /// # #[cfg(not(feature = "nightly"))]
    /// # use allocator_api2::alloc::Global;
    /// #
    /// let mut vec = BoundedVec::from([
    ///     "foo",
    ///     "bar", "Bar", "bar",
    ///     "baz", "Baz", "BaZ",
    ///     "quux", "Quux", "QuuX",
    ///     "garply"
    /// ]);
    /// vec.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    ///
    /// assert_eq!(vec.as_slice(), &["foo", "bar", "baz", "quux", "garply"]);
    /// ```
    pub fn dedup_by<F>(&mut self, mut same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        let len = self.len();
        if len <= 1 {
            return;
        }

        // Check if we ever want to remove anything.
        // This allows to use copy_non_overlapping in next cycle.
        // And avoids any memory writes if we don't need to remove anything.
        let mut first_duplicate_idx: usize = 1;
        let start = self.as_mut_ptr();
        while first_duplicate_idx != len {
            let found_duplicate = unsafe {
                // SAFETY: first_duplicate always in range [1..len)
                // Note that we start iteration from 1 so we never overflow.
                let prev = start.add(first_duplicate_idx.wrapping_sub(1));
                let current = start.add(first_duplicate_idx);
                // We explicitly say in docs that references are reversed.
                same_bucket(&mut *current, &mut *prev)
            };
            if found_duplicate {
                break;
            }
            first_duplicate_idx += 1;
        }
        // Don't need to remove anything.
        // We cannot get bigger than len.
        if first_duplicate_idx == len {
            return;
        }

        // INVARIANT: vec.len() > read > write > write-1 >= 0
        struct FillGapOnDrop<'a, T, A>
        where
            A: alloc::Allocator,
        {
            // Offset of the element we want to check if it is duplicate
            read: usize,

            // Offset of the place where we want to place the non-duplicate
            // when we find it.
            write: usize,

            // The Vec that would need correction if `same_bucket` panicked
            vec: &'a mut BoundedVec<T, A>,
        }

        impl<'a, T, A> Drop for FillGapOnDrop<'a, T, A>
        where
            A: alloc::Allocator,
        {
            fn drop(&mut self) {
                // This code gets executed when `same_bucket` panics.
                //
                // SAFETY: invariant guarantees that `read - write`
                // and `len - read` never overflow and that the copy is always
                // in-bounds.
                unsafe {
                    let ptr = self.vec.as_mut_ptr();
                    let len = self.vec.len();

                    // How many items were left when `same_bucket` panicked.
                    // Basically vec[read..].len()
                    let items_left = len.wrapping_sub(self.read);

                    // Pointer to first item in vec[write..write+items_left] slice
                    let dropped_ptr = ptr.add(self.write);
                    // Pointer to first item in vec[read..] slice
                    let valid_ptr = ptr.add(self.read);

                    // Copy `vec[read..]` to `vec[write..write+items_left]`.
                    // The slices can overlap, so `copy_nonoverlapping` cannot be used
                    core::ptr::copy(valid_ptr, dropped_ptr, items_left);

                    // How many items have been already dropped
                    // Basically vec[read..write].len()
                    let dropped = self.read.wrapping_sub(self.write);

                    self.vec.set_len(len - dropped);
                }
            }
        }

        // Drop items while going through Vec, it should be more efficient than
        // doing slice partition_dedup + truncate

        // Construct gap first and then drop item to avoid memory corruption if `T::drop` panics.
        let mut gap: FillGapOnDrop<'_, T, A> = FillGapOnDrop {
            read: first_duplicate_idx + 1,
            write: first_duplicate_idx,
            vec: self,
        };

        unsafe {
            // SAFETY: we checked that first_duplicate_idx in bounds before.
            // If drop panics, `gap` would remove this item without drop.
            core::ptr::drop_in_place(start.add(first_duplicate_idx));
        }

        // SAFETY: Because of the invariant, read_ptr, prev_ptr and write_ptr
        // are always in-bounds and read_ptr never aliases prev_ptr
        unsafe {
            while gap.read < len {
                let read_ptr = start.add(gap.read);
                let prev_ptr = start.add(gap.write.wrapping_sub(1));

                // We explicitly say in docs that references are reversed.
                let found_duplicate = same_bucket(&mut *read_ptr, &mut *prev_ptr);
                if found_duplicate {
                    // Increase `gap.read` now since the drop may panic.
                    gap.read += 1;
                    // We have found duplicate, drop it in-place
                    core::ptr::drop_in_place(read_ptr);
                } else {
                    let write_ptr = start.add(gap.write);

                    // read_ptr cannot be equal to write_ptr because at this point
                    // we guaranteed to skip at least one element (before loop starts).
                    core::ptr::copy_nonoverlapping(read_ptr, write_ptr, 1);

                    /* We have filled that place, so go further */
                    gap.write += 1;
                    gap.read += 1;
                }
            }

            // Technically we could let `gap` clean up with its Drop, but
            // when `same_bucket` is guaranteed to not panic, this bloats a little
            // the codegen, so we just do it manually
            gap.vec.set_len(gap.write);
            core::mem::forget(gap);
        }
    }
}

impl<T, A> Extend<T> for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[track_caller]
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.extend_from_iter(iterable.into_iter())
    }
}

impl<'a, T, A> Extend<&'a T> for BoundedVec<T, A>
where
    T: Copy,
    A: alloc::Allocator,
{
    #[track_caller]
    fn extend<I>(&mut self, iterable: I)
    where
        I: IntoIterator<Item = &'a T>,
    {
        self.extend_from_iter(iterable.into_iter().cloned())
    }
}

impl<T, A> BoundedVec<T, A>
where
    T: Clone,
    A: alloc::Allocator + Clone,
{
    #[inline]
    pub(crate) fn from_slice_in(slice: &[T], alloc: A) -> BoundedVec<T, A> {
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
        unsafe {
            vec.set_len(slice.len());
        }

        vec
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
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, A> AsMut<[T]> for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, A> AsRef<BoundedVec<T, A>> for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[inline]
    fn as_ref(&self) -> &BoundedVec<T, A> {
        self
    }
}

impl<T, A> AsMut<BoundedVec<T, A>> for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[inline]
    fn as_mut(&mut self) -> &mut BoundedVec<T, A> {
        self
    }
}

impl<T, A> ops::Deref for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, A> ops::DerefMut for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[inline]
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
            inner: Box::new_uninit_slice_in(0, A::default()),
            length: 0,
        }
    }
}

impl<T, I, A> ops::Index<I> for BoundedVec<T, A>
where
    I: slice::SliceIndex<[T]>,
    A: alloc::Allocator,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        ops::Index::index(&**self, index)
    }
}

impl<T, I, A> ops::IndexMut<I> for BoundedVec<T, A>
where
    I: slice::SliceIndex<[T]>,
    A: alloc::Allocator,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        ops::IndexMut::index_mut(&mut **self, index)
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
        unsafe {
            cloned_vec.set_len(self.len());
        }

        cloned_vec
    }
}

impl<T, A> hash::Hash for BoundedVec<T, A>
where
    T: hash::Hash,
    A: alloc::Allocator,
{
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        hash::Hash::hash(self.as_slice(), state)
    }
}

impl<T, A> Drop for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    fn drop(&mut self) {
        unsafe {
            let initialized_slice = core::ptr::slice_from_raw_parts_mut(self.inner.as_mut_ptr().cast::<T>(), self.length);

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

impl From<&str> for BoundedVec<u8, alloc::Global> {
    #[inline]
    fn from(s: &str) -> BoundedVec<u8, alloc::Global> {
        From::from(s.as_bytes())
    }
}

impl<T, const N: usize> From<[T; N]> for BoundedVec<T, alloc::Global> {
    #[inline]
    fn from(array: [T; N]) -> BoundedVec<T, alloc::Global> {
        let length = array.len();
        let mut me = ManuallyDrop::new(Box::new(array));
        let data = me.as_mut_ptr() as *mut MaybeUninit<T>;
        let inner = unsafe { Box::from_raw(slice::from_raw_parts_mut(data, length)) };

        Self { inner, length }
    }
}

impl<T, A> From<Box<[T], A>> for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[inline]
    fn from(boxed: Box<[T], A>) -> Self {
        let length = boxed.len();
        let (data, alloc) = Box::into_raw_with_allocator(boxed);
        let raw_ptr = data.cast::<MaybeUninit<T>>();
        let inner = unsafe { Box::from_raw_in(slice::from_raw_parts_mut(raw_ptr, length), alloc) };

        Self { inner, length }
    }
}

impl<T, A, const N: usize> From<Box<[T; N], A>> for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    #[inline]
    fn from(boxed: Box<[T; N], A>) -> Self {
        let length = boxed.len();
        let (data, alloc) = Box::into_raw_with_allocator(boxed);
        let raw_ptr = data.cast::<MaybeUninit<T>>();
        let inner = unsafe { Box::from_raw_in(slice::from_raw_parts_mut(raw_ptr, length), alloc) };

        Self { inner, length }
    }
}

impl<T, A, const N: usize> TryFrom<BoundedVec<T, A>> for [T; N]
where
    A: alloc::Allocator,
{
    type Error = BoundedVec<T, A>;

    fn try_from(mut vec: BoundedVec<T, A>) -> Result<[T; N], BoundedVec<T, A>> {
        if vec.len() != N {
            return Err(vec);
        }

        unsafe { vec.set_len(0) };

        let array = unsafe { core::ptr::read(vec.as_ptr() as *const [T; N]) };

        Ok(array)
    }
}

/*
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
*/

impl<T, A> IntoIterator for BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    type Item = T;
    type IntoIter = IntoIter<T, A>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            let mut me = ManuallyDrop::new(self);
            let alloc = ManuallyDrop::new(core::ptr::read(me.allocator()));
            let inner = me.as_non_null();
            let begin = inner.as_ptr();
            let end = if crate::zst::is_zst::<T>() {
                begin.wrapping_byte_add(me.len())
            } else {
                begin.add(me.len()) as *const T
            };
            let cap = me.inner.len();

            IntoIter::from_parts(inner, cap, alloc, inner, end)
        }
    }
}

impl<'a, T, A> IntoIterator for &'a BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, A> IntoIterator for &'a mut BoundedVec<T, A>
where
    A: alloc::Allocator,
{
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T1, T2, A1, A2> PartialEq<BoundedVec<T2, A2>> for BoundedVec<T1, A1>
where
    T1: PartialEq<T2>,
    A1: alloc::Allocator,
    A2: alloc::Allocator,
{
    fn eq(&self, other: &BoundedVec<T2, A2>) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }
}

impl<T1, T2, A> PartialEq<BoundedVec<T2, A>> for &[T1]
where
    T1: PartialEq<T2>,
    A: alloc::Allocator,
{
    fn eq(&self, other: &BoundedVec<T2, A>) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }
}

impl<T1, T2, A> PartialEq<BoundedVec<T2, A>> for &mut [T1]
where
    T1: PartialEq<T2>,
    A: alloc::Allocator,
{
    fn eq(&self, other: &BoundedVec<T2, A>) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }
}

impl<T1, T2, A> PartialEq<BoundedVec<T2, A>> for [T1]
where
    T1: PartialEq<T2>,
    A: alloc::Allocator,
{
    fn eq(&self, other: &BoundedVec<T2, A>) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }
}

impl<T1, T2, A> PartialEq<&[T2]> for BoundedVec<T1, A>
where
    T1: PartialEq<T2>,
    A: alloc::Allocator,
{
    fn eq(&self, other: &&[T2]) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }
}

impl<T1, T2, A> PartialEq<&mut [T2]> for BoundedVec<T1, A>
where
    T1: PartialEq<T2>,
    A: alloc::Allocator,
{
    fn eq(&self, other: &&mut [T2]) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }
}

impl<T1, T2, A> PartialEq<[T2]> for BoundedVec<T1, A>
where
    T1: PartialEq<T2>,
    A: alloc::Allocator,
{
    fn eq(&self, other: &[T2]) -> bool {
        PartialEq::eq(&self[..], &other[..])
    }
}

impl<T1, T2, A, const N: usize> PartialEq<[T2; N]> for BoundedVec<T1, A>
where
    T1: PartialEq<T2>,
    A: alloc::Allocator,
{
    fn eq(&self, other: &[T2; N]) -> bool {
        PartialEq::eq(&self[..], &other[..])
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
