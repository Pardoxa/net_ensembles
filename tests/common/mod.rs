use net_ensembles::Node;
use net_ensembles::GenericGraph;
use net_ensembles::traits::*;
use std::fmt::Debug;

pub fn equal_graphs<T, A>(g1: &GenericGraph<T, A>, g2: &GenericGraph<T, A>)
where T: Node + SerdeStateConform,
      A: Debug + AdjContainer<T> + SerdeStateConform
{
    assert_eq!(g1.edge_count(), g2.edge_count());
    assert_eq!(g1.vertex_count(), g2.vertex_count());
    for (n0, n1) in g2.container_iter().zip(g1.container_iter()) {
        assert_eq!(n1.id(), n1.id());
        assert_eq!(n1.degree(), n1.degree());

        for (i, j) in n1.neighbors().zip(n0.neighbors()) {
            assert_eq!(i, j);
        }
    }
}
