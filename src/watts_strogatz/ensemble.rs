use {
    super::OriginalEdge,
    crate::{
        AdjContainer,
        Node,
        GenericGraph,
        watts_strogatz::WSContainer,
        WithGraph,
        HasRng,
        SimpleSample,
        GraphIteratorsMut,
        GraphIterators,
        SerdeStateConform,
        iter::*,
        generic_graph::*,
        traits::*
    },
    rand::{
        Rng,
        distributions::{
            Uniform,
            Distribution
        }
    },
    std::{
        mem::swap,
        num::*,
        convert::AsRef
    }
};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

/// # Specific [GenericGraph] used for watts-strogatz small World graph
pub type WSGraph<T> = GenericGraph<T,WSContainer<T>>;

/// # Implements small-world graph ensemble
/// * Note the implemented traits!
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct SmallWorldWS<T, R>
{
    graph: WSGraph<T>,
    rewire_prob: f64,
    rng: R,
    neighbor_distance: NonZeroU32
}

/// # short for [SmallWorldWS]
/// See [SmallWorldWS] for the implemented traits
pub type WS<T, R> = SmallWorldWS<T, R>;

impl<T, R> WS<T, R>
{
    /// # Returns neigbor distance of the initial ring
    /// A neighbor distance of 1 means, in the original 
    /// ring structure every node is connected to its two nearest neigbors,
    /// a neighbor distance of 2 means, it is connected to its 4 nearest neigbors and so on
    ///
    /// The graph will contain neighbor_distance * system size (i.e. graph.vertex_count())
    /// connections
    pub fn neighbor_distance(&self) -> NonZeroU32
    {
        self.neighbor_distance
    }

    /// # retunrs rewire probability the ensemble is set to
    /// On average, a fraction of rewire_prob nodes should be rewired
    pub fn rewire_prob(&self) -> f64
    {
        self.rewire_prob
    }
}

/// # Error variants
/// Possible Errors which can be encountered during the initial creation of an instance 
/// of a [WS]
#[derive(Debug, Clone, Copy)]
pub enum WSCreationError{
    /// It is impossible to create the initial ring structure.
    /// You have to either increase the system size of the graph (i.e., the number of nodes)
    /// or reduce the neigbor distance!
    ImpossibleRingRequest,
    /// Something went wrong during the rewireing.
    /// This should not happen. If you encounter this error,
    /// please file a bug report on Github with a minimal example to reproduce the bug. 
    /// Thanks!
    ImpossibleEdgeRequest
}

impl<T, R> WS<T, R>
where T: Node,
    R: Rng
{
    /// # Initialize a [WS] - a small-world ensemble
    /// * `n` is the system size, i.e., how many nodes there should be in the created graphs
    /// * `neighbor_distance` is needed for the initial ring structure. See also [WS::neigbor_distance](`Self::neighbor_distance`)
    /// * `rewire_probability` - each edge will be rewired with a probability of `rewire_probability` - see also [WS::rewire_prob]
    /// * `rng` - random number generator
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
            return Err(WSCreationError::ImpossibleRingRequest);
        }
        let mut graph = WSGraph::new(n as usize);
        let res = graph.init_ring(neighbor_distance.get() as usize);
        if res.is_err() {
            return Err(WSCreationError::ImpossibleEdgeRequest);
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

impl<T, R> SmallWorldWS<T, R>
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

impl<T, R> WithGraph<T, WSGraph<T>> for SmallWorldWS<T, R>
{
    fn at(&self, index: usize) -> &T
    {
        self.graph.at(index)
    }

    fn at_mut(&mut self, index: usize) -> &mut T {
        self.graph.at_mut(index)
    }

    fn graph(&self) -> &WSGraph<T> {
        self.as_ref()
    }

    fn sort_adj(&mut self) {
        self.graph.sort_adj()
    }
}

impl<T, R> HasRng<R> for SmallWorldWS<T, R>
where R: Rng
{
    fn swap_rng(&mut self, rng: &mut R) {
        swap(&mut self.rng, rng);
    }

    fn rng(&mut self) -> &mut R {
        &mut self.rng
    }
}

impl<T, R> SimpleSample for SmallWorldWS<T, R>
where R: Rng
{


    fn randomize(&mut self) {
        self.graph.init_ring(self.neighbor_distance.get() as usize).unwrap();
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

impl<T, R> AsRef<WSGraph<T>> for SmallWorldWS<T, R>
{
    #[inline]
    fn as_ref(&self) -> &WSGraph<T>{
        &self.graph
    }
}

impl<T, R> GraphIteratorsMut<T, WSGraph<T>, WSContainer<T>> for SmallWorldWS<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
{
    fn contained_iter_neighbors_mut(&mut self, index: usize) ->
        NContainedIterMut<T, WSContainer<T>>
    {
        self.graph.contained_iter_neighbors_mut(index)
    }

    fn contained_iter_neighbors_mut_with_index(&mut self, index: usize)
        -> INContainedIterMut<'_, T, WSContainer<T>>
    {
        self.graph.contained_iter_neighbors_mut_with_index(index)
    }

    fn contained_iter_mut(&mut self) ->  ContainedIterMut<T, WSContainer<T>> {
        self.graph.contained_iter_mut()
    }
}

impl<T, R> GraphIterators<T, WSGraph<T>, WSContainer<T>> for SmallWorldWS<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng
{
    fn contained_iter(&self) -> ContainedIter<'_, T, WSContainer<T>> {
        self.graph().contained_iter()
    }

    fn contained_iter_neighbors(&self, index: usize) -> NContainedIter<'_, T, WSContainer<T>> {
        self.graph.contained_iter_neighbors(index)   
    }

    fn contained_iter_neighbors_with_index(&self, index: usize) -> NIContainedIter<T, WSContainer<T>> {
        self.graph.contained_iter_neighbors_with_index(index)
    }

    fn container_iter(&self) -> core::slice::Iter<'_, WSContainer<T>> {
        self.graph.container_iter()
    }

    fn container_iter_neighbors(&self, index: usize) -> NContainerIter<'_, T, WSContainer<T>> {
        self.graph.container_iter_neighbors(index)
    }

    fn dfs(&self, index: usize) -> Dfs<'_, T, WSContainer<T>> {
        self.graph.dfs(index)
    }

    fn dfs_with_index(&self, index: usize) -> DfsWithIndex<'_, T, WSContainer<T>> {
        self.graph.dfs_with_index(index)
    }

    fn bfs_index_depth(&self, index: usize) -> Bfs<'_, T, WSContainer<T>> {
        self.graph.bfs_index_depth(index)   
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_pcg::Pcg64;
    use rand::SeedableRng;
    use crate::Dot;
    use crate::EmptyNode;
    use crate::WithGraph;
    use crate::dot_constants::*;
    use std::fs::File;
    use std::io::BufWriter;

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

    #[test]
    fn plot()
    {
        let sys_size = 10;
        let n = 2;
        let rng = Pcg64::seed_from_u64(72389458937632);
        let mut ensemble = WS::<EmptyNode, _>::new(
            sys_size,
            unsafe{NonZeroU32::new_unchecked(n)},
            0.1,
            rng
        ).unwrap();

        let file = File::create("ws10.dot")
            .unwrap();

        for _ in 0..10 {
            ensemble.randomize();
        }
        
        let writer = BufWriter::new(file);

        ensemble.graph().dot_with_indices(
            writer,
            dot_options!(NO_OVERLAP, MARGIN_0)
        ).unwrap();

        assert!(ensemble.graph().is_connected().unwrap())
    }
}

impl<T, R> Contained<T> for SmallWorldWS<T, R>
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