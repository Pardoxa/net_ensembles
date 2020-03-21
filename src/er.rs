use crate::node::Node;
use graphlib::Graph;
use graphlib::VertexId;
use rand_core::SeedableRng;

#[allow(dead_code)]
pub struct ER<T: Node, R: SeedableRng> {
    graph: Graph<T>,
    prob: f32,
    ids: Vec<VertexId>,
    rng: R,
}
/// This is going to become an Erdős-Rényi graph
#[allow(dead_code)]
impl<T: Node, R: rand::Rng + SeedableRng> ER<T, R> {
    fn random(&mut self) {
        for i in 0..self.ids.len() {
            for j in i+1..self.ids.len() {
                if self.rng.gen::<f32>() <= self.prob {
                    self.graph.add_edge(&self.ids[i], &self.ids[j]).unwrap();
                }
            }
        }
    }

    pub fn new(size: usize, prob: f32, rng: R) -> Self {
        let mut graph: Graph<T> = Graph::with_capacity(size);
        let mut ids = Vec::new();
        for _i in 0..size {
            ids.push(graph.add_vertex(T::new_empty()));
        }
        let mut e = ER {
            graph,
            prob,
            ids,
            rng,
        };
        e.random();
        e
    }

    pub fn get_graph(&self) -> &Graph<T> {
        &self.graph
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use rand_pcg::Pcg64;
    use crate::node::TestNode;

    #[test]
    fn test_graph_construction() {
        let rng = Pcg64::seed_from_u64(76);
        let e = ER::<TestNode, Pcg64>::new(20, 0.3, rng);
        assert!(e.get_graph().edge_count() > 0);
        assert_eq!(20, e.get_graph().vertex_count());
    }
}
