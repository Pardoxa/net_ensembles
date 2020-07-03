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
    random_edge_halfs: Vec<usize>,  // optimization to lessen the number of required allocations
    random_edge_halfs_backup: Vec<usize>, // optimization to lessen the number of required allocations
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
        assert!(
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
            rng,
            random_edge_halfs: Vec::new(),
            random_edge_halfs_backup: Vec::new(),
        };
        res.init_edge_halfs();
        res.randomize();
        res
    }

    /// Swaps the degeedistribution for a new one and draws a new network according to this distribution
    /// **Note** `new_degree_distribution.len()` has to be of the same length as `self.degree_distribution.len()`
    /// will panic otherwise
    /// * returns old degree distribution
    pub fn swap_distribution_vec(&mut self, mut new_degree_distribution: Vec<usize>) -> Vec<usize>
    {
        assert_eq!(self.degree_distribution.len(), new_degree_distribution.len(),
        "degree distributions need the same length"
        );
        assert!(
            new_degree_distribution.iter()
                .all(|&degree| degree < new_degree_distribution.len() - 1),
            "Impossible degree distribution - not enough vertices for at least on of the requested degrees"
        );
        mem::swap(&mut self.degree_distribution, &mut new_degree_distribution);
        self.init_edge_halfs();
        self.randomize();
        new_degree_distribution
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

    fn init_edge_halfs(&mut self)
    {
        self.random_edge_halfs_backup.clear();
        let ptr = self.degree_distribution.as_ptr();
        let len = self.degree_distribution.len();
        self.random_edge_halfs_backup.extend(
            (0..len)
            .flat_map(
                |i| 
                {
                    let times: usize = unsafe { *ptr.add(i)};
                    iter::repeat(i).take(times)

                }
            )
        );
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
        self.random_edge_halfs_backup.shuffle(&mut self.rng);
        self.random_edge_halfs.clear();
        self.random_edge_halfs.extend_from_slice(&self.random_edge_halfs_backup);

        while self.random_edge_halfs.len() > 0 {
            let added = self.add_multiple_random_edges();
            // if adding did not work, we have to try again!
            if !added {
                self.random_edge_halfs_backup.shuffle(&mut self.rng);
                self.random_edge_halfs.clear();
                self.random_edge_halfs.extend_from_slice(&self.random_edge_halfs_backup);
                self.graph.clear_edges();
            }
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum ConfigurationModelStep {
    Error,
    Added((usize, usize), (usize, usize)),

}

impl<T, R> MarkovChain<ConfigurationModelStep, ()> for ConfigurationModel<T, R>
    where   T: Node + SerdeStateConform,
            R: rand::Rng,
{

    /// # Markov step
    /// * use this to perform a markov step, e.g., to create a markov chain
    /// * result `ConfigurationModelStep` can be used to undo the step with `self.undo_step(result)`
    fn m_step(&mut self) -> ConfigurationModelStep {
        let mut vertex_list = Vec::with_capacity(2);
        // draw two vertices that are not connected
        while vertex_list.len() < 2 {
            vertex_list.extend(self.random_edge_halfs_backup.choose_multiple(&mut self.rng, 2));
            if vertex_list[0] == vertex_list[1]
            {
                vertex_list.clear();
            }
        }
        
        let edge_1: (usize, usize) = (vertex_list[0], *self.graph.vertices[vertex_list[0]].adj.choose(&mut self.rng).unwrap());
        let edge_2: (usize, usize) = (vertex_list[1], *self.graph.vertices[vertex_list[1]].adj.choose(&mut self.rng).unwrap());

        if edge_2.0 == edge_1.0 || edge_1.1 == edge_2.1 {
            return ConfigurationModelStep::Error;
        }

        // try to add new edges, return on error
        match self.graph.add_edge(edge_1.0, edge_2.0)
        {
            Err(..) => return ConfigurationModelStep::Error,
            _ => ()
        };
        match self.graph.add_edge(edge_1.1, edge_2.1){
            Err(..) => {
                self.graph.remove_edge(edge_1.0, edge_2.0).unwrap();
                return ConfigurationModelStep::Error
            },
            _ => ()
        };

        // remove old edges, panic on error
        self.graph.remove_edge(edge_1.0, edge_1.1).expect("Fatal error in removing edges");
        self.graph.remove_edge(edge_2.0, edge_2.1).expect("Fatal error in removing edges");
        

        return ConfigurationModelStep::Added(edge_1, edge_2)
        
    }

    /// # Undo a markcov step
    /// * adds removed edge, or removes added edge, or does nothing
    /// * if it returns an Err value, you probably used the function wrong
    /// ## Important:
    /// Restored graph is the same as before the random step **except** the order of nodes
    /// in the adjacency list might be shuffled!
    fn undo_step(&mut self, step: ConfigurationModelStep) -> () {
        let (edge1, edge2) = match step {
            ConfigurationModelStep::Error => return,
            ConfigurationModelStep::Added(edge1, edge2) => (edge1, edge2)
        };
        self.graph.add_edge(edge2.0, edge2.1).unwrap();
        self.graph.add_edge(edge1.0, edge1.1).unwrap();
        self.graph.remove_edge(edge1.1, edge2.1).unwrap();
        self.graph.remove_edge(edge1.0, edge2.0).unwrap();

    }

    fn undo_step_quiet(&mut self, step: ConfigurationModelStep) {
        self.undo_step(step)
    }

}

impl<T, R> ConfigurationModel<T, R>
where T: Node,
    R: rand::Rng,
{

    fn add_multiple_random_edges(&mut self) -> bool
    {
        let mut node1 = self.random_edge_halfs.pop().unwrap();
        let mut counter = self.random_edge_halfs.len() - 1;
        loop {
            if node1 == self.random_edge_halfs[counter]{
                counter = match counter.checked_sub(1){
                    Some(val) => val,
                    None => break false,
                };
                continue;
            }
            let node2 = self.random_edge_halfs.swap_remove(counter);
            if self.graph
                .add_edge(node1, node2)
                .is_err()
            {
                    return false;
            }
            if self.random_edge_halfs.len() > 1 {
                node1 = self.random_edge_halfs.pop().unwrap();
                counter = self.random_edge_halfs.len() - 1;
            }else{
                break true;
            }

        }

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