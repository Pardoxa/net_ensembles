# Changelog

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
