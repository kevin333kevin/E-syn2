use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use serde_json::{Value, Map};
use std::collections::HashMap;
use serde_json::json;
use std::fs;
use std::error::Error as StdError;
use serde::{Deserialize, Serialize};
// Define Data Structures
use std::collections::HashSet;
use rayon::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
struct GraphData {
    nodes: HashMap<String, Node>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Node {
    op: String,
    children: Vec<String>,
    eclass: String,
    cost: f32,
}




fn process_json(input_file: &str, a: u32) -> Result<(), Box<dyn StdError>> {
    // 读取输入文件
    let json_content = fs::read_to_string(input_file)
.map_err(|err| format!("Failed to read JSON file: {}", err))?;

    // Parse the JSON content
    let data: Value = serde_json::from_str(&json_content)?;
    let choices: HashMap<String, String> = serde_json::from_value(data["choices"].clone())?;
    let values: HashSet<&str> = choices.values().map(|v| v.as_str()).collect();
    println!("values: {:?}", values.len());

    // 读取 graph_internal_serd.json 文件
    let current_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let parent_dir = current_dir.parent().unwrap();
    let input_dir = parent_dir.join("extraction-gym/data/my_data");

    println!("Input Directory: {:?}", input_dir);

    let json_files: Vec<PathBuf> = input_dir.read_dir()?.filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.is_file() && path.extension().map(|ext| ext == "json").unwrap_or(false) {
            Some(path)
        } else {
            None
        }
    }).collect();

    let graph_file = json_files.get(0).ok_or("No JSON file found")?;
    let graph_content = fs::read_to_string(graph_file)
        .map_err(|err| format!("Failed to read graph data file: {}", err))?;

    // Parse the graph data
    let graph_data: GraphData = serde_json::from_str(&graph_content)?;
    // 构建新的结果字典，只保留存在于 values 集合中的键
    let new_nodes: HashMap<String, Node> = graph_data.nodes
        .into_iter()
        .filter(|(key, _)| values.contains(key.as_str()))
        .collect();
    println!("new_nodes: {:?}", new_nodes.len());

    // Build the final result dictionary
    let result = serde_json::json!({
        "nodes": new_nodes,
    });

    // Get the input file name
    let file_name = Path::new(input_file)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    // Build the output file path
    let output_file = if a == 1 {
        parent_dir.join("process_json/out_process_result")
                  .join(file_name)
                  .with_extension("json")
    } else {
        parent_dir.join("process_json/out_process_dag_result")
                  .join(file_name)
                  .with_extension("json")
    };

    // Output the result
    let output_content = serde_json::to_string_pretty(&result)?;
    fs::write(output_file, output_content)?;
    
    Ok(())
}


fn process_json1(input_file: &str, a: u32) -> Result<(), Box<dyn StdError>> {
    // Read the input file
    let json_content =fs::read_to_string(input_file).map_err(|err| format!("Failed to read JSON file: {}", err))?;
    // Parse the JSON content
    let data: GraphData = serde_json::from_str(&json_content).map_err(|err| format!("Failed to parse JSON: {}", err))?;

    println!("Input Directory process_json1: {:?}", input_file);
 //   println!("graph_data.nodes{:?}", data.nodes.len());
    // Process the nodes
    let mut new_nodes: HashMap<String, Node> = HashMap::new();
    for (key, mut value) in data.nodes {
        let new_key = key.split('.').next().unwrap().to_string();
        value.children = value
            .children
            .iter()
            .map(|child| child.split('.').next().unwrap().to_string())
            .collect();
        new_nodes.insert(new_key, value);
    }
    println!("new_nodes process_json1:{:?}", new_nodes.len());
    // Build the final result struct
    let result = GraphData { nodes: new_nodes };

    // Get the input file name
    let file_name = Path::new(input_file)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid input file name"))?;
    let current_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let parent_dir = current_dir.parent().unwrap();
    // Build the output directory path
    let output_file = if a == 1 {
        parent_dir.join("process_json/out_process_result").join(file_name).with_extension("json")
    } else {
        parent_dir.join("process_json/out_process_dag_result").join(file_name).with_extension("json")
    };
    // let output_dir = Path::new(output_dir);
    // fs::create_dir_all(output_dir).map_err(|err| format!("Failed to create output directory: {}", err))?;

    // // Build the output file path
    // let output_file = output_dir.join(file_name).with_extension("json");
    println!("output Directory: {:?}", output_file);
    // Serialize the result to JSON
    let output_content =
        serde_json::to_string_pretty(&result).map_err(|err| format!("Failed to serialize result: {}", err))?;
    let mut output_file = fs::File::create(output_file)?;
    output_file
        .write_all(output_content.as_bytes())
        .map_err(|err| format!("Failed to write output file: {}", err))?;

    Ok(())
}


fn main() {
    let output_dir = Path::new("out_process_result");
    fs::create_dir_all(output_dir).unwrap_or_else(|_| panic!("Failed to create output directory: {:?}", output_dir));

    let current_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let parent_dir = current_dir.parent().unwrap();
    let input_dir = parent_dir.join("extraction-gym/out_json/my_data");
    //println!("Input Directory: {:?}", input_dir);

    if let Ok(entries) = fs::read_dir(&input_dir) {
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

        file_paths.par_iter().for_each(|path| {
            let input_file = path.to_str().expect("Invalid input file path");

            // Generate new path with .json extension
            let mut new_path = PathBuf::from(&input_file);
            new_path.set_extension("json");

            // Rename the file with .json extension
            if let Err(err) = fs::rename(&path, &new_path) {
                println!("Failed to rename file: {:?}", err);
                return;
            }

            // Process the JSON file
            if let Err(err) = process_json(new_path.to_str().unwrap(), 1) {
                eprintln!("Error processing file {}: {}", new_path.to_str().unwrap(), err);
            }
        });
    } else {
        println!("Failed to read directory: {:?}", input_dir);
    }






    let input_dir = parent_dir.join("extraction-gym/out_dag_json/my_data");
    let output_dir_py = "out_process_dag_result";
    fs::create_dir_all(output_dir_py).unwrap_or_else(|_| panic!("Failed to create output directory: {:?}", output_dir_py));


    if let Ok(entries) = fs::read_dir(&input_dir) {
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

        file_paths.par_iter().for_each(|path| {
            let input_file = path.to_str().expect("Invalid input file path");

            // Generate new path with .json extension
            let mut new_path = PathBuf::from(&input_file);
            new_path.set_extension("json");

            // Rename the file with .json extension
            if let Err(err) = fs::rename(&path, &new_path) {
                println!("Failed to rename file: {:?}", err);
                return;
            }

            // Process the JSON file
            if let Err(err) = process_json(new_path.to_str().unwrap(), 0) {
                eprintln!("Error processing file {}: {}", new_path.to_str().unwrap(), err);
            }
        });
    } else {
        println!("Failed to read directory: {:?}", input_dir);
    }



  
    let input_dir = parent_dir.join("process_json/out_process_result");
     println!("Input Directory: {:?}", input_dir);
     if let Ok(entries) = fs::read_dir(&input_dir) {
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

        file_paths.par_iter().for_each(|path| {
            let input_file = path.to_str().expect("Invalid input file path");

            // Call the process_json function with your desired parameters
            if let Err(err) = process_json1(input_file, 1) {
                eprintln!("Error processing file {}: {}", input_file, err);
            }
        });
    } else {
        println!("Failed to read directory: {:?}", input_dir);
    }




    let input_dir = parent_dir.join("process_json/out_process_dag_result");
    if let Ok(entries) = fs::read_dir(&input_dir) {
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

        file_paths.par_iter().for_each(|path| {
            let input_file = path.to_str().expect("Invalid input file path");

            // Call the process_json function with your desired parameters
            if let Err(err) = process_json1(input_file, 0) {
                eprintln!("Error processing file {}: {}", input_file, err);
            }
        });
    } else {
        println!("Failed to read directory: {:?}", input_dir);
    }

let input_dir = parent_dir.join("extraction-gym/data/my_data");
println!("Input Directory: {:?}", input_dir);
let files = std::fs::read_dir(&input_dir).unwrap();
let json_files: Vec<PathBuf> = files
.filter_map(|entry| {
    let entry = entry.unwrap();
    let path = entry.path();
    if path.is_file() && path.extension().unwrap_or_default() == "json" {
        Some(path)
    } else {
        None
    }
})
.collect();

let graph_file = &json_files[0];
let mut source_data = String::new();
File::open(&graph_file)
    .unwrap()
    .read_to_string(&mut source_data)
    .unwrap();
let source_data: Value = serde_json::from_str(&source_data).unwrap();
let root_eclasses = if let Some(root_eclasses) = source_data["root_eclasses"].as_array() {
    root_eclasses.to_owned()
} else {
    vec![]
};
let current_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
let parent_dir = current_dir.parent().unwrap();
let output_dir = parent_dir.join("process_json/out_process_dag_result");
for entry in std::fs::read_dir(&output_dir).unwrap() {
    let entry = entry.unwrap();
    let target_file_path = entry.path();
    if target_file_path.is_file() && target_file_path.extension().unwrap_or_default() == "json" {
        let mut target_data = String::new();
        File::open(&target_file_path)
            .unwrap()
            .read_to_string(&mut target_data)
            .unwrap();
        let mut target_data: Value = serde_json::from_str(&target_data).unwrap();
        target_data["root_eclasses"] = json!(root_eclasses);
        File::create(&target_file_path)
            .unwrap()
            .write_all(serde_json::to_string_pretty(&target_data).unwrap().as_bytes())
            .unwrap();
    }
}
let output_dir = parent_dir.join("process_json/out_process_result");
for entry in std::fs::read_dir(&output_dir).unwrap() {
    let entry = entry.unwrap();
    let target_file_path = entry.path();
    if target_file_path.is_file() && target_file_path.extension().unwrap_or_default() == "json" {
        let mut target_data = String::new();
        File::open(&target_file_path)
            .unwrap()
            .read_to_string(&mut target_data)
            .unwrap();
        let mut target_data: Value = serde_json::from_str(&target_data).unwrap();
        target_data["root_eclasses"] = json!(root_eclasses);
        File::create(&target_file_path)
            .unwrap()
            .write_all(serde_json::to_string_pretty(&target_data).unwrap().as_bytes())
            .unwrap();
    }
}



}

