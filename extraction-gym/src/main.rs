mod extract;

pub use extract::*;

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

pub type Cost = NotNan<f64>;

pub const INFINITY: Cost = unsafe { NotNan::new_unchecked(std::f64::INFINITY) };

fn main() {
    env_logger::init();

    let extractors = get_fast_extractors();

    let mut args = pico_args::Arguments::from_env();

    let extractor_name = get_extractor_name(&mut args);
    if extractor_name == "print" {
        print_extractor_names(&extractors);
        return;
    }

    let cost_function = get_cost_function(&mut args);
    let out_filename = get_output_filename(&mut args);
    let filename = get_input_filename(&mut args);

    let modified_filename = modify_filename(&filename, "data/", "out_json/");
    let modified_filename1 = modify_filename(&filename, "data/", "out_dag_json/");

    let rest = args.finish();
    if !rest.is_empty() {
        panic!("Unknown arguments: {:?}", rest);
    }

    let mut out_file = std::fs::File::create(out_filename.clone()).unwrap();

    let egraph = parse_egraph(&filename);

    let extractor = get_extractor(&extractors, &extractor_name);

    let modified_name = format_modified_name(&modified_filename, &extractor_name);
    let modified_name1 = format_modified_name(&modified_filename1, &extractor_name);

    let start_time = std::time::Instant::now();

    let result = extract_result(extractor, &egraph, &egraph.root_eclasses, &cost_function);

    let us = start_time.elapsed().as_micros();
    assert!(result
        .find_cycles(&egraph, &egraph.root_eclasses)
        .is_empty());

    let (dag_cost, dag_cost_with_extraction_result) =
        result.dag_cost_with_extraction_result(&egraph, &egraph.root_eclasses);
    print_dag_cost(dag_cost);

    result.record_costs_random(10, 0.5, &egraph, &dag_cost_with_extraction_result);

    write_json_result(&modified_name, &result);
    write_json_result(&modified_name1, &dag_cost_with_extraction_result);

    log_result(&filename, &extractor_name, dag_cost, us);
    write_output_file(
        &mut out_file,
        &filename,
        &modified_name1,
        &extractor_name,
        dag_cost,
        us,
    );
}

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
    .filter(|(index, _)| *index == 1)
    .map(|(_, item)| item)
    .collect()
}

fn get_extractor_name(args: &mut pico_args::Arguments) -> String {
    args.opt_value_from_str("--extractor")
        .unwrap()
        .unwrap_or_else(|| "bottom-up".into())
}

fn print_extractor_names(extractors: &IndexMap<&str, Box<dyn Extractor>>) {
    for name in extractors.keys() {
        println!("{}", name);
    }
}

fn get_cost_function(args: &mut pico_args::Arguments) -> String {
    args.opt_value_from_str("--cost-function")
        .unwrap()
        .unwrap_or_else(|| "node_depth_cost".into())
}

fn get_output_filename(args: &mut pico_args::Arguments) -> PathBuf {
    args.opt_value_from_str("--out")
        .unwrap()
        .unwrap_or_else(|| "out.json".into())
}

fn get_input_filename(args: &mut pico_args::Arguments) -> String {
    args.free_from_str().unwrap()
}

fn modify_filename(filename: &str, old_prefix: &str, new_prefix: &str) -> String {
    filename.replacen(old_prefix, new_prefix, 1)
}

fn parse_egraph(filename: &str) -> EGraph {
    EGraph::from_json_file(filename)
        .with_context(|| format!("Failed to parse {filename}"))
        .unwrap()
}

fn get_extractor<'a>(
    extractors: &'a IndexMap<&str, Box<dyn Extractor>>,
    extractor_name: &str,
) -> &'a Box<dyn Extractor> {
    extractors
        .get(extractor_name)
        .with_context(|| format!("Unknown extractor: {extractor_name}"))
        .unwrap()
}

fn format_modified_name(modified_filename: &str, extractor_name: &str) -> String {
    format!(
        "{}_{}",
        &modified_filename[..modified_filename.len() - 5],
        extractor_name,
    )
}

fn extract_result(
    extractor: &Box<dyn Extractor>,
    egraph: &EGraph,
    root_eclasses: &[ClassId],
    cost_function: &str,
) -> ExtractionResult {
    extractor.extract(egraph, root_eclasses, cost_function)
}

fn print_dag_cost(dag_cost: Cost) {
    print!("-------------------------------------------\n");
    print!("dag cost: {}\n", dag_cost);
    print!("-------------------------------------------\n");
}

fn write_json_result<T: serde::Serialize>(filename: &str, data: &T) {
    let json_result = to_string_pretty(data).unwrap();
    let _ = fs::create_dir_all("out_json/my_data");
    let __ = fs::write(filename, json_result);
}

fn log_result(filename: &str, extractor_name: &str, dag_cost: Cost, us: u128) {
    log::info!("{filename:40}\t{extractor_name:10}\t{dag_cost:5}\t{us:5}");
}

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