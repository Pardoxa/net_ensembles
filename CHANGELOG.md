# Changelog

## v0.1.0 -> v0.2.0

* `Ensemble` trait -> `SimpleSample` + `MarkovChain` trait
* `mc_step` of former `Ensemble` trait is now `m_step` in `MarkovChain` trait
* fix for `SwEnsemble` (`Ensemble` trait was `pub(crate)` and not `pub` due to returned type)
* added member `contained_iter` in `GenericGraph` (returns iterator)
* added member `container_iter_neighbors` to iterate over `AdjContainer` of neighbors of specific vertex
* added member `contained_iter_neighbors` to iterate over additional information of neighbors of specific vertex
* new trait `WithGraph` for graph ensembles
