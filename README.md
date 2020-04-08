# Rust library for random graph ensembles

Implements simple sampling and monte carlo (or rather markov-) steps,
that can be used to create a markov chain.

This is intended to be used for various different use cases.
As such, you can easily define additional data that should be stored at each vertex.


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

## For each vertex

### methods and more
* degree
* check adjacency with other nodes
* access additional data

## Iterators
* iterate over indices stored in adjacency list

# Documentation:

You can find the Documentation [here](https://pardoxa.github.io/net_ensembles/net_ensembles/).

# Notes

No warranties whatsoever, but since
I am writing this library for my own scientific simulations,
I do my best to avoid errors.

You can learn more about me and my research on my [homepage](www.yfeld.de).

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

* I plan to publish this to crates.io soon
* I might move the `Ensemble` trait into a different crate in the Future.
  If I do, the trait will be reexported at the same position as currently
