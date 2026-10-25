# Bounded Vector Library

## Introduction

The **bounded_vec** crate is a library for working with fixed capacity growable arrays. The data type `BoundedVec` is a 
deterministic, fixed capacity vector implementation designed for `no_std` environments where implicit heap allocations 
and dynamic resizing are prohibited. Bounded vectors are beneficial when you need collection semantics within a strict, 
pre-allocated maximum capacity.

## Getting Started

To use this library in your project, add the **bounded_vec** crate as a dependency in
your `Cargo.toml` file

```toml
[dependencies]
bounded_vec = "1.0.0"
```

This library has a `nightly` feature to unlock using custom memory allocators. To use `nightly` add

```toml
[dependencies.bounded_vec]
version = "1.0.0"
features = ["nightly"]
```

or

```toml
[dependencies]
bounded_vec = { version = "1.0.0", features = ["nightly"] }
```

to your `Cargo.toml` file. Optionally, you can add the crate declaration

```rust
extern crate bounded_vec;
```

to your `lib.rs` or `main.rs` file, but this is not strictly necessary.

## Testing

To run the tests for the library, run

```text
cargo test
```

on stable Rust. Run

```text
cargo +nightly test --features "nightly" 
```

to run the tests with nightly features enabled. 

## Benchmarking

To run the benchmarks for the library, run

```text
cargo bench
```

on stable Rust. Run

```text
cargo +nightly bench --features "nightly"
```

to run the benchmarks with nightly features enabled.

## Features

This library has the following features:

* Support for custom memory allocators on all collections on both Rust nightly and Rust stable, where stable support
  comes through the `allocator_api2` crate. This is a temporary feature until the `allocator_api` feature stabilizes.
* Extensive documentation including formal axiomatic semantics for the trickier to understand mutation operations.
* The API is designed to be almost identical to the standard library vector collection counterparts in terms of both 
  the feature set, and the method names, to make them more familiar to work with. The main thing missing is any
  methods that would change the capacity of the bounded vector, since it is designed to have a fixed capacity.
* Vector capacity is defined at runtime. Operations that exceed capacity do not trigger reallocation. They either 
  panic or return an error depending on the API contract.
 