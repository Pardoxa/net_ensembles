use crate::*;
use er_c::draw_two_from_range;
use crate::spacial::*;
use rand::Rng;
use std::{io::Write,f64::consts::PI};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};


/// # Implements a special Ensemble
///
/// * You can generate a dot file which includes special information.
/// * **NOTE** You should use **neato** for that to work 
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct SpacialEnsemble<T, R> 
where T: Node {
    graph: SpacialGraph<T>,
    rng: R,
    f: f64,
    alpha: f64,
    sqrt_n_pi: f64,
}


impl<T, R> SpacialEnsemble<T, R> 
    where 
        T: Node, 
        R: Rng,
{
    /// Generate a new Spacial ensemble with 
    /// * `n` nodes
    /// * `rng` as random number generator
    /// * `f` - see paper
    /// * `alpha` - see paper
    pub fn new(n: usize, mut rng: R, f: f64, alpha: f64) -> Self
    {
        let mut graph = SpacialGraph::new(n);

        graph.vertices
            .iter_mut()
            .for_each(|v|
                {
                    v.x = rng.gen();
                    v.y = rng.gen();
                }
            );

        let mut res = Self{
            graph,
            rng,
            alpha,
            f,
            sqrt_n_pi: (n as f64 * PI).sqrt()
        };
        res.randomize();
        res
    }

    #[inline]
    fn prob_unchecked(&self, i: usize, j: usize) -> f64
    {
        let distance = unsafe{
            self.graph
                .vertices
                .get_unchecked(i)
                .distance(self.graph.vertices.get_unchecked(j))
        };
        let prob = self.f * 
            (1.0 + self.sqrt_n_pi * distance / self.alpha)
            .powf(-self.alpha);
        prob
    }
}


impl<T, R> SimpleSample for SpacialEnsemble<T, R>
where   T: Node + SerdeStateConform,
        R: rand::Rng,
{
    /// # Randomizes the edges according to Er probabilities
    /// * this is used by `ErEnsembleC::new` to create the initial topology
    /// * you can use this for sampling the ensemble
    /// * runs in `O(vertices * vertices)`
    fn randomize(&mut self) {
        self.graph.clear_edges();
        // iterate over all possible edges once
        for i in 0..self.graph.vertex_count() {
            for j in i+1..self.graph.vertex_count() {
                let prob = self.prob_unchecked(i, j);
                if self.rng.gen::<f64>() <= prob {
                    // in these circumstances equivalent to 
                    // self.graph.add_edge(i, j).unwrap();
                    // but without checking for existing edges and other errors -> a bit more efficient
                    unsafe{
                        self.graph
                            .vertices
                            .get_unchecked_mut(i)
                            .adj
                            .push(j);
                        self.graph
                            .vertices
                            .get_unchecked_mut(j)
                            .adj
                            .push(i);
                    }
                    
                    self.graph.edge_count += 1;
                }
            }
        }
    }
}

/// # Returned by markov steps
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum SpacialStep {
    /// nothing was changed
    Nothing,
    /// an edge was added
    AddedEdge((usize, usize)),
    /// an edge was removed
    RemovedEdge((usize, usize)),

    /// an error occured. Did you try to remove steps in the wrong order?
    Error,
}

impl<T, R> MarkovChain<SpacialStep, SpacialStep> for 
    SpacialEnsemble<T, R>
where 
    T: Node,
    R: Rng,
{
    fn m_step(&mut self) -> SpacialStep {
        let edge = draw_two_from_range(&mut self.rng, self.graph.vertex_count());
        let prob = self.prob_unchecked(edge.0, edge.1);
        if self.rng.gen::<f64>() <= prob {
            let success = self.graph.add_edge(edge.0, edge.1);
            match success{
                Ok(_) => SpacialStep::AddedEdge(edge),
                Err(_) => SpacialStep::Nothing,
            }
        } else {
            let success =  self.graph.remove_edge(edge.0, edge.1);
            match success {
                Ok(_)  => SpacialStep::RemovedEdge(edge),
                Err(_) => SpacialStep::Nothing,
            }
        }
    }

    fn m_steps_quiet(&mut self, count: usize) {
        for _ in 0..count {
            let (i, j) = draw_two_from_range(&mut self.rng, self.graph.vertex_count());
            let prob = self.prob_unchecked(i, j);
            if self.rng.gen::<f64>() <= prob {
                let _ = self.graph.add_edge(i, j);
            } else {
                let _ =  self.graph.remove_edge(i, j);
            }
        }
    }

    fn undo_step(&mut self, step: &SpacialStep) -> SpacialStep {
        match step {
            SpacialStep::AddedEdge(edge) => {
                let res = self.graph
                    .remove_edge(edge.0, edge.1);
                match res {
                    Ok(_) => SpacialStep::RemovedEdge(*edge),
                    _ => SpacialStep::Error,
                }
            },
            SpacialStep::RemovedEdge(edge) => {
                let res = self.graph
                    .add_edge(edge.0, edge.1);
                match res {
                    Ok(_) => SpacialStep::AddedEdge(*edge),
                    _ => SpacialStep::Error,
                }
            },
            SpacialStep::Nothing | SpacialStep::Error => *step,
            
        }
    }

    /// * panics if `step` is error, or cannot be undone
    /// The latter means, you are undoing the steps in the wrong order
    fn undo_step_quiet(&mut self, step: &SpacialStep) {
        match step {
            SpacialStep::AddedEdge(edge) =>
            {
                self.graph.remove_edge(edge.0, edge.1)
                    .expect("tried to remove non existing edge!");
            },
            SpacialStep::RemovedEdge(edge) => {
                self.graph
                    .add_edge(edge.0, edge.1)
                    .expect("Tried to add existing edge!");
            },
            SpacialStep::Nothing => (),
            SpacialStep::Error => unreachable!("You tried to undo an error! MarcovChain undo_step_quiet")
        }
    }
}

/// You should use **neato** if you want the correct spacial placement of nodes
impl<T, R> Dot for SpacialEnsemble<T, R>
where T: Node
{
    fn dot_from_indices<F, W, S1, S2>(&self, writer: W, dot_options: S1, mut f: F) -> Result<(), std::io::Error>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
        W: Write,
        F: FnMut(usize) -> S2 
    {
        self.graph.dot_from_indices(
            writer,
            dot_options,
            |index| {
                format!(
                    "{}\" pos=\"{:.2},{:.2}!",
                    f(index).as_ref(),
                    self.graph.container(index).x * 100.0,
                    100.0 * self.graph.container(index).y
                )
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_pcg::Pcg64;
    use rand::SeedableRng;
    use std::fs::*;
    #[test]
    fn spacial_print() {
        let rng = Pcg64::seed_from_u64(12232);
        let mut e = SpacialEnsemble::<EmptyNode, _>::new(50, rng, 0.95, 3.0);
        
        e.m_steps_quiet(2000);
        let f = File::create("Spacial.dot")
            .unwrap();
        e.dot(f, "").unwrap();
    }
}