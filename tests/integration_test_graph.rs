
use net_ensembles::*;
use std::fs::File;
use std::io::prelude::*;

fn create_graph_1() -> Graph<TestNode> {
    let mut graph: Graph<TestNode> = Graph::new(20);

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

    let empty_graph: Graph<TestNode> = Graph::new(20);
    assert_eq!(0, empty_graph.leaf_count());
}

#[test]
fn connected_components() {
    let graph: Graph<TestNode> = Graph::new(20);
    assert_eq!(vec![1;20], graph.connected_components());

    let graph2: Graph<TestNode> = Graph::new(0);
    assert_eq!(Vec::<u32>::new(), graph2.connected_components());
}

#[test]
fn multiple_connected_components() {
    let graph = create_graph_1();
    let components = graph.connected_components();
    assert_eq!(components[0], 6);
    assert_eq!(components[1], 4);
    assert_eq!(components[2], 3);
    for i in 3..components.len() {
        assert_eq!(components[i], 1);
    }
}

#[test]
fn q_core_empty_graph() {
    let graph: Graph<TestNode> = Graph::new(0);
    assert_eq!(graph.q_core(1), None);
    assert_eq!(graph.q_core(2), None);

    let graph2: Graph<TestNode> = Graph::new(1);

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
    let mut graph: Graph<TestNode> = Graph::new(20);
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
    let mut graph: Graph<TestNode> = Graph::new(20);
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(0, 2).unwrap();
    graph.add_edge(1, 2).unwrap();
    assert_eq!(graph.q_core(2), Some(3));
}

#[test]
fn check_is_connected() {
    let mut graph: Graph<TestNode> = Graph::new(10);
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
fn dot_labeled() {
    let graph = create_graph_1();
    let s = graph.to_dot_with_labels(DEFAULT_DOT_OPTIONS, |_, index| format!("Hey {}!", index));
    let mut read_in = File::open("TestData/label_test.dot").expect("unable to open file");
    let mut data = String::new();
    // let mut f = File::create("label_test.dot").expect("Unable to create file");
    // f.write_all(s.as_bytes()).expect("Unable to write data");
    read_in.read_to_string(&mut data).expect("unable to read file");
    assert_eq!(data, s);
}

#[test]
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
    let mut graph: Graph<TestNode> = Graph::new(20);
    // create complete graph
    for i in 0..graph.vertex_count() {
        for j in i+1..graph.vertex_count() {
            graph.add_edge(i, j).unwrap();
        }
    }
    assert_eq!(graph.average_neighbor_count(), 19.0);


    let empty: Graph<TestNode> = Graph::new(20);

    assert_eq!(empty.average_neighbor_count(), 0.0);
}

#[test]
fn diameter_test() {
    let mut graph: Graph<TestNode> = Graph::new(5);
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
    let mut graph: Graph<TestNode> = Graph::new(6);
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
    let graph: Graph<TestNode> = Graph::new(6);

    let components = graph.vertex_biconnected_components(false);
    assert_eq!(components, vec![]);

    let mut graph: Graph<TestNode> = Graph::new(6);
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(1, 2).unwrap();
    graph.add_edge(2, 0).unwrap();

    let components = graph.vertex_biconnected_components(false);
    assert_eq!(components, vec![3]);

    let mut graph: Graph<TestNode> = Graph::new(5);
    graph.add_edge(0, 1).unwrap();
    graph.add_edge(1, 2).unwrap();
    graph.add_edge(2, 0).unwrap();
    graph.add_edge(2, 3).unwrap();
    graph.add_edge(4, 3).unwrap();
    graph.add_edge(4, 0).unwrap();
    assert_eq!(vec![5], graph.vertex_biconnected_components(false));
}
