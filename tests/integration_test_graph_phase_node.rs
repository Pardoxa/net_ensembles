use net_ensembles::*;
use std::fs::File;
use std::io::prelude::*;
use rand_pcg::Pcg64;
use rand::SeedableRng;
use rand::Rng;
mod common;
use common::{PhaseNode, equal_graphs};


#[cfg(feature = "serde_support")]
use serde_json;

#[test]
fn phase_test() {
    let mut graph: Graph<PhaseNode> = Graph::new(4);
    for i in 0..4 {
        graph.add_edge(i, (i + 1) % 4).unwrap();
    }

    for i in 0..4 {
        assert_eq!(
            graph.at(i).get_phase(),
            i as f64 * 10.0
        );
    }

    for i in 0..4 {
        graph.at_mut(i).set_phase(i as f64 * 0.5);
    }

    for i in 0..4 {
        assert_eq!(
            graph.at(i).get_phase(),
            i as f64 * 0.5
        );
    }
    #[allow(deprecated)]
    let dot = graph.to_dot_with_labels_from_contained(
        "",
        |contained, index| format!("Phase: {} at index {}", contained.get_phase(), index)
    );

    let mut read_in = File::open("TestData/phase_test.dot").expect("unable to open file");
    let mut test_data = String::new();
    read_in.read_to_string(&mut test_data).expect("unable to read file");
    assert_eq!(test_data, dot);
}

#[cfg(feature = "serde_support")]
#[test]
#[ignore]
fn graph_parsing_big_random() {
    graph_parsing_compare_random(23545635745, 1000);
}

#[cfg(feature = "serde_support")]
fn graph_parsing_compare_random(seed: u64, size: usize) {
    // now check with a random graph
    let mut rng = Pcg64::seed_from_u64(seed);
    let mut graph: Graph<PhaseNode> = Graph::new(size);
    for i in 0..size {
        for j in i+1..size {
            if rng.gen::<f64>() <= 0.6 {
                graph.add_edge(i, j).unwrap();
            }
        }
    }

    let s = serde_json::to_string(&graph).unwrap();
    let parsed_graph: Graph::<PhaseNode> = serde_json::from_str(&s).unwrap();
    // check, that graphs are equal
    for i in 0..size {
        assert_eq!(
            graph.at(i).get_phase(),
            parsed_graph.at(i).get_phase()
        );
    }

    equal_graphs(&graph, &parsed_graph);
}

#[test]
fn clone(){
    let mut rng = Pcg64::seed_from_u64(123174123);
    let mut graph: Graph<PhaseNode> = Graph::new(100);
    for i in 0..100 {
        for j in i+1..100 {
            if rng.gen::<f64>() <= 0.6 {
                graph.add_edge(i, j).unwrap();
            }
        }
    }

    let clone = graph.clone();
    for i in 0..100 {
        assert_eq!(
            graph.at(i).get_phase(),
            clone.at(i).get_phase()
        );
    }

    equal_graphs(&graph, &clone);
}
