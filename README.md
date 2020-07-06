# Rust library for random graph ensembles
[![Crate](https://img.shields.io/crates/v/net_ensembles.svg)](https://crates.io/crates/net_ensembles)
[![Rust unit tests - master](https://github.com/Pardoxa/net_ensembles/workflows/Rust%20unit%20tests%20-%20master/badge.svg?branch=master)](https://github.com/Pardoxa/net_ensembles)
[![Docs](https://docs.rs/net_ensembles/badge.svg)](https://docs.rs/net_ensembles/)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.38+-lightgray.svg)

Implements simple sampling and monte carlo (or rather markov-) steps,
that can be used to create a markov chain.

This is intended to be used for various different use cases.
As such, you can easily define additional data that should be stored at each vertex.

## Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
net_ensembles = "0.4"
# for feature "serde_support" (enabled by default) also use
serde = { version = "1.0", features = ["derive"] }
```
If you do not need `serde` support, add this instead:
```toml
[dependencies]
net_ensembles = { version = "0.4", default-features = false  }
```

## Release Notes

See [changelog](CHANGELOG.md).
Note that savestates (created with serde) of v0.4 are incompatible
with savestates generated by older versions and vise versa.
It is likely, that savestates generated with v0.4 will be compatible 
with all following versions, though no guarantees are made.

## currently implemented graph ensembles

* Erdős-Rényi (x2)
* small-world

## work in progress
* Barabási-Albert
* Configuration Model

### Note

On a 64 bit system drawing an usize consumes more than on a 32 bit system, 
therefore ensembles drawn etc. are affected by the size of `usize`.

## Graph

* you can always visualize the current graph by creating a `.dot` file from it.
  There are different options for that, choose which one fits you best.

### Implements measurable quantities

- average degree
- connected components
- diameter
- is_connected
- leaf count
- q_core
- transitivity
- biconnected component
- vertex_load (closely related, often equal to betweeness)

### Iterators

* depth first search from index
* breadth first search from index
* over additional data
* …

## For each vertex

### methods and more
* degree
* check adjacency with other nodes
* access additional data

### Iterators
* iterate over indices stored in adjacency list
* …

# Documentation:

* [current working branch](https://pardoxa.github.io/net_ensembles/master/doc/net_ensembles/)
* [releases](https://docs.rs/net_ensembles/)

# Notes

No warranties whatsoever, but since
I am writing this library for my own scientific simulations,
I do my best to avoid errors.

You can learn more about me and my research on my [homepage](https://www.yfeld.de).

If you notice any bugs, or want to request new features: do not hesitate to
open a new [issue](https://github.com/Pardoxa/net_ensembles/issues) on the repository.

## vertices

* The number of vertices has to be decided when creating a graph and cannot be changed later - at least for now.
* I might add a method to add vertices if requested or I need it myself.

Due to implementation details, where I prioritize fast access of vertices,
it is unlikely, that I will implement the option to remove vertices.
If I do, it will likely be a relatively costly operation, so keep that in mind.


## crates.io

* I might move the `sampling` module into a different crate in the future.
  If I do, everything will likely be reexported at the same position as currently

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
