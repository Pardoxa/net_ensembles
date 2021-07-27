//! # Small-world ensemble
//! In this specific small-world ensemble each vertex has at least degree 2.
//! That means, this small-world ensemble will never exhibit leaves.
//!
//! I implemented the same model, as I used in my paper
//! > Yannick Feld and Alexander K. Hartmann,
//! > "Large-deviations of the basin stability of power grids"
//! > *Chaos*&nbsp;**29**:113113&nbsp;(2019), DOI: [10.1063/1.5121415](https://dx.doi.org/10.1063/1.5121415)
//!
//! where it is described in more detail.
//!
//! You can find a list of my publications on my [homepage](https://www.yfeld.de/#publications).
//! # Citations
//! > D. J. Watts and S. H. Strogatz, "Collective dynamics on 'small-world' networks,"
//!   Nature **393**, 440-442 (1998), DOI:&nbsp;[10.1038/30918](https://doi.org/10.1038/30918)
use crate::{traits::*, sw_graph::*, iter::*};
use std::{borrow::Borrow, io::Write};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

const ROOT_EDGES_PER_VERTEX: usize = 2;

/// # Returned by markov steps
/// * information about the performed step and possible errors
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum SwChangeState {
    /// ERROR adjecency list invalid?
    InvalidAdjecency,
    /// Can not add edge twice
    BlockedByExistingEdge,
    /// Nothing happend
    Nothing,
    /// old edge: (Rewire.0, Rewire.1), new edge (Rewire.0, Rewire.2)
    Rewire(usize, usize, usize),
    /// old edge: (Reset.0, Reset.1), new edge (Reset.0, Reset.2)
    Reset(usize, usize, usize),
    /// A GraphError occurred
    GError(GraphErrors),
}

impl SwChangeState {
    /// checks if self is `Nothing` variant
    pub fn is_nothing(&self) -> bool {
        if let SwChangeState::Nothing = self {
            true
        }else{
            false
        }
    }

    /// checks if self is `Nothing` or `BlockedByExistingEdge`
    pub fn is_nothing_or_blocked(&self) -> bool {
        match self {
            SwChangeState::Nothing |
            SwChangeState::BlockedByExistingEdge => true,
            _                                    => false
        }
    }

    /// result is equal to `!self.is_nothing_or_blocked()`
    pub fn not_nothing_or_blocked(&self) -> bool {
        match self {
            SwChangeState::Nothing |
            SwChangeState::BlockedByExistingEdge => false,
            _                                    => true
        }
    }

    /// # valid states:
    /// * `SwChangeState::Rewire(..)`
    /// * `SwChangeState::Reset(..)`
    /// * `SwChangeState::Nothing`
    /// * `SwChangeState::BlockedByExistingEdge`
    /// # invalid states:
    /// * `SwChangeState::InvalidAdjecency`
    /// * `SwChangeState::GError(..)`
    pub fn is_valid(&self) -> bool {
        match self {
            SwChangeState::Rewire(..) |
            SwChangeState::Reset(..) |
            SwChangeState::Nothing |
            SwChangeState::BlockedByExistingEdge => true,
            SwChangeState::InvalidAdjecency |
            SwChangeState::GError(..)            => false,
        }
    }
}

/// # Implements small-world graph ensemble
/// * for more details look at [documentation](index.html) of module `sw`
/// ## Sampling
/// * for markov steps look at [```MarkovChain``` trait](../sampling/traits/trait.MarkovChain.html)
/// * for simple sampling look at [```SimpleSample``` trait](./sampling/traits/trait.SimpleSample.html)
/// ## Other
/// * for topology functions look at [`GenericGraph`](../generic_graph/struct.GenericGraph.html)
/// * to access underlying topology or manipulate additional data look at [```WithGraph``` trait](../traits/trait.WithGraph.html)
/// * to use or swap the random number generator, look at [```HasRng``` trait](../traits/trait.HasRng.html)
/// # Minimal example
/// ```
/// use net_ensembles::{SwEnsemble, EmptyNode};
/// use net_ensembles::traits::*; // I recommend always using this
/// use rand_pcg::Pcg64; //or whatever you want to use as rng
/// use net_ensembles::rand::SeedableRng; // I use this to seed my rng, but you can use whatever
///
/// let rng = Pcg64::seed_from_u64(12);
///
/// // now create small-world ensemble with 200 nodes
/// // and a rewiring probability of 0.3 for each edge
/// let sw_ensemble = SwEnsemble::<EmptyNode, Pcg64>::new(200, 0.3, rng);
/// ```
/// # Simple sampling example
/// ```
/// use net_ensembles::{SwEnsemble, EmptyNode};
/// use net_ensembles::traits::*; // I recommend always using this
/// use rand_pcg::Pcg64; //or whatever you want to use as rng
/// use net_ensembles::rand::SeedableRng; // I use this to seed my rng, but you can use whatever
/// use std::fs::File;
/// use std::io::{BufWriter, Write};
///
/// let rng = Pcg64::seed_from_u64(122);
///
/// // now create small-world ensemble with 100 nodes
/// // and a rewiring probability of 0.3 for each edge
/// let mut sw_ensemble = SwEnsemble::<EmptyNode, Pcg64>::new(100, 0.3, rng);
///
/// // setup file for writing
/// let f = File::create("simple_sample_sw_example.dat")
///     .expect("Unable to create file");
/// let mut f = BufWriter::new(f);
/// f.write_all(b"#diameter bi_connect_max average_degree\n")
///     .unwrap();
///
/// // simple sample for 10 steps
/// sw_ensemble.simple_sample(10,
///     |ensemble|
///     {
///         let diameter = ensemble.graph()
///             .diameter()
///             .unwrap();
///
///         let bi_connect_max = ensemble.graph()
///             .clone()
///             .vertex_biconnected_components(false)[0];
///
///         let average_degree = ensemble.graph()
///             .average_degree();
///
///         write!(f, "{} {} {}\n", diameter, bi_connect_max, average_degree)
///             .unwrap();
///     }
/// );
///
/// // or just collect this into a vector to print or do whatever
/// let vec = sw_ensemble.simple_sample_vec(10,
///     |ensemble|
///     {
///         let diameter = ensemble.graph()
///             .diameter()
///             .unwrap();
///
///         let transitivity = ensemble.graph()
///             .transitivity();
///         (diameter, transitivity)
///     }
/// );
/// println!("{:?}", vec);
/// ```
/// # Save and load example
/// * only works if feature ```"serde_support"``` is enabled
/// * Note: ```"serde_support"``` is enabled by default
/// * I need the ```#[cfg(feature = "serde_support")]``` to ensure the example does compile if
///  you opt out of the default feature
/// * you can do not have to use ```serde_json```, look [here](https://docs.serde.rs/serde/) for more info
/// ```
/// use net_ensembles::traits::*; // I recommend always using this
/// use serde_json;
/// use rand_pcg::Pcg64;
/// use net_ensembles::{SwEnsemble, EmptyNode, rand::SeedableRng};
/// use std::fs::File;
///
/// let rng = Pcg64::seed_from_u64(95);
/// // create small-world ensemble
/// let sw_ensemble = SwEnsemble::<EmptyNode, Pcg64>::new(200, 0.3, rng);
///
/// #[cfg(feature = "serde_support")]
/// {
///     // storing the ensemble in a file:
///
///     let sw_file = File::create("store_SW.dat")
///           .expect("Unable to create file");
///
///     // or serde_json::to_writer(sw_file, &sw_ensemble);
///     serde_json::to_writer_pretty(sw_file, &sw_ensemble);
///
///     // loading ensemble from file:
///
///     let mut read = File::open("store_SW.dat")
///         .expect("Unable to open file");
///
///     let sw: SwEnsemble::<EmptyNode, Pcg64> = serde_json::from_reader(read).unwrap();
/// }
///
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct SwEnsemble<T: Node, R>
{
    graph: SwGraph<T>,
    r_prob: f64,
    rng: R,
}

impl <T, R> SwEnsemble<T, R>
    where   T: Node + SerdeStateConform,
            R: rand::Rng,
{
    /// # Initialize
    /// * create new SwEnsemble graph with `n` vertices
    /// * `r_prob` is probability of rewiring for each edge
    /// * `rng` is consumed and used as random number generator in the following
    /// * internally uses `SwGraph<T>::new(n)`
    pub fn new(n: usize, r_prob: f64, rng: R) -> Self {
        let mut graph = SwGraph::new(n);
        graph.init_ring_2();
        let mut result =
            SwEnsemble {
                graph,
                r_prob,
                rng,
            };
        result.randomize();
        result
    }

    /// # **Experimental!** Connect the connected components
    /// * resets edges, to connect the connected components
    /// * intended as starting point for a markov chain, if you require connected graphs
    /// * do **not** use this to independently (simple-) sample connected networks,
    ///   as this will skew the statistics 
    /// * **This is still experimental, this member might change the internal functionallity
    ///   resulting in different connected networks, without prior notice**
    /// * **This member might be removed in braking releases**
    pub fn make_connected(&mut self)
    {
        while !self.is_connected().unwrap_or(true){
            let (num, ids) = self.graph.connected_components_ids();
            let mut new_connection = vec![false; num];
            let vc_usize = self.vertex_count();

            for index in  0..vc_usize {
                let min = ids[index].min(ids[(index + 1) % vc_usize]);
                if !new_connection[min as usize] && ids[index] != ids[(index + 1) % vc_usize] {
                    // find out, where the desired edge is pointing to right now
                    new_connection[min as usize] = true;
                    let to = *self.graph.container_mut(index)
                        .iter_raw_edges()
                        .find(
                            |edge|
                            edge.originally_to().map(|c| c) == Some((index + 1) % vc_usize)
                        )
                        .unwrap()
                        .to();
                    self.graph.reset_edge(
                        index,
                        to
                    );
                }
            }
        }



    }

    /// draw number <= high but not index
    fn draw_remaining(&mut self, index: usize, high: usize) -> usize {
        let num = self.rng.gen_range(0..high);

        if num < index {
            num
        } else {
            num + 1
        }
    }

    /// edge `(index0, index1)` has to be rooted at `index0`
    fn randomize_edge(&mut self, index0: usize, index1: usize) -> SwChangeState {
        let vertex_count = self.graph.vertex_count();

        if self.rng.gen::<f64>() <= self.r_prob {
            let rewire_index = self.
            draw_remaining(index0, vertex_count - 1);
            self.graph.rewire_edge(index0, index1, rewire_index)
        }else {
            self.graph.reset_edge(index0, index1)
        }
    }

    /// sanity check performed in debug mode
    fn debug_error_check(state: SwChangeState) -> bool {
        match state {
            SwChangeState::GError(_)                => panic!("GError"),
            SwChangeState::InvalidAdjecency         => panic!("InvalidAdjecency"),
            _                                       => true
        }
    }

    /// * draws random edge `(i0, i1)`
    /// * edge rooted at `i0`
    /// * uniform probability
    /// * result dependent on order of adjecency lists
    /// * `mut` because it uses the `rng`
    pub fn draw_edge(&mut self) -> (usize, usize) {
        // each vertex has the same number of root nodes
        // the root nodes have an order -> adjecency lists
        let rng_num = self.rng.gen_range(0..self.graph.edge_count());
        let v_index = rng_num / ROOT_EDGES_PER_VERTEX;
        let e_index = rng_num % ROOT_EDGES_PER_VERTEX;

        let mut iter = self.graph
            .container(v_index)
            .iter_raw_edges()
            .filter(|x| x.is_root());

        let &to = iter
            .nth(e_index)
            .unwrap()
            .to();

        (v_index, to)
    }

    /// * returns rewiring probability
    pub fn r_prob(&self) -> f64 {
        self.r_prob
    }

    /// * set new value for rewiring probability
    /// ## Note
    /// * will only set the value, which will be used from now on
    /// * if you also want to create a new sample, call `randomize` afterwards
    pub fn set_r_prob(&mut self, r_prob: f64) {
        self.r_prob = r_prob;
    }

    /// * retuns `GenericGraph::contained_iter_neighbors_mut`
    /// * otherwise you would not have access to this function, since no mut access to
    ///   the graph is allowed
    pub fn contained_iter_neighbors_mut(&mut self, index: usize) -> NContainedIterMut<T, SwContainer<T>>
    {
            self.graph.contained_iter_neighbors_mut(index)
    }
}

impl<T, R> GraphIteratorsMut<T, SwGraph<T>, SwContainer<T>> for SwEnsemble<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
{
    fn contained_iter_neighbors_mut(&mut self, index: usize) ->
        NContainedIterMut<T, SwContainer<T>>
    {
        self.graph.contained_iter_neighbors_mut(index)
    }

    fn contained_iter_neighbors_mut_with_index(&mut self, index: usize)
        -> INContainedIterMut<'_, T, SwContainer<T>>
    {
        self.graph.contained_iter_neighbors_mut_with_index(index)
    }

    fn contained_iter_mut(&mut self) ->  ContainedIterMut<T, SwContainer<T>> {
        self.graph.contained_iter_mut()
    }
}


impl<T, R> WithGraph<T, SwGraph<T>> for SwEnsemble<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
{
    fn at(&self, index: usize) -> &T {
        self.graph.at(index)
    }

    fn at_mut(&mut self, index: usize) -> &mut T{
        self.graph.at_mut(index)
    }

    fn graph(&self) -> &SwGraph<T> {
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
        self.graph.sort_adj();
    }
}


impl<T, R> AsRef<SwGraph<T>> for SwEnsemble<T, R>
where T: Node,
      R: rand::Rng
{
    #[inline]
    fn as_ref(&self) -> &SwGraph<T>{
        &self.graph
    }
}

impl<T, R> Borrow<SwGraph<T>> for SwEnsemble<T, R>
where T: Node,
      R: rand::Rng
{
    #[inline]
    fn borrow(&self) -> &SwGraph<T> {
        &self.graph
    }
}

impl<T, R> HasRng<R> for SwEnsemble<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
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

impl<T, R> SimpleSample for SwEnsemble<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
{
    /// # Randomizes the edges according to small-world model
    /// * this is used by `SwEnsemble::new` to create the initial topology
    /// * you can use this for sampling the ensemble
    /// * runs in `O(vertices)`
    fn randomize(&mut self){
        let count = self.graph
            .vertex_count();

        for i in 0..count {
            let vertex = self.graph
                .get_mut_unchecked(i);

            let mut root_iter = vertex
                .iter_raw_edges()
                .filter(|edge| edge.is_root())
                .map(|edge| edge.to());

            debug_assert_eq!(ROOT_EDGES_PER_VERTEX, 2);

            let first   = *root_iter.next().unwrap();
            let second  = *root_iter.next().unwrap();
            debug_assert!(root_iter.next().is_none());

            let state = self.randomize_edge(i, first);
            debug_assert!(Self::debug_error_check(state));

            let state = self.randomize_edge(i, second);
            debug_assert!(Self::debug_error_check(state));

        }
    }
}

impl<T, R> MarkovChain<SwChangeState, SwChangeState> for SwEnsemble<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
        {

    /// # Markov step
    /// * use this to perform a markov step
    /// * keep in mind, that it is not unlikely for a step to do `Nothing` as it works by
    /// drawing an edge and then reseting it with `r_prob`, else the edge is rewired
    /// * result `SwChangeState` can be used to undo the step with `self.undo_step(result)`
    /// * result should never be `InvalidAdjecency` or `GError` if used on a valid graph
    fn m_step(&mut self) -> SwChangeState {
        let edge = self.draw_edge();
        self.randomize_edge(edge.0, edge.1)
    }

    /// # Undo a markov step
    /// * *rewires* edge to old state
    /// * Note: cannot undo `InvalidAdjecency` or `GError`,
    /// will just return `InvalidAdjecency` or `GError` respectively
    /// * returns result of *rewire*
    /// ## Important:
    /// Restored graph is the same as before the random step **except** the order of nodes
    /// in the adjacency list might be shuffled!
    fn undo_step(&mut self, step: &SwChangeState) -> SwChangeState {
        match step {
            SwChangeState::Rewire(root, old_to, new_to) |
            SwChangeState::Reset (root, old_to, new_to)  => self.graph.rewire_edge(*root, *new_to, *old_to), // swap old to and new to in rewire
            SwChangeState::Nothing |
            SwChangeState::BlockedByExistingEdge |
            SwChangeState::InvalidAdjecency |
            SwChangeState::GError(_)                     => *step
        }
    }

    /// # Undo a Monte Carlo step
    /// * *rewires* edge to old state
    /// * **panics** if you try to undo `InvalidAdjecency` or `GError`
    /// * **panics** if rewire result (`SwChangeState`) is invalid (i.e. `!result.is_valid()`)
    /// ## Important:
    /// Restored graph is the same as before the random step **except** the order of nodes
    /// in the adjacency list might be shuffled!
    fn undo_step_quiet(&mut self, step: &SwChangeState) {
        match step {
            SwChangeState::Rewire(root, old_to, new_to) |
            SwChangeState::Reset (root, old_to, new_to)  => {
                // swap old to and new to in rewire to undo step
                let state = self.graph.rewire_edge(*root, *new_to, *old_to);
                if !state.is_valid() {
                    panic!("undo step - rewire error: {:?}", state);
                }
            },
            SwChangeState::Nothing |
            SwChangeState::BlockedByExistingEdge => (),
            SwChangeState::InvalidAdjecency      => panic!("undo_step - {:?} - corrupt step?", step),
            SwChangeState::GError(error)         => panic!("undo_step - GError {} - corrupt step?", error)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::EmptyNode;
    use rand_pcg::Pcg64;
    use rand::SeedableRng;

    #[test]
    #[ignore]
    fn sw_make_connected() {
        let rng = Pcg64::seed_from_u64(7526);
        let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(20, 0.0, rng);
        e.set_r_prob(0.2);

        // the following code creates 3 connected components
        while e.is_connected().unwrap(){
            e.m_steps_quiet(10);
        }

        let mut comp = e.connected_components();
        let mut steps = Vec::new();
        while comp.len() < 3{
            e.m_steps(10, &mut steps);
            comp = e.connected_components();

            if  comp.len() < 2{
                e.undo_steps_quiet(&steps);
            }

        }

        // now I reconnect them
        e.make_connected();
        assert!(e.is_connected().unwrap());
    }


}


impl<T, R> Dot for SwEnsemble<T, R>
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

impl<T, R> Contained<T> for SwEnsemble<T, R>
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