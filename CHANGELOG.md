# Changelog

## v0.3.0 &rarr; master

# GenericGraph
* add member `connected_components_ids`

# ensembles
* add member `make_connected` for ErEnsembleC
* add member `make_connected` for SwEnsemble

## v0.2.2 &rarr; v0.3.0

## optimizations
* optimization for `q_core`
* major optimization for `vertex_load`

## traits
* implemented `Borrow<GenericGraph>` for the ensembles
* implemented `AsRef<GenericGraph>` for the ensembles
* new trait `MeasurableGraphQuantities<G>`
* implemented trait `MeasurableGraphQuantities` for ensembles
* new trait `Dot`
* implemented `Dot` for `GenericGraph`
* new trait `DotExtra`
* implemented `DotExtra` for `GenericGraph`
* new trait `Metropolis`

## macro
* add `dot_options!` macro for ease of use


## new iterator
*  `INContainedIterMut` to mutably iterate over neighbors
  of specified vertex and also get the indices of the neighbors
* added member in `GraphIteratorsMut<T, G, A>` and `GenericGraph`
  to return `INContainedIterMut`

## fixes
* trait `GraphIteratorsMut<'a, T, G, A>` &rarr; `GraphIteratorsMut<T, G, A>`
* trait `GraphIterators<'a, T, G, A>` &rarr; `GraphIterators<T, G, A>`

## deprecated
* `to_dot*` members of `GenericGraph`. Use members of `Dot` or `DotExtra`
  trait instead

## other
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
