//! # Erdős-Rényi with constant number of edges
//! * Draw from an Erdős-Rényi graph ensemble
//! * In this model, all possible edges are equally likely
//! * The number of edges is fixed
//!
//! # Citations
//! > P. Erdős and A. Rényi, "On the evolution of random graphs,"
//!   Publ. Math. Inst. Hungar. Acad. Sci. **5**, 17-61 (1960)
use crate::graph::Graph;
use crate::Node;
use crate::traits::{Ensemble, EnsembleRng};
use rand::seq::SliceRandom;
use rand::distributions::{Distribution, Uniform};

/// Storing the information about which edges were deleted or added
#[derive(Debug)]
pub struct ErStepM{
    /// removed edge
    pub(crate) removed: (u32, u32),
    pub(crate) i_removed: usize,
    pub(crate) inserted: (u32, u32),
    pub(crate) i_inserted: usize,
}

impl ErStepM{
    fn invert(&mut self){
        std::mem::swap(
            &mut self.removed,
            &mut self.inserted
        );
    }
}

/// # Implements Erdős-Rényi graph ensemble
/// Constant number of edges
#[derive(Debug, Clone)]
pub struct ErEnsembleM<T: Node, R: rand::Rng> {
    graph: Graph<T>,
    m: usize,
    rng: R,
    all_edges: Vec<(u32, u32)>,
    possible_edges: Vec<(u32, u32)>,
    current_edges: Vec<(u32, u32)>,
    current_uniform: rand::distributions::uniform::Uniform<usize>,
    possible_uniform: rand::distributions::uniform::Uniform<usize>,
}

impl<T, R> EnsembleRng<ErStepM, ErStepM, R> for ErEnsembleM<T, R>
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


impl <T, R> Ensemble<ErStepM, ErStepM> for ErEnsembleM<T, R>
    where   T: Node,
            R: rand::Rng,
{
    /// * undo a monte carlo step, return result-state
    /// * if you want to undo more than one step
    /// see [`undo_steps`](#method.undo_steps)
    fn undo_step(&mut self, mut step: ErStepM) -> ErStepM {
        step.invert();
        self.step(&step);
        step
    }

    /// * undo a monte carlo step, **panic** on invalid result state
    /// * for undoing multiple steps see [`undo_steps_quiet`](#method.undo_steps_quiet)
    fn undo_step_quiet(&mut self, mut step: ErStepM) -> (){
        step.invert();
        self.step(&step);
    }

    /// # Randomizes self according to  model
    /// * this is intended for creation of initial sample
    /// * used in [`simple_sample`](#method.simple_sample)
    /// and [`simple_sample_vec`](#method.simple_sample_vec)
    fn randomize(&mut self){
        self.graph.clear_edges();

        self.shuffle_all_edges();

        // uses mem-copy
        self.current_edges
            .copy_from_slice(&self.all_edges[..self.m]);

        for edge in self.current_edges.iter()
        {
            self.graph
                .add_edge(edge.0, edge.1)
                .unwrap();
        }

        // uses mem-copy
        self.possible_edges
            .copy_from_slice(&self.all_edges[self.m..]);
    }

    /// # Monte Carlo step
    /// * use this to perform a Monte Carlo step
    /// * for doing multiple mc steps at once, use [`mc_steps`](#method.mc_steps)
    fn mc_step(&mut self) -> ErStepM{
        let index_current   = self.current_uniform .sample(&mut self.rng);
        let index_possible  = self.possible_uniform.sample(&mut self.rng);

        let step = ErStepM{
            removed:  self.current_edges[index_current],
            i_removed: index_current,
            inserted: self.possible_edges[index_possible],
            i_inserted: index_possible
        };

        self.step(&step);

        step
    }
}

impl<T: Node, R: rand::Rng> ErEnsembleM<T, R> {
    fn step(&mut self, step: &ErStepM){
        self.graph
            .remove_edge(step.removed.0, step.removed.1)
            .unwrap();

        self.graph
            .add_edge(step.inserted.0, step.inserted.1)
            .unwrap();

        std::mem::swap(
            &mut self.current_edges[step.i_removed],
            &mut self.possible_edges[step.i_inserted]
        );
    }

    /// # Initialize
    /// create new ErEnsembleM graph with:
    /// * `n` vertices
    /// * `m` edges
    /// * `rng` is consumed and used as random number generator in the following
    /// * internally uses `Graph<T>::new(n)`
    /// * generates random edges according to ER model
    pub fn new(n: u32, m: usize, rng: R) -> Self {
        let graph: Graph<T> = Graph::new(n);

        let p_edges = (n * (n - 1)) / 2;
        let mut vec = Vec::with_capacity(p_edges as usize);
        for i in 0..n {
            for j in i+1..n {
                vec.push((i, j));
            }
        }

        let current_uniform     = Uniform::from(0..m);
        let possible_uniform    = Uniform::from(0..(p_edges as usize - m));

        let mut e = ErEnsembleM {
            graph,
            m,
            rng,
            all_edges: vec,
            possible_edges: vec![(0, 0); p_edges as usize - m],   // randomize will mem_copy - slice needs to be big enough
            current_edges: vec![(0, 0); m], // randomize will mem_copy - slice needs to be big enough
            current_uniform,
            possible_uniform
        };
        e.randomize();
        e
    }

    fn graph_mut(&mut self) -> &mut Graph<T> {
        &mut self.graph
    }

    /// Return total number of edges
    pub fn get_m(&self) -> usize {
        self.m
    }

    fn shuffle_all_edges(&mut self) -> () {
        self.all_edges
            .shuffle(&mut self.rng);
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

    /// access additional information at vertex
    pub fn at(&self, index: usize) -> & T {
        self.graph.at(index)
    }

    /// mutable access of additional information at index
    pub fn at_mut(&mut self, index: usize) -> &mut T {
        self.graph.at_mut(index)
    }
}
