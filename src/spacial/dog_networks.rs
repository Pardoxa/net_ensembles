use rand::{Rng, distributions::Uniform, distributions::Distribution, seq::SliceRandom};
use std::iter::*;
use super::*;
use crate::{Node, AdjContainer, Dot};
use rand_distr::Poisson;
use std::io::Write;

pub struct DogEnsemble<T, R>
{
    graph: SpacialGraph<T>,
    rng: R,
    kappa: f64,
    tau: f64,
    lambda: usize
}

/// You should use **neato** if you want the correct spacial placement of nodes
impl<T, R> Dot for DogEnsemble<T, R>
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

impl<T,R> DogEnsemble<T, R>
where R: Rng,
    T: Node
{
    pub fn new(n: usize, mut rng: R, kappa: f64, tau: f64, lambda: usize) -> Self
    {
        let mut graph = SpacialGraph::new(n);

        let iter = LatinHypercubeSampling2D::new(n, &mut rng);

        let scaling_factor = 1.0/ n as f64;

        graph.vertices
            .iter_mut()
            .zip(iter)
            .for_each(|(v, val)|
                {
                    v.x = val.0 as f64 * scaling_factor;
                    v.y = val.1 as f64 * scaling_factor;
                }
            );

        let mut res = Self{
            graph,
            rng, 
            tau,
            lambda,
            kappa
        };

        res.randomize();

        res

    }

    pub fn randomize(&mut self)
    {
        self.graph.clear_edges();
        let uniform = Uniform::new(0.0, 1.0);
        for i in 0..self.graph.vertex_count()
        {
            for j in 0..i{
                let (node_i, node_j) = self.graph.get_2_mut(i, j);
                let dist = node_i.distance(node_j);
                let prob = (-dist*self.kappa).exp();
                let num = uniform.sample(&mut self.rng);
                if num < prob {
                    unsafe{
                        let _ = node_i.push(node_j);
                    }
                    self.graph.edge_count += 1;
                }
            }
        }

        let proportion = 1.0 - self.tau;
        let num = (self.graph.vertex_count() as f64 * proportion).round() as usize;

        let mut all_possible_nodes: Vec<_> = (0..self.graph.vertex_count()).collect();
        all_possible_nodes.shuffle(&mut self.rng);

        let special_nodes = &all_possible_nodes[0..num];

        let poi = Poisson::new(self.lambda as f64)
            .unwrap();

        let index_uniform = Uniform::new(0, self.graph.vertex_count());

        let mut degree_sum: usize = self.graph.degree_iter().sum();
        for &node in special_nodes
        {
            let m = poi.sample(&mut self.rng) as usize;
            for _ in 0..m{
                let deg_factor = 1.0 / degree_sum as f64;
                loop {
                    let index = index_uniform.sample(&mut self.rng);
                    let prob = self.graph.degree(index).unwrap() as f64 * deg_factor;

                    if index != node 
                        && uniform.sample(&mut self.rng) < prob
                        && self.graph.add_edge(node, index).is_ok() 
                    {
                        break;   
                    }
                }
                degree_sum += 2;
            }
            
        }

    }

    pub fn graph(&self) -> &SpacialGraph<T>
    {
        &self.graph
    }
}


pub struct LatinHypercubeSampling2D<R>
{
    x: Vec<usize>,
    y: Vec<usize>,
    rng: R
}


impl<R> LatinHypercubeSampling2D<R>
{
    pub fn new(samples: usize, rng: R) -> Self
    {
        let x: Vec<_> = (0..samples).collect();
        let y = x.clone();
        Self { x, y, rng }
    }
}

impl<R: Rng> ExactSizeIterator for LatinHypercubeSampling2D<R>
{
    fn len(&self) -> usize {
        self.x.len()
    }
}

impl<R: Rng> FusedIterator for LatinHypercubeSampling2D<R> {}

impl<R> Iterator for LatinHypercubeSampling2D<R>
where R: Rng
{
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x.is_empty()
        {
            None
        } else {
            let uniform = Uniform::new(0, self.x.len());
            let x_idx = uniform.sample(&mut self.rng);
            let y_idx = uniform.sample(&mut self.rng);

            let x = self.x.swap_remove(x_idx);
            let y = self.y.swap_remove(y_idx);
            Some((x,y))
        }
    }
}