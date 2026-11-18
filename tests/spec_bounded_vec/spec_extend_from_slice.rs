use crate::spec_bounded_vec::common::strategy_bounded_vec_max_len;
use bounded_vec::BoundedVec;

use core::fmt;
use std::alloc;
use std::string::String;

use proptest::prelude::*;

fn prop_extend_from_slice_values<T, A>(values: BoundedVec<T, A>, extension_values: BoundedVec<T, A>) -> Result<(), TestCaseError>
where
    T: PartialEq + Clone + Default + fmt::Debug,
    A: alloc::Allocator + Clone + Default + fmt::Debug,
{
    let result = {
        let mut _result = BoundedVec::with_capacity_in(values.len() + extension_values.len(), values.allocator().clone());
        _result.extend(values.iter().cloned());
        _result.extend_from_slice(extension_values.as_slice());
        _result
    };

    for i in 0..values.len() {
        prop_assert_eq!(&result[i], &values[i]);
    }

    for j in 0..extension_values.len() {
        prop_assert_eq!(&result[values.len() + j], &extension_values[j]);
    }

    Ok(())
}

fn prop_extend_from_slice_len<T, A>(values: BoundedVec<T, A>, extension_values: BoundedVec<T, A>) -> Result<(), TestCaseError>
where
    T: PartialEq + Clone + Default + fmt::Debug,
    A: alloc::Allocator + Clone + Default + fmt::Debug,
{
    let result = {
        let mut _result = BoundedVec::with_capacity_in(values.len() + extension_values.len(), values.allocator().clone());
        _result.extend(values.iter().cloned());
        _result.extend_from_slice(&extension_values);
        _result
    };

    prop_assert_eq!(result.len(), values.len() + extension_values.len());

    Ok(())
}

macro_rules! generate_props {
    ($module_name:ident, $typ:ty, $alloc_typ:ty, $max_length:expr, $vec_gen:ident) => {
        mod $module_name {
            use super::*;
            proptest! {
                #[test]
                fn prop_extend_from_slice_values(
                    values1 in super::$vec_gen::<$typ, $alloc_typ>($max_length),
                    values2 in super::$vec_gen::<$typ, $alloc_typ>($max_length),
                ) {
                    let values1: super::BoundedVec<$typ, $alloc_typ> = values1;
                    let values2: super::BoundedVec<$typ, $alloc_typ> = values2;
                    super::prop_extend_from_slice_values(values1, values2)?
                }

                #[test]
                fn prop_extend_from_slice_len(
                    values1 in super::$vec_gen::<$typ, $alloc_typ>($max_length),
                    values2 in super::$vec_gen::<$typ, $alloc_typ>($max_length),
                ) {
                    let values1: super::BoundedVec<$typ, $alloc_typ> = values1;
                    let values2: super::BoundedVec<$typ, $alloc_typ> = values2;
                    super::prop_extend_from_slice_len(values1, values2)?
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
