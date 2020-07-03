#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

use crate::{traits::*, graph::*, iter::*};
use std::{borrow::*, convert::*, iter, iter::*};
use rand::seq::*;
use std::mem;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// Generate networks with a given degree distribution
/// * the degree of each vertex is fixed (see self.degree_distribution),
///  while the actual edges will be drawn randomly
/// * No self loops allowed
pub struct ConfigurationModel<T, R>
where T: Node
{
    graph: Graph<T>,
    degree_distribution: Vec<usize>,
    rng: R,
}

impl<T, R> ConfigurationModel<T, R>
where T: Node,
{
    /// Get the degree distribution of the vertices
    pub fn degree_distribution(&self) -> &Vec<usize>
    {
        &self.degree_distribution
    }
}

impl<T, R> ConfigurationModel<T, R>
where T: Node,
    R: rand::Rng
{
    /// create configuration model from a degree distribution
    /// * **Note** sum of degree distribution has to be even, will panic otherwise
    /// * degree_distribution has to have a length grater 1
    /// * sum of degree_distribution has to be even - otherwise there would be a dangling edge half
    /// * drawn graphs will consist of `degree_distribution.len()` vertices, where 
    /// a vertex i will have degree `degree_distribution[i]`
    pub fn from_vec(degree_distribution: Vec<usize>, rng: R) -> Self
    {
        assert!(
            degree_distribution.len() > 1,
            "degree distribution has to have lenght grater than 1"
        );
        debug_assert!(
            degree_distribution.iter()
                .all(|&degree| degree < degree_distribution.len() - 1),
            "Impossible degree distribution - not enough vertices for at least on of the requested degrees"
        );
        let mut sum = 0;
        for val in degree_distribution.iter()
        {
            sum += val;
        }
        assert!(
            sum % 2 == 0, 
            "Sum of degree distribution has to be even, otherwise there would be a dangling edge half, which is invalid"
        );
        let graph = Graph::<T>::new(degree_distribution.len());
        let mut res = Self{
            graph,
            degree_distribution,
            rng
        };
        res.randomize();
        res
    }

    /// Swaps the degeedistribution for a new one and draws a new network according to this distribution
    /// **Note** `new_degree_distribution.len()` has to be of the same length as `self.degree_distribution.len()`
    /// will panic otherwise
    /// * returns old degree distribution
    pub fn swap_distribution_vec(&mut self, mut new_degree_distribution: Vec<usize>) -> Vec<usize>
    {
        assert_eq!(self.degree_distribution.len(), new_degree_distribution.len());
        mem::swap(&mut self.degree_distribution, &mut new_degree_distribution);
        self.randomize();
        new_degree_distribution
    }
}


impl<T, R> AsRef<Graph<T>> for ConfigurationModel<T, R>
where T: Node
{
    #[inline]
    fn as_ref(&self) -> &Graph<T>{
        &self.graph
    }
}

impl<T, R> Borrow<Graph<T>> for ConfigurationModel<T, R>
where T: Node
{
    #[inline]
    fn borrow(&self) -> &Graph<T> {
        &self.graph
    }
}


impl<T, R> HasRng<R> for ConfigurationModel<T, R>
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

impl<T, R> GraphIteratorsMut<T, Graph<T>, NodeContainer<T>> for ConfigurationModel<T, R>
where   T: Node
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


impl<T, R> WithGraph<T, Graph<T>> for ConfigurationModel<T, R>
where   T: Node,
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

impl<T, R> SimpleSample for ConfigurationModel<T, R>
where   T: Node,
        R: rand::Rng,
{
    /// # Randomizes the edges according to the configuration Model
    fn randomize(&mut self) {
        self.graph.clear_edges();
        let mut edge_halfs = Vec::from_iter(
            (0..self.degree_distribution.len())
                .flat_map(|i| iter::repeat(i).take(self.degree_distribution[i]))
        );
        edge_halfs.shuffle(&mut self.rng);
        let mut edge_halfs_clone = edge_halfs.clone();

        while edge_halfs.len() > 0 {
            let added = self.add_random_edge(&mut edge_halfs);
            // if adding did not work, we have to try again!
            if !added {
                edge_halfs_clone.shuffle(&mut self.rng);
                edge_halfs.clear();
                edge_halfs.extend_from_slice(&edge_halfs_clone);
                self.graph.clear_edges();
            }
        }
    }
}

impl<T, R> ConfigurationModel<T, R>
where T: Node,
    R: rand::Rng,
{
    fn add_random_edge(&mut self, edge_halfs: &mut Vec<usize>) -> bool
    {
        let node1 = edge_halfs.pop().unwrap();
        for i in (0..edge_halfs.len()).rev()
        {
            if node1 == edge_halfs[i]{
                continue;
            }
            let node2 = edge_halfs.remove(i);
            // shuffle if it did not removed last entry
            // to get correct statistics
            if i != edge_halfs.len(){
                edge_halfs.shuffle(&mut self.rng);
            }
            return self.graph
                .add_edge(node1, node2)
                .is_ok();
        }
        false
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use rand_pcg::Pcg64;
    use crate::*;
    use rand::SeedableRng;

    #[test]
    #[should_panic(expected = "Impossible degree distribution")]
    fn impossible_degree_distribution() {
        let rng = Pcg64::seed_from_u64(12);

        let degree_distribution = vec![1,2,3];
        let _e: ConfigurationModel<EmptyNode, _> = ConfigurationModel::from_vec(degree_distribution, rng);
    }

    #[test]
    fn valid_degree_distributions()
    {
        let mut rng = Pcg64::seed_from_u64(12322);
        let degree_distribution = vec![1,2,3,1,2,3];
        let ensemble: ConfigurationModel<EmptyNode, _> 
            = ConfigurationModel::from_vec(degree_distribution.clone(), Pcg64::from_rng(&mut rng).unwrap());
        
        for i in 0..ensemble.vertex_count()
        {
            assert_eq!(ensemble.graph().degree(i), Some(degree_distribution[i]));
        }

        let sw: SwEnsemble<EmptyNode, _> = SwEnsemble::new(1000, 0.1, Pcg64::from_rng(&mut rng).unwrap());
        let degree_distribution: Vec<_> = sw.container_iter().map(|c| c.degree()).collect();
        let ensemble: ConfigurationModel<EmptyNode, _> 
            = ConfigurationModel::from_vec(degree_distribution.clone(), Pcg64::from_rng(&mut rng).unwrap());

        for i in 0..ensemble.vertex_count()
        {
            assert_eq!(ensemble.graph().degree(i), Some(degree_distribution[i]));
        }

    }

}