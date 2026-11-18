#![deny(unsafe_op_in_unsafe_fn)]
#![deny(private_interfaces)]
#![deny(clippy::missing_safety_doc)]
extern crate std;

mod spec_bounded_vec;
mod test_bounded_vec;
mod test_drop;
mod test_panic_safety;
