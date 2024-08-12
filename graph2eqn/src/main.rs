use serde::Deserialize;
use rustc_hash::FxHashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

// Define Data Structures
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

fn is_cyclic_graph(nodes: &FxHashMap<String, Node>) -> bool {
    let mut visited = FxHashMap::default();
    let mut rec_stack = FxHashMap::default();

    for node_id in nodes.keys() {
        if !visited.contains_key(node_id) {
            let cyclic_nodes = is_cyclic_util(nodes, node_id, &mut visited, &mut rec_stack);
            if !cyclic_nodes.is_empty() {
                for node in cyclic_nodes {
                    println!("{}", node);
                }
                return true;
            }
        }
    }

    false
}

fn is_cyclic_util(
    nodes: &FxHashMap<String, Node>,
    node_id: &str,
    visited: &mut FxHashMap<String, bool>,
    rec_stack: &mut FxHashMap<String, bool>,
) -> Vec<String> {
    visited.insert(node_id.to_string(), true);
    rec_stack.insert(node_id.to_string(), true);

    let mut cyclic_nodes = Vec::new();

    if let Some(node) = nodes.get(node_id) {
        for child_id in &node.children {
            if !visited.get(child_id).unwrap_or(&false) {
                let mut child_cyclic_nodes = is_cyclic_util(nodes, child_id, visited, rec_stack);
                if !child_cyclic_nodes.is_empty() {
                    cyclic_nodes.push(child_id.to_string());
                    cyclic_nodes.append(&mut child_cyclic_nodes);
                    return cyclic_nodes;
                }
            } else if *rec_stack.get(child_id).unwrap_or(&false) {
                cyclic_nodes.push(child_id.to_string());
                return cyclic_nodes;
            }
        }
    }

    rec_stack.remove(node_id);
    cyclic_nodes
}

fn parse_json(json_str: &str) -> Graph {
    serde_json::from_str(json_str).expect("JSON was not well-formatted")
}

fn string_to_unique_id(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn dag_to_equations_small(
    nodes: &FxHashMap<String, Node>,
    node_id: &str,
    visited: &mut FxHashMap<String, String>,
    visit_count: &mut FxHashMap<String, usize>,
) -> String {
    *visit_count.entry(node_id.to_string()).or_insert(0) += 1;

    if visited.contains_key(node_id) {
        return format!("new_n_{}", node_id);
    }

    let node = nodes.get(node_id).unwrap();
    let expression = match node.op.as_str() {
        "&" => {
            let operands: Vec<String> = node
                .children
                .iter()
                .map(|child_id| dag_to_equations_small(nodes, child_id, visited, visit_count))
                .collect();
            operands.join(" & ")
        }
        _ => {
            let operands: Vec<String> = node
                .children
                .iter()
                .map(|child_id| dag_to_equations_small(nodes, child_id, visited, visit_count))
                .collect();
            if operands.is_empty() {
                node.op.clone()
            } else if operands.len() == 1 {
                format!("{}({})", node.op, operands[0])
            } else {
                format!("({} {} {})", operands[0], node.op, operands[1])
            }
        }
    };

    if visit_count[node_id] > 1 && (expression.contains(" ") || expression.contains("(")) {
        visited.insert(node_id.to_string(), expression.clone());
    }

    expression
}

fn dag_to_equations_large(
    nodes: &FxHashMap<String, Node>,
    node_id: &str,
    visited: &mut FxHashMap<String, String>,
    visit_count: &mut FxHashMap<String, usize>,
) -> String {
    *visit_count.entry(node_id.to_string()).or_insert(0) += 1;

    if visited.contains_key(node_id) {
        return format!("new_n_{}", node_id);
    }

    let node = nodes.get(node_id).unwrap();
    let expression = match node.op.as_str() {
        "&" => {
            node.children
                .iter()
                .map(|child_id| dag_to_equations_large(nodes, child_id, visited, visit_count))
                .collect::<Vec<_>>()
                .join(" & ")
        }
        _ => {
            let operands: Vec<String> = node
                .children
                .iter()
                .map(|child_id| dag_to_equations_large(nodes, child_id, visited, visit_count))
                .collect();
    
            match operands.len() {
                0 => node.op.clone(),
                1 => {
                    let operand = &operands[0];
                    if operand.len() > 50 {
                        let new_node_id = string_to_unique_id(operand);
                        visited.insert(new_node_id.to_string(), operand.clone());
                        format!("{}(new_n_{})", node.op, new_node_id)
                    } else {
                        format!("{}({})", node.op, operand)
                    }
                }
                2 => {
                    let lhs = &operands[0];
                    let rhs = &operands[1];
                    let (lhs_id, rhs_id) = (
                        if lhs.len() > 50 {
                            let id = string_to_unique_id(lhs);
                            visited.insert(id.to_string(), lhs.clone());
                            Some(id)
                        } else {
                            None
                        },
                        if rhs.len() > 50 {
                            let id = string_to_unique_id(rhs);
                            visited.insert(id.to_string(), rhs.clone());
                            Some(id)
                        } else {
                            None
                        },
                    );
    
                    format!(
                        "({} {} {})",
                        lhs_id.map_or_else(|| lhs.clone(), |id| format!("new_n_{}", id)),
                        node.op,
                        rhs_id.map_or_else(|| rhs.clone(), |id| format!("new_n_{}", id))
                    )
                }
                _ => unreachable!(),
            }
        }
    };

    if visit_count[node_id] > 1 && (expression.contains(" ") || expression.contains("(")) {
        visited.insert(node_id.to_string(), expression.clone());
    }

    expression
}

fn format_synopsys_single(equation: &str) -> Vec<String> {
    equation
        .split('&')
        .map(|part| part.trim().to_string())
        .collect()
}

fn read_prefix_mapping(file_path: &str) -> FxHashMap<String, String> {
    let file = File::open(file_path).expect("Unable to open file");
    let reader = BufReader::new(file);
    let mut mapping = FxHashMap::default();

    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        if line.starts_with("OUTORDER = ") {
            let parts: Vec<&str> = line
                .trim_start_matches("OUTORDER = ")
                .trim_end_matches(';')
                .split_whitespace()
                .collect();
            for (index, part) in parts.iter().enumerate() {
                mapping.insert(format!("p[{}]", index), part.to_string());
            }
            break;
        }
    }

    mapping
}

fn write_to_file(
    variables: &Vec<String>,
    parts: Vec<String>,
    file_name: &str,
    f_prefix: &str,
    visited: FxHashMap<String, String>,
    prefix_mapping: &FxHashMap<String, String>,
) {
    let mut file = File::create(file_name).expect("Unable to create file");

    writeln!(file, "INORDER = {};", variables.join(" ")).expect("Unable to write to file");

    for (index, part) in parts.iter().enumerate() {
        let f_number = format!("{}[{}]", f_prefix, index);
        let mapped_prefix = prefix_mapping.get(&f_number).unwrap_or(&f_number);
        writeln!(file, "{} = {};", mapped_prefix, part).expect("Unable to write to file");
    }

    let outorder: Vec<String> = (0..parts.len())
        .map(|i| format!("{}[{}]", f_prefix, i))
        .map(|f_number| {
            prefix_mapping
                .get(&f_number)
                .unwrap_or(&f_number)
                .to_string()
        })
        .collect();

    writeln!(file, "OUTORDER = {};", outorder.join(" ")).expect("Unable to write to file");

    for (node_id, expr) in visited.iter() {
        writeln!(file, "new_n_{} = {};", node_id, expr).expect("Unable to write to file");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: <program> <path_to_input_json_file> <path_to_output_circuit_file> [mode] [check_cyclic]");
        println!("Mode can be 'small' or 'large'. If not specified, 'small' is used as default.");
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    let output_path = &args[2];
    let mode = if args.len() >= 4 { &args[3] } else { "large" };
    let check_cyclic = args.len() >= 5 && args[4] == "1";

    let mut file = File::open(file_path).expect("Unable to open file");
    let mut json_str = String::new();
    file.read_to_string(&mut json_str).expect("Unable to read file");

    let graph = parse_json(&json_str);

    if check_cyclic {
        println!("Checking for cyclic graph");
        if is_cyclic_graph(&graph.nodes) {
            println!("Error: The graph is cyclic.");
            return;
        }
    }

    let root_nodes = &graph.root_eclasses;
    let prefix_mapping = read_prefix_mapping("../e-rewriter/circuit0.eqn");

    for (i, root) in root_nodes.iter().enumerate() {
        let mut visited = FxHashMap::default();
        let mut visit_count = FxHashMap::default();

        //print the mode
        println!("Mode: {}", mode);
        
        let equation = match mode {
            "small" => dag_to_equations_small(&graph.nodes, root, &mut visited, &mut visit_count),
            "large" => dag_to_equations_large(&graph.nodes, root, &mut visited, &mut visit_count),
            _ => {
                println!("Invalid mode '{}'. Using 'small' mode as default.", mode);
                dag_to_equations_small(&graph.nodes, root, &mut visited, &mut visit_count)
            }
        };

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
            &format!("{}", output_path),
            "p",
            visited,
            &prefix_mapping,
        );
        
        println!("Finished graph to equation conversion for circuit {} using {} mode", i + 1, mode);
    }
}