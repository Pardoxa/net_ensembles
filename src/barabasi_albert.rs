//! Implementation of a Barabási-Albert Model


use crate::graph::Graph;
use crate::GenericGraph;
use crate::Node;
use crate::traits::*;
use crate::iter::{INContainedIterMut, NContainedIterMut, ContainedIterMut};
use crate::graph::NodeContainer;
use std::borrow::Borrow;
use std::convert::AsRef;
use rand::seq::SliceRandom;

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
    pub fn new(n: usize, rng: R) -> Self {
        assert!(n > 1);
        let source_graph: Graph::<T> = Graph::new(1);
        let ba_graph: Graph<T> = Graph::new(n);
        let mut e = BAensemble {
            ba_graph,
            source_graph,
            rng,
            m: 1,
        };
        e.randomize();
        e
    }

    /// Generate a new BA graph ensemble with a specified source graph
    /// * **panics** if `source_graph` contains any vertices with degree 0
    /// * `m`: how many edges should each newly added vertex have originally
    /// * `rng`: Random number generator
    /// * `n`: Number of nodes, `n > source_graph.vertex_count()` has to be true *panics* otherwise
    pub fn new_from_graph(n:usize, rng: R, m: usize, source_graph: &Graph<T>) -> Self
    {
        assert!(
            source_graph.container_iter().all(|container| container.degree() > 0),
            "Source graph is not allowed to contain any vertices without edges!"
        );
        assert!(n > source_graph.vertex_count());
        let mut ba_graph: Graph<T> = Graph::new(n);
        for i in 0..source_graph.vertex_count() {
            *ba_graph.at_mut(i) = (*source_graph.at(i)).clone();
        }
        let mut e = Self {
            m,
            rng,
            ba_graph,
            source_graph: source_graph.clone(),
        };
        e.randomize();
        e
    }

    /// Generate a new BA graph ensemble with a specified generic source graph
    /// * **panics** if `generic_source_graph` contains any vertices with degree 0
    /// * `m`: how many edges should each newly added vertex have originally
    /// * `rng`: Random number generator
    /// * `n`: Number of nodes, `n > source_graph.vertex_count()` has to be true *panics* otherwise
    pub fn new_from_generic_graph<A2: AdjContainer<T>>(n:usize, rng: R, m: usize, generic_source_graph: &GenericGraph<T, A2>) -> Self
    {
        assert!(
            generic_source_graph.container_iter().all(|container| container.degree() > 0),
            "Source graph is not allowed to contain any vertices without edges!"
        );
        assert!(n > generic_source_graph.vertex_count());
        let source_graph: Graph<T> = generic_source_graph.into();
        let mut ba_graph: Graph<T> = Graph::new(n);
        for i in 0..source_graph.vertex_count() {
            *ba_graph.at_mut(i) = (*source_graph.at(i)).clone();
        }
        let mut e = Self {
            m,
            rng,
            ba_graph,
            source_graph: source_graph.clone(),
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
        NContainedIterMut<T, NodeContainer<T>>
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
        self.ba_graph.clear_edges();
        
        // "copy" original graph
        for i in 0..self.source_graph.vertex_count() {
            for &j in self.source_graph
                .container(i)
                .neighbors()
                .filter(|&&j| i < j) {
                let _ = self.ba_graph.add_edge(i, j);
            }
        }

        
        let mut prev: Vec<_> = (0..self.source_graph.vertex_count()).collect();
        prev.reserve(self.ba_graph.vertex_count() - prev.len());
        let mut random_order: Vec<_> = (self.source_graph.vertex_count()..self.ba_graph.vertex_count()).collect();
        random_order.shuffle(&mut self.rng);

        let final_edge_count = self.source_graph.edge_count() + self.m * (self.ba_graph.vertex_count() - prev.len());
        let mut deg_vec: Vec<_> = Vec::with_capacity(2 * final_edge_count);

        // deg_vec should contain the index of every vertex i exactly deg(i) times
        for (i, container) in self.source_graph.container_iter().enumerate() {
            for _ in 0..container.degree() {
                deg_vec.push(i);
            }
        }

        for i in random_order {
            deg_vec.shuffle(&mut self.rng);
            for &sample in deg_vec.iter()
            {
                // try to add the edge
                let _ = self.ba_graph.add_edge(i, sample);

                if self.ba_graph.container(i).degree() == self.m {
                    for &neighbor in self.ba_graph.container(i).neighbors() {
                        deg_vec.push(neighbor);
                    }
                    for _ in 0..self.m {
                        deg_vec.push(i);
                    }
                    break;
                }

            }
            

        }
    }
}