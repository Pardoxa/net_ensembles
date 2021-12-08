# Changelog

## v0.5.0 &rarr; master

* Remove unneeded trait bounds for Graph iterators
* adding module `dual_graph`

## GenericGraph

* adding `from_vec`
* adding member `degree_iter`
* adding member `degree_histogram`
* adding member `cloned_subgraph`
* adding iterator `dfs_mut`
* adding iterator `bfs_index_depth_mut`

## Graph

* implemented `FromIterator` trait

## NodeContainer

* add `AdjList` trait

## v0.4.0 &rarr; v0.5.0

### sampling
* move sampling to new crate - sampling is reexported at old location

### Iterators
* adding new breadth first iterator `BfsFiltered`
* improve `size_hint` for a few iterators

### GenericGraph
* adding member `bfs_filtered` 
* adding member `degree_vec`

### Graph
* adding `From<&GenericGraph>`
* adding `Graph::complete_graph`

### minor optimizations
* ErEnsembleC - `randomize` (and therefore `new`, as that uses randomize) 
* GenericGraph - `new` roughly twice as fast now, though it was fast to begin with

### Ensemble
* adding Barabási-Albert ensemble - work in progress
* adding ConfigurationModel - work in progress
* adding Spacial Ensemble - work in progress
* moved `sort_adj`to `WithGraph`trait
* `ErEnsembleC`: removed `R: Rng` trait bound for struct, added `Dot` trait
* `ErEnsembleM`: removed `R: Rng` trait bound for struct, added `Dot` trait
* `SwEnsemble`: removed `R: Rng` trait bound for struct, added `Dot` trait

### traits
* WithGraph now has `sort_adj()` member
* MarkovChain now has member `m_steps_quiet()`
* adding traits `Histogram`, `HistogramVal` and `HistogramIntervalDistance`
* HasRng - changed `swap_rng(&mut self, rng: Rng) -> Rng` to `swap_rng(&mut self, rng: &mut Rng)`
  and moved HasRng into sampling module. Reexported at old position.

### module sampling
* added `WangLandauAdaptive` for performing WangLandau simulation
* added `EntropicAdaptive` for performing Entropic sampling simulation
* added Histograms 

### Markov Steps
* Adding `Serialize + Deserialize` for ErStepC
* Adding `Clone + Serialize + Deserialize` for ErStepM
* Adding `Cloone + Serialize + Deserializ` for SwChangeState

### and more

* I did not list all changes, you are welcome to take a look at the documentation


## v0.3.0 &rarr; v0.4.0
## Major breaking change: `u32 -> usize`
* **Almost all `u32` where changed into `usize`**. 
  In fact so many, that I will not list them all, sorry for the inconvinience.
  Why? Because `u32` was a very bad choice and I had to convert it into `usize` almost everywhere anyway. 
  Currently (almost?) no one uses this library anyway. That might change, meaning:
  if I do not do this change now, it will only become increasingly inconvinient in the future,
  so I'd be stuck with it.
* This increases the performance of some iterators
* **IMPORTANT**: This will sadly affect the state of the rng, as I am now generating usize instead of u32.
  Therefore this affects the individual networks drawn for any given seed.

### optimizations
* optimization for `diameter`


### GenericGraph
* add member `connected_components_ids`
* add member `contained_iter_neighbors_with_index` which returns new iterator

### MetropolisState
* rename `fn to_rng` &rarr; `fn into_rng`
* add member `is_finished`
* add member `set_m_beta`
* add member `set_temperature`
* derive Debug

### MetropolisSave
* add member `is_finished`

### ensembles
* add **experimental** member `make_connected` for `ErEnsembleC`
* add **experimental** member `make_connected` for `SwEnsemble`

### traits
* `GraphIterators` added member `contained_iter_neighbors_with_index(&self, index: usize)`

###  Iterator
* new Iterator `NIContainedIter`; similar to `INContainedIterMut`
  but differs in mutability

## v0.2.2 &rarr; v0.3.0

### optimizations
* optimization for `q_core`
* major optimization for `vertex_load`

### traits
* implemented `Borrow<GenericGraph>` for the ensembles
* implemented `AsRef<GenericGraph>` for the ensembles
* new trait `MeasurableGraphQuantities<G>`
* implemented trait `MeasurableGraphQuantities` for ensembles
* new trait `Dot`
* implemented `Dot` for `GenericGraph`
* new trait `DotExtra`
* implemented `DotExtra` for `GenericGraph`
* new trait `Metropolis`

### macro
* add `dot_options!` macro for ease of use


### new iterator
*  `INContainedIterMut` to mutably iterate over neighbors
  of specified vertex and also get the indices of the neighbors
* added member in `GraphIteratorsMut<T, G, A>` and `GenericGraph`
  to return `INContainedIterMut`

### fixes
* trait `GraphIteratorsMut<'a, T, G, A>` &rarr; `GraphIteratorsMut<T, G, A>`
* trait `GraphIterators<'a, T, G, A>` &rarr; `GraphIterators<T, G, A>`

### deprecated
* `to_dot*` members of `GenericGraph`. Use members of `Dot` or `DotExtra`
  trait instead

### other
* `fn is_adjacent(&self, other_id: &u32) -> bool;` &rarr; `fn is_adjacent(&self, other_id: u32) -> bool;`
* now `dot_options` from `to_dot_with_labels*` do not have to be a string,
they only have to implement `AsRef<str>`, and the closure `f` only has to
return something, that implements `AsRef<str>`, not necessarily a `String`

## v0.2.1 &rarr; v0.2.2

### iterators
* iterator optimizations (`nth` + `FusedIterator` + …)
* added iterator: `contained_iter_mut`
* added iterator: `contained_iter_neighbors_mut`

### traits
* added `GraphIterators` and generically implemented it for the Ensembles
* added `GraphIteratorsMut`

## v0.1.0 &rarr; v0.2.1

### traits
* ~`Ensemble`~ &#8680; `SimpleSample` + `MarkovChain`
* `mc_step` of former `Ensemble` trait is now `m_step` in `MarkovChain` trait
* new trait `WithGraph` for graph ensembles
* ```EnsembleRng``` &#8680; ```HasRng```
* ```Node``` removed ```parse_str``` and ```make_string```, since serde takes care of that now

### bug fix
* fix for `SwEnsemble` (`Ensemble` trait was `pub(crate)` and not `pub` due to returned type)

### `GenericGraph` - new iterators
* added member `contained_iter` (returns iterator)
* added member `container_iter_neighbors` to iterate over `AdjContainer` of neighbors of specific vertex
* added member `contained_iter_neighbors` to iterate over additional information of neighbors of specific vertex

### serde
* added trait ```SerdeStateConform```
* added blanked implementation for ```SerdeStateConform```
* derived ```Serialize``` and ```Deserialize``` for most types

### features
* added feature "serde_support" (enabled by default)

### other
* removed ```Display``` trait. Use serde instead.
* reexported ```rand```
