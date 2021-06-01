use std::mem::swap;
use std::num::*;
use rand::Rng;
use rand::distributions::{Uniform, Distribution};
use std::convert::AsRef;
use crate::AdjContainer;
use crate::Node;
use crate::GenericGraph;
use crate::watts_strogatz::WSContainer;
use crate::WithGraph;
use crate::{HasRng, SimpleSample};

use super::OriginalEdge;

pub type WSGraph<T> = GenericGraph<T,WSContainer<T>>;

pub struct SmallWorldWS<T, R>
{
    graph: WSGraph<T>,
    rewire_prob: f64,
    rng: R,
    neighbor_distance: NonZeroU32
}

pub type WS<T, R> = SmallWorldWS<T, R>;

impl<T, R> WS<T, R>
{
    pub fn neighbor_distance(&self) -> NonZeroU32
    {
        self.neighbor_distance
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
        n: u32,
        neighbor_distance: NonZeroU32,
        rewire_probability: f64,
        rng: R
    ) -> Result<Self, WSCreationError>
    {
        let n = n;
        let minimum_n = 1 + 2 * neighbor_distance.get();
        if n < minimum_n {
            return Err(WSCreationError::ImpossibleRequest);
        }
        let mut graph = WSGraph::new(n as usize);
        let res = graph.init_ring(neighbor_distance.get() as usize);
        if res.is_err() {
            return Err(WSCreationError::ImpossibleRequest);
        }
        let mut s = 
            Self
            {
                neighbor_distance,
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

impl<T, R> WS<T, R>
where R: Rng
{
    // Adding a random edge, that does not exist right now and is not "originalEdge"
    fn add_random_edge(&mut self, mut original_edge: OriginalEdge)
    {
        let n = self.graph.vertex_count();
        let die = Uniform::from(0..n as u32);
        let mut first = die.sample(self.rng());
        let mut second = die.sample(self.rng());
        loop{
            // no self loops
            // not the same edge again!
            // no existing edge
            if first == second ||
                (first == original_edge.from && second == original_edge.to)
                || (first == original_edge.to && second == original_edge.from)
                || self.graph.get_mut_unchecked(first as usize)
                    .is_adjacent(second as usize)
            {
                first = die.sample(self.rng());
                second = die.sample(self.rng());
            } else {
                break;
            }

        }

        let mut create_edge = |from: u32, to: u32, mut edge: OriginalEdge| {
            edge.set_origin_false();
            let from_usize = from as usize;
            let to_usize = to as usize;
            let (vertex_from, vertex_to) = self.graph
                .get_2_mut(from_usize, to_usize);
            
            // create edge from --> to
            let (vec_to, vec_original) = vertex_from.edges_mut();
            vec_to.push(to_usize);
            vec_original.push(edge);
            // create edge to --> from
            let (vec_to, vec_original) = vertex_to.edges_mut();
            edge.swap_direction();
            vec_to.push(from_usize);
            vec_original.push(edge)
        };

        if first == original_edge.to || second == original_edge.from {
            original_edge.swap_direction();
            create_edge(second, first, original_edge)
        } else {
            create_edge(first, second, original_edge)
        }
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
        let n = self.graph.vertex_count();
        let mut rewire_vec = Vec::with_capacity(2 * self.neighbor_distance.get() as usize);

        for i in 0..n{
            let vertex = self.graph
                .get_mut_unchecked(i);
            
            let (i_to, i_original_edges) = vertex.edges_mut();
            let len = i_original_edges.len();
            debug_assert!(len == i_to.len());
            
            for j in (0..len).rev()
            {
                if i_original_edges[j].is_at_origin() 
                    && self.rng.gen::<f64>() <= self.rewire_prob
                {
                    // remove (one direction of) edge to be rewired
                    let edge = i_original_edges.swap_remove(j);
                    rewire_vec.push(edge);
                    i_to.swap_remove(j);
                }
            }

            // remove other direction of all edges that are to be rewired
            rewire_vec.iter()
                .for_each(
                    |edge|
                    {
                        let to = edge.to();
                        let vertex = self.graph.get_mut_unchecked(to);
                        let from = edge.from();
                        vertex.swap_remove_elem(from);
                    }
                );

            rewire_vec.iter()
                .for_each(|&edge| self.add_random_edge(edge));
            
            rewire_vec.clear();
        }
        
    }
}

impl<T, R> AsRef<WSGraph<T>> for WS<T, R>
where T: Node,
      R: rand::Rng
{
    #[inline]
    fn as_ref(&self) -> &WSGraph<T>{
        self.graph()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_pcg::Pcg64;
    use rand::SeedableRng;
    use crate::EmptyNode;
    use crate::WithGraph;

    #[test]
    fn creation()
    {
        let sys_size = 300;
        let mut rng_rng = Pcg64::seed_from_u64(925859072932);

        for n in 1..10 {
            let rng = Pcg64::from_rng(&mut rng_rng)
                .unwrap();
            let mut ensemble = WS::<EmptyNode, _>::new(
                sys_size,
                unsafe{NonZeroU32::new_unchecked(n)},
                0.1,
                rng
            ).unwrap();
            assert_eq!(
                ensemble.graph().average_degree(),
                (2 * n) as f32
            );
            for _ in 0..20 {
                ensemble.randomize();
                assert_eq!(
                    ensemble.graph().average_degree(),
                    (2 * n) as f32
                );
                for i in 0..sys_size {
                    let v = ensemble.graph.get_mut_unchecked(i as usize);
                    let (to, origin) = v.edges_mut();
                    assert_eq!(to.len(), origin.len());
                    to.iter()
                        .zip(origin.iter())
                        .for_each(
                            |(&to, edge)|
                            {
                                if edge.is_at_origin(){
                                    assert_eq!(
                                        edge.from,
                                        i
                                    );
                                    assert_eq!(
                                        edge.to,
                                        to as u32
                                    );
                                }else {
                                    println!("{:?}",edge);
                                    println!("i: {}, to: {}", i, to);
                                    assert!(
                                        edge.from != i || 
                                            edge.to != to as u32
                                    );
                                }
                            }
                        )
                }
            }

        }
        
    }
}