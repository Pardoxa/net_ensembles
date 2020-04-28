# Rust library for random graph ensembles
[![Crate](https://img.shields.io/crates/v/net_ensembles.svg)](https://crates.io/crates/net_ensembles)

Implements simple sampling and monte carlo (or rather markov-) steps,
that can be used to create a markov chain.

This is intended to be used for various different use cases.
As such, you can easily define additional data that should be stored at each vertex.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
net_ensembles = "0.2"
```

## For the whole graph:

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

* [changelog](CHANGELOG.md)
* [current working branch](https://pardoxa.github.io/net_ensembles/master/doc/net_ensembles/)
* [v0.1.0](https://pardoxa.github.io/net_ensembles/v0.1.0/doc/net_ensembles/)
* [v0.2.1](https://pardoxa.github.io/net_ensembles/v0.2.1/doc/net_ensembles/)
* [v0.2.2](https://pardoxa.github.io/net_ensembles/v0.2.2/doc/net_ensembles/)

# Notes

No warranties whatsoever, but since
I am writing this library for my own scientific simulations,
I do my best to avoid errors.

You can learn more about me and my research on my [homepage](https://www.yfeld.de).

If you like my library but feel like there is an iterator missing or something
like that: feel free to create an issue on the repository, I might add it.

## currently implemented network ensembles

* Erdős-Rényi (x2)
* small-world

## vertices

* The number of vertices has to be decided when creating a graph and cannot be changed later - at least for now.
* I might add a method to add vertices if requested or I need it myself.

Due to implementation details, where I prioritize fast access of vertices,
it is unlikely, that I will implement the option to remove vertices.
If I do, it will likely be a relatively costly operation, so keep that in mind.

## crates.io

* I might move the `MarkovChain` and `SimpleSample` trait into a different crate in the future.
  If I do, the traits will be reexported at the same position as currently

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
