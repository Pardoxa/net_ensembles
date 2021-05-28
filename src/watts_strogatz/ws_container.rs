use crate::{AdjContainer, GraphErrors, IterWrapper};
#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};
use permutation;

#[derive(Debug,Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub(crate) struct OriginalEdge{
    from: u32,
    to: u32,
}

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

    fn swap_remove_elem(&mut self, elem: usize)
    {
        let index = self.get_index(elem);
        self.to.swap_remove(index);
        self.original.swap_remove(index);
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
                    to: other.id as u32
                }
            );
            other.original.push(
                OriginalEdge{
                    from: other.id as u32,
                    to: self.id as u32
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
}