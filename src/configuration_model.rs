//! Contains Configuration Model 
//!
#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

use{
    crate::{
        traits::*,
        graph::*,
        iter::*,
        GenericGraph
    },
    std::{
        borrow::*,
        convert::*,
        iter,
        iter::*,
        mem
    },
    rand::seq::*
};


#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// Generate networks with a given degree distribution
/// * the degree of each vertex is fixed (see self.degree_vec),
///  while the actual edges will be drawn randomly
/// * No self loops allowed
pub struct ConfigurationModel<T, R>
where T: Node
{
    graph: Graph<T>,
    degree_vec: Vec<usize>,
    rng: R,
    random_edge_halfs: Vec<usize>,  // optimization to lessen the number of required allocations
    random_edge_halfs_backup: Vec<usize>, // optimization to lessen the number of required allocations
}

impl<T, R> ConfigurationModel<T, R>
where T: Node,
{
    /// Get reference to the degree vector of the vertices,
    /// faster than `self.graph().degree_vec()`,
    /// since the former has to construct the vector, while the latter just 
    /// returns a reference to an existing vector
    pub fn degree_vec(&self) -> &Vec<usize>
    {
        &self.degree_vec
    }
}

impl<T, R> ConfigurationModel<T, R>
where T: Node,
    R: rand::Rng
{
    /// # create configuration model from a constant degree
    /// * drawn graphs will consist of `degree_vec.len()` vertices, where 
    /// a vertex *i* will have degree `degree_vec[i]`
    /// * `size`: number of nodes in the resulting graphs
    /// * returns `None` if resulting degree vector is invalid
    pub fn from_const(constant: usize, size: usize, rng: R) -> Option<Self>
    {
        if constant >= size - 1 || size * constant % 2 != 0 {
            None
        } else {
            Some(
                Self::from_vec_unchecked(
                    vec![constant; size]
                    , rng
                )
            )
        }
    }

    /// # create ConfigurationModel from a generic graph
    /// * same as [`from_vec_unchecked`](#method.from_vec_unchecked), but creates degree vector from a generic graph
    pub fn from_generic_graph<T1, A1>(generic_graph: &GenericGraph<T1, A1>, rng: R) -> Self
    where T1: Node,
        A1: AdjContainer<T1>
    {
        Self::from_vec_unchecked(generic_graph.degree_vec(), rng)
    }

    /// # create ConfigurationModel from a generic graph and clones underlying Data
    /// * `degree_vec` will be extracted from `generic_graph`
    /// * Data will be cloned from the `generic_graph`
    /// * returns `ConfigurationModel` model
    /// * `model.graph()` will have the same topology as `generic_graph`
    /// after this creation. This will of cause change, if you call `randomize`
    /// or do markov steps
    pub fn clone_from_generic_graph<A>(generic_graph: &GenericGraph<T, A>, rng: R) -> Self
    where T: Clone,
        A: AdjContainer<T>
    {
        let graph = Graph::from(generic_graph);
        let mut res = Self{
            graph,
            degree_vec: generic_graph.degree_vec(),
            rng,
            random_edge_halfs: Vec::new(),
            random_edge_halfs_backup: Vec::new(),
        };
        res.init_edge_halfs();
        res
    }

    /// # create configuration model from a degree vector
    /// * drawn graphs will consist of `degree_vec.len()` vertices, where 
    /// a vertex *i* will have degree `degree_vec[i]`
    /// * returns `None` if degree vector is invalid
    pub fn from_vec(degree_vec: Vec<usize>, rng: R) -> Option<Self>
    {
        if Self::degree_vec_is_valid(&degree_vec){
            Some(Self::from_vec_unchecked(degree_vec, rng))
        } else {
            None
        }
       
    }

    /// # create configuration model from a degree vector
    /// * same as [`from_vec`](#method.from_vec), but it does not check if the 
    /// `degree_vec` is valid - that is on you now
    pub fn from_vec_unchecked(degree_vec: Vec<usize>, rng: R) -> Self
    {
        let graph = Graph::<T>::new(degree_vec.len());
        let mut res = Self{
            graph,
            degree_vec,
            rng,
            random_edge_halfs: Vec::new(),
            random_edge_halfs_backup: Vec::new(),
        };
        res.init_edge_halfs();
        res.randomize();
        res
    }

    /// # check if a vector (slice) is a vaild degree distribution
    /// * sum of `degree_vec` needs to be even
    /// * `degree_vec.len()` has to be greater than `1`
    /// * no entry can request a degree larger than `degree_vec.len()-2`
    pub fn degree_vec_is_valid(degree_vec: &[usize]) -> bool
    {
        if degree_vec.len() <= 1 {
            return false;
        }
        if !degree_vec.iter()
            .all(|&degree| degree < degree_vec.len() - 1)
        {
            return false;
        }
        let mut sum = 0;
        for val in degree_vec.iter().copied()
        {
            sum += val;
        }
        sum % 2 == 0
    }

    /// # asserts, that a vector is a vaild degree distribution
    /// * similar to [`degree_vec_is_valid`](#method.degree_vec_is_valid), but asserts instead
    /// * intended for debugging: see why the `degree_vec` is invalid
    pub fn assert_degree_vec_valid(degree_vec: &[usize]){
        assert!(
            degree_vec.len() > 1,
            "degree vec has to have lenght grater than 1"
        );
        assert!(
            degree_vec.iter()
                .all(|&degree| degree < degree_vec.len() - 1),
            "Impossible degree vec - not enough vertices for at least on of the requested degrees"
        );
        let mut sum = 0;
        for val in degree_vec.iter().copied()
        {
            sum += val;
        }
        assert!(
            sum % 2 == 0, 
            "Sum of degree vec has to be even, otherwise there would be a dangling edge half, which is invalid"
        );
    }

    /// # Swaps the degree vector for a new one and draws a new network accordingly
    /// **Note** `new_degree_vec.len()` has to be of the same length as `self.degree_vec.len()`
    /// will **panic** otherwise
    /// * **panics** if new_degree_vec is invalid
    /// * returns previous `degree_vec`
    pub fn swap_degree_vec(&mut self, new_degree_vec: Vec<usize>) -> Vec<usize>
    {
        Self::assert_degree_vec_valid(&new_degree_vec);
        self.swap_degree_vec_unchecked(new_degree_vec)
    }

    /// # Swaps the degree_vec for a new one and draws a new network accordingly
    /// * same as [`swap_degree_vec`](#method.swap_degree_vec) but does not assert, that the degree vector is valid 
    /// * **panics** if `self.degree_vec.len() != new_degree_vec.len()` (the only thing still checked for, as that check is really really cheap)
    pub fn swap_degree_vec_unchecked(&mut self, mut new_degree_vec: Vec<usize>) -> Vec<usize>
    {
        assert_eq!(self.degree_vec.len(), new_degree_vec.len(),
        "degree_vecs need the same length"
        );
        mem::swap(&mut self.degree_vec, &mut new_degree_vec);
        self.init_edge_halfs();
        self.randomize();
        new_degree_vec
    }

    /// # Use the degree vector of a generic graph
    /// * asserts, that `generic_graph` and `self.graph()` have the same number of vertices
    /// * similar to `self.swap_degree_vec_unchecked(generic_graph.degree_vec())`
    /// but does not create new vector and as such does not return old degree vector. 
    /// If you need the old degree vector, you can use `self.degree_vec().clone()` before calling this method
    pub fn degree_vec_from_generic_graph<T1, A1>(&mut self, generic_graph: &GenericGraph<T1, A1>)
    where T1: Node,
        A1: AdjContainer<T1>
    {
        assert_eq!(self.degree_vec.len(), generic_graph.vertices.len());
        for i in 0..self.degree_vec.len(){
            self.degree_vec[i] = generic_graph.vertices[i].degree();
        }
        self.init_edge_halfs();
        self.randomize();
    }

    /// initialize or update the edge halfs vectors for later usage
    fn init_edge_halfs(&mut self)
    {
        self.random_edge_halfs_backup.clear();
        let ptr = self.degree_vec.as_ptr();
        let len = self.degree_vec.len();
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
    fn rng(&mut self) -> &mut R {
        &mut self.rng
    }

    fn swap_rng(&mut self, rng: &mut R) {
        std::mem::swap(&mut self.rng, rng);
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

        while !self.random_edge_halfs.is_empty() {
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

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// Markov step of configuration model
pub enum ConfigurationModelStep {
    /// step did not succeed
    Error,
    /// * Step did succeed and can be undone using this
    /// * contains the newly added edges, which is enough information to know which edges were removed
    Added((usize, usize), (usize, usize)),

}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
/// Result of undoing a step via Markov Chain method of ConfigurationModel
pub enum UndoStepErrorCM {
    /// Could not add edge (usize, usize), because of GraphError
    UnableToAddEdge((usize, usize), GraphErrors),
    /// Could not remove edge (usize, usize), because of GraphError
    UnableToRemoveEdge((usize, usize), GraphErrors)

}

impl<T, R> MarkovChain<ConfigurationModelStep, Result<(), UndoStepErrorCM>> for ConfigurationModel<T, R>
    where   T: Node + SerdeStateConform,
            R: rand::Rng,
{

    /// # Markov step
    /// * use this to perform a markov step, e.g., to create a markov chain
    /// * result `ConfigurationModelStep` can be used to undo the step with `self.undo_step(result)`
    /// # How it works
    /// * it draws two distinct vertices, weighted with the Vertex degree
    /// * then for each vertex a random edge is drawn from the respective adjacency list.
    ///  let these edges be *edge1 = (n, j)* and *edge2 = (k, l)*. These edges are removed and 
    /// the edges *(n, k)* and *(j, l)* are added. 
    /// * If the above would result in an invalid topology,
    /// nothing is added or removed and `ConfigurationModelStep::Error` is returned
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
        if let Err(..) = self.graph.add_edge(edge_1.0, edge_2.0){
            return ConfigurationModelStep::Error
        }

        if let Err(..) = self.graph.add_edge(edge_1.1, edge_2.1)
        {
            self.graph.remove_edge(edge_1.0, edge_2.0).unwrap();
            return ConfigurationModelStep::Error
        }

        // remove old edges, panic on error
        self.graph.remove_edge(edge_1.0, edge_1.1).expect("Fatal error in removing edges");
        self.graph.remove_edge(edge_2.0, edge_2.1).expect("Fatal error in removing edges");
        

        ConfigurationModelStep::Added(edge_1, edge_2)
        
    }

    /// # Undo a markcov step
    /// * adds removed edge and removes added edge, or does nothing
    /// ## Important:
    /// Restored graph is the same as before the random step **except** the order of nodes
    /// in the adjacency list might be shuffled!
    /// # Error
    /// If an error is encountered, this will revert the graph to the state, before trying to undo
    /// the step. The returned Error gives a hint for why this did not succeed.
    fn undo_step(&mut self, step: &ConfigurationModelStep) -> Result<(), UndoStepErrorCM> {
        let (edge1, edge2) = match step {
            ConfigurationModelStep::Error => return Ok(()),
            ConfigurationModelStep::Added(edge1, edge2) => (edge1, edge2)
        };
        
        match self.graph.add_edge(edge2.0, edge2.1) {
            Err(error) => Err(UndoStepErrorCM::UnableToAddEdge(*edge2, error)),
            Ok(..) => {
                match self.graph.add_edge(edge1.0, edge1.1) {
                    Err(error) => {
                        self.graph.remove_edge(edge2.0, edge2.1).unwrap();
                        Err(UndoStepErrorCM::UnableToAddEdge(*edge1, error))
                    },
                    Ok(..) => {
                        match self.graph.remove_edge(edge1.1, edge2.1){
                            Err(error) => {
                                self.graph.remove_edge(edge2.0, edge2.1).unwrap();
                                self.graph.remove_edge(edge1.0, edge1.1).unwrap();
                                Err(UndoStepErrorCM::UnableToRemoveEdge((edge1.1, edge2.1), error))
                            },
                            Ok(..) =>{
                                match self.graph.remove_edge(edge1.0, edge2.0)
                                {
                                    Err(error) => {
                                        self.graph.remove_edge(edge2.0, edge2.1).unwrap();
                                        self.graph.remove_edge(edge1.0, edge1.1).unwrap();
                                        self.graph.add_edge(edge1.1, edge2.1).unwrap();
                                        Err(UndoStepErrorCM::UnableToRemoveEdge((edge1.0, edge2.0), error))
                                    },
                                    Ok(..) => Ok(())
                                }
                            }
                        }
                    }
                }
            }
        }

    }

    /// # Undo a markcov step
    /// * adds removed edge and removes added edge, or does nothing
    /// * as long as you know, that you undo the steps in the correct order,
    ///  this is the prefered method as this more efficent
    /// * **panics** if an Error is encountered
    fn undo_step_quiet(&mut self, step: &ConfigurationModelStep) {
        let (edge1, edge2) = match step {
            ConfigurationModelStep::Error => return,
            ConfigurationModelStep::Added(edge1, edge2) => (edge1, edge2)
        };
        self.graph.add_edge(edge2.0, edge2.1).unwrap();
        self.graph.add_edge(edge1.0, edge1.1).unwrap();
        self.graph.remove_edge(edge1.1, edge2.1).unwrap();
        self.graph.remove_edge(edge1.0, edge2.0).unwrap();
    }

}

impl<T, R> Contained<T> for ConfigurationModel<T, R>
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

#[cfg(test)]
mod testing {
    use super::*;
    use rand_pcg::Pcg64;
    use crate::*;
    use rand::SeedableRng;

    #[test]
    fn impossible_degree_distribution() {
        let rng = Pcg64::seed_from_u64(12);

        let degree_vec = vec![1,2,3];
        
        assert!(ConfigurationModel::<EmptyNode, _>::from_vec(degree_vec, rng).is_none());
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn assert_degree_vec() {
        let mut rng = Pcg64::seed_from_u64(12);
        let mut sw: SwEnsemble<EmptyNode, _> = SwEnsemble::new(1000, 0.1, Pcg64::from_rng(&mut rng).unwrap());

        
        let mut ensemble: ConfigurationModel<EmptyNode, _> 
                = ConfigurationModel::from_generic_graph(sw.graph(), Pcg64::from_rng(&mut rng).unwrap());
        for i in 0..5 {
            if i != 0 {
                sw.randomize();
                ensemble.degree_vec_from_generic_graph(sw.graph());
            }
            for j in 0..5 {
                if j != 0 {
                    ensemble.randomize();
                }
                assert_eq!(&ensemble.graph().degree_vec(), ensemble.degree_vec());
            }
        
        }
        
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn degree_distribution_is_valids()
    {
        let mut rng = Pcg64::seed_from_u64(12322);
        let degree_vec = vec![1,2,3,1,2,3];
        let ensemble: ConfigurationModel<EmptyNode, _> 
            = ConfigurationModel::from_vec(degree_vec.clone(), Pcg64::from_rng(&mut rng).unwrap()).unwrap();
        
        for i in 0..ensemble.vertex_count()
        {
            assert_eq!(ensemble.graph().degree(i), Some(degree_vec[i]));
        }

        let sw: SwEnsemble<EmptyNode, _> = SwEnsemble::new(1000, 0.1, Pcg64::from_rng(&mut rng).unwrap());
        let degree_vec: Vec<_> = sw.container_iter().map(|c| c.degree()).collect();
        let ensemble: ConfigurationModel<EmptyNode, _> 
            = ConfigurationModel::from_vec(degree_vec.clone(), Pcg64::from_rng(&mut rng).unwrap()).unwrap();

        for i in 0..ensemble.vertex_count()
        {
            assert_eq!(ensemble.graph().degree(i), Some(degree_vec[i]));
        }

    }

}