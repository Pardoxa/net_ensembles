//! Implementation of a Barabási-Albert Model

use{
    crate::{
        graph::{Graph,NodeContainer},
        GenericGraph,
        Node,
        traits::*,
        iter::{INContainedIterMut, NContainedIterMut,ContainedIterMut}
    },
    std::{
        borrow::Borrow,
        convert::AsRef
    },
    rand::{
        seq::SliceRandom,
        distributions::WeightedIndex,
        prelude::*
    }
};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// Implements a Barabási-Albert Graph ensemble
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct BAensemble<T, R>
where T: Node,
      R: rand::Rng {
    source_graph: Graph<T>,
    ba_graph: Graph<T>,
    rng: R,
    m: usize,
    weights: Vec<usize>,
}

impl<T, R> AsRef<Graph<T>> for BAensemble<T, R>
where T: Node,
      R: rand::Rng
{
    #[inline]
    fn as_ref(&self) -> &Graph<T>{
        &self.ba_graph
    }
}

impl<T, R> Borrow<Graph<T>> for BAensemble<T, R>
where T: Node,
      R: rand::Rng
{
    #[inline]
    fn borrow(&self) -> &Graph<T> {
        &self.ba_graph
    }
}

impl<T, R> WithGraph<T, Graph<T>> for BAensemble<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
{
    fn at(&self, index: usize) -> &T {
        self.ba_graph.at(index)
    }

    fn at_mut(&mut self, index: usize) -> &mut T {
        self.ba_graph.at_mut(index)
    }

    fn graph(&self) -> &Graph<T> {
        self.borrow()
    }

    fn sort_adj(&mut self) {
        self.ba_graph.sort_adj();
    }
}

impl<T, R> BAensemble<T, R>
where T: Node + SerdeStateConform,
      R: rand::Rng
{
    /// # Initialize
    /// * create simplest form of Barabási-Albert graph 
    /// * `m` = 1
    /// * `n`: Number of nodes, `n > 1` has to be true *panics* otherwise
    /// * `rng`:  Rng to use
    /// 
    pub fn new(n: usize, rng: R, m: usize, source_n: usize) -> Self {
        assert!(source_n >= 2);
        assert!(n > source_n);
        let source_graph: Graph::<T> = Graph::complete_graph(source_n);
        let ba_graph: Graph<T> = Graph::new(n);
        let mut e = BAensemble {
            ba_graph,
            source_graph,
            rng,
            m,
            weights: vec![0; n],
        };
        e.randomize();
        e
    }

    /// Generate a new BA graph ensemble with a specified source graph
    /// * **panics** if `source_graph` contains any vertices with degree 0
    /// * `m`: how many edges should each newly added vertex have originally
    /// * `rng`: Random number generator
    /// * `n`: Number of nodes, `n > source_graph.vertex_count()` has to be true *panics* otherwise
    pub fn new_from_graph<B>(n:usize, rng: R, m: usize, source_graph: B) -> Self
    where B: Borrow<Graph<T>>
    {
        assert!(
            source_graph.borrow().container_iter().all(|container| container.degree() > 0),
            "Source graph is not allowed to contain any vertices without edges!"
        );
        assert!(n > source_graph.borrow().vertex_count());
        let mut ba_graph: Graph<T> = Graph::new(n);
        for i in 0..source_graph.borrow().vertex_count() {
            *ba_graph.at_mut(i) = (*source_graph.borrow().at(i)).clone();
        }
        let mut e = Self {
            m,
            rng,
            ba_graph,
            source_graph: source_graph.borrow().clone(),
            weights: vec![0; n],
        };
        e.randomize();
        e
    }

    /// Generate a new BA graph ensemble with a specified generic source graph
    /// * **panics** if `generic_source_graph` contains any vertices with degree 0
    /// * `m`: how many edges should each newly added vertex have originally
    /// * `rng`: Random number generator
    /// * `n`: Number of nodes, `n > source_graph.vertex_count()` has to be true *panics* otherwise
    pub fn new_from_generic_graph<A2, B>(n:usize, rng: R, m: usize, generic_source_graph: B) -> Self
    where
        A2: AdjContainer<T>,
        B: Borrow<GenericGraph<T, A2>>
    {
        assert!(
            generic_source_graph.borrow().container_iter().all(|container| container.degree() > 0),
            "Source graph is not allowed to contain any vertices without edges!"
        );
        assert!(n > generic_source_graph.borrow().vertex_count());
        let source_graph: Graph<T> = generic_source_graph.borrow().into();
        let mut ba_graph: Graph<T> = Graph::new(n);
        for i in 0..source_graph.vertex_count() {
            *ba_graph.at_mut(i) = (*source_graph.at(i)).clone();
        }
        let mut e = Self {
            m,
            rng,
            ba_graph,
            source_graph,
            weights: vec![0; n],
        };
        e.randomize();
        e

    }

    /// get reference to original graph, which is at the core of the BA graph
    pub fn source_graph(&self) -> &Graph<T>
    {
        &self.source_graph
    }
}

impl<T, R> GraphIteratorsMut<T, Graph<T>, NodeContainer<T>> for BAensemble<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
{
    fn contained_iter_neighbors_mut(&mut self, index: usize) ->
        NContainedIterMut<T, NodeContainer<T>, IterWrapper>
    {
        self.ba_graph.contained_iter_neighbors_mut(index)
    }

    fn contained_iter_neighbors_mut_with_index(&mut self, index: usize)
        -> INContainedIterMut<'_, T, NodeContainer<T>>
    {
        self.ba_graph.contained_iter_neighbors_mut_with_index(index)
    }

    fn contained_iter_mut(&mut self) ->  ContainedIterMut<T, NodeContainer<T>> {
        self.ba_graph.contained_iter_mut()
    }
}

impl<T, R> SimpleSample for BAensemble<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng,
{
    /// # Randomizes the Barabási-Albert (BA) graph
    /// * this essentially deletes the BA graph and creates a new one using the initial graph
    fn randomize(&mut self) {
        self.ba_graph.reset_from_graph(&self.source_graph);
        
        let mut random_order: Vec<_> = (self.source_graph.vertex_count()..self.ba_graph.vertex_count()).collect();
        random_order.shuffle(&mut self.rng);


        // init weights
        for i in 0..self.source_graph.vertex_count(){
            self.weights[i] = self.source_graph.vertices[i].degree();
        }
        for i in self.source_graph.vertex_count()..self.ba_graph.vertex_count() {
            self.weights[i] = 0;
        }

        for i in random_order {
            let dist = WeightedIndex::new(&self.weights).unwrap();
            while self.ba_graph.container(i).degree() < self.m
            {
                let index = dist.sample(&mut self.rng);
                // try to add the edge
                let _ = self.ba_graph.add_edge(i, index);
            }

            // update weights
            self.weights[i] = self.ba_graph.container(i).degree();
            for &index in self.ba_graph.container(i).neighbors() {
                self.weights[index] += 1;
            }
        }
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use rand_pcg::Pcg64;
    use crate::*;
    use rand::SeedableRng;

    #[test]
    fn creation() {
        let rng = Pcg64::seed_from_u64(12);
        let _e: BAensemble<EmptyNode, _> =  BAensemble::new(100, rng, 1, 2);
        let rng = Pcg64::seed_from_u64(122321232);
        let mut er: ErEnsembleC<EmptyNode, _> = ErEnsembleC::new(10, 3.0, rng);
        // create valid graph
        while er.graph().container_iter().any(|container| container.degree() < 1) {
            er.randomize();
        }
        let rng = Pcg64::seed_from_u64(1878321232);
        let _ba = BAensemble::new_from_graph(20, rng, 2, er.graph());
        

        let rng= Pcg64::seed_from_u64(1878321232);
        let sw: SwEnsemble<EmptyNode, _> = SwEnsemble::new(10, 0.1, rng);
        let rng= Pcg64::seed_from_u64(78321232);
        let _ba2 = BAensemble::new_from_generic_graph(50, rng, 2, sw);
        
    }
}