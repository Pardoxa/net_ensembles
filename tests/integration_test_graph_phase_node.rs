use net_ensembles::*;
use std::fs::File;
use std::io::prelude::*;
use rand_pcg::Pcg64;
use rand_core::SeedableRng;
use rand::Rng;


fn equal_graphs<T: Node>(g1: Graph<T>, g2: Graph<T>) {
    assert_eq!(g1.edge_count(), g2.edge_count());
    assert_eq!(g1.vertex_count(), g2.vertex_count());
    for (n0, n1) in g2.node_container_iter().zip(g1.node_container_iter()) {
        assert_eq!(n1.get_id(), n1.get_id());
        assert_eq!(n1.neighbor_count(), n1.neighbor_count());

        for (i, j) in n1.neighbors().zip(n0.neighbors()) {
            assert_eq!(i, j);
        }
    }
}


#[derive(Debug, Clone)]
pub struct PhaseNode {phase: f64,}

impl PhaseNode {
    pub fn set_phase(&mut self, phase: f64) {
        self.phase = phase;
    }

    pub fn get_phase(&self) -> f64 {
        self.phase
    }
}

impl Node for PhaseNode {
    fn new_from_index(index: u32) -> Self {
        PhaseNode { phase: 10.0 * index as f64 }
    }

    fn make_string(&self) -> Option<String> {
        Some(format!("phase: {},", self.phase))
    }

    /// Override this, if you want to load the stored the network
    fn parse_str(to_parse: &str) -> Option<(&str, Self)>
        where Self: Sized
    {
        let identifier = "phase: ";
        let mut split_index = to_parse.find(identifier)?;
        split_index += identifier.len();
        let remaining_to_parse = &to_parse[split_index..];

        split_index = remaining_to_parse.find(",")?;
        let (phase_str, mut remaining_to_parse) = remaining_to_parse.split_at(split_index);
        remaining_to_parse = &remaining_to_parse[1..];
        let phase = phase_str.parse::<f64>().ok()?;
        let node = PhaseNode{ phase };

        Some((remaining_to_parse, node))
    }
}


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

    let dot = graph.to_dot_with_labels(
        "",
        |contained, index| format!("Phase: {} at index {}", contained.get_phase(), index)
    );

    let mut read_in = File::open("TestData/phase_test.dot").expect("unable to open file");
    let mut test_data = String::new();
    read_in.read_to_string(&mut test_data).expect("unable to read file");
    assert_eq!(test_data, dot);
}


#[test]
fn graph_parsing() {
    let mut graph: Graph<PhaseNode> = Graph::new(4);
    for i in 0..4 {
        graph.add_edge(i, (i + 1) % 4).unwrap();
    }

    println!("{}", graph);
    let g = graph.to_string();
    let try_parse = Graph::<PhaseNode>::parse_str(&g);

    let (_, parsed_graph) = try_parse.unwrap();

    println!("parsed: {}", parsed_graph);

    // check, that graphs are equal
    equal_graphs(graph, parsed_graph);

    // bigger graph
    let mut graph: Graph<PhaseNode> = Graph::new(40);
    for i in 0..40 {
        for j in i+1..40 {
            graph.add_edge(i, j).unwrap();
        }
    }

    let s = graph.to_string();
    let try_parse = Graph::<PhaseNode>::parse_str(&s);
    let (_, parsed_graph) = try_parse.unwrap();
    // check, that graphs are equal
    equal_graphs(graph, parsed_graph);

    graph_parsing_compare_random(232, 30);
}

#[test]
#[ignore]
fn graph_parsing_big_random() {
    graph_parsing_compare_random(23545635745, 1000);
}

fn graph_parsing_compare_random(seed: u64, size: u32) {
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

    let s = graph.to_string();
    let try_parse = Graph::<PhaseNode>::parse_str(&s);
    let (_, parsed_graph) = try_parse.unwrap();
    // check, that graphs are equal
    for i in 0..size as usize {
        assert_eq!(
            graph.at(i).get_phase(),
            parsed_graph.at(i).get_phase()
        );
    }

    equal_graphs(graph, parsed_graph);
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
    for i in 0..100 as usize {
        assert_eq!(
            graph.at(i).get_phase(),
            clone.at(i).get_phase()
        );
    }

    equal_graphs(graph, clone);
}
