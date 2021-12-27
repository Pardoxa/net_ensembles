//! # Generic implementation for Topology
//! * contains multiple measurable quantities
//! * used by `Graph<T>` and `SwGraph<T>`

mod iterators;
pub use iterators::*;

#[allow(clippy::module_inception)]
mod generic_graph;
pub use generic_graph::*;

#[cfg(test)]
mod tests{
    use super::*;
    use crate::{EmptyNode, graph::NodeContainer, CountingNode};
    use sampling::histogram::{Histogram, HistogramVal};
    use crate::{Graph, AdjContainer};

    #[test]
    fn degree_dist()
    {
        let mut network = GenericGraph::<EmptyNode, NodeContainer<EmptyNode>>::new(10);

        network.init_ring(1).unwrap();

        let hist = network.degree_histogram();

        assert_eq!(hist.hist().len(), 1);
        assert_eq!(hist.hist()[0], 10);
        assert_eq!(hist.first_border(), 2);
        assert_eq!(hist.second_last_border(), 2);

        let network = GenericGraph::<EmptyNode, NodeContainer<EmptyNode>>::new(1);

        let hist = network.degree_histogram();
        let mut iter = hist.bin_hits_iter();
        assert_eq!(iter.next(), Some((0, 1)));
        assert_eq!(iter.next(), None);

    }

    #[test]
    fn subgraph()
    {
        let graph = Graph::<EmptyNode>::complete_graph(10);

        let subgraph = graph.cloned_subgraph(vec![3, 2, 1, 9, 3])
            .unwrap();

        assert_eq!(subgraph.vertex_count(), 4);
        assert_eq!(subgraph.edge_count(), (4*3) / 2);

        let c = subgraph.container(1);
        let mut iter = c.neighbors();
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
        drop(subgraph);
        let subgraph = graph.cloned_subgraph(vec![0]).unwrap();
        assert_eq!(subgraph.edge_count(), 0);
        assert_eq!(subgraph.vertex_count(), 1);
        drop(graph);

        let mut graph = Graph::<EmptyNode>::new(7);

        graph.add_edge(0, 3).unwrap();
        graph.add_edge(0, 1).unwrap();
        graph.add_edge(2, 3).unwrap();
        graph.add_edge(0, 5).unwrap();

        
        let mut subgraph = graph.cloned_subgraph(vec![0, 1, 3])
            .unwrap();
        subgraph.sort_adj();

        assert_eq!(subgraph.vertex_count(), 3);
        assert_eq!(subgraph.edge_count(), 2);

        let c = subgraph.container(0);
        assert_eq!(c.degree(), 2);
        let mut iter = c.neighbors();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), None);

        let c = subgraph.container(1);
        assert_eq!(c.degree(), 1);
        
        let mut iter = c.neighbors();
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), None);

        let c = subgraph.container(2);
        let mut iter = c.neighbors();
        assert_eq!(c.degree(), 1);
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), None);

        assert!(graph.cloned_subgraph(vec![]).is_none());
        assert!(graph.cloned_subgraph(vec![199,199,329029]).is_none());
    }

    #[test]
    fn test_dfs_mut_magic()
    {
        let mut graph = Graph::<CountingNode>::complete_graph(10);

        let mut iter = graph.dfs_mut(0);

        let first = iter.next().unwrap();
        first.index = 3;
        iter.next();
        let third = iter.next().unwrap();
        third.index = 23;
        graph.at_mut(3).index = 4;
    }
}