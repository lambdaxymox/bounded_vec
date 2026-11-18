use crate::spec_bounded_vec::common::{
    SingleBoundedValue,
    strategy_bounded_vec_len,
    strategy_bounded_vec_max_len,
};
use bounded_vec::BoundedVec;

use core::fmt;
use std::alloc;
use std::string::String;

use proptest::prelude::*;

fn strategy_prop_into_iter_take<T, A>(max_length: usize) -> impl Strategy<Value = (BoundedVec<T, A>, usize)>
where
    T: PartialEq + Clone + Default + fmt::Debug + Arbitrary + SingleBoundedValue,
    A: alloc::Allocator + Clone + Default + fmt::Debug,
{
    (1..=max_length).prop_flat_map(move |length| (strategy_bounded_vec_len(length), 0..=length))
}

fn prop_into_iter_back_to_vec<T, A>(values: BoundedVec<T, A>) -> Result<(), TestCaseError>
where
    T: PartialEq + Clone + Default + fmt::Debug,
    A: alloc::Allocator + Clone + Default + fmt::Debug,
{
    let vec = values.clone();
    let mut result = BoundedVec::with_capacity_in(values.capacity(), values.allocator().clone());
    for value in vec.into_iter() {
        result.push(value);
    }

    let expected = values.as_slice();
    let result = result.as_slice();

    prop_assert_eq!(result, expected);

    Ok(())
}

fn prop_into_iter_take<T, A>((values, count): (BoundedVec<T, A>, usize)) -> Result<(), TestCaseError>
where
    T: PartialEq + Clone + Default + fmt::Debug,
    A: alloc::Allocator + Clone + Default + fmt::Debug,
{
    let mut vec = BoundedVec::with_capacity_in(values.capacity(), values.allocator().clone());
    vec.extend(values.iter().cloned());

    let mut expected = BoundedVec::with_capacity_in(values.capacity(), values.allocator().clone());
    for value in values.iter().cloned().take(count) {
        expected.push(value);
    }

    let mut result = BoundedVec::with_capacity_in(values.capacity(), values.allocator().clone());
    result.extend(values.into_iter().take(count));

    prop_assert_eq!(result, expected);

    Ok(())
}

fn prop_into_iter_take_none<T, A>(values: BoundedVec<T, A>) -> Result<(), TestCaseError>
where
    T: PartialEq + Clone + Default + fmt::Debug,
    A: alloc::Allocator + Clone + Default + fmt::Debug,
{
    let mut vec = BoundedVec::with_capacity_in(values.capacity(), values.allocator().clone());
    vec.extend(values.iter().cloned());

    let mut result = BoundedVec::with_capacity_in(values.capacity(), values.allocator().clone());
    result.extend(values.into_iter().take(0));

    prop_assert!(result.is_empty());

    Ok(())
}

fn prop_into_iter_ordering<T, A>(values: BoundedVec<T, A>) -> Result<(), TestCaseError>
where
    T: PartialEq + Clone + Default + fmt::Debug,
    A: alloc::Allocator + Clone + Default + fmt::Debug,
{
    let vec = values.clone();
    let mut iter = vec.clone().into_iter();
    for i in 0..vec.len() {
        let expected = vec.get(i).cloned();
        let result = iter.next();

        prop_assert_eq!(result, expected);
    }

    prop_assert_eq!(iter.next(), None);

    Ok(())
}

macro_rules! generate_props {
    ($module_name:ident, $typ:ty, $alloc_typ:ty, $max_length:expr, $vec_gen:ident) => {
        mod $module_name {
            use super::*;
            proptest! {
                #[test]
                fn prop_into_iter_back_to_vec(values in super::$vec_gen::<$typ, $alloc_typ>($max_length)) {
                    let values: super::BoundedVec<$typ, $alloc_typ> = values;
                    super::prop_into_iter_back_to_vec(values)?
                }

                #[test]
                fn prop_into_iter_take(values in super::strategy_prop_into_iter_take::<$typ, $alloc_typ>($max_length)) {
                    let values: (super::BoundedVec<$typ, $alloc_typ>, usize) = values;
                    super::prop_into_iter_take(values)?
                }

                #[test]
                fn prop_into_iter_take_none(values in super::$vec_gen::<$typ, $alloc_typ>($max_length)) {
                    let values: super::BoundedVec<$typ, $alloc_typ> = values;
                    super::prop_into_iter_take_none(values)?
                }

                #[test]
                fn prop_into_iter_ordering(values in super::$vec_gen::<$typ, $alloc_typ>($max_length)) {
                    let values: super::BoundedVec<$typ, $alloc_typ> = values;
                    super::prop_into_iter_ordering(values)?
                }
            }
        }
    };
}

generate_props!(unit, (), alloc::Global, 32, strategy_bounded_vec_max_len);
generate_props!(u8, u8, alloc::Global, 32, strategy_bounded_vec_max_len);
generate_props!(u16, u16, alloc::Global, 32, strategy_bounded_vec_max_len);
generate_props!(u32, u32, alloc::Global, 32, strategy_bounded_vec_max_len);
generate_props!(u64, u64, alloc::Global, 32, strategy_bounded_vec_max_len);
generate_props!(usize, usize, alloc::Global, 32, strategy_bounded_vec_max_len);
generate_props!(string, String, alloc::Global, 32, strategy_bounded_vec_max_len);
