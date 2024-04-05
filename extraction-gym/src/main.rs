// Module for extraction-related functionality
mod extract;

// Re-export the contents of the `extract` module
pub use extract::*;

// Import necessary dependencies
use egraph_serialize::*;

use anyhow::Context;
use indexmap::IndexMap;
use ordered_float::NotNan;
use serde_json::to_string_pretty;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

// Define a type alias for the cost value
pub type Cost = NotNan<f64>;

// Define a constant for infinity cost
pub const INFINITY: Cost = unsafe { NotNan::new_unchecked(std::f64::INFINITY) };

// Function to get the fast extractors
// Returns: An `IndexMap` mapping extractor names to their corresponding `Extractor` implementations
fn get_fast_extractors() -> IndexMap<&'static str, Box<dyn Extractor>> {
    [
        ("bottom-up", extract::bottom_up::BottomUpExtractor.boxed()),
        (
            "faster-bottom-up",
            extract::faster_bottom_up::FasterBottomUpExtractor.boxed(),
        ),
        (
            "greedy-dag",
            extract::greedy_dag::GreedyDagExtractor.boxed(),
        ),
        (
            "faster-greedy-dag",
            extract::faster_greedy_dag::FasterGreedyDagExtractor.boxed(),
        ),
        (
            "global-greedy-dag",
            extract::global_greedy_dag::GlobalGreedyDagExtractor.boxed(),
        ),
    ]
    .into_iter()
    .enumerate()
    //.filter(|(index, _)| *index == 1)
    .map(|(_, item)| item)
    .collect()
}

// Function to get the extractor name from the command-line arguments
// Input: A mutable reference to the `pico_args::Arguments` instance
// Returns: The extractor name as a `String`, defaulting to "bottom-up" if not provided
fn get_extractor_name(args: &mut pico_args::Arguments) -> String {
    args.opt_value_from_str("--extractor")
        .unwrap()
        .unwrap_or_else(|| "faster-bottom-up".into())
}

// Function to print the extractor names
// Input: A reference to the `IndexMap` of extractors
fn print_extractor_names(extractors: &IndexMap<&str, Box<dyn Extractor>>) {
    for name in extractors.keys() {
        println!("{}", name);
    }
}

// Function to get the cost function from the command-line arguments
// Input: A mutable reference to the `pico_args::Arguments` instance
// Returns: The cost function as a `String`, defaulting to "node_depth_cost" if not provided
fn get_cost_function(args: &mut pico_args::Arguments) -> String {
    args.opt_value_from_str("--cost-function")
        .unwrap()
        .unwrap_or_else(|| "node_depth_cost".into())
}

// Function to get the output filename from the command-line arguments
// Input: A mutable reference to the `pico_args::Arguments` instance
// Returns: The output filename as a `PathBuf`, defaulting to "out.json" if not provided
fn get_output_filename(args: &mut pico_args::Arguments) -> PathBuf {
    args.opt_value_from_str("--out")
        .unwrap()
        .unwrap_or_else(|| "out.json".into())
}

// Function to get the input filename from the command-line arguments
// Input: A mutable reference to the `pico_args::Arguments` instance
// Returns: The input filename as a `String`
fn get_input_filename(args: &mut pico_args::Arguments) -> String {
    args.free_from_str().unwrap()
}

// Function to modify a filename by replacing a prefix
// Input:
//   - `filename`: The original filename
//   - `old_prefix`: The prefix to replace
//   - `new_prefix`: The new prefix to use
// Returns: The modified filename as a `String`
fn modify_filename(filename: &str, old_prefix: &str, new_prefix: &str) -> String {
    filename.replacen(old_prefix, new_prefix, 1)
}

// Function to parse an e-graph from a JSON file
// Input: The filename of the JSON file
// Returns: The parsed `EGraph` instance
fn parse_egraph(filename: &str) -> EGraph {
    EGraph::from_json_file(filename)
        .with_context(|| format!("Failed to parse {filename}"))
        .unwrap()
}

// Function to get the extractor based on the extractor name
// Input:
//   - `extractors`: A reference to the `IndexMap` of extractors
//   - `extractor_name`: The name of the extractor to retrieve
// Returns: A reference to the `Box<dyn Extractor>` corresponding to the extractor name
fn get_extractor<'a>(
    extractors: &'a IndexMap<&str, Box<dyn Extractor>>,
    extractor_name: &str,
) -> &'a Box<dyn Extractor> {
    // print all extractors
    // for name in extractors.keys() {
    //     println!("{}", name);
    // }
    extractors
        .get(extractor_name)
        .with_context(|| format!("Unknown extractor: {extractor_name}"))
        .unwrap()
}

// Function to format a modified filename with the extractor name
// Input:
//   - `modified_filename`: The modified filename
//   - `extractor_name`: The name of the extractor
// Returns: The formatted filename as a `String`
fn format_modified_name(modified_filename: &str, extractor_name: &str) -> String {
    format!(
        "{}_{}.json",
        &modified_filename[..modified_filename.len() - 5],
        extractor_name,
    )
}

// Function to extract the result using the selected extractor
// Input:
//   - `extractor`: A reference to the `Box<dyn Extractor>` representing the extractor
//   - `egraph`: A reference to the `EGraph` instance
//   - `root_eclasses`: A reference to the root e-classes
//   - `cost_function`: The cost function to use
// Returns: The `ExtractionResult` obtained from the extraction process
fn extract_result(
    extractor: &Box<dyn Extractor>,
    egraph: &EGraph,
    root_eclasses: &[ClassId],
    cost_function: &str,
) -> ExtractionResult {
    extractor.extract(egraph, root_eclasses, cost_function)
}

// Function to print the DAG cost
// Input: The DAG cost as a `Cost` value
fn print_dag_cost(dag_cost: Cost) {
    print!("-------------------------------------------\n");
    print!("dag cost: {}\n", dag_cost);
    print!("-------------------------------------------\n");
}

// Function to write a JSON result to a file
// Input:
//   - `filename`: The filename to write the JSON result to
//   - `data`: A reference to the data to serialize and write as JSON
fn write_json_result<T: serde::Serialize>(filename: &str, data: &T) {
    let json_result = to_string_pretty(data).unwrap();
    let _ = fs::create_dir_all("out_json");
    let __ = fs::write(filename, json_result);
}

// Function to log the result
// Input:
//   - `filename`: The filename associated with the result
//   - `extractor_name`: The name of the extractor used
//   - `dag_cost`: The DAG cost
//   - `us`: The elapsed time in microseconds
fn log_result(filename: &str, extractor_name: &str, dag_cost: Cost, us: u128) {
    log::info!("{filename:40}\t{extractor_name:10}\t{dag_cost:5}\t{us:5}");
}

// Function to write the result to the output file
// Input:
//   - `out_file`: A mutable reference to the output file
//   - `filename`: The filename associated with the result
//   - `modified_name1`: The modified filename
//   - `extractor_name`: The name of the extractor used
//   - `dag_cost`: The DAG cost
//   - `us`: The elapsed time in microseconds
fn write_output_file(
    out_file: &mut File,
    filename: &str,
    modified_name1: &str,
    extractor_name: &str,
    dag_cost: Cost,
    us: u128,
) {
    writeln!(
        out_file,
        r#"{{ 
    "name": "{filename}",
    "md_name": "{modified_name1}",
    "extractor": "{extractor_name}", 
    "dag": {dag_cost}, 
    "micros": {us}
}}"#
    )
    .unwrap();
}

// Main function
fn main() {
    // Initialize the logger
    env_logger::init();

    // Get the fast extractors
    let extractors = get_fast_extractors();

    // Parse command-line arguments
    let mut args = pico_args::Arguments::from_env();

    // Get the extractor name from the arguments
    let extractor_name = get_extractor_name(&mut args);
    if extractor_name == "print" {
        // Print the extractor names and exit
        print_extractor_names(&extractors);
        return;
    }

    // Get the cost function from the arguments
    let cost_function = get_cost_function(&mut args);
    // Get the output filename from the arguments
    let out_filename = get_output_filename(&mut args);
    // Get the input filename from the arguments
    let filename = get_input_filename(&mut args);

    // Modify the filename for JSON output
    let modified_filename_for_tree_cost = modify_filename(&filename, "input/", "out_json/");
    let modified_filename_for_dag_cost = modify_filename(&filename, "input/", "out_dag_json/");

    // Check for any remaining arguments
    let rest = args.finish();
    if !rest.is_empty() {
        panic!("Unknown arguments: {:?}", rest);
    }

    // Create the output file
    let mut out_file = std::fs::File::create(out_filename.clone()).unwrap();

    // Parse the e-graph from the input file
    let egraph = parse_egraph(&filename);

    // Get the extractor based on the extractor name
    let extractor = get_extractor(&extractors, &extractor_name);

    // Format the modified filename with the extractor name
    let modified_name_for_tree_cost = format_modified_name(&modified_filename_for_tree_cost, &extractor_name);
    let modified_name_for_dag_cost = format_modified_name(&modified_filename_for_dag_cost, &extractor_name);

    // Record the start time
    let start_time = std::time::Instant::now();

    // Extract the result using the selected extractor
    let tree_cost_extraction_result = extract_result(extractor, &egraph, &egraph.root_eclasses, &cost_function);

    // Calculate the elapsed time in microseconds
    let us = start_time.elapsed().as_micros();
    // Assert that the result has no cycles
    assert!(tree_cost_extraction_result
        .find_cycles(&egraph, &egraph.root_eclasses)
        .is_empty());

    // Calculate the DAG cost and the DAG cost with extraction result
    let (dag_cost, dag_cost_extraction_result) =
        tree_cost_extraction_result.calculate_dag_cost_with_extraction_result(&egraph, &egraph.root_eclasses);
    // Print the DAG cost
    print_dag_cost(dag_cost);

    // Record random costs based on the extraction result
    tree_cost_extraction_result.record_costs_random(10, 0.5, &egraph, &dag_cost_extraction_result);

    // Write the JSON result to files
    write_json_result(&modified_name_for_tree_cost, &tree_cost_extraction_result);
    write_json_result(&modified_name_for_dag_cost, &dag_cost_extraction_result);

    // Log the result
    log_result(&filename, &extractor_name, dag_cost, us);
    // Write the result to the output file (log file)
    write_output_file(
        &mut out_file,
        &filename,
        &modified_name_for_dag_cost,
        &extractor_name,
        dag_cost,
        us,
    );

    // print time consumption of tree-based extraction as seconds
    println!("Time consumption of tree-based extraction: {} seconds", us as f64 / 1000000.0);
}
