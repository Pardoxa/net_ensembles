# Rust library for network-ensembles

# Documentation:

You can find the Documentation [here](https://www.yfeld.de/lib/doc/net_ensembles/).

### currently implemented network ensembles

* Erdős-Rényi

### planing to implement network ensembles

* small-world

## Note

### vertices

The number of vertices has to be decided when creating a graph and cannot be changed later - at least for now.

I might add a method to add vertices if requested or I need it myself.

Due to implementation details, where I prioritize fast access of vertices,
it is unlikely, that I will implement the option to remove vertices.
If I do, it will likely be a relatively costly operation, so keep that in mind.

### edges

You are free to create or remove edges as you see fit
