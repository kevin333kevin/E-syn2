use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error as StdError;

#[derive(Debug, Deserialize, Serialize)]
struct GraphData {
    nodes: HashMap<String, Node>,
    root_eclasses: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Node {
    op: String,
    children: Vec<String>,
    eclass: String,
    cost: f64,
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
) -> Result<String, Box<dyn StdError>> {
    println!("Begin to process the extraction result - prepare for graph2eqn");

    println!("Processing JSON with choices...");
    let processed_json = process_json_with_choices(dag_cost_json, saturated_graph_json)?;

    println!("Simplifying keys...");
    let simplified_json = process_json_simplify_keys(&processed_json)?;

    println!("Updating root eclasses...");
    let final_json = update_root_eclasses(saturated_graph_json, &simplified_json)?;

    println!("Extraction result to eqn process completed successfully");
    Ok(final_json)
}

pub fn process_circuit_conversion(
    extraction_result: &crate::ExtractionResult,
    saturated_graph_json: &str,
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
    let result = extraction_result_to_eqn(dag_cost_json, saturated_graph_json);

    match &result {
        Ok(_) => println!("Circuit conversion completed successfully"),
        Err(e) => println!("Error in circuit conversion: {}", e),
    }

    result
}