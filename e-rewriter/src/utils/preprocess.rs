use egg::*;
use std::collections::HashMap;
use std::io::BufReader;
use std::fs;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fs::File;
use std::io::BufRead;
use std::env;
use ::serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::path::Path;
use serde_json::Value;
use serde::__private::fmt::Display;



// parse equation 2 to change the variables' names avoid of The variables of the nodes in equation 1 duplicated in the hash table
pub fn process_text(file_content: &str) -> String {
    let mut processed_content = String::new();
    let mut egraph: egg::EGraph<SymbolLang, ()> = EGraph::default();
    let mut vars = HashMap::new();
    
    for line in file_content.lines() {
        if line.starts_with("INORDER") {
            processed_content.push_str(line);
            processed_content.push('\n');
            let inputs = line.trim_start_matches("INORDER = ").trim_end_matches(';').split_whitespace().collect::<Vec<_>>();
            for (_, input) in inputs.iter().enumerate() {
                let id = egraph.add_expr(&input.parse().unwrap());
                vars.insert(input.to_string(), id);
            }
            //println!("vars: {:?}", vars);
        }
            
        else if line.starts_with("OUTORDER") {
            let mut parts = line.split('=');
            if let Some(variables) = parts.nth(1) {
                let modified_variables = variables
                    .split_whitespace()
                    .map(|var| {
                        format!("p_{}", var)
                    })
                    .collect::<Vec<String>>()
                    .join(" ");
                processed_content.push_str(&format!("OUTORDER = {}\n", modified_variables));
            }
        } else if let Some(eq_index) = line.find('=') {
            let (lhs, rhs) = line.split_at(eq_index);
            let rhs_trimmed = rhs.trim_start_matches('=').trim();
            let op_index = rhs_trimmed.find(|c| c == '*' || c == '+');

            if let Some(op_index) = op_index {
                let rhs_lhs = rhs_trimmed[..op_index].trim_end_matches(' ');
                let rhs_rhs = rhs_trimmed[op_index + 2..].trim_end_matches(';');
              //  print!("lhs:{}\nrhs_lhs:{}\nrhs_rhs:{}\n",lhs,rhs_lhs,rhs_rhs);
                let lhs_modified = format!("p_{}", lhs.trim());
                let rhs_lhs_modified = if vars.contains_key(rhs_lhs)  {
                    rhs_lhs.to_owned()
                } else if rhs_lhs.starts_with('!') && vars.contains_key(&rhs_lhs[1..]) {
                    rhs_lhs.to_owned()
                } else if rhs_lhs.starts_with('!') {
                    format!("!p_{}", &rhs_lhs[1..])
                } else {
                    format!("p_{}", rhs_lhs.trim())
                };
                let rhs_rhs_modified = if vars.contains_key(rhs_rhs) {
                    rhs_rhs.to_owned()
                } else if rhs_rhs.starts_with('!') && vars.contains_key(&rhs_rhs[1..]) {
                    rhs_rhs.to_owned()
                } else if rhs_rhs.starts_with('!') {
                    format!("!p_{}", &rhs_rhs[1..])
                } else {
                    format!("p_{}", rhs_rhs.trim())
                };

                let modified_line = format!("{} = {} {} {};", lhs_modified, rhs_lhs_modified, &rhs_trimmed[op_index..op_index+1], rhs_rhs_modified);
                processed_content.push_str(&modified_line);
                processed_content.push('\n');
            } else {
                processed_content.push_str(line);
                processed_content.push('\n');
            }
        } else {
            processed_content.push_str(line);
            processed_content.push('\n');
        }
    }
    processed_content
}



// parse your input file ------ add node into egg::egraph ------ here your input is a mut egraph
// each time this function is called ,return a mut egraph after adding nodes by parsing a eqn file and a root node  
fn add_node_2_egraph(egraph:&mut egg::EGraph<SymbolLang,()> ,reader:BufReader<File>,vars: &mut HashMap<String, Id>,out: &mut HashMap<String, u64>,count_out:&mut i32,id2concat :&mut Vec<Id>) -> (egg::EGraph<SymbolLang, ()>,egg::Id ){
    fn string_to_unique_id(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        let line = line.trim().trim_end_matches(';');

        if line.starts_with('#') || line.is_empty() {
            continue;
        } else if line.starts_with("INORDER") {
            let inputs = line.trim_start_matches("INORDER = ").split_whitespace();
            for input in inputs {
                //let id = egraph.add_expr(&input.parse().unwrap());
                let id = egraph.add(SymbolLang::leaf(input)); // 将 "NOT" 替换为每个输入的字符串
                vars.insert(input.to_string(), id);
            }
        } else if line.starts_with("OUTORDER") {
            let output = line.trim_start_matches("OUTORDER = ").trim();
            for output in output.split_whitespace() {
                let id_u64 = string_to_unique_id(output);
                out.insert(output.to_string(), id_u64);
            }
    
        } else {
          //  print!("line:  {}\n",line);
            let parts: Vec<&str> = line.split('=').map(str::trim).collect();
            let left = parts[0];
            let right = parts[1];

            let id = if right.contains('+') {
                let operands: Vec<&str> = right.split('+').map(str::trim).collect();
                let lhs = if operands[0].starts_with('!') {
                    let var = &operands[0][1..];
                    let id = vars[var];
                    egraph.add(SymbolLang::new("Not", vec![id]))
                } else {
                    vars[operands[0]]
                };
                let rhs = if operands[1].starts_with('!') {
                    let var = &operands[1][1..];
                    let id = vars[var];
                    egraph.add(SymbolLang::new("Not", vec![id]))
                } else {
                    vars[operands[1]]
                };
                egraph.add(SymbolLang::new("Or", vec![lhs, rhs]))
            } else if right.contains('*') {
                let operands: Vec<&str> = right.split('*').map(str::trim).collect();
                let lhs = if operands[0].starts_with('!') {
                    let var = &operands[0][1..];
                    let id = vars[var];
                    egraph.add(SymbolLang::new("Not", vec![id]))
                } else {
                    vars[operands[0]]
                };
                let rhs = if operands[1].starts_with('!') {
                    let var = &operands[1][1..];
                    let id = vars[var];
                    egraph.add(SymbolLang::new("Not", vec![id]))
                } else {
                    vars[operands[1]]
                };
                egraph.add(SymbolLang::new("And", vec![lhs, rhs]))
            } else {
                panic!("Unknown operator");
            };

            if out.contains_key(left) {
                id2concat.push(id);
                *count_out += 1;
            }
            vars.insert(left.to_string(), id);
        }
    }
    let mut concat = Vec::new();
    for i in 0..*count_out - 1 {
        if i == 0 {
            let id = egraph.add(SymbolLang::new("Concat", vec![id2concat[i as usize], id2concat[(i + 1) as usize]]));
            concat.push(id);
        } else {
            let id = egraph.add(SymbolLang::new("Concat", vec![concat[(i - 1) as usize], id2concat[(i + 1) as usize]]));
            concat.push(id);
        }
    }
        println!("out size: {}", out.len());
        println!("count_out: {}", count_out);
        
    // for num in &concat {
    //     println!("concat: {}", usize::from(*num));
    // }
    let last_element: egg::Id = concat.pop().unwrap();
    println!("Last element: {:?}", last_element);
        // for (key, value) in vars.iter() {
        //     println!("Key: {}, Value: {:?}", key, value);
        // }
    egraph.rebuild();
   // egraph.dot().to_dot("tmp1.dot").unwrap();
   // egraph.dot().to_pdf("tmp1.pdf").unwrap();
    (egraph.clone(),last_element)
}


// parse 2 eqn file-----> to add them into 1 egraph 
// return root0 ---- eqn1's root  & roo1 ---- eqn2's root

pub fn process_file_2file(file_name0: &str, file_name1: &str) ->(egg::Id,egg::Id){
    let file = File::open(file_name0).expect("Unable to open the eqn file");
    let reader = BufReader::new(file);
    let file1 = File::open(file_name1).expect("Unable to open the eqn file");
    let reader1 = BufReader::new(file1);
    let mut egraph: egg::EGraph<SymbolLang, ()> = EGraph::default();
    let mut vars = HashMap::new();
    let mut out = HashMap::new();
    let mut count_out: i32 = 0;
    let mut id2concat = Vec::new();
    let root_id0: egg::Id;
    let root_id1: egg::Id;
    
    let (mut egraph_in,root_id0) = add_node_2_egraph(&mut egraph, reader, &mut vars, &mut out, &mut count_out, &mut id2concat);
    let mut count_out: i32 = 0;
    let mut id2concat = Vec::new();
    (egraph_in,root_id1)  = add_node_2_egraph(&mut egraph_in, reader1, &mut vars, &mut out, &mut count_out, &mut id2concat);

    let json_str = serde_json::to_string_pretty(&egraph_in).unwrap();

    let output_dir = Path::new(file_name0).parent().unwrap_or(Path::new(""));
    let output_file = format!("{}.json", PathBuf::from(file_name0).file_name().unwrap().to_string_lossy());
    let output_path = output_dir.join(output_file);
    fs::write(output_path, json_str).expect("Failed to write JSON file");
    (root_id0,root_id1)
}


#[derive(Debug, Deserialize, Serialize, Clone)]
struct Node {
    op: String,
    children: Vec<u32>,
}

//sub function for process_json_prop
pub fn process_data(data: &mut serde_json::Value) {
    if let Some(entries) = data.as_array_mut() {
        for entry in entries.iter_mut() {
            if let Some(entry_obj) = entry.as_object_mut() {
                process_entry(entry_obj);
            }
        }
    } else if let Some(obj) = data.as_object_mut() {
        process_entry(obj);
        for (_, value) in obj.iter_mut() {
            process_data(value);
        }
    }
}

//sub function for process_json_prop
pub fn process_entry(entry_obj: &mut serde_json::Map<String, serde_json::Value>) {
    // 处理 "op" 和 "children" 键值对
    if let Some(op_value) = entry_obj.remove("op") {
        if let Some(children_value) = entry_obj.remove("children") {
            entry_obj.insert(op_value.as_str().unwrap().to_owned(), children_value);
        }
    }

    for (key, value) in entry_obj.clone().into_iter() {
        if let serde_json::Value::Array(ref arr) = value {
            if arr.is_empty() {
                entry_obj.remove(&key);
                entry_obj.insert("Symbol".to_owned(), serde_json::Value::String(key.clone()));
            }
        }
        if key == "Not" {
            if let serde_json::Value::Array(ref arr1) = value {
                if arr1.len() == 1 {
                    let new_value = arr1[0].clone();
                    entry_obj.insert(key, new_value);
                }
            }
        }
    }
}

//transfer egg's symbol language version json format into your defined language version json format 
//The reason is that u must use egg's symbol language 's trait add to add node into graph ,which the self-defined language by define_language! macro definition dosesn't support add node
//
pub fn process_json_prop(json_file: &str) -> String {

    let json_str = fs::read_to_string(json_file).expect("Failed to read JSON file");
    let mut data: Value = serde_json::from_str(&json_str).unwrap();

    // 处理 "memo"
    if let Some(classes) = data.get_mut("classes").and_then(|classes| classes.as_object_mut()) {
        for class in classes.values_mut() {
            if let Some(nodes) = class.get_mut("nodes").and_then(|nodes| nodes.as_array_mut()) {
                for node in nodes.iter_mut() {
                //    println!("Processed node: {:?}", node);
                    process_data(node);
                }
            }
            if let Some(parents) = class.get_mut("parents").and_then(|parents| parents.as_array_mut()) {
                for parent in parents.iter_mut() {
                //    println!("Processed parent: {:?}", parent);
                    process_data(parent);
                    
                }
            }
        }
    }

    if let Some(memo) = data.get_mut("memo").and_then(|memo| memo.as_array_mut()) {
        for entry in memo.iter_mut() {
            process_data(entry);
        }
    }

    // 将修改后的数据转换回 JSON 字符串
    let modified_json_str = serde_json::to_string_pretty(&data).unwrap();
    // 构造保存修改后的文件路径
    let json_file_path = PathBuf::from(json_file);
    let modified_json_file = json_file_path.with_file_name(format!("md_{}", json_file_path.file_name().unwrap().to_str().unwrap()));
    // 将修改后的 JSON 字符串写入文件
    fs::write(&modified_json_file, modified_json_str).expect("Failed to write modified JSON file");
    modified_json_file.to_str().unwrap().to_owned()
}


// egraph_serialize::EGraph used in extraction gym
// you need to transfer egg's EGraph to  Egraph_serialize's EGraph for extraction gym input
pub fn egg_to_serialized_egraph<L, A>(egraph: &egg::EGraph<L, A>) -> egraph_serialize::EGraph
where
    L: Language + Display,
    A: Analysis<L>,
{
    use egraph_serialize::*;
    let mut out = EGraph::default();
    for class in egraph.classes() {
        for (i, node) in class.nodes.iter().enumerate() {
            out.add_node(
                format!("{}.{}", class.id, i),
                Node {
                    op: node.to_string(),
                    children: node
                        .children()
                        .iter()
                        .map(|id| NodeId::from(format!("{}.0", id)))
                        .collect(),
                    eclass: ClassId::from(format!("{}", class.id)),
                    cost: Cost::new(1.0).unwrap(),
                },
            )
        }
    }
    out
}






pub fn process_file_1file(file_name: &str) -> (egg::Id){
    let file = File::open(file_name).expect("Unable to open the eqn file");
    let reader = BufReader::new(file);
    let mut egraph:egg::EGraph<SymbolLang,()> = EGraph::default();
    let mut vars = HashMap::new();
    let mut out = HashMap::new();
    let mut count_out = 0;
    let mut id2concat = Vec::new();

    fn string_to_unique_id(s: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    let id0 =egraph.add(SymbolLang::leaf("0"));
    vars.insert("0".to_string(), id0);
    let id1 =egraph.add(SymbolLang::leaf("1"));
    vars.insert("1".to_string(), id1);
    for line in reader.lines() {
        let line = line.expect("Unable to read line");
        let line = line.trim().trim_end_matches(';');
        //print!("line:  {}\n",line);
        if line.starts_with('#') || line.is_empty() {
            continue;
        } else if line.starts_with("INORDER") {
            let inputs = line.trim_start_matches("INORDER = ").split_whitespace();
            for input in inputs {
                let id = egraph.add(SymbolLang::leaf(input)); // 将 "NOT" 替换为每个输入的字符串
                vars.insert(input.to_string(), id);
            }
        } else if line.starts_with("OUTORDER") {
            let output = line.trim_start_matches("OUTORDER = ").trim();
            for output in output.split_whitespace() {
                let id_u64 = string_to_unique_id(output);
                out.insert(output.to_string(), id_u64);
            }
        } else {
            let parts: Vec<&str> = line.split('=').map(str::trim).collect();
            let left = parts[0];
            let right = parts[1];
           // print!("right {}\n",right);
            let id = if right.contains('+') {
                let operands: Vec<&str> = right.split('+').map(str::trim).collect();
                let lhs = if operands[0].starts_with('!') {
                    let var = &operands[0][1..];
                    let id = vars[var];
                    egraph.add(SymbolLang::new("Not", vec![id]))
                } else {
                    vars[operands[0]]
                };
                let rhs = if operands[1].starts_with('!') {
                    let var = &operands[1][1..];
                    let id = vars[var];
                    egraph.add(SymbolLang::new("Not", vec![id]))
                } else {
                    vars[operands[1]]
                };
                egraph.add(SymbolLang::new("Or", vec![lhs, rhs]))
            } else if right.contains('*') {
                let operands: Vec<&str> = right.split('*').map(str::trim).collect();
                let lhs = if operands[0].starts_with('!') {
                    let var = &operands[0][1..];
                    let id = vars[var];
                    egraph.add(SymbolLang::new("Not", vec![id]))
                } else {
                    vars[operands[0]]
                };
                let rhs = if operands[1].starts_with('!') {
                    let var = &operands[1][1..];
                    let id = vars[var];
                    egraph.add(SymbolLang::new("Not", vec![id]))
                } else {
                    vars[operands[1]]
                };

                egraph.add(SymbolLang::new("And", vec![lhs, rhs]))
            } else if right.starts_with('!'){
                let var =&right[1..];
                let id =vars[var];
                egraph.add(SymbolLang::new("Not", vec![id]))
            }else{
                vars[right]
            } ;

            if out.contains_key(left) {
                id2concat.push(id);
                count_out += 1;
            }
            vars.insert(left.to_string(), id);
        }
    }
    

    let mut concat = Vec::new();

    for i in 0..count_out - 1 {
        if i == 0 {
            let id = egraph.add(SymbolLang::new("Concat", vec![id2concat[i as usize], id2concat[(i + 1) as usize]]));
            concat.push(id);
        } else {
            let id = egraph.add(SymbolLang::new("Concat", vec![concat[(i - 1) as usize], id2concat[(i + 1) as usize]]));
            concat.push(id);
        }
    }
    let last_element: egg::Id = concat.pop().unwrap();
    egraph.rebuild();
    let json_str = serde_json::to_string_pretty(&egraph).unwrap();

    let output_dir = Path::new(file_name).parent().unwrap_or(Path::new(""));
    let output_file = format!("{}.json", PathBuf::from(file_name).file_name().unwrap().to_string_lossy());
    let output_path = output_dir.join(output_file);
    fs::write(output_path, json_str).expect("Failed to write JSON file");
    last_element
}