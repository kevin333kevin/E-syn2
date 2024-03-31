use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error as StdError;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};


// Struct representing the graph data
// Contains a HashMap of nodes with string keys and Node values
#[derive(Debug, Deserialize, Serialize)]
struct GraphData {
    nodes: HashMap<String, Node>,
}

// Struct representing a node in the graph
// Contains fields for the operation (op), children nodes, equivalence class (eclass), and cost
#[derive(Debug, Deserialize, Serialize)]
struct Node {
    op: String,
    children: Vec<String>,
    eclass: String,
    cost: f32,
}

// Function to process a JSON file with choices
// Input:
//   - input_file: Path to the input JSON file
//   - output_dir: Path to the output directory
//   - a: An additional parameter (not used in this function)
// Output:
//   - Result<(), Box<dyn StdError>>: Returns Ok(()) if successful, or an error if encountered
fn process_json_with_choices(
    input_file: &str,
    output_dir: &str,
    a: u32,
) -> Result<(), Box<dyn StdError>> {
    // Read the JSON content from the input file
    let json_content = fs::read_to_string(input_file)?;
    let data: Value = serde_json::from_str(&json_content)?;

    // Extract the choices from the JSON data
    let choices: HashMap<String, String> = serde_json::from_value(data["choices"].clone())?;
    let values: HashSet<&str> = choices.values().map(|v| v.as_str()).collect();

    // Get the current directory and parent directory
    let current_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let parent_dir = current_dir.parent().unwrap();
    let input_dir = parent_dir.join("extraction-gym/input");

    // Find all JSON files in the input directory
    let json_files: Vec<PathBuf> = input_dir
        .read_dir()?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension().map(|ext| ext == "json").unwrap_or(false) {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // Get the first JSON file (assuming there is only one)
    let graph_file = json_files.get(0).ok_or("No JSON file found")?;
    let graph_content = fs::read_to_string(graph_file)?;
    let graph_data: GraphData = serde_json::from_str(&graph_content)?;

    // Filter the nodes based on the choices
    let new_nodes: HashMap<String, Node> = graph_data
        .nodes
        .into_iter()
        .filter(|(key, _)| values.contains(key.as_str()))
        .collect();

    // Create the result JSON object
    let result = serde_json::json!({
        "nodes": new_nodes,
    });

    // Get the file name from the input file path
    let file_name = Path::new(input_file).file_name().unwrap().to_str().unwrap();

    // Create the output file path
    let output_file = Path::new(output_dir).join(file_name).with_extension("json");
    let output_content = serde_json::to_string_pretty(&result)?;

    // Write the result to the output file
    fs::write(output_file, output_content)?;

    Ok(())
}

// Function to process a JSON file and simplify the keys
// Input:
//   - input_file: Path to the input JSON file
//   - output_dir: Path to the output directory
//   - a: An additional parameter (not used in this function)
// Output:
//   - Result<(), Box<dyn StdError>>: Returns Ok(()) if successful, or an error if encountered
fn process_json_simplify_keys(
    input_file: &str,
    output_dir: &str,
    a: u32,
) -> Result<(), Box<dyn StdError>> {
    // Read the JSON content from the input file
    let json_content = fs::read_to_string(input_file)?;
    let data: GraphData = serde_json::from_str(&json_content)?;

    // Create a new HashMap to store the simplified nodes
    let mut new_nodes: HashMap<String, Node> = HashMap::new();

    // Iterate over the nodes and simplify the keys
    for (key, mut value) in data.nodes {
        let new_key = key.split('.').next().unwrap().to_string();
        value.children = value
            .children
            .iter()
            .map(|child| child.split('.').next().unwrap().to_string())
            .collect();
        new_nodes.insert(new_key, value);
    }

    // Create the result GraphData with the simplified nodes
    let result = GraphData { nodes: new_nodes };

    // Get the file name from the input file path
    let file_name = Path::new(input_file)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid input file name"))?;

    // Create the output file path
    let output_file = Path::new(output_dir).join(file_name).with_extension("json");
    let output_content = serde_json::to_string_pretty(&result)?;
    let mut output_file = fs::File::create(output_file)?;

    // Write the result to the output file
    output_file.write_all(output_content.as_bytes())?;

    Ok(())
}

// Function to process files in a directory
// Input:
//   - input_dir: Path to the input directory
//   - output_dir: Path to the output directory
//   - process_func: Function to process each file (either process_json_with_choices or process_json_simplify_keys)
//   - a: An additional parameter to pass to the processing function
// Output:
//   - None
fn process_files_in_directory(
    input_dir: &Path,
    output_dir: &str,
    process_func: fn(&str, &str, u32) -> Result<(), Box<dyn StdError>>,
    a: u32,
) {
    // Create the output directory if it doesn't exist
    fs::create_dir_all(output_dir)
        .unwrap_or_else(|_| panic!("Failed to create output directory: {:?}", output_dir));

    // Read the files in the input directory
    if let Ok(entries) = fs::read_dir(input_dir) {
        // Collect the file paths
        let file_paths: Vec<PathBuf> = entries
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        Some(path)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // process file one by one
        for path in file_paths {
            // get the length of file_paths
            // let len = file_paths.len();
            // println!("Processing file {}/{}", len, len);
            let input_file = path.to_str().expect("Invalid input file path");

            // Rename the file to have a .json extension
            let mut new_path = PathBuf::from(input_file);
            new_path.set_extension("json");

            if let Err(err) = fs::rename(&path, &new_path) {
                println!("Failed to rename file: {:?}", err);
                continue;
            }

            // Process the file using the provided processing function
            if let Err(err) = process_func(new_path.to_str().unwrap(), output_dir, a) {
                eprintln!(
                    "Error processing file {}: {}",
                    new_path.to_str().unwrap(),
                    err
                );
            }
        }
    } else {
        println!("Failed to read directory: {:?}", input_dir);
    }
}

// Function to update root eclasses in the output files
// Input:
//   - graph_file: Path to the graph file containing the root eclasses
//   - output_dir: Path to the output directory containing the files to update
// Output:
//   - None
fn update_root_eclasses(graph_file: &Path, output_dir: &Path) {
    // print the output directory
    //println!("Output directory: {:?}", output_dir);
    // Read the graph data from the graph file
    let mut source_data = String::new();

    //println!("Graph file: {:?}", graph_file);

    File::open(graph_file)
        .unwrap()
        .read_to_string(&mut source_data)
        .unwrap();
    let source_data: Value = serde_json::from_str(&source_data).unwrap();

    // Extract the root eclasses from the graph data
    let root_eclasses = if let Some(root_eclasses) = source_data["root_eclasses"].as_array() {
        root_eclasses.to_owned()
    } else {
        vec![]
    };

    // Iterate over the files in the output directory
    for entry in fs::read_dir(output_dir).unwrap() {
        let entry = entry.unwrap();
        let target_file_path = entry.path();

        // Check if the file is a JSON file
        if target_file_path.is_file() && target_file_path.extension().unwrap_or_default() == "json"
        {
            // Read the target file data
            let mut target_data = String::new();
            File::open(&target_file_path)
                .unwrap()
                .read_to_string(&mut target_data)
                .unwrap();
            let mut target_data: Value = serde_json::from_str(&target_data).unwrap();

            // Update the root eclasses in the target data
            target_data["root_eclasses"] = json!(root_eclasses);

            // Write the updated data back to the target file
            File::create(&target_file_path)
                .unwrap()
                .write_all(
                    serde_json::to_string_pretty(&target_data)
                        .unwrap()
                        .as_bytes(),
                )
                .unwrap();
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "process_json")]
#[command(version)]
#[command(about = "Process extracted graphs based on user input", long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Sets the input graph file",
        required = true,
    )]
    graph_file: String,
    #[arg(
        short,
        long,
        value_name = "DIR",
        help = "Sets the input extraction result directory",
        required = true,
    )]
    extraction_result_dir: String,
    #[arg(
        short,
        long,
        help = "Extracts the graph based on DAG or tree-based extraction",
    )]
    extract_dag: bool,
}

fn main() {
    let args = Args::parse();

    let input_saturacted_graph_file = Path::new(&args.graph_file);
    let input_extraction_result_dir = &args.extraction_result_dir;
    let dag_based = args.extract_dag;

    // print whether it is DAG-based or not
    println!("DAG-based: {}", dag_based);

    // Get the current directory and parent directory
    let current_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let parent_dir = current_dir.parent().unwrap();

    if dag_based {
        println!("Processing DAG-based extracted graphs...");
        // Process files in out_dag_json directory
        let out_dag_json_dir = Path::new(input_extraction_result_dir);
        let out_process_dag_result_dir = "out_process_dag_result";
        process_files_in_directory(
            &out_dag_json_dir,
            out_process_dag_result_dir,
            process_json_with_choices,
            0,
        ); // a=1 to use choices, a=0 to use DAG

        // Process files in out_process_dag_result directory
        let out_process_dag_result_dir = parent_dir.join("process_json/out_process_dag_result");
        process_files_in_directory(
            &out_process_dag_result_dir,
            out_process_dag_result_dir.to_str().unwrap(),
            process_json_simplify_keys,
            0,
        ); // a=1 to use choices, a=0 to use DAG

        // Update root eclasses in the output files for dag_based
        update_root_eclasses(
            &input_saturacted_graph_file,
            &parent_dir.join("process_json/out_process_dag_result"),
        );
    } else {
        // Process files in out_json directory
        let out_json_dir = Path::new(input_extraction_result_dir);
        let out_process_result_dir = "out_process_result";
        process_files_in_directory(
            &out_json_dir,
            out_process_result_dir,
            process_json_with_choices,
            1,
        ); // a=1 to use choices, a=0 to use DAG

        // Process files in out_process_result directory
        let out_process_result_dir = parent_dir.join("process_json/out_process_result");
        process_files_in_directory(
            &out_process_result_dir,
            out_process_result_dir.to_str().unwrap(),
            process_json_simplify_keys,
            1,
        ); // a=1 to use choices, a=0 to use DAG

        // Update root eclasses in the output files for non-dag_based
        update_root_eclasses(
            &input_saturacted_graph_file,
            &parent_dir.join("process_json/out_process_result"),
        );
    }
}
