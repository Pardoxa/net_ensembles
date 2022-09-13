use{
    crate::{
        AdjContainer,
        GraphErrors,
        IterWrapper
    },
    std::mem::swap,
    permutation
};

use rand::seq::SliceRandom;
#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};


#[derive(Debug,Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub(crate) struct OriginalEdge{
    pub from: u32,
    pub to: u32,
    pub is_at_origin: bool
}

impl OriginalEdge{
    pub fn is_at_origin(&self) -> bool 
    {
        self.is_at_origin
    }

    pub fn to(&self) -> usize
    {
        self.to as usize
    }

    pub fn from(&self) -> usize
    {
        self.from as usize
    }

    pub fn swap_direction(&mut self)
    {
        swap(&mut self.from, &mut self.to)
    }

    pub fn set_origin_false(&mut self)
    {
        self.is_at_origin = false;
    }
}

/// # Used for accessing neighbor information from a graph
/// * Contains Adjacency list and internal id (normally the index in the graph)
/// * also contains user specified data, i.e., `T` 
/// * see trait [AdjContainer]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct WSContainer<T>
{
    id: usize,
    node: T,
    to: Vec<usize>,
    original: Vec<OriginalEdge>
}

impl<T> WSContainer<T>{
    fn get_index(&self, elem: usize) -> usize
    {
        self.to.iter()
            .position(|&e| e == elem)
            .expect("Fatal error in get_index")
    }

    pub(crate) fn swap_remove_elem(&mut self, elem: usize)
    {
        let index = self.get_index(elem);
        self.to.swap_remove(index);
        self.original.swap_remove(index);
    }

    //pub(crate) fn iter_original_edges(&self) -> impl Iterator<Item=&OriginalEdge>
    //{
    //    self.original
    //        .iter()
    //        .filter(|e| e.is_at_origin)
    //}

    pub(crate) fn edges_mut(&mut self) -> (&mut Vec<usize>, &mut Vec<OriginalEdge>)
    {
        (&mut self.to, &mut self.original)
    }


}

impl<T> AdjContainer<T> for WSContainer<T>
{
    fn new(id: usize, node: T) -> Self {
        Self{
            id,
            node,
            to: Vec::new(),
            original: Vec::new()
        }
    }

    fn contained(&self) -> &T {
        &self.node
    }

    fn contained_mut(&mut self) -> &mut T {
        &mut self.node
    }

    fn neighbors(&self) -> IterWrapper {
        IterWrapper::GenericIter(self.to.iter())
    }

    fn degree(&self) -> usize {
        self.to.len()
    }

    fn id(&self) -> usize {
        self.id
    }

    fn get_adj_first(&self) -> Option<&usize> {
        self.to.first()
    }

    fn is_adjacent(&self, other_id: usize) -> bool {
        self.to.contains(&other_id)
    }

    #[doc(hidden)]
    unsafe fn clear_edges(&mut self){
        self.to.clear();
        self.original.clear();
    }

    #[doc(hidden)]
    unsafe fn push(&mut self, other: &mut Self) -> Result<(), GraphErrors>
    {
        if self.is_adjacent(other.id){
            Err(GraphErrors::EdgeExists)
        }else {
            self.to.push(other.id);
            other.to.push(self.id);
            self.original.push(
                OriginalEdge{
                    from: self.id as u32,
                    to: other.id as u32,
                    is_at_origin: true
                }
            );
            other.original.push(
                OriginalEdge{
                    from: other.id as u32,
                    to: self.id as u32,
                    is_at_origin: true
                }
            );

            Ok(())
        }
    }

    #[doc(hidden)]
    unsafe fn remove(&mut self, other: &mut Self)
        -> Result<(), GraphErrors>
    {
        if !self.is_adjacent(other.id()){
            return Err(GraphErrors::EdgeDoesNotExist);
        }
        self.swap_remove_elem(other.id());
        other.swap_remove_elem(self.id());
        Ok(())
    }

    fn sort_adj(&mut self) {
        let p = permutation::sort(self.to.as_slice());
        self.to = p.apply_slice(&self.to[..]);
        self.original = p.apply_slice(&self.original[..]);
    }

    fn shuffle_adj<R: rand::Rng>(&mut self, rng: &mut R) {
        let mut list: Vec<_> = (0..self.original.len()).collect();
        list.shuffle(rng);
        let new_to: Vec<_> = list.iter().map(|&idx| self.to[idx]).collect();
        let new_original: Vec<_> = list.iter().map(|&idx| self.original[idx]).collect();
        self.to = new_to;
        self.original = new_original;
    }

}