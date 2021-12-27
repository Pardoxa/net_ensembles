use {
    crate::{
        AdjContainer,
        GenericGraph
    },
    std::{
        marker::PhantomData,
        collections::VecDeque
    }
};


/// Depth first search Iterator
pub struct Dfs<'a, T, A>
where   T: 'a,
        A: AdjContainer<T>
{
        vertices: &'a [A],
        handled: Vec<bool>,
        stack: Vec<usize>,
        marker: PhantomData<T>
}


impl<'a, T, A> Dfs<'a, T, A>
where   T: 'a,
        A: AdjContainer<T>
{
    pub(crate) fn new(graph: &'a GenericGraph<T, A>, index: usize) -> Self {
        let mut handled: Vec<bool> = vec![false; graph.vertex_count()];
        let mut stack: Vec<usize> = Vec::with_capacity(graph.vertex_count());
        
        if index < handled.len()
        {
            stack.push(index);
            handled[index] = true;
        }

        Dfs {
            vertices: graph.vertices.as_slice(),
            handled,
            stack,
            marker: PhantomData
        }
    }
}

impl<'a, T, A> Iterator for Dfs<'a, T, A>
where   T: 'a,
        A: AdjContainer<T>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.stack.pop()?;
        let container = &self.vertices[index];
        for &i in container.neighbors() {
            if !self.handled[i] {
                self.handled[i] = true;
                self.stack.push(i);
            }
        }
        Some(container.contained())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.stack.len(), Some(self.handled.len()))
    }
}

/// # Depth first search iterator
/// * used to iterate over what is contained in a [`GenericGraph`](crate::GenericGraph::dfs_mut)
/// ## Borrow checker works
/// I am using the dark arts of unsafe rust to make this 
/// Iterator work. The following is a doc test to assert
/// that the borrow checker still forbids double mutable access
/// ```compile_fail
/// use net_ensembles::{Graph, CountingNode};
/// let mut graph = Graph::<CountingNode>::complete_graph(10);
/// 
/// let mut iter = graph.dfs_mut(0);
/// 
/// let first = iter.next().unwrap();
/// first.index = 3;
/// iter.next();
/// let third = iter.next().unwrap();
/// graph.at_mut(3).index = 4;
/// drop(iter);
/// third.index = 23;
/// ```
pub struct DfsMut<'a, T, A>
where A: AdjContainer<T>
{
    graph: *mut A,
    handled: Vec<bool>,
    stack: Vec<usize>,
    marker: PhantomData<&'a mut T>
}

impl<'a, T, A> DfsMut<'a, T, A>
where A: AdjContainer<T>
{
    pub(crate) fn new(graph: &'a mut GenericGraph<T, A>, index: usize) -> Self
    {
        let mut handled = vec![false; graph.vertex_count()];
        let mut stack: Vec<usize> = Vec::with_capacity(graph.vertex_count());

        if index < handled.len()
        {
            stack.push(index);
            handled[index] = true;
        }

        DfsMut{
            graph: graph.vertices.as_mut_ptr(),
            handled,
            stack,
            marker: PhantomData
        }
    }
}

impl<'a, T, A> Iterator for DfsMut<'a, T, A>
where A: AdjContainer<T> + 'a,
    T: 'a
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item>
    {
        let index = self.stack.pop()?;
        let container = unsafe{
            &mut *self.graph.add(index)
        };
        for &i in container.neighbors() {
            if !self.handled[i] {
                self.handled[i] = true;
                self.stack.push(i);
            }
        }

        Some(container.contained_mut())
    }
}

/// Depth first search Iterator with **index** of corresponding nodes
pub struct DfsWithIndex<'a, T, A>
where   T: 'a,
        A: AdjContainer<T>
{
        vertices: &'a [A],
        handled: Vec<bool>,
        stack: Vec<usize>,
        marker: PhantomData<T>
}

impl<'a, T, A> DfsWithIndex<'a, T, A>
    where   T: 'a,
            A: AdjContainer<T>
{

    pub(crate) fn new(graph: &'a GenericGraph<T, A>, index: usize) -> Self {
        let mut handled: Vec<bool> = vec![false; graph.vertex_count()];
        let mut stack: Vec<usize> = Vec::with_capacity(graph.vertex_count());
        
        if index < handled.len()
        {
            stack.push(index);
            handled[index] = true;
        }
        
        DfsWithIndex {
            vertices: graph.vertices.as_slice(),
            handled,
            stack,
            marker: PhantomData
        }
    }

}

impl<'a, T, A> Iterator for DfsWithIndex<'a, T, A>
where   T: 'a,
        A: AdjContainer<T>
{
        type Item = (usize, &'a T);

        fn next(&mut self) -> Option<Self::Item> {
            let index = self.stack.pop()?;
            let container = &self.vertices[index];
            for &i in container.neighbors() {
                if !self.handled[i] {
                    self.handled[i] = true;
                    self.stack.push(i);
                }
            }
            Some((index, container.contained()))
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.stack.len(), Some(self.vertices.len()))
        }
}

/// # Breadth first search Iterator with **index** and **depth** of corresponding nodes
/// * iterator returns tuple: `(index, node, depth)`, where `node` is what 
/// is contained at the vertex corresponding to the index, i.e., the `&T`
pub struct Bfs<'a, T, A>
where   T: 'a,
        A: AdjContainer<T>
{
        vertices: &'a [A],
        handled: Vec<bool>,
        queue0: VecDeque<usize>,
        queue1: VecDeque<usize>,
        depth: usize,
        marker: PhantomData<T>
}

impl<'a, T, A> Bfs<'a, T, A>
where   T: 'a,
        A: AdjContainer<T>
{
        pub(crate) fn new(graph: &'a GenericGraph<T, A>, index: usize) -> Self {
            let mut handled= vec![false; graph.vertex_count()];
            let mut queue0 = VecDeque::with_capacity(graph.vertex_count() / 2);
            let queue1 = VecDeque::with_capacity(graph.vertex_count() / 2);
            
            if index < graph.vertex_count() {
                queue0.push_back(index);
                handled[index] = true;
            }

            Bfs {
                vertices: graph.vertices.as_slice(),
                handled,
                queue0,
                queue1,
                depth: 0,
                marker: PhantomData
            }
        }

        pub(crate) fn reuse(&mut self, index: usize) {

            self.handled.fill(false);
 
            self.queue0.clear();
            self.queue1.clear();
            self.depth = 0;

            if index < self.vertices.len() {
                self.queue0.push_back(index);
                self.handled[index] = true;
            }
        }
}


/// # Iterator
/// - returns tuple: `(index, node, depth)`
impl<'a, T, A> Iterator for Bfs<'a, T, A>
where   T: 'a,
        A: AdjContainer<T>
{
        type Item = (usize, &'a T, usize);
        fn next(&mut self) -> Option<Self::Item> {
            // if queue0 is not empty, take element from queue, push neighbors to other queue
            if let Some(index) = self.queue0.pop_front() {
                let container = &self.vertices[index];
                for &i in container.neighbors() {
                    if !self.handled[i] {
                        self.handled[i] = true;
                        self.queue1.push_back(i);
                    }
                }
                Some((index, container.contained(), self.depth))
            } else if self.queue1.is_empty() {
                None
            } else {
                std::mem::swap(&mut self.queue0, &mut self.queue1);
                self.depth += 1;
                self.next()
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.queue0.len() + self.queue1.len(), Some(self.vertices.len()))
        }
}

/// # Breadth first search Iterator with **index** and **depth** of corresponding nodes
/// * iterator returns tuple: `(index, node, depth)`, where `node` is what 
/// is contained at the vertex corresponding to the index, i.e., the `&mut T`
pub struct BfsMut<'a, T, A>
where   T: 'a,
        A: AdjContainer<T>
{
        vertices: *mut A,
        handled: Vec<bool>,
        queue0: VecDeque<usize>,
        queue1: VecDeque<usize>,
        depth: usize,
        marker: PhantomData<&'a mut T>
}

impl<'a, T, A> BfsMut<'a, T, A>
where   T: 'a,
        A: AdjContainer<T>
{
    pub(crate) fn new(graph: &'a mut GenericGraph<T, A>, index: usize) -> Self {
        let mut handled= vec![false; graph.vertex_count()];
        let mut queue0 = VecDeque::with_capacity(graph.vertex_count() / 2);
        let queue1 = VecDeque::with_capacity(graph.vertex_count() / 2);
        
        if index < graph.vertex_count() {
            queue0.push_back(index);
            handled[index] = true;
        }

        BfsMut {
            vertices: graph.vertices.as_mut_ptr(),
            handled,
            queue0,
            queue1,
            depth: 0,
            marker: PhantomData
        }
    }
}

/// # Iterator
/// - returns tuple: `(index, node, depth)`
impl<'a, T, A> Iterator for BfsMut<'a, T, A>
where   T: 'a,
        A: AdjContainer<T> + 'a
{
        type Item = (usize, &'a mut T, usize);
        fn next(&mut self) -> Option<Self::Item> {
            // if queue0 is not empty, take element from queue, push neighbors to other queue
            if let Some(index) = self.queue0.pop_front() {
                let container = unsafe{
                    &mut *self.vertices.add(index)
                };
                for &i in container.neighbors() {
                    if !self.handled[i] {
                        self.handled[i] = true;
                        self.queue1.push_back(i);
                    }
                }
                Some((index, container.contained_mut(), self.depth))
            } else if self.queue1.is_empty() {
                None
            } else {
                std::mem::swap(&mut self.queue0, &mut self.queue1);
                self.depth += 1;
                self.next()
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.queue0.len() + self.queue1.len(), Some(self.handled.len()))
        }
}

// # Breadth first search Iterator with **index** and **depth** of corresponding nodes
/// * iterator returns tuple: `(index, node, depth)`
/// * iterator uses filter to decide, if a vertex should be considered
pub struct BfsFiltered<'a, 'b, T, A, F>
where   T: 'a,
        A: AdjContainer<T>,
        F: FnMut(&T, usize) -> bool,
{
        vertices: &'a [A],
        handled: Vec<bool>,
        queue0: VecDeque<usize>,
        queue1: VecDeque<usize>,
        depth: usize,
        filter_fn: &'b mut  F,
        marker: PhantomData<T>
}

impl<'a, 'b, T, A, F>  BfsFiltered<'a, 'b, T, A, F>
where   T: 'a,
        A: AdjContainer<T>,
        F: 'b + FnMut(&T, usize) -> bool,
{
    pub(crate) fn new(graph: &'a GenericGraph<T, A>, index: usize, filter: &'b mut F) -> Option<Self>
    {
        if index >= graph.vertex_count() || !filter(graph.at(index), index) {
            return None;
        }
        let mut handled= vec![false; graph.vertex_count()];
        let mut queue0 = VecDeque::with_capacity(graph.vertex_count() / 2);
        let queue1 = VecDeque::with_capacity(graph.vertex_count() / 2);
        
        queue0.push_back(index);
        handled[index] = true;

        Some(
            Self{
                handled,
                vertices: graph.vertices.as_slice(),
                filter_fn: filter,
                queue0,
                queue1,
                depth: 0,
                marker: PhantomData
            }
        )
    }

    /// At any state of the iterator, you can check if a given, valid Vertex, was encountered yet
    /// * Note: That can mean, that said vertex is still in the queue
    /// * **panics** if index is out of bounds
    pub fn is_handled(&self, index: usize) -> bool
    {
        self.handled[index]
    }

    /// Efficiently reuse the iterator, now possibly starting at a new index
    /// * returns Err(self) without changing self, if index out of Bounds 
    /// or filter (filter_fn) of (vertex_at_index, index) is false
    /// * otherwise: prepares iterator to be used again and returns Ok(self)
    pub fn reuse(mut self, index: usize) -> Result<Self, Self>
    {
        if index > self.vertices.len() || !(self.filter_fn)(self.vertices[index].contained(), index) {
            return Err(self);
        }
        for i in 0..self.handled.len() {
            self.handled[i] = false;
        }
        self.queue0.clear();
        self.queue1.clear();
        self.depth = 0;

        
        self.queue0.push_back(index);
        self.handled[index] = true;
        Ok(self)
    }
}

impl<'a, 'b, T, A, F> Iterator for BfsFiltered<'a, 'b, T, A, F>
where   T: 'a,
        A: AdjContainer<T>,
        F: 'b + FnMut(&T, usize) -> bool,
{
    type Item = (usize, &'a T, usize);
    fn next(&mut self) -> Option<Self::Item> {
        // if queue0 is not empty, take element from queue, push neighbors to other queue
        if let Some(index) = self.queue0.pop_front() {
            let container = self.vertices.get(index)?;
            for &i in container.neighbors() {
                if self.handled[i] || !(self.filter_fn)(self.vertices[index].contained(), i)
                {
                    continue;
                }
                
                self.handled[i] = true;
                self.queue1.push_back(i);
                
            }
            Some((index, container.contained(), self.depth))
        } else if self.queue1.is_empty() {
            None
        } else {
            std::mem::swap(&mut self.queue0, &mut self.queue1);
            self.depth += 1;
            self.next()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.queue0.len() + self.queue1.len(), Some(self.vertices.len()))
    }
}



