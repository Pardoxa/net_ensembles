//! # Small-world ensemble
//! In this specific small-world ensemble each vertex has at least degree 2.
//! That means, this small-world ensemble will never exhibit leaves.
//!
//! I implemented the same model, as I used in my paper
//! > Yannick Feld and Alexander K. Hartmann,
//! > "Large-deviations of the basin stability of power grids"
//! > *Chaos* **29**:113113 (2019), DOI: [10.1063/1.5121415](https://dx.doi.org/10.1063/1.5121415)
//!
//! where it is described in more detail.
//!
//! You can find a list of my publications on my [homepage](https://www.yfeld.de/#publications).
use crate::SwGraph;
use crate::traits::*;
use crate::SwChangeState;

/// # Implements small-world graph ensemble
/// * for more details look at [documentation](index.html) of module `sw`
#[derive(Debug, Clone)]
pub struct SwEnsemble<T: Node, R: rand::Rng> {
    graph: SwGraph<T>,
    r_prob: f64,
    rng: R,
}

impl <T, R> SwEnsemble<T, R>
    where   T: Node,
            R: rand::Rng,
{
    /// # Initialize
    /// * create new SwEnsemble graph with `n` vertices
    /// * `r_prob` is probability of rewiring for each edge
    /// * `rng` is consumed and used as random number generator in the following
    /// * internally uses `SwGraph<T>::new(n)`
    pub fn new(n: u32, r_prob: f64, rng: R) -> Self {
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

    fn draw_remaining(&mut self, index: u32, high: u32) -> u32 {
        let num = self.rng.gen_range(0, high);

        if num < index {
            num
        } else {
            num + 1
        }
    }

    fn randomize_edge(&mut self, index0: u32, index1: u32) -> SwChangeState {
        let vertex_count = self.graph.vertex_count();

        if self.rng.gen::<f64>() <= self.r_prob {
            let rewire_index = self.
            draw_remaining(index0, vertex_count - 1);
            self.graph.rewire_edge(index0, index1, rewire_index)
        }else {
            self.graph.reset_edge(index0, index1)
        }
    }

    fn debug_error_check(state: SwChangeState) -> bool {
        match state {
            SwChangeState::GError(_)                => panic!("GError"),
            SwChangeState::InvalidAdjecency         => panic!("InvalidAdjecency"),
            _                                       => true
        }
    }

    /// # Randomizes the edges according to small-world model
    /// * this is used by `SwEnsemble::new` to create the initial topology
    /// * you can use this for sampling the ensemble
    /// * runs in `O(vertices)`
    pub fn randomize(&mut self){
        let count = self.graph
            .vertex_count();

        for i in 0..count {
            let vertex = self.graph
                .get_mut_unchecked(i as usize);

            let mut root_iter = vertex
                .iter_raw_edges()
                .filter(|edge| edge.is_root())
                .map(|edge| edge.to());

            let first   = *root_iter.next().unwrap();
            let second  = *root_iter.next().unwrap();
            debug_assert!(root_iter.next().is_none());

            let state = self.randomize_edge(i, first);
            debug_assert!(Self::debug_error_check(state));

            let state = self.randomize_edge(i, second);
            debug_assert!(Self::debug_error_check(state));

        }
    }

    /// # Sort adjecency lists
    /// If you depend on the order of the adjecency lists, you can sort them
    /// # Performance
    /// * internally uses [pattern-defeating quicksort](https://github.com/orlp/pdqsort)
    /// as long as that is the standard
    /// * sorts an adjecency list with length `d` in worst-case: `O(d log(d))`
    /// * is called for each adjecency list, i.e., `self.vertex_count()` times
    pub fn sort_adj(&mut self) {
        self.graph.sort_adj();
    }

    /// * returns reference to the underlying topology aka, the `SwGraph<T>`
    /// * use this to call functions regarding the topology
    pub fn graph(&self) -> &SwGraph<T> {
        &self.graph
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


    /// # Access RNG
    /// If, for some reason, you want access to the internal random number generator: Here you go
    pub fn rng(&mut self) -> &mut R {
        &mut self.rng
    }
}
