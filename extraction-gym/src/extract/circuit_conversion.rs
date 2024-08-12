use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error as StdError;
use rustc_hash::FxHashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};



#[derive(Debug, Deserialize, Serialize)]
struct GraphData {
    nodes: HashMap<String, Node>,
    root_eclasses: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Graph {
    nodes: FxHashMap<String, Node>,
    root_eclasses: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Node {
    op: String,
    children: Vec<String>,
    eclass: String,
    cost: f64,
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

fn generate_eqn_content(
    variables: &Vec<String>,
    parts: Vec<String>,
    f_prefix: &str,
    visited: FxHashMap<String, String>,
    prefix_mapping: &FxHashMap<String, String>,
) -> String {
    let mut content = String::new();

    content.push_str(&format!("INORDER = {};\n", variables.join(" ")));

    for (index, part) in parts.iter().enumerate() {
        let f_number = format!("{}[{}]", f_prefix, index);
        let mapped_prefix = prefix_mapping.get(&f_number).unwrap_or(&f_number);
        content.push_str(&format!("{} = {};\n", mapped_prefix, part));
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

    content.push_str(&format!("OUTORDER = {};\n", outorder.join(" ")));

    for (node_id, expr) in visited.iter() {
        content.push_str(&format!("new_n_{} = {};\n", node_id, expr));
    }

    content
}

fn json_to_eqn(json_str: &str, prefix_mapping_path: &str, mode: &str) -> Result<String, Box<dyn StdError>> {
    let graph: Graph = serde_json::from_str(json_str)?;

    if is_cyclic_graph(&graph.nodes) {
        return Err("The graph is cyclic.".into());
    }

    let root_nodes = &graph.root_eclasses;
    let prefix_mapping = read_prefix_mapping(prefix_mapping_path);

    let mut final_content = String::new();

    for (i, root) in root_nodes.iter().enumerate() {
        let mut visited = FxHashMap::default();
        let mut visit_count = FxHashMap::default();

        println!("Mode: {}", mode);
        
        let equation = match mode {
            "small" => dag_to_equations_small(&graph.nodes, root, &mut visited, &mut visit_count),
            "large" => dag_to_equations_large(&graph.nodes, root, &mut visited, &mut visit_count),
            _ => {
                println!("Invalid mode '{}'. Using 'large' mode as default.", mode);
                dag_to_equations_large(&graph.nodes, root, &mut visited, &mut visit_count)
            }
        };

        let mut variables = vec![];
        for node in graph.nodes.values() {
            if node.children.is_empty() && !variables.contains(&node.op) && node.op != "1" && node.op != "0" {
                variables.push(node.op.clone());
            }
        }

        let parts = format_synopsys_single(&equation);

        let content = generate_eqn_content(
            &variables,
            parts,
            "p",
            visited,
            &prefix_mapping,
        );

        final_content.push_str(&content);
        final_content.push_str("\n");
        
        println!("Finished graph to equation conversion for circuit {} using {} mode", i + 1, mode);
    }

    Ok(final_content)
}


fn process_json_with_choices(
    extraction_result_json: &str,
    saturated_graph_json: &str,
) -> Result<String, Box<dyn StdError>> {
    println!("Processing extraction result with choices...");
    
    println!("Parsing extraction result JSON...");
    let extraction_data: Value = serde_json::from_str(extraction_result_json)
        .map_err(|e| {
            println!("Error parsing extraction result JSON: {}", e);
            println!("First 100 characters of extraction result JSON: {:?}", &extraction_result_json.chars().take(100).collect::<String>());
            e
        })?;

    println!("Parsing saturated graph JSON...");
    let saturated_graph: GraphData = serde_json::from_str(saturated_graph_json)
        .map_err(|e| {
            println!("Error parsing saturated graph JSON: {}", e);
            println!("First 100 characters of saturated graph JSON: {:?}", &saturated_graph_json.chars().take(100).collect::<String>());
            e
        })?;

    println!("Extracting choices from extraction result...");
    let choices: HashMap<String, String> = serde_json::from_value(extraction_data["choices"].clone())
        .map_err(|e| {
            println!("Error extracting choices: {}", e);
            println!("Choices data: {:?}", extraction_data["choices"]);
            e
        })?;

    let values: HashSet<&str> = choices.values().map(|v| v.as_str()).collect();

    println!("Filtering nodes based on choices...");
    let new_nodes: HashMap<String, Node> = saturated_graph
        .nodes
        .into_iter()
        .filter(|(key, _)| values.contains(key.as_str()))
        .collect();

    println!("Creating result JSON...");
    let result = serde_json::json!({
        "nodes": new_nodes,
    });

    println!("Serializing result to JSON string...");
    Ok(serde_json::to_string_pretty(&result)?)
}

fn process_json_simplify_keys(input_json: &str) -> Result<String, Box<dyn StdError>> {
    println!("Processing JSON to simplify keys...");
    
    println!("Parsing input JSON...");
    let data: Value = serde_json::from_str(input_json)
        .map_err(|e| {
            println!("Error parsing input JSON: {}", e);
            println!("First 100 characters of input JSON: {:?}", &input_json.chars().take(100).collect::<String>());
            e
        })?;

    println!("Simplifying keys...");
    let mut new_nodes: HashMap<String, Node> = HashMap::new();

    if let Some(nodes) = data["nodes"].as_object() {
        for (key, value) in nodes {
            let new_key = key.split('.').next().unwrap().to_string();
            let mut node: Node = serde_json::from_value(value.clone())?;
            node.children = node.children
                .iter()
                .map(|child| child.split('.').next().unwrap().to_string())
                .collect();
            new_nodes.insert(new_key, node);
        }
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Input JSON does not contain a 'nodes' object",
        )));
    }

    let result = serde_json::json!({ "nodes": new_nodes });
    
    println!("Serializing result to JSON string...");
    Ok(serde_json::to_string_pretty(&result)?)
}

fn update_root_eclasses(graph_json: &str, target_json: &str) -> Result<String, Box<dyn StdError>> {
    println!("Updating root eclasses...");

    println!("Parsing graph JSON...");
    let source_data: Value = serde_json::from_str(graph_json)
        .map_err(|e| {
            println!("Error parsing graph JSON: {}", e);
            println!("First 100 characters of graph JSON: {:?}", &graph_json.chars().take(100).collect::<String>());
            e
        })?;

    println!("Parsing target JSON...");
    let mut target_data: Value = serde_json::from_str(target_json)
        .map_err(|e| {
            println!("Error parsing target JSON: {}", e);
            println!("First 100 characters of target JSON: {:?}", &target_json.chars().take(100).collect::<String>());
            e
        })?;

    println!("Extracting root eclasses...");
    let root_eclasses = source_data["root_eclasses"].as_array().unwrap_or(&Vec::new()).to_owned();
    target_data["root_eclasses"] = serde_json::json!(root_eclasses);

    println!("Serializing updated JSON to string...");
    Ok(serde_json::to_string_pretty(&target_data)?)
}

pub fn extraction_result_to_eqn(
    dag_cost_json: &str,
    saturated_graph_json: &str,
    prefix_mapping_path: &str,
    mode: &str,
) -> Result<String, Box<dyn StdError>> {
    println!("Begin to process the extraction result - prepare for graph2eqn");

    println!("Processing JSON with choices...");
    let processed_json = process_json_with_choices(dag_cost_json, saturated_graph_json)?;

    println!("Simplifying keys...");
    let simplified_json = process_json_simplify_keys(&processed_json)?;

    println!("Updating root eclasses...");
    let final_json = update_root_eclasses(saturated_graph_json, &simplified_json)?;

    println!("Converting to EQN format...");
    let eqn_content = json_to_eqn(&final_json, prefix_mapping_path, mode)?;

    println!("Extraction result to eqn process completed successfully");
    Ok(eqn_content)
}


pub fn process_circuit_conversion(
    extraction_result: &crate::ExtractionResult,
    saturated_graph_json: &str,
    prefix_mapping_path: &str,
    mode: &str,
) -> Result<String, Box<dyn StdError>> {
    println!("Begin to process the extraction result - prepare for graph2eqn");
    
    println!("Extracting DAG cost JSON from ExtractionResult...");
    let dag_cost_json = extraction_result.dag_cost_json.as_ref()
        .ok_or_else(|| {
            let err = std::io::Error::new(std::io::ErrorKind::NotFound, "DAG cost JSON not found");
            println!("Error: DAG cost JSON not found in ExtractionResult");
            err
        })?;

    println!("DAG cost JSON extracted successfully");
    println!("First 100 characters of DAG cost JSON: {:?}", &dag_cost_json.chars().take(100).collect::<String>());

    println!("Calling extraction_result_to_eqn...");
    let result = extraction_result_to_eqn(dag_cost_json, saturated_graph_json, prefix_mapping_path, mode);

    match &result {
        Ok(_) => println!("Circuit conversion completed successfully"),
        Err(e) => println!("Error in circuit conversion: {}", e),
    }

    result
}