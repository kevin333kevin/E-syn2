use serde::Deserialize;
use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::ExtractionResult;
use crate::EGraph;

#[derive(Deserialize, Debug)]
struct Node {
    op: String,
    children: Vec<String>,
    eclass: String,
    cost: f64,
}

#[derive(Deserialize, Debug)]
struct Graph {
    nodes: FxHashMap<String, Node>,
    root_eclasses: Vec<String>,
}

fn string_to_unique_id(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn dag_to_equations(
    nodes: &FxHashMap<String, Node>,
    node_id: &str,
    visited: &mut FxHashMap<String, String>,
    visit_count: &mut FxHashMap<String, usize>,
) -> String {
    // Implementation of dag_to_equations_large from graph2eqn
    // ...
}

fn format_synopsys_single(equation: &str) -> Vec<String> {
    equation
        .split('&')
        .map(|part| part.trim().to_string())
        .collect()
}

fn read_prefix_mapping(file_path: &str) -> FxHashMap<String, String> {
    // Implementation from graph2eqn
    // ...
}

pub fn extraction_result_to_eqn(
    extraction_result: &ExtractionResult,
    egraph: &EGraph,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let nodes: FxHashMap<String, Node> = extraction_result.choices.iter().map(|(key, value)| {
        let node = egraph.nodes.get(value).unwrap();
        (key.clone(), Node {
            op: node.op.clone(),
            children: node.children.iter().map(|c| c.to_string()).collect(),
            eclass: node.eclass.clone(),
            cost: node.cost,
        })
    }).collect();

    let graph = Graph {
        nodes,
        root_eclasses: egraph.root_eclasses.clone(),
    };

    let prefix_mapping = read_prefix_mapping("../e-rewriter/circuit0.eqn");

    for (i, root) in graph.root_eclasses.iter().enumerate() {
        let mut visited = FxHashMap::default();
        let mut visit_count = FxHashMap::default();
        
        let equation = dag_to_equations(&graph.nodes, root, &mut visited, &mut visit_count);

        let mut variables = vec![];
        for node in graph.nodes.values() {
            if node.children.is_empty() && !variables.contains(&node.op) && node.op != "1" && node.op != "0" {
                variables.push(node.op.clone());
            }
        }

        let parts = format_synopsys_single(&equation);

        write_to_file(
            &variables,
            parts,
            output_path,
            "p",
            visited,
            &prefix_mapping,
        )?;
    }

    Ok(())
}

fn write_to_file(
    variables: &Vec<String>,
    parts: Vec<String>,
    file_name: &str,
    f_prefix: &str,
    visited: FxHashMap<String, String>,
    prefix_mapping: &FxHashMap<String, String>,
) -> std::io::Result<()> {
    // Implementation from graph2eqn
    // ...
    Ok(())
}