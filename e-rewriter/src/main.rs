use egg::*; 
use rayon::iter::ParallelDrainRange;
use serde::Serialize;
use std::fs;
use std::io;
// use sprs::io;
// use crate::io;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::env;
use std::io::prelude::*;
use std::time::Instant;
use std::path::Path;
mod utils;
use utils::{language::*, preprocess::*, extract_new::*};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub fn preprocess_file(file_name: &str) -> Result<(), io::Error> {
    // Open the file for reading
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);

    // Prepare a string to hold the new contents of the file
    let mut new_contents = String::new();

    // Flags to detect INORDER and OUTORDER sections
    let mut in_inorder_section = false;
    let mut in_outorder_section = false;

    for line in reader.lines() {
        let line = line?;

        // Check if we're entering the INORDER or OUTORDER sections
        if line.trim().starts_with("INORDER") {
            in_inorder_section = true;
            new_contents.push_str(&line.trim());
            new_contents.push(' ');
        } else if line.trim().starts_with("OUTORDER") {
            in_outorder_section = true;
            new_contents.push_str(&line.trim());
            new_contents.push(' ');
        } else if in_inorder_section || in_outorder_section {
            new_contents.push_str(&line.trim());
            if line.trim().ends_with(";") {
                // End of section, reset flags
                in_inorder_section = false;
                in_outorder_section = false;
                // if this line end with ;, push \n to new_contents
                new_contents.push('\n');
                // continue to next line
                continue;
            }
            new_contents.push(' ');

        } else {
            new_contents.push_str(&line);
            new_contents.push('\n');
        }
    }

    // Open the same file for writing
    let mut file = OpenOptions::new().write(true).truncate(true).open(file_name)?;

    // Write the new contents to the file
    file.write_all(new_contents.as_bytes())?;

    Ok(())
}


fn main() ->Result<(), Box<dyn std::error::Error>> {
    

    //1.read eqn file
    let args: Vec<String> = env::args().collect();
    let input_path1 = &args[1];

    // preprocess input file
    preprocess_file(&input_path1)?;
    println!("Finished preprocessing input file");

    //-----------------------------------------------------------------------------------------------------   
    //2.transfer eqn file into egraph format in egg
    let root_id0 = process_file_1file(input_path1);
    println!("root: {:?}", root_id0);
    let mut root_ids: Vec<usize> = Vec::new();
    root_ids.push(root_id0.into());
    let json_file1 = format!("{}.json", input_path1);


    //4.transfer egg::egraph symbol language 's json into your defined language's json 
    let md_json_file1=process_json_prop(&json_file1);
    let json_data1 = fs::read_to_string(&md_json_file1).expect("Unable to read the JSON file");
    let mut egraphin: egg::EGraph<Prop, ()> = serde_json::from_str(&json_data1).unwrap();
    egraphin.rebuild();

    
    // save graphin into josn file
    let json_rep_test = serde_json::to_string_pretty(&egraphin).unwrap();
    let file_path = env::current_dir().unwrap().join("dot_graph/graphin.json");
    fs::write(&file_path, json_rep_test.clone()).expect("Failed to write JSON to file");

    // read from json file and print info
    let json_contents = fs::read_to_string(&file_path).expect("Failed to read JSON file");
    let mut egraph_new_test: egg::EGraph<Prop, ()> = serde_json::from_str(&json_contents).unwrap();
    egraph_new_test.rebuild();

    // generate dot file
    //5.add a whole graph root to connect 2 egraph's root
    let current_dir = env::current_dir().unwrap();
    let output_dir = current_dir.join("dot_graph");
    fs::create_dir_all(&output_dir).unwrap();

    // let output_path = output_dir.join("fooin.pdf");
    // egraph_new_test.dot().to_pdf(&output_path).unwrap();
    println!("total");
    println!("input node{}", egraph_new_test.total_size());
    println!("input class{}", egraph_new_test.number_of_classes());


    //-----------------------------------------------------------------------------------------------------   
    //6.transfer egg::egraph to serialized_egraph and save it into json file

    let json_rep_test_egraph_serd = egg_to_serialized_egraph(&egraph_new_test);    
    let file_path_1: PathBuf = env::current_dir().unwrap().join("dot_graph/graph_in_serd.json");
    let file = File::create(&file_path_1)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &json_rep_test_egraph_serd)?;


    //add root to graph_in_serd.json
    let root_eclasses_value: serde_json::Value = root_ids
    .clone()
    .into_iter()
    .map(|id| serde_json::Value::String(id.to_string())) // int to string
    .collect();
    let file = File::create(&file_path_1)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &json_rep_test_egraph_serd)?;
    let json_string = std::fs::read_to_string(&file_path_1)?;
    let mut json_data: serde_json::Value = serde_json::from_str(&json_string)?;
    json_data["root_eclasses"] =  root_eclasses_value.clone();
    print!("root_eclasses_value{}",root_eclasses_value);
    let file = File::create(&file_path_1)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &json_data)?;
    

    // visualize graph

    // let egraph_with_root =egraph_serialize::EGraph::from_json_file(&file_path_1).unwrap();
    // let svg_path = file_path_1.with_extension("svg");
    // egraph_with_root.to_svg_file(&svg_path).unwrap();
    // let svg_path1 = file_path_1.with_extension("dot");
    // egraph_with_root.to_dot_file(&svg_path1).unwrap();

    

    //-----------------------------------------------------------------------------------------------------   
    //7.ruuner configure
    let runner_iteration_limit = 10000000;
    let egraph_node_limit = 200000000;
  //  let egraph_node_limit = 10 *egraph_new_test.total_size();
    let start = Instant::now();
    let mut runner1 = Runner::default()
        .with_explanations_enabled()
        .with_egraph(egraph_new_test)
        .with_time_limit(std::time::Duration::from_secs(50))
        .with_iter_limit(runner_iteration_limit)
        .with_node_limit(egraph_node_limit);

    runner1.roots = root_ids.iter().cloned().map(Id::from).collect();
    let runner =runner1.run(&make_rules());
    let duration= start.elapsed();
    println!("Runner stopped: {:?}. Time take for runner: {:?}, Classes: {}, Nodes: {}, Size: {} \n\n",
            runner.stop_reason, duration, runner.egraph.number_of_classes(),
            runner.egraph.total_number_of_nodes(), runner.egraph.total_size());
    println!("root{:?}", runner.roots);
    let root = runner.roots[0];


    //save output egraph from runner (input for extraction gym)
    let json_rep_test_egraph = serde_json::to_string_pretty(&runner.egraph).unwrap();
    let json_rep_test_egraph_serd = egg_to_serialized_egraph(&runner.egraph);
    


    println!("egraph after runner");
    println!("egraph node{}", runner.egraph.total_size());
    println!("egraph class{}", runner.egraph.number_of_classes());
    let base_path = env::current_dir().expect("Failed to get current directory");
    let file_path = base_path.join("dot_graph/graph_internal.json");
    fs::write(&file_path, json_rep_test_egraph.clone()).expect("Failed to write JSON to file");

    //add root nodes into json
    println!("write root");
    let file_path_1 = base_path.join("dot_graph/graph_internal_serd.json");
    let root_eclasses_value: serde_json::Value = root_ids
    .clone()
    .into_iter()
    .map(|id| serde_json::Value::String(id.to_string())) // 将整数转换为字符串
    .collect();
    let file = File::create(&file_path_1)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &json_rep_test_egraph_serd)?;
    let json_string = std::fs::read_to_string(&file_path_1)?;
    let mut json_data: serde_json::Value = serde_json::from_str(&json_string)?;
    json_data["root_eclasses"] = root_eclasses_value;
    let file = File::create(&file_path_1)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &json_data)?;
    println!("done");


    // -------------------------------------------------------------
    // egg extraction
    let extractor_base_0  = Extractor2::new(&runner.egraph, egg::AstSize);
   // let extractor_base_1  = Extractor2::new(&runner.egraph, egg::AstDepth);
    let (best_cost_base_0,best_base_0 )=extractor_base_0.find_best(root);
   // let (best_cost_base_1,best_base_1 )=extractor_base_1.find_best(root);
    println!("best{}",best_cost_base_0);
    //println!("test_expr{}",best_base_0);
    
    let mut results: BTreeMap<i32, RecExpr<Prop>> = BTreeMap::new();
    results.insert(0, best_base_0.clone());
//    results.insert(1, best_base_1.clone()); 

    // let min_keys: Vec<i32> = vec![0, 1];
    // let mut count = 0;
    // let output_directory = "test_data_beta_runner/";
    // fs::create_dir_all(output_directory).unwrap();
    // for min_key in min_keys.iter() {
    //     let output = results
    //         .get(min_key)
    //         .map(|result| result.to_string())
    //         .unwrap_or_default();
    //     let output_file_name = format!("output_from_egg{}.txt", count);
    //     let output_file_path = Path::new(output_directory).join(output_file_name);
    //     if let Ok(mut output_file) = File::create(output_file_path) {
    //         output_file.write_all(output.as_bytes()).ok();
    //     } 
    //     count += 1;
    // }
    //---------------------------------------------------





    //some codes for visualize


    // let mut egraph = egraph_serialize::EGraph::from_json_file(&file_path_1).unwrap();
    // //生成原始的 SVG 文件
    // let svg_path = file_path_1.with_extension("svg");
    // egraph.to_svg_file(&svg_path).unwrap();
    // // 生成内联叶子节点的 SVG 文件
    // egraph.inline_leaves();
    // let inlined_svg_path = file_path_1.with_extension("inlined.svg");
    // egraph.to_svg_file(&inlined_svg_path).unwrap();
    // // 饱和内联叶子节点后的 SVG 文件
    // egraph.saturate_inline_leaves();
    // let saturated_svg_path = file_path_1.with_extension("inlined-saturated.svg");
    // egraph.to_svg_file(&saturated_svg_path).unwrap();
    // let saturated_svg_path = file_path_1.with_extension("inlined-saturated.dot");
    // egraph.to_dot_file(&saturated_svg_path).unwrap();

    //#[cfg(feature = "display")]{
    // let filename="/data/cchen/E-Syn2/extraction-gym-new/extraction-gym/out_process_dag_result/graph_internal_serd_bottom-up.json".to_string();
    // let egraph = egraph_serialize::EGraph::from_json_file(&filename).unwrap();
    // let svg_path = "/data/cchen/E-Syn2/extraction-gym-new/extraction-gym/graph_internal_serd_bottom-up.svg".to_string();
    // egraph.to_svg_file(&svg_path).unwrap();
    //}

    // let filename="/data/cchen/extraction-gym-new/extraction-gym/out_process_dag_result/graph_internal_serd_bottom-up".to_string();
    // let egraph = egraph_serialize::EGraph::from_json_file(&filename)
    // .unwrap();
    // let svg_path = "/data/cchen/extraction-gym-new/extraction-gym/out_process_dag_result/graph_internal_serd_bottom-up.svg";
    // egraph.to_svg_file(&svg_path).unwrap();
   
   
   // egraph1.to_svg_file(&svg_path).unwrap();
    // runner.egraph.dot().to_png("/data/cchen/E-Brush/image/process.png").unwrap();
   
    Ok(())
}