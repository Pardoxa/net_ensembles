
use net_ensembles::*;
use std::fs::File;
use std::io::prelude::*;
use net_ensembles::dot_constants::*;

fn create_graph_1() -> Graph<EmptyNode> {
    let mut graph: Graph<EmptyNode> = Graph::new(20);

    graph.add_edge(0, 1).unwrap();
    graph.add_edge(0, 2).unwrap();
    graph.add_edge(1, 2).unwrap();

    // line of 6 vertices - no connected component
    for i in 5..10 {
        graph.add_edge(i, i + 1).unwrap();
    }

    // 4 completly connected nodes, each with 3 neighbors
    for i in 11..15 {
        for j in i+1..15 {
            graph.add_edge(i, j).unwrap();
        }
    }
    graph
}

#[test]
fn leaf_count() {
    let graph = create_graph_1();
    assert_eq!(2, graph.leaf_count());

    let empty_graph: Graph<EmptyNode> = Graph::new(20);
    assert_eq!(0, empty_graph.leaf_count());
}

#[test]
fn connected_components() {
    let graph: Graph<EmptyNode> = Graph::new(20);
    assert_eq!(vec![1;20], graph.connected_components());

    let graph2: Graph<EmptyNode> = Graph::new(0);
    assert_eq!(Vec::<usize>::new(), graph2.connected_components());
}

#[test]
fn multiple_connected_components() {
    let graph = create_graph_1();
    let components = graph.connected_components();
    assert_eq!(components[0], 6);
    assert_eq!(components[1], 4);
    assert_eq!(components[2], 3);
    for &component in components.iter().skip(3) {
        assert_eq!(component, 1);
    }
}

#[test]
fn q_core_empty_graph() {
    let graph: Graph<EmptyNode> = Graph::new(0);
    assert_eq!(graph.q_core(1), None);
    assert_eq!(graph.q_core(2), None);

    let graph2: Graph<EmptyNode> = Graph::new(1);

    assert_eq!(graph2.q_core(1), None);
    assert_eq!(graph2.q_core(2), Some(0));
}

#[test]
fn q_core_multiple_components() {
    let graph = create_graph_1();

    assert_eq!(graph.q_core(2), Some(4));
    assert_eq!(graph.q_core(3), Some(4));
    assert_eq!(graph.q_core(4), Some(0));
    assert_eq!(graph.q_core(1), None);
}

#[test]
fn q_core_complete_graph() {
    let mut graph: Graph<EmptyNode> = Graph::new(20);
    // create complete graph
    for i in 0..graph.vertex_count() {
        for j in i+1..graph.vertex_count() {
            graph.add_edge(i, j).unwrap();
        }
    }

    // since this is a complete graph, q core should always consist of 20 nodes
    // as long as q < 20, as every node has 19 neighbors
    for i in 2..20 {
        assert_eq!(graph.q_core(i), Some(20));
    }

    assert_eq!(graph.q_core(20), Some(0));
}

#[test]
fn q_core() {
    let mut graph: Graph<EmptyNode> = Graph::new(20);
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(0, 2).unwrap();
    graph.add_edge(1, 2).unwrap();
    assert_eq!(graph.q_core(2), Some(3));
}

#[test]
fn check_is_connected() {
    let mut graph: Graph<EmptyNode> = Graph::new(10);
    assert!(!graph.is_connected().unwrap());

    // almost connect graph
    for i in 0..8 {
        graph.add_edge(i, i+1).unwrap();
    }
    assert!(!graph.is_connected().unwrap());

    // connect graph
    graph.add_edge(8, 9).unwrap();
    assert!(graph.is_connected().unwrap());
}

#[test]
#[allow(deprecated)]
fn dot_labeled() {
    let graph = create_graph_1();
    let s = graph.to_dot_with_labels_from_contained(EXAMPLE_DOT_OPTIONS, |_, index| format!("Hey {}!", index));
    let mut read_in = File::open("TestData/label_test.dot").expect("unable to open file");
    let mut data = String::new();
    // let mut f = File::create("label_test.dot").expect("Unable to create file");
    // f.write_all(s.as_bytes()).expect("Unable to write data");
    read_in.read_to_string(&mut data).expect("unable to read file");
    assert_eq!(data, s);
}

#[test]
#[allow(deprecated)]
fn dot() {
    let graph = create_graph_1();
    let s = graph.to_dot();
    let mut read_in = File::open("TestData/dotTest.dot").expect("unable to open file");
    let mut data = String::new();
    read_in.read_to_string(&mut data).expect("unable to read file");
    assert_eq!(data, s);
}

#[test]
fn average_neighbor_count () {
    let mut graph: Graph<EmptyNode> = Graph::new(20);
    // create complete graph
    for i in 0..graph.vertex_count() {
        for j in i+1..graph.vertex_count() {
            graph.add_edge(i, j).unwrap();
        }
    }
    assert_eq!(graph.average_degree(), 19.0);


    let empty: Graph<EmptyNode> = Graph::new(20);

    assert_eq!(empty.average_degree(), 0.0);
}

#[test]
fn diameter_test() {
    let mut graph: Graph<EmptyNode> = Graph::new(5);
    for i in 0..4 {
        graph.add_edge(i, i + 1).unwrap();
    }
    assert_eq!(4, graph.diameter().unwrap());

    graph = Graph::new(4);

    for i in 0..4 {
        graph.add_edge(i, (i + 1) % 4).unwrap();
    }
    assert_eq!(2, graph.diameter().unwrap());

    graph = Graph::new(40);
    for i in 0..40 {
        graph.add_edge(i, (i + 1) % 40).unwrap();
    }
    assert_eq!(20, graph.diameter().unwrap());

    graph = Graph::new(40);
    for i in 0..40 {
        for j in i+1..40 {
            graph.add_edge(i, j).unwrap();
        }
    }
    assert_eq!(1, graph.diameter().unwrap());

    graph = Graph::new(1);
    graph.diameter();
}


#[test]
fn bi_test1() {
    let mut graph: Graph<EmptyNode> = Graph::new(6);
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(1, 2).unwrap();
    graph.add_edge(2, 0).unwrap();
    graph.add_edge(3, 4).unwrap();
    graph.add_edge(5, 4).unwrap();

    let components = graph.clone().vertex_biconnected_components(false);
    assert_eq!(components, vec![3,2,2]);
    assert_eq!(graph.vertex_biconnected_components(true), vec![3]);
}

#[test]
fn bi_test2() {
    let graph: Graph<EmptyNode> = Graph::new(6);

    let components = graph.vertex_biconnected_components(false);
    assert_eq!(components, vec![]);

    let mut graph: Graph<EmptyNode> = Graph::new(6);
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(1, 2).unwrap();
    graph.add_edge(2, 0).unwrap();

    let components = graph.vertex_biconnected_components(false);
    assert_eq!(components, vec![3]);

    let mut graph: Graph<EmptyNode> = Graph::new(5);
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(1, 2).unwrap();
    graph.add_edge(2, 0).unwrap();
    graph.add_edge(2, 3).unwrap();
    graph.add_edge(4, 3).unwrap();
    graph.add_edge(4, 0).unwrap();
    assert_eq!(vec![5], graph.vertex_biconnected_components(false));
}

#[test]
fn vertex_load() {
    let mut graph: Graph<EmptyNode> = Graph::new(4);
    // create complete graph
    for i in 0..graph.vertex_count() {
        for j in i+1..graph.vertex_count() {
            graph.add_edge(i, j).unwrap();
        }
    }
    let edge_b = graph.vertex_load(true);
    assert_eq!(edge_b, vec![3.0, 3.0, 3.0, 3.0]);
    let edge_b = graph.vertex_load(false);
    assert_eq!(edge_b, vec![0.0, 0.0, 0.0, 0.0]);

    let graph: Graph<EmptyNode> = Graph::new(4);
    let edge_b = graph.vertex_load(true);
    assert_eq!(edge_b, vec![0.0, 0.0, 0.0, 0.0]);

    // https://www.researchgate.net/publication/304065361_A_computationally_lightweight_and_localized_centrality_metric_in_lieu_of_betweenness_centrality_for_complex_network_analysis/figures?lo=1
    let mut graph: Graph<EmptyNode> = Graph::new(8);
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(0, 2).unwrap();
    graph.add_edge(2, 1).unwrap();
    graph.add_edge(5, 1).unwrap();
    graph.add_edge(2, 3).unwrap();
    graph.add_edge(2, 4).unwrap();
    graph.add_edge(4, 3).unwrap();
    graph.add_edge(5, 3).unwrap();
    graph.add_edge(4, 5).unwrap();
    graph.add_edge(4, 7).unwrap();
    graph.add_edge(4, 6).unwrap();
    graph.add_edge(5, 7).unwrap();
    graph.add_edge(5, 6).unwrap();
    graph.add_edge(6, 7).unwrap();
    let edge_b = graph.vertex_load(false);
    println!("edge_betweenness: {:?}", edge_b);
    assert_eq!(edge_b, vec![0.0, 4.666666666666666, 8.0, 0.6666666666666665, 8.666666666666668, 10.0, 0.0, 0.0]);
}

#[test]
fn transitivity() {
    let graph: Graph<EmptyNode> = Graph::new(3);
    assert!(graph.transitivity().is_nan());

    let mut graph: Graph<EmptyNode> = Graph::new(3);
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(0, 2).unwrap();

    assert_eq!(0.0, graph.transitivity());

    graph.add_edge(2, 1).unwrap();

    assert_eq!(1.0, graph.transitivity());

    graph = Graph::new(6);
    graph.add_edge(0,1).unwrap();
    graph.add_edge(1,2).unwrap();
    graph.add_edge(2,3).unwrap();
    graph.add_edge(3,4).unwrap();
    graph.add_edge(4,0).unwrap();
    graph.add_edge(4,1).unwrap();
    graph.add_edge(3,5).unwrap();

    assert_eq!(3.0 / 11.0, graph.transitivity());

    graph = Graph::new(6);
    graph.add_edge(1,2).unwrap();
    graph.add_edge(2,3).unwrap();
    graph.add_edge(3,4).unwrap();
    graph.add_edge(4,5).unwrap();
    graph.add_edge(5,1).unwrap();
    graph.add_edge(5,2).unwrap();
    graph.add_edge(4,0).unwrap();

    assert_eq!(3.0 / 11.0, graph.transitivity());
}

#[test]
fn iter_neighbors() {
    let mut graph: Graph<EmptyNode> = Graph::new(6);

    for i in 0..graph.vertex_count() {
        let mut iter = graph.contained_iter_neighbors(i);
        assert!(iter.next().is_none());
    }

    for i in 0..graph.vertex_count() {
        let mut iter = graph.container_iter_neighbors(i);
        assert!(iter.next().is_none());
    }

    for i in 0..6 {
        graph.add_edge(i, (i + 1) %  6).unwrap();
    }

    for i in 0..graph.vertex_count() {
        let iter = graph.contained_iter_neighbors(i);
        assert!(iter.len() == 2);
    }

    for i in 0..graph.vertex_count() {
        let mut iter = graph.container_iter_neighbors(i);
        assert!(iter.len() == 2);
        let next = iter.next();
        let id = next.unwrap().id();
        assert!(
            id == (i + 5) % 6 ||
            id == (i + 1) % 6
        );

    }

    graph.sort_adj();

    for i in 0..graph.vertex_count() {
        let mut iter = graph.container_iter_neighbors(i);
        assert!(iter.len() == 2);
        assert!(
            iter.next().unwrap().id()
            <
            iter.next().unwrap().id()
        );

        let mut iter2 = graph.container_iter_neighbors(i).rev();
        assert!(
            iter2.next().unwrap().id()
            >
            iter2.next().unwrap().id()
        )
    }

}

#[test]
#[should_panic]
fn iter_panic() {
    let graph: Graph<EmptyNode> = Graph::new(6);
    graph.contained_iter_neighbors(6);
}
