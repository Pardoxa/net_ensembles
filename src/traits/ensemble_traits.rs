use crate::{AdjContainer, traits::*, iter::*, GenericGraph};
use crate::generic_graph::{Dfs, DfsWithIndex, Bfs};
use crate::{sw_graph::SwContainer, graph::NodeContainer};

/// # Access internal random number generator
pub trait HasRng<Rng>
where Rng: rand::Rng
{
    /// # Access RNG
    /// If, for some reason, you want access to the internal random number generator: Here you go
    fn rng(&mut self) -> &mut Rng;

    /// # If you need to exchange the internal rng
    /// * returns old rng
    fn swap_rng(&mut self, rng: Rng) -> Rng;
}

/// # Create a markov chain by doing markov steps
pub trait MarkovChain<S, Res> {
    /// * undo a markov step, return result-state
    /// * if you want to undo more than one step
    /// see [`undo_steps`](#method.undo_steps)
    fn undo_step(&mut self, step: S) -> Res;

    /// * undo a markov, **panic** on invalid result state
    /// * for undoing multiple steps see [`undo_steps_quiet`](#method.undo_steps_quiet)
    fn undo_step_quiet(&mut self, step: S);

    /// # Markov step
    /// * use this to perform a markov step step
    /// * for doing multiple markov steps at once, use [`m_steps`](#method.m_steps)
    fn m_step(&mut self) -> S;

    /// #  Markov steps
    /// * use this to perform multiple markov steps at once
    /// * result `Vec<S>` can be used to undo the steps with `self.undo_steps(result)`
    fn m_steps(&mut self, count: usize) -> Vec<S> {
        let mut vec = Vec::with_capacity(count);
        for _ in 0..count {
            vec.push(
                self.m_step()
            );
        }
        vec
    }

    /// # Undo markov steps
    /// * Note: uses undo_step in correct order and returns result
    /// ## Important:
    /// * look at specific implementation of `undo_step`, every thing mentioned there applies to each step
    fn undo_steps(&mut self, steps: Vec<S>) -> Vec<Res> {
        steps.into_iter()
            .rev()
            .map(|step| self.undo_step(step))
            .collect()
    }

    /// # Undo markov steps
    /// * Note: uses `undo_step_quiet` in correct order
    /// ## Important:
    /// * look at specific implementation of `undo_step_quiet`, every thing mentioned there applies to each step
    fn undo_steps_quiet(&mut self, steps: Vec<S>) {
        let iter = steps.into_iter()
            .rev();
        for step in iter {
            self.undo_step_quiet(step);
        }
    }

    /// # Metropolis Monte Carlo
    ///
    ///
    /// |               | meaning                                                                      |
    /// |---------------|------------------------------------------------------------------------------|
    /// | `rng`         | the Rng used to decide, if a state should be accepted or rejected            |
    /// | `temperature` | used in metropolis probability                                               |
    /// | `stepsize`    | is used for each markov step, i.e., `self.m_steps(stepsize)` is called       |
    /// | `steps`       | is the number of steps with size `stepsize`, that this method should perform |
    /// | `valid_self`  | checks, if the markov steps produced a valid state                           |
    /// | `energy`      | should calculate the "energy" of the system used for acceptance probability  |
    /// | `measure`     | called after each step                                                       |
    ///
    /// **Important**: if possible, treat `energy(&mut Self)` as `energy(&Self)`. This will be safer.
    ///
    /// **Note**: instead of the `temperature` T the literature sometimes uses &beta;. The relation between them is:
    /// &beta; = T⁻¹
    ///
    /// **Note**: If `valid_self` returns `false`, the state will be rejected. If you do not need this,
    /// use `|_| true`
    ///
    /// **`measure`**: function is intended for storing measurable quantities etc.
    /// Is called at the end of each iteration. As for the parameter:
    ///
    /// | type        | name suggestion  | description                                                                                                                                  |
    /// |-------------|------------------|----------------------------------------------------------------------------------------------------------------------------------------------|
    /// | `&mut Self` | `current_state`  | current `self`. After stepping and accepting/rejecting                                                                                       |
    /// | `usize`     | `i`              | counter, starts at 0, each step counter increments by 1                                                                                      |
    /// | `f64`       | `current_energy` | `energy(&mut self)`. Assumes that `energy` of a state deterministic and invariant under: `let steps = self.steps(n); self.undo_steps(steps);`|
    /// | `bool`      | `rejected`       | `true` if last step was rejected. That should mean, that the current state is the same as the last state.                                    |
    ///
    /// Citation see, e.g,
    /// > M. E. J. Newman and G. T. Barkema, "Monte Carlo Methods in Statistical Physics"
    ///   *Clarendon Press*, 1999, ISBN:&nbsp;978-0-19-8517979
    #[allow(clippy::clippy::too_many_arguments)]
    fn monte_carlo_metropolis<F, G, H, Rng>(
        &mut self,
        mut rng: Rng,
        temperature: f64,
        stepsize: usize,
        steps: usize,
        mut valid_self: F,
        mut energy: G,
        mut measure: H,
    )
    where
        F: FnMut(&mut Self) -> bool,
        G: FnMut(&mut Self) -> f64,
        H: FnMut(&mut Self, usize, f64, bool),
        Rng: rand::Rng
    {
        let mut old_energy = energy(self);
        let mut current_energy: f64;
        let mut last_steps: Vec<_>;
        let mut a_prob: f64;
        let mbeta = -1.0 / temperature;

        for i in 0..steps {
            last_steps = self.m_steps(stepsize);
            current_energy = energy(self);

            // calculate acacceptance probability
            let mut rejected = !valid_self(self);
            if !rejected {
                // I only have to calculate this for a valid state
                a_prob = 1.0_f64.min((mbeta * (current_energy - old_energy)).exp());
                rejected = rng.gen::<f64>() > a_prob;
            }


            // if step is NOT accepted
            if rejected {
                self.undo_steps_quiet(last_steps);
                current_energy = old_energy;
            } else {
                old_energy = current_energy;
            }
            measure(self, i, current_energy, rejected);
        }
    }
}

/// For easy sampling of your ensemble
pub trait SimpleSample{
    /// # Randomizes self according to  model
    /// * this is intended for creation of initial sample
    /// * used in [`simple_sample`](#method.simple_sample)
    /// and [`simple_sample_vec`](#method.simple_sample_vec)
    fn randomize(&mut self);

    /// # do the following `times` times:
    /// 1) `f(self)`
    /// 2) `self.randomize()`
    fn simple_sample<F>(&mut self, times: usize, mut f: F)
        where F: FnMut(&Self) -> ()
    {
        for _ in 0..times {
            f(self);
            self.randomize();
        }
    }

    /// # do the following `times` times:
    /// 1) `res = f(self)`
    /// 2) `self.randomize()`
    /// ## res is collected into Vector
    fn simple_sample_vec<F, G>(&mut self, times: usize, mut f: F) -> Vec<G>
        where F: FnMut(&Self) -> G
    {
        let mut vec = Vec::with_capacity(times);
        for _ in 0..times {
            vec.push(f(self));
            self.randomize();
        }
        vec
    }
}

/// unify graph ensembles in a trait
pub trait WithGraph<T, G> {
    /// * access additional information at index
    fn at(&self, index: usize) -> &T;

    /// * mutable access to additional information at index
    fn at_mut(&mut self, index: usize) -> &mut T;

    /// * returns reference to the underlying topology aka, the `GenericGraph`
    /// * use this to call functions regarding the topology
    fn graph(&self) -> &G;
}

///  Collection mut Graph iterators
pub trait GraphIteratorsMut<T, G, A>
where
    T: Node,
    A: AdjContainer<T>
{
    /// * iterate over mutable additional information of neighbors of vertex `index`
    /// * iterator returns `&mut T`
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    fn contained_iter_neighbors_mut(&mut self, index: usize) -> NContainedIterMut<'_, T, A>;

    /// * iterate over mutable additional information of neighbors of vertex `index`
    /// * iterator returns `(index_neighbor: usize, neighbor: &mut T)`
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    fn contained_iter_neighbors_mut_with_index(&mut self, index: usize) -> INContainedIterMut<'_, T, A>;

    /// * get iterator over mutable additional information stored at each vertex in order of the indices
    /// * iterator returns a `Node` (for example `EmptyNode` or whatever you used)
    fn contained_iter_mut(&mut self) -> ContainedIterMut<'_, T, A>;
}

/// Collection of Graph iterators
pub trait GraphIterators<T, G, A>
    where
        T: Node,
        A: AdjContainer<T>
{
    /// * get iterator over additional information stored at each vertex in order of the indices
    /// * iterator returns a `Node` (for example `EmptyNode` or whatever you used)
    /// * similar to `self.container_iter().map(|container| container.contained())`
    fn contained_iter(&self) -> ContainedIter<'_, T, A>;

    /// * iterate over additional information of neighbors of vertex `index`
    /// * iterator returns `&T`
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    fn contained_iter_neighbors(&self, index: usize) -> NContainedIter<'_, T, A>;

    /// * get iterator over AdjContainer in order of the indices
    /// * iterator returns `AdjContainer<Node>`, i.e., `A`
    fn container_iter(&self) -> core::slice::Iter<'_, A>;

    /// * iterate over additional information of neighbors of vertex `index`
    /// * iterator returns `&T`
    /// * `sort_adj` will affect the order
    /// * **panics** if index out of bounds
    fn container_iter_neighbors(&self, index: usize) -> NContainerIter<'_, T, A>;

    /// # returns `Iterator`
    ///
    /// * the iterator will iterate over the vertices in depth first search order,
    /// beginning with vertex `index`.
    /// * iterator returns `node`
    ///
    /// Order
    ///------------------------
    /// Order is guaranteed to be in DFS order, however
    /// if this order is not unambigouse
    /// adding edges and especially removing edges will shuffle the order.
    ///
    /// Note:
    /// ----------------------
    /// Will only iterate over vertices within the connected component that contains vertex `index`
    fn dfs(&self, index: u32) -> Dfs<'_, T, A>;

    /// # returns `Iterator`
    ///
    /// * the iterator will iterate over the vertices in depth first search order,
    /// beginning with vertex `index`.
    /// * Iterator returns tuple `(index, node)`
    ///
    /// Order
    ///------------------------
    /// Order is guaranteed to be in DFS order, however
    /// if this order is not unambigouse
    /// adding edges and especially removing edges will shuffle the order.
    ///
    /// Note:
    /// ----------------------
    /// Will only iterate over vertices within the connected component that contains vertex `index`
    fn dfs_with_index(&self, index: u32) -> DfsWithIndex<'_, T, A>;

    /// # returns `Iterator`
    ///
    /// * the iterator will iterate over the vertices in breadth first search order,
    /// beginning with vertex `index`.
    /// * Iterator returns tuple `(index, node, depth)`
    ///
    /// ### depth
    /// * starts at 0 (i.e. the first element in the iterator will have `depth = 0`)
    /// * `depth` equals number of edges in the *shortest path* from the *current* vertex to the
    /// *first* vertex (i.e. to the vertex with index `index`)
    ///
    /// Order
    ///------------------------
    /// Order is guaranteed to be in BFS order, however
    /// if this order is not unambigouse
    /// adding edges and especially removing edges will shuffle the order.
    ///
    /// Note:
    /// ----------------------
    /// Will only iterate over vertices within the connected component that contains vertex `index`
    fn bfs_index_depth(&self, index: u32) -> Bfs<'_, T, A>;
}


impl<T, E> GraphIterators<T, GenericGraph<T, NodeContainer<T>>, NodeContainer<T>> for E
where
    T: Node,
    E: WithGraph<T, GenericGraph<T, NodeContainer<T>>>,
{
    fn contained_iter(&self) -> ContainedIter<'_, T, NodeContainer<T>>
    {
        self.graph().contained_iter()
    }

    fn contained_iter_neighbors(&self, index: usize) -> NContainedIter<'_, T, NodeContainer<T>>
    {
        self.graph().contained_iter_neighbors(index)
    }

    fn container_iter(&self) -> core::slice::Iter<'_, NodeContainer<T>>
    {
        self.graph().container_iter()
    }

    fn container_iter_neighbors(&self, index: usize) -> NContainerIter<'_, T, NodeContainer<T>>
    {
        self.graph().container_iter_neighbors(index)
    }

    fn dfs(&self, index: u32) -> Dfs<'_, T, NodeContainer<T>>
    {
        self.graph().dfs(index)
    }

    fn dfs_with_index(&self, index: u32) -> DfsWithIndex<'_, T, NodeContainer<T>>
    {
        self.graph().dfs_with_index(index)
    }

    fn bfs_index_depth(&self, index: u32) -> Bfs<'_, T, NodeContainer<T>>
    {
        self.graph().bfs_index_depth(index)
    }
}

impl<T, E> GraphIterators<T, GenericGraph<T, SwContainer<T>>, SwContainer<T>> for E
where
    T: Node,
    E: WithGraph<T, GenericGraph<T, SwContainer<T>>>,
{
    fn contained_iter(&self) -> ContainedIter<'_, T, SwContainer<T>>
    {
        self.graph().contained_iter()
    }

    fn contained_iter_neighbors(&self, index: usize) -> NContainedIter<'_, T, SwContainer<T>>
    {
        self.graph().contained_iter_neighbors(index)
    }

    fn container_iter(&self) -> core::slice::Iter<'_, SwContainer<T>>
    {
        self.graph().container_iter()
    }

    fn container_iter_neighbors(&self, index: usize) -> NContainerIter<'_, T, SwContainer<T>>
    {
        self.graph().container_iter_neighbors(index)
    }

    fn dfs(&self, index: u32) -> Dfs<'_, T, SwContainer<T>>
    {
        self.graph().dfs(index)
    }

    fn dfs_with_index(&self, index: u32) -> DfsWithIndex<'_, T, SwContainer<T>>
    {
        self.graph().dfs_with_index(index)
    }

    fn bfs_index_depth(&self, index: u32) -> Bfs<'_, T, SwContainer<T>>
    {
        self.graph().bfs_index_depth(index)
    }
}
