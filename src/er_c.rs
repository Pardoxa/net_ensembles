//! # Erdős-Rényi ensemble
//!
//! # Minimal example:
//! ```
//! use net_ensembles;
//! use net_ensembles::traits::*;
//! use rand_pcg::Pcg64;
//! use rand::SeedableRng;
//!
//! // Define your own node
//! #[derive(Clone)]
//! struct ExampleNode { }
//!
//! // it has to implement the Node trait
//! impl Node for ExampleNode {
//!     fn new_from_index(index: u32) -> Self {
//!         ExampleNode { }
//!     }
//! }
//!
//! // now choose your random number generator
//! let rng = Pcg64::seed_from_u64(76);
//! // the following creates an ErEnsembleC graph with 20 vertices and a connectivity of 0.3
//! // and uses the random number generator `rng`
//! let e = net_ensembles::ErEnsembleC::<ExampleNode, Pcg64>::new(20, 0.3, rng);
//! assert_eq!(20, e.graph().vertex_count());
//! ```
//! Take a look at the struct `ErEnsembleC` for details
use crate::graph::Graph;
use crate::GraphErrors;
use crate::Node;

/// # Returned by Monte Carlo Steps
#[derive(Debug, Clone)]
pub enum ErStepC {
    /// nothing was changed
    Nothing,
    /// an edge was added
    AddedEdge((u32, u32)),
    /// an edge was removed
    RemovedEdge((u32, u32)),
}

/// # Implements Erdős-Rényi graph ensemble
#[derive(Debug, Clone)]
pub struct ErEnsembleC<T: Node, R: rand::Rng> {
    graph: Graph<T>,
    prob: f64,
    c_target: f64,
    rng: R,
}

impl<T: Node, R: rand::Rng> ErEnsembleC<T, R> {
    /// # Initialize
    /// * create new ErEnsembleC graph with `n` vertices
    /// * target connectivity `c_target`
    /// * `rng` is consumed and used as random number generator in the following
    /// * internally uses `Graph<T>::new(n)`
    /// * generates random edges according to ErEnsembleC probability (see `ErEnsembleC::randomize`)
    pub fn new(n: u32, c_target: f64, rng: R) -> Self {
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


    /// # Randomizes the edges according to Er probabilities
    /// * this is used by `ErEnsembleC::new` to create the initial topology
    /// * you can use this for sampling the ensemble
    /// * runs in `O(vertices * vertices)`
    pub fn randomize(&mut self) {
        self.graph.clear_edges();
        // iterate over all possible edges once
        for i in 0..self.graph.vertex_count() {
            for j in i+1..self.graph.vertex_count() {
                if self.rng.gen::<f64>() <= self.prob {
                    self.graph.add_edge(i, j).unwrap();
                }
            }
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

    /// # Monte Carlo step
    /// * use this to perform a Monte Carlo step
    /// * result `ErStepC` can be used to undo the step with `self.undo_step(result)`
    pub fn random_step(&mut self) -> ErStepC {
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

    /// # Monte Carlo steps
    /// * use this to perform multiple Monte Carlo steps at once
    /// * result `Vec<ErStepC>` can be used to undo the steps with `self.undo_steps(result)`
    pub fn random_steps(&mut self, count: usize) -> Vec<ErStepC> {
        (0..count)
            .map(|_| self.random_step())
            .collect()
    }

    /// # Undo a Monte Carlo step
    /// * adds removed edge, or removes added edge, or does nothing
    /// * if it returns an Err value, you probably used the function wrong
    /// ## Important:
    /// Restored graph is the same as before the random step **except** the order of nodes
    /// in the adjacency list might be shuffled!
    pub fn undo_step(&mut self, step: ErStepC) -> Result<(),GraphErrors> {
        match step {
            ErStepC::AddedEdge(edge)     => self.graph.remove_edge(edge.0, edge.1),
            ErStepC::Nothing             => Ok(()),
            ErStepC::RemovedEdge(edge)   => self.graph.add_edge(edge.0, edge.1)
        }
    }

    /// # Undo a Monte Carlo step
    /// * adds removed edges, removes added edge etc. in the correct order
    /// * if it returns an Err value, you probably used the function wrong
    /// ## Important:
    /// Restored graph is the same as before the random steps **except** the order of nodes
    /// in the adjacency list might be shuffled!
    pub fn undo_steps(&mut self, mut steps: Vec<ErStepC>) -> Result<(),GraphErrors> {
        while let Some(step) = steps.pop() {
            self.undo_step(step)?;
        }
        Ok(())
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

    /// returns reference to the underlying topology aka, the `Graph<T>`
    ///
    /// Use this to call functions regarding the topology
    pub fn graph(&self) -> &Graph<T> {
        &self.graph
    }

    fn graph_mut(&mut self) -> &mut Graph<T> {
        &mut self.graph
    }

    /// # Access RNG
    /// If, for some reason, you want access to the internal random number generator: Here you go
    pub fn rng(&mut self) -> &mut R {
        &mut self.rng
    }


}

/// high is exclusive
fn draw_two_from_range<T: rand::Rng>(rng: &mut T, high: u32) -> (u32, u32){
    let first = rng.gen_range(0, high);
    let second = rng.gen_range(0, high - 1);
    return if second < first {
        (first, second)
    } else {
        (first, second + 1)
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use rand_pcg::Pcg64;
    use crate::TestNode;
    use rand::SeedableRng;

    #[test]
    fn test_edge_count() {
        // create empty graph
        let rng = Pcg64::seed_from_u64(12);
        let mut e = ErEnsembleC::<TestNode, Pcg64>::new(100, 0.0, rng);
        let ec = e.graph().edge_count();
        assert_eq!(0, ec);
        // empty graph should not be connected:
        assert!(
            !e.graph()
                .is_connected()
                .expect("test_edge_count error 1")
        );

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
