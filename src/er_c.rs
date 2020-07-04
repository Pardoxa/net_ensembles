//! # Erdős-Rényi ensemble with target connectivity
//! * Draw from an Erdős-Rényi graph ensemble with a target connectivity.
//! * In this model, all possible edges are equally likely
//! * The number of edges is variable
//!
//! # Citations
//! > P. Erdős and A. Rényi, "On the evolution of random graphs,"
//!   Publ. Math. Inst. Hungar. Acad. Sci. **5**, 17-61 (1960)

use crate::{traits::*, iter::*, graph::*};
use std::borrow::Borrow;
use std::convert::AsRef;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// # Returned by markov steps
#[derive(Debug, Clone)]
pub enum ErStepC {
    /// nothing was changed
    Nothing,
    /// an edge was added
    AddedEdge((usize, usize)),
    /// an edge was removed
    RemovedEdge((usize, usize)),
    /// a GraphError occured and is wrapped here
    GError(GraphErrors),
}

impl ErStepC {
    /// `true` if `self` is not `GError` variant
    pub fn is_valid(&self) -> bool {
        match self {
            Self::GError(_)     => false,
            _                   => true,
        }
    }

    /// `panic!` if `self` is `GError` variant
    pub fn valid_or_panic(&self) {
        if let Self::GError(error) = self {
            panic!("ErStepC - invalid - {}", error)
        }
    }

    /// `panic!(msg)` if `self` is `GError` variant
    pub fn valid_or_panic_msg(&self, msg: &str) {
        if let Self::GError(error) = self {
            panic!("ErStepC - invalid {}- {}", msg, error)
        }
    }
}

/// # Implements Erdős-Rényi graph ensemble
/// * variable number of edges
/// * targets a connectivity
/// ## Sampling
/// * for *simple sampling* look at [```SimpleSample``` trait](./sampling/traits/trait.SimpleSample.html)
/// * for *markov steps* look at [```MarkovChain``` trait](../sampling/traits/trait.MarkovChain.html)
/// ## Other
/// * for topology functions look at [`GenericGraph`](../generic_graph/struct.GenericGraph.html)
/// * to access underlying topology or manipulate additional data look at [```WithGraph``` trait](../traits/trait.WithGraph.html)
/// * to use or swap the random number generator, look at [```HasRng``` trait](../traits/trait.HasRng.html)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ErEnsembleC<T, R>
where T: Node,
      R: rand::Rng
{
    graph: Graph<T>,
    prob: f64,
    c_target: f64,
    rng: R,
}

impl<T, R> AsRef<Graph<T>> for ErEnsembleC<T, R>
where T: Node,
      R: rand::Rng
{
    #[inline]
    fn as_ref(&self) -> &Graph<T>{
        &self.graph
    }
}

impl<T, R> Borrow<Graph<T>> for ErEnsembleC<T, R>
where T: Node,
      R: rand::Rng
{
    #[inline]
    fn borrow(&self) -> &Graph<T> {
        &self.graph
    }
}


impl<T, R> HasRng<R> for ErEnsembleC<T, R>
    where   T: Node,
            R: rand::Rng,
{
    /// # Access RNG
    /// If, for some reason, you want access to the internal random number generator: Here you go
    fn rng(&mut self) -> &mut R {
        &mut self.rng
    }

    /// # Swap random number generator
    /// * returns old internal rng
    fn swap_rng(&mut self, mut rng: R) -> R {
        std::mem::swap(&mut self.rng, &mut rng);
        rng
    }
}

impl<T, R> SimpleSample for ErEnsembleC<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng,
{
    /// # Randomizes the edges according to Er probabilities
    /// * this is used by `ErEnsembleC::new` to create the initial topology
    /// * you can use this for sampling the ensemble
    /// * runs in `O(vertices * vertices)`
    fn randomize(&mut self) {
        self.graph.clear_edges();
        // iterate over all possible edges once
        for i in 0..self.graph.vertex_count() {
            for j in i+1..self.graph.vertex_count() {
                if self.rng.gen::<f64>() <= self.prob {
                    // in these circumstances equivalent to 
                    // self.graph.add_edge(i, j).unwrap();
                    // but without checking for existing edges and other errors -> a bit more efficient
                    self.graph.vertices[i].adj.push(j);
                    self.graph.vertices[j].adj.push(i);
                    self.graph.edge_count += 1;
                }
            }
        }
    }
}

impl<T, R> MarkovChain<ErStepC, ErStepC> for ErEnsembleC<T, R>
    where   T: Node + SerdeStateConform,
            R: rand::Rng,
{

    /// # Markov step
    /// * use this to perform a markov step, e.g., to create a markov chain
    /// * result `ErStepC` can be used to undo the step with `self.undo_step(result)`
    fn m_step(&mut self) -> ErStepC {
        let edge = draw_two_from_range(&mut self.rng, self.graph.vertex_count());

        // Try to add edge. else: remove edge
        if self.rng.gen::<f64>() <= self.prob {

            let success = self.graph.add_edge(edge.0, edge.1);
            match success {
                Ok(_)  => ErStepC::AddedEdge(edge),
                Err(_) => ErStepC::Nothing,
            }

        } else {

            let success =  self.graph.remove_edge(edge.0, edge.1);
            match success {
                Ok(_)  => ErStepC::RemovedEdge(edge),
                Err(_) => ErStepC::Nothing,
            }
        }
    }

    /// # Undo a markcov step
    /// * adds removed edge, or removes added edge, or does nothing
    /// * if it returns an Err value, you probably used the function wrong
    /// ## Important:
    /// Restored graph is the same as before the random step **except** the order of nodes
    /// in the adjacency list might be shuffled!
    fn undo_step(&mut self, step: ErStepC) -> ErStepC {
        match step {
            ErStepC::AddedEdge(edge)     => {
                let res = self.graph.remove_edge(edge.0, edge.1);
                match res {
                    Err(err)        => ErStepC::GError(err),
                    Ok(_)           => ErStepC::RemovedEdge(edge),
                }
            },
            ErStepC::RemovedEdge(edge)   => {
                let res = self.graph.add_edge(edge.0, edge.1);
                match res {
                    Err(err)     => ErStepC::GError(err),
                    Ok(_)        => ErStepC::AddedEdge(edge),
                }
            },
            ErStepC::Nothing |
            ErStepC::GError(_)   => step,
        }
    }

    /// # Undo a markov step
    /// * adds removed edge, or removes added edge, or does nothing
    /// * if it returns an Err value, you probably used the function wrong
    /// ## Important:
    /// Restored graph is the same as before the random step **except** the order of nodes
    /// in the adjacency list might be shuffled!
    fn undo_step_quiet(&mut self, step: ErStepC) {
        match step {
            ErStepC::AddedEdge(edge)     => {
                let res = self.graph.remove_edge(edge.0, edge.1);
                if res.is_err() {
                    panic!("ErEnsembleC - undo_step - panic {:?}", res.unwrap_err());
                }
            },
            ErStepC::RemovedEdge(edge)   => {
                let res = self.graph.add_edge(edge.0, edge.1);
                if res.is_err() {
                    panic!("ErEnsembleC - undo_step - panic {:?}", res.unwrap_err());
                }
            },
            _       => step.valid_or_panic_msg("ErEnsembleC - quiet")
        }
    }
}

impl<T, R> ErEnsembleC<T, R>
where T: Node + SerdeStateConform,
      R: rand::Rng
{
    /// # Initialize
    /// create new `ErEnsembleC` with:
    /// * `n` vertices
    /// * target connectivity `c_target`
    /// * `rng` is consumed and used as random number generator in the following
    /// * internally uses `Graph<T>::new(n)`
    /// * generates random edges according to ER model
    pub fn new(n: usize, c_target: f64, rng: R) -> Self {
        let prob = c_target / (n - 1) as f64;
        let graph: Graph<T> = Graph::new(n);
        let mut e = ErEnsembleC {
            graph,
            c_target,
            prob,
            rng,
        };
        e.randomize();
        e
    }

    /// # **Experimental!** Connect the connected components
    /// * adds edges, to connect the connected components
    /// * panics if no vertices are in the graph
    /// * intended as starting point for a markov chain, if you require connected graphs
    /// * do **not** use this to independently (simple-) sample connected networks,
    ///   as this will skew the statistics
    /// * **This is still experimental, this member might change the internal functionallity
    ///   resulting in different connected networks, without prior notice**
    /// * **This member might be removed in braking releases**
    pub fn make_connected(&mut self){
        let mut suggestions = self.graph.suggest_connections();
        let mut last_suggestion = suggestions.pop().unwrap();
        while let Some(suggestion) = suggestions.pop(){
            self.graph.add_edge(last_suggestion, suggestion).unwrap();
            last_suggestion = suggestion;
        }
    }

    /// returns target connectivity
    /// # Explanation
    /// The target connectivity `c_target` is used to
    /// calculate the probability `p`, that any two vertices `i` and `j` (where `i != j`)
    /// are connected.
    ///
    /// `p = c_target / (N - 1)`
    /// where `N` is the number of vertices in the graph
    pub fn target_connectivity(&self) -> f64 {
        self.c_target
    }

    /// * set new value for target connectivity
    /// ## Note
    /// * will only set the value (and probability), which will be used from now on
    /// * if you also want to create a new sample, call `randomize` afterwards
    pub fn set_target_connectivity(&mut self, c_target: f64) {
        let prob = c_target / (self.graph().vertex_count() - 1) as f64;
        self.prob = prob;
        self.c_target = c_target;
    }

    fn graph_mut(&mut self) -> &mut Graph<T> {
        &mut self.graph
    }

    /// # Sort adjecency lists
    /// If you depend on the order of the adjecency lists, you can sort them
    /// # Performance
    /// * internally uses [pattern-defeating quicksort](https://github.com/orlp/pdqsort)
    /// as long as that is the standard
    /// * sorts an adjecency list with length `d` in worst-case: `O(d log(d))`
    /// * is called for each adjecency list, i.e., `self.vertex_count()` times
    pub fn sort_adj(&mut self) {
        self.graph_mut().sort_adj();
    }
}

impl<T, R> GraphIteratorsMut<T, Graph<T>, NodeContainer<T>> for ErEnsembleC<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
{
    fn contained_iter_neighbors_mut(&mut self, index: usize) ->
        NContainedIterMut<T, NodeContainer<T>>
    {
        self.graph.contained_iter_neighbors_mut(index)
    }

    fn contained_iter_neighbors_mut_with_index(&mut self, index: usize)
        -> INContainedIterMut<'_, T, NodeContainer<T>>
    {
        self.graph.contained_iter_neighbors_mut_with_index(index)
    }

    fn contained_iter_mut(&mut self) ->  ContainedIterMut<T, NodeContainer<T>> {
        self.graph.contained_iter_mut()
    }
}


impl<T, R> WithGraph<T, Graph<T>> for ErEnsembleC<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
{
    fn at(&self, index: usize) -> &T{
        self.graph.at(index)
    }

    fn at_mut(&mut self, index: usize) -> &mut T{
        self.graph.at_mut(index)
    }

    fn graph(&self) -> &Graph<T> {
        self.borrow()
    }
}

/// high is exclusive
fn draw_two_from_range<T: rand::Rng>(rng: &mut T, high: usize) -> (usize, usize){
    let first = rng.gen_range(0, high);
    let second = rng.gen_range(0, high - 1);

    if second < first {
        (first, second)
    } else {
        (first, second + 1)
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use rand_pcg::Pcg64;
    use crate::EmptyNode;
    use rand::SeedableRng;

    #[test]
    fn test_edge_count() {
        // create empty graph
        let rng = Pcg64::seed_from_u64(12);
        let mut e = ErEnsembleC::<EmptyNode, Pcg64>::new(100, 0.0, rng);
        let ec = e.graph().edge_count();
        assert_eq!(0, ec);
        // empty graph should not be connected:
        assert!(
            !e.graph()
                .is_connected()
                .expect("test_edge_count error 1")
        );
        assert!(e.graph.dfs(100).next().is_none());

        // add edge
        e.graph_mut()
            .add_edge(0, 1)
            .unwrap();
        let ec_1 = e.graph().edge_count();
        assert_eq!(1, ec_1);

        let mut res = e.graph_mut()
            .add_edge(0, 1);
        assert!(res.is_err());

        // remove edge
        e.graph_mut()
            .remove_edge(0, 1)
            .unwrap();
        let ec_0 = e.graph().edge_count();
        assert_eq!(0, ec_0);

        res = e.graph_mut()
            .remove_edge(0, 1);
        assert!(res.is_err());
    }

    #[test]
    fn draw_2(){
        let mut rng = Pcg64::seed_from_u64(762132);
        for _i in 0..100 {
            let (first, second) = draw_two_from_range(&mut rng, 2);
            assert!(first != second);
        }
        for i in 2..100 {
            let (first, second) = draw_two_from_range(&mut rng, i);
            assert!(first != second);
        }
    }
}
