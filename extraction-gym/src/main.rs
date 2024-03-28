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
    // only keep fast version
    let extractors: IndexMap<&str, Box<dyn Extractor>> = [
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
    .collect();

    // default extractor

    // let extractors: IndexMap<&str, Box<dyn Extractor>> = [
    //     ("bottom-up", extract::bottom_up::BottomUpExtractor.boxed()),
    //     (
    //         "faster-bottom-up",
    //         extract::faster_bottom_up::FasterBottomUpExtractor.boxed(),
    //     ),
    //     (
    //         "greedy-dag",
    //         extract::greedy_dag::GreedyDagExtractor.boxed(),
    //     ),
    //     (
    //         "faster-greedy-dag",
    //         extract::faster_greedy_dag::FasterGreedyDagExtractor.boxed(),
    //     ),
    //     (
    //         "global-greedy-dag",
    //         extract::global_greedy_dag::GlobalGreedyDagExtractor.boxed(),
    //     ),
    // ]
    // .into_iter()
    // .collect();

    let mut args = pico_args::Arguments::from_env();

    let extractor_name: String = args
        .opt_value_from_str("--extractor")
        .unwrap()
        .unwrap_or_else(|| "bottom-up".into());
    if extractor_name == "print" {
        for name in extractors.keys() {
            println!("{}", name);
        }
        return;
    }

    let cost_function: String = args
        .opt_value_from_str("--cost-function")
        .unwrap()
        .unwrap_or_else(|| "node_depth_cost".into());

    let out_filename: PathBuf = args
        .opt_value_from_str("--out")
        .unwrap()
        .unwrap_or_else(|| "out.json".into());
    // let out_filename1: PathBuf = args
    //     .opt_value_from_str("--out_json")
    //     .unwrap()
    //     .unwrap_or_else(|| "out.json".into());
    let filename: String = args.free_from_str().unwrap();
    let modified_filename = filename.replacen("data/", "out_json/", 1);
    let modified_filename1 = filename.replacen("data/", "out_dag_json/", 1);
    //println!("{}", modified_filename);
    // println!("{}", filename);
    let rest = args.finish();
    if !rest.is_empty() {
        panic!("Unknown arguments: {:?}", rest);
    }

    let mut out_file = std::fs::File::create(out_filename.clone()).unwrap();

    let egraph = EGraph::from_json_file(&filename)
        .with_context(|| format!("Failed to parse {filename}"))
        .unwrap();

    let extractor = extractors
        .get(extractor_name.as_str())
        .with_context(|| format!("Unknown extractor: {extractor_name}"))
        .unwrap();

    let modified_name = format!(
        "{}_{}",
        &modified_filename[..modified_filename.len() - 5],
        extractor_name,
    );
    let modified_name1 = format!(
        "{}_{}",
        &modified_filename1[..modified_filename1.len() - 5],
        extractor_name,
    );
    let start_time = std::time::Instant::now();
    
    let result = extractor.extract(&egraph, &egraph.root_eclasses, &cost_function);

    let us = start_time.elapsed().as_micros();
    assert!(result
        .find_cycles(&egraph, &egraph.root_eclasses)
        .is_empty());
    //let tree = result.tree_cost(&egraph, &egraph.root_eclasses);
    //let dag = result.dag_cost(&egraph, &egraph.root_eclasses);

    //help me print dag cost
    // print!("-------------------------------------------\n");
    // print!("dag cost: {}\n", dag);
    // print!("-------------------------------------------\n");
    let (dag_cost, dag_cost_with_extraction_result) =
        result.dag_cost_with_extraction_result(&egraph, &egraph.root_eclasses);
    print!("-------------------------------------------\n");
    print!("dag cost: {}\n", dag_cost);
    print!("-------------------------------------------\n");
    result.record_costs_random(10, 0.5, &egraph, &dag_cost_with_extraction_result); // record random costs
    let json_result = to_string_pretty(&result).unwrap();
    let _ = fs::create_dir_all("out_json/my_data");
    let __ = fs::write(&modified_name, json_result);
    let _ = fs::create_dir_all("out_dag_json/my_data");
    let json_dag_result = to_string_pretty(&dag_cost_with_extraction_result).unwrap();
    let __ = fs::write(&modified_name1, json_dag_result);

    //println!("{}", json_result);
    log::info!("{filename:40}\t{extractor_name:10}\t{dag_cost:5}\t{us:5}");
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
