//! # Erdős-Rényi with constant number of edges
//! * Draw from an Erdős-Rényi graph ensemble
//! * In this model, all possible edges are equally likely
//! * The number of edges is fixed
//!
//! # Citations
//! > P. Erdős and A. Rényi, "On the evolution of random graphs,"
//!   Publ. Math. Inst. Hungar. Acad. Sci. **5**, 17-61 (1960)
//!
use {
    crate::{graph::*, iter::*, traits::*},
    std::{borrow::Borrow, convert::AsRef, io::Write},
    rand::seq::SliceRandom
};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// Storing the information about which edges were deleted or added
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ErStepM{
    /// removed edge
    pub(crate) removed: (usize, usize),
    pub(crate) i_removed: usize,
    pub(crate) inserted: (usize, usize),
    pub(crate) i_inserted: usize,
}

impl ErStepM{
    #[allow(unused)]
    fn invert(&mut self){
        std::mem::swap(
            &mut self.removed,
            &mut self.inserted
        );
    }

    fn inverted(&self) -> Self{
        Self{
            removed: self.inserted,
            inserted: self.removed,
            i_inserted: self.i_inserted,
            i_removed: self.i_removed,
        }
    }
}

/// # Implements Erdős-Rényi graph ensemble
/// Constant number of edges
/// * **Note** simple sampling of this ensemble is somewhat inefficient right now -
///   I might change it in the future, though that will change the results of the simple sampling
///   (Not on average of cause)
/// * for *simple sampling* look at [```SimpleSample``` trait](./sampling/traits/trait.SimpleSample.html)
/// * for *markov steps* look at [```MarkovChain``` trait](../sampling/traits/trait.MarkovChain.html)
/// ## Other
/// * for topology functions look at [`GenericGraph`](../generic_graph/struct.GenericGraph.html)
/// * to access underlying topology or manipulate additional data look at [```WithGraph``` trait](../traits/trait.WithGraph.html)
/// * to use or swap the random number generator, look at [```HasRng``` trait](../traits/trait.HasRng.html)
///
/// # Save and load example
/// * only works if feature ```"serde_support"``` is enabled
/// * Note: ```"serde_support"``` is enabled by default
/// * I need the ```#[cfg(feature = "serde_support")]``` to ensure the example does compile if
/// * you can do not have to use ```serde_json```, look [here](https://docs.serde.rs/serde/) for more info
///  you opt out of the default feature
/// ```
/// use net_ensembles::traits::*; // I recommend always using this
/// use serde_json;
/// use rand_pcg::Pcg64;
/// use net_ensembles::{ErEnsembleM, EmptyNode, rand::SeedableRng};
/// use std::fs::File;
/// use std::io::{BufWriter, BufReader};
///
/// let rng = Pcg64::seed_from_u64(95);
/// // create Erdős-Rényi ensemble with 200 vertices and 600 edges
/// let er_ensemble = ErEnsembleM::<EmptyNode, Pcg64>::new(200, 600, rng);
///
/// #[cfg(feature = "serde_support")]
/// {
///     // storing the ensemble in a file:
///
///     let er_m_file = File::create("store_ER_m.dat")
///           .expect("Unable to create file");
///     let buf_writer = BufWriter::new(er_m_file);
///
///     // or serde_json::to_writer(buf_writer, &er_ensemble);
///     serde_json::to_writer_pretty(buf_writer, &er_ensemble);
///
///     // loading ensemble from file:
///
///     let read = File::open("store_ER_m.dat")
///         .expect("Unable to open file");
///     let mut buf_reader = BufReader::new(read); 
///
///     let er: ErEnsembleM::<EmptyNode, Pcg64> = serde_json::from_reader(buf_reader).unwrap();
/// }
///
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct ErEnsembleM<T: Node, R>
{
    graph: Graph<T>,
    m: usize,
    rng: R,
    all_edges: Vec<(usize, usize)>,
    possible_edges: Vec<(usize, usize)>,
    current_edges: Vec<(usize, usize)>,
}


impl<T, R> AsRef<Graph<T>> for ErEnsembleM<T, R>
where T: Node,
      R: rand::Rng
{
    #[inline]
    fn as_ref(&self) -> &Graph<T>{
        &self.graph
    }
}

impl<T, R> Borrow<Graph<T>> for ErEnsembleM<T, R>
where T: Node,
      R: rand::Rng
{
    #[inline]
    fn borrow(&self) -> &Graph<T> {
        &self.graph
    }
}

impl<T, R> HasRng<R> for ErEnsembleM<T, R>
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
    fn swap_rng(&mut self, rng: &mut R) {
        std::mem::swap(&mut self.rng, rng);
    }
}

impl<T, R> SimpleSample for ErEnsembleM<T, R>
    where   T: Node + SerdeStateConform,
            R: rand::Rng,
{
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
}

impl <T, R> MarkovChain<ErStepM, ErStepM> for ErEnsembleM<T, R>
    where   T: Node + SerdeStateConform,
            R: rand::Rng,
{
    /// * undo a markov step, return result-state
    /// * if you want to undo more than one step
    /// see [`undo_steps`](#method.undo_steps)
    fn undo_step(&mut self, step: &ErStepM) -> ErStepM {
        let step = step.inverted();
        self.step(&step);
        step
    }

    /// * undo a markov step, **panic** on invalid result state
    /// * for undoing multiple steps see [`undo_steps_quiet`](#method.undo_steps_quiet)
    fn undo_step_quiet(&mut self, step: &ErStepM) {
        let step = step.inverted();
        self.step(&step);
    }

    /// # Markov step
    /// * use this to perform a markov step
    /// * for doing multiple mc steps at once, use [`m_steps`](#method.m_steps)
    fn m_step(&mut self) -> ErStepM{
        let index_current   = self.rng.gen_range(0..self.current_edges.len());
        let index_possible  = self.rng.gen_range(0..self.possible_edges.len());

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

impl<T, R> ErEnsembleM<T, R>
where T: Node + SerdeStateConform,
      R: rand::Rng
{
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
    pub fn new(n: usize, m: usize, rng: R) -> Self {
        let graph: Graph<T> = Graph::new(n);

        let p_edges = (n * (n - 1)) / 2;

        // panic, if you try to create a graph with to many edges
        assert!(
            m <= p_edges,
            "A complete graph with {} vertices has {} edges. \
             You requested {} edges, i.e., to many. Panic at function `new` of struct {}",
            n,
            p_edges,
            m,
            std::any::type_name::<Self>()
        );

        let mut vec = Vec::with_capacity(p_edges);
        for i in 0..n {
            for j in i+1..n {
                vec.push((i, j));
            }
        }

        let mut e = ErEnsembleM {
            graph,
            m,
            rng,
            all_edges: vec,
            possible_edges: vec![(0, 0); p_edges - m],   // randomize will mem_copy - slice needs to be big enough
            current_edges: vec![(0, 0); m], // randomize will mem_copy - slice needs to be big enough
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

    fn shuffle_all_edges(&mut self) {
        self.all_edges
            .shuffle(&mut self.rng);
    }

}

impl<T, R> GraphIteratorsMut<T, Graph<T>, NodeContainer<T>> for ErEnsembleM<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
{
    fn contained_iter_neighbors_mut(&mut self, index: usize) ->
        NContainedIterMut<T, NodeContainer<T>, IterWrapper>
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

impl<T, R> WithGraph<T, Graph<T>> for ErEnsembleM<T, R>
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

    /// # Sort adjecency lists
    /// If you depend on the order of the adjecency lists, you can sort them
    /// # Performance
    /// * internally uses [pattern-defeating quicksort](https://github.com/orlp/pdqsort)
    /// as long as that is the standard
    /// * sorts an adjecency list with length `d` in worst-case: `O(d log(d))`
    /// * is called for each adjecency list, i.e., `self.vertex_count()` times
    fn sort_adj(&mut self) {
        self.graph_mut().sort_adj();
    }
}

impl<T, R> Dot for ErEnsembleM<T, R>
where T: Node
{
    fn dot_from_indices<F, W, S1, S2>(&self, writer: W, dot_options: S1, f: F)
        -> Result<(), std::io::Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        W: Write,
        F: FnMut(usize) -> S2 {
        self.graph
            .dot_from_indices(writer, dot_options, f)
    }

    fn dot<S, W>(&self, writer: W, dot_options: S) -> Result<(), std::io::Error>
    where
        S: AsRef<str>,
        W: Write {
        self.graph
            .dot(writer, dot_options)
    }

    fn dot_string<S>(&self, dot_options: S) -> String
    where
        S: AsRef<str> {
        self.graph.dot_string(dot_options)
    }

    fn dot_string_from_indices<F, S1, S2>(&self, dot_options: S1, f: F) -> String
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        F: FnMut(usize) -> S2 {
        self.graph
            .dot_string_from_indices(dot_options, f)
    }

    fn dot_string_with_indices<S>(&self, dot_options: S) -> String
    where
        S: AsRef<str> {
        self.graph
            .dot_string_with_indices(dot_options)
    }

    fn dot_with_indices<S, W>(
            &self, writer: W,
            dot_options: S
        ) -> Result<(), std::io::Error>
    where
        S: AsRef<str>,
        W: Write {
        self.graph
            .dot_with_indices(writer, dot_options)
    }
}

impl<T, R> Contained<T> for ErEnsembleM<T, R>
where T: Node
{
    fn get_contained(&self, index: usize) -> Option<&T> {
        self.graph.get_contained(index)
    }

    fn get_contained_mut(&mut self, index: usize) -> Option<&mut T> {
        self.graph.get_contained_mut(index)
    }

    unsafe fn get_contained_unchecked(&self, index: usize) -> &T {
        self.graph.get_contained_unchecked(index)
    }

    unsafe fn get_contained_unchecked_mut(&mut self, index: usize) -> &mut T {
        self.graph.get_contained_unchecked_mut(index)
    }
}