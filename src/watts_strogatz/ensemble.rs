use std::mem::swap;
use std::num::NonZeroUsize;
use rand::Rng;

use crate::Node;
use crate::GenericGraph;
use crate::watts_strogatz::WSContainer;
use crate::WithGraph;
use crate::{HasRng, SimpleSample};

pub type WSGraph<T> = GenericGraph<T,WSContainer<T>>;

pub struct SmallWorldWS<T, R>
{
    graph: WSGraph<T>,
    rewire_prob: f64,
    rng: R,
    neigbor_distance: NonZeroUsize
}

pub type WS<T, R> = SmallWorldWS<T, R>;

impl<T, R> WS<T, R>
{
    pub fn neigbor_distance(&self) -> NonZeroUsize
    {
        self.neigbor_distance
    }

    pub fn rewire_prob(&self) -> f64
    {
        self.rewire_prob
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WSCreationError{
    ImpossibleRequest,
}

impl<T, R> WS<T, R>
where T: Node,
    R: Rng
{
    pub fn new(
        n: usize,
        neigbor_distance: NonZeroUsize,
        rewire_probability: f64,
        rng: R
    ) -> Result<Self, WSCreationError>
    {
        let minimum_n = 1 + 2 * neigbor_distance.get();
        if n < minimum_n {
            return Err(WSCreationError::ImpossibleRequest);
        }
        let mut graph = WSGraph::new(n);
        let res = graph.init_ring(neigbor_distance.get());
        if res.is_err() {
            return Err(WSCreationError::ImpossibleRequest);
        }
        let mut s = 
            Self
            {
                neigbor_distance,
                rng,
                rewire_prob: rewire_probability,
                graph
            };
        s.randomize();
        Ok(
            s
        )
    }
}

impl<T, R> WithGraph<T, WSGraph<T>> for WS<T, R>
{
    fn at(&self, index: usize) -> &T
    {
        self.graph.at(index)
    }

    fn at_mut(&mut self, index: usize) -> &mut T {
        self.graph.at_mut(index)
    }

    fn graph(&self) -> &WSGraph<T> {
        &self.graph
    }

    fn sort_adj(&mut self) {
        self.graph.sort_adj()
    }
}

impl<T, R> HasRng<R> for WS<T, R>
where R: Rng
{
    fn swap_rng(&mut self, rng: &mut R) {
        swap(&mut self.rng, rng);
    }

    fn rng(&mut self) -> &mut R {
        &mut self.rng
    }
}

impl<T, R> SimpleSample for WS<T, R>
where R: Rng
{
    fn randomize(&mut self) {
        unimplemented!()
    }
}