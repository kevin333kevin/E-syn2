use std::env;
use serde_json::{Value,Map};
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use egg::*;
use std::fs;
use std::collections::BTreeMap;
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let input_path1 = &args[1];

    let json_data = fs::read_to_string(input_path1)?;
    let mut json_value: Value = serde_json::from_str(&json_data)?;

    if let Some(nodes) = json_value.get_mut("nodes").and_then(Value::as_object_mut) {
        let sorted_nodes: BTreeMap<_, _> = nodes.iter()
            .map(|(key, value)| {
                let key_int = key.parse::<u64>().unwrap_or(0);
                (key_int, (key.clone(), value.clone()))
            })
            .collect();

        let sorted_json_map: Map<String, Value> = sorted_nodes.values()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();

        if let Some(nodes_value) = json_value.as_object_mut() {
            nodes_value.insert("nodes".to_string(), Value::Object(sorted_json_map));
        }
    }

    let output_path = PathBuf::from("out.json");
    let sorted_json_string = serde_json::to_string_pretty(&json_value)?;

    fs::write(output_path, sorted_json_string)?;

// 打印排序后的 JSON 值
    let mut egraph: EGraph<SymbolLang, ()> = EGraph::default();
    let mut vars:HashMap<usize, Id> = HashMap::new();
    
    let mut nodes_to_remove = Vec::new();
    if let Some(nodes) = json_value.get_mut("nodes").as_deref().and_then(Value::as_object) {
        for (key, value) in nodes {
            let key_int = key.parse::<usize>().unwrap_or(0);
            
            if let Some(children) = value.get("children").and_then(Value::as_array) {
                if children.is_empty() {
                    let id = egraph.add(SymbolLang::leaf(key));
        
                    vars.insert(key_int, id);
                    // println!("Node ID: {}", key);
                    // println!("id: {}", id);
                    // println!("Node Value: {:?}", value);
                    // println!("----------------------");
    
                    nodes_to_remove.push(key.clone());
                }
            }
        }
    }
    
    // 根据记录的节点键移除节点
    if let Some(nodes) = json_value.get_mut("nodes").and_then(Value::as_object_mut) {
        for key in &nodes_to_remove {
            nodes.remove(key);
        }

    }
   
    fn process_nodes(egraph: &mut EGraph<SymbolLang, ()>, vars: &mut HashMap<usize, Id>, json_value: &mut Value) {
            
            if let Some(nodes) = json_value.get_mut("nodes").and_then(Value::as_object_mut) {
                let mut nodes_to_remove = Vec::new();
                for (key, value) in nodes.iter_mut() {
                    let key_int = key.parse::<usize>().unwrap_or(0);
                    
                    if let Some(children) = value.get("children").and_then(Value::as_array) {
                        if let Some(op) = value.get("op").and_then(Value::as_str) {
                            if op == "*" {
                              //  println!("Node ID: {}", key);
                                if let (Some(lhs), Some(rhs)) = (children.get(0).and_then(Value::as_str), children.get(1).and_then(Value::as_str)) {
                                    if let (Ok(lhs_value), Ok(rhs_value)) = (lhs.parse::<usize>(), rhs.parse::<usize>()) {
                                        let lhs_id: Id = match vars.get(&lhs_value) {
                                            Some(&id) => id,
                                            None => {
                                      //          println!("Skipping node due to missing reference: {}", lhs_value);
                                                continue;
                                            }
                                        };
                                        let rhs_id: Id = match vars.get(&rhs_value) {
                                            Some(&id) => id,
                                            None => {
                                    //            println!("Skipping node due to missing reference: {}", rhs_value);
                                                continue;
                                            }
                                        };
                                        let id = egraph.add(SymbolLang::new("And", vec![lhs_id, rhs_id]));
                                        vars.insert(key_int, id);
                                        
                                        // println!("Node ID: {}", key);
                                        // println!("Node Value: {:?}", value);
                                        // println!("lhs: {}", lhs_id);
                                        // println!("rhs: {}", rhs_id);
                                        // println!("id: {}", id);
                                        // println!("----------------------");
                                        
                                        nodes_to_remove.push(key.clone());

                                    }
                                }
                            } else if op == "!" {
                             //   println!("Node ID: {}", key);
                                if let Some(lhs) = children.get(0).and_then(Value::as_str) {
                                    if let Ok(lhs_value) = lhs.parse::<usize>() {
                                        let lhs_id: Id = match vars.get(&lhs_value) {
                                            Some(&id) => id,
                                            None => {
                                        //        println!("Skipping node due to missing reference: {}", lhs_value);
                                                continue;
                                            }
                                        };
                                        let id = egraph.add(SymbolLang::new("Not", vec![lhs_id]));
                                        vars.insert(key_int, id);
                                        
                                        // println!("Node ID: {}", key);
                                        // println!("Node Value: {:?}", value);
                                        // println!("lhs: {}", lhs_id);
                                        // println!("id: {}", id);
                                        // println!("----------------------");
                                        
                                        nodes_to_remove.push(key.clone());
                                    }
                                }
                            } else if op == "+" {
                              //  println!("Node ID: {}", key);
                                if let (Some(lhs), Some(rhs)) = (children.get(0).and_then(Value::as_str), children.get(1).and_then(Value::as_str)) {
                                    if let (Ok(lhs_value), Ok(rhs_value)) = (lhs.parse::<usize>(), rhs.parse::<usize>()) {
                                        let lhs_id: Id = match vars.get(&lhs_value) {
                                            Some(&id) => id,
                                            None => {
                                              //  println!("Skipping node due to missing reference: {}", lhs_value);
                                                continue;
                                            }
                                        };
                                        let rhs_id: Id = match vars.get(&rhs_value) {
                                            Some(&id) => id,
                                            None => {
                                             //   println!("Skipping node due to missing reference: {}", rhs_value);
                                                continue;
                                            }
                                        };
                                        let id = egraph.add(SymbolLang::new("Or", vec![lhs_id, rhs_id]));
                                        vars.insert(key_int, id);
                                        
                                        // println!("Node ID: {}", key);
                                        // println!("Node Value: {:?}", value);
                                        // println!("lhs: {}", lhs_id);
                                        // println!("rhs: {}", rhs_id);
                                        // println!("id: {}", id);
                                        // println!("----------------------");
                                        
                                        nodes_to_remove.push(key.clone());
            
                                    }
                                }
                            } else if op == "&" {
                            //    println!("Node ID: {}", key);
                                if let (Some(lhs), Some(rhs)) = (children.get(0).and_then(Value::as_str), children.get(1).and_then(Value::as_str)) {
                                    if let (Ok(lhs_value), Ok(rhs_value)) = (lhs.parse::<usize>(), rhs.parse::<usize>()) {
                                        let lhs_id: Id =match vars.get(&lhs_value) {
                                            Some(&id) => id,
                                            None => {
                                          //      println!("Skipping node due to missing reference: {}", lhs_value);
                                                continue;
                                            }
                                        };
                                        let rhs_id: Id = match vars.get(&rhs_value) {
                                            Some(&id) => id,
                                            None => {
                                       //         println!("Skipping node due to missing reference: {}", rhs_value);
                                                continue;
                                            }
                                        };
                                        let id = egraph.add(SymbolLang::new("Concat", vec![lhs_id, rhs_id]));
                                        vars.insert(key_int, id);
                                        
                                        // println!("Node ID: {}", key);
                                        // println!("Node Value: {:?}", value);
                                        // println!("lhs: {}", lhs_id);
                                        // println!("rhs: {}", rhs_id);
                                        // println!("id: {}", id);
                                        // println!("----------------------");
                                        
                                        nodes_to_remove.push(key.clone());

                                    }
                                }
                            }
                        }
                    }
                }
                for key in nodes_to_remove {
                    nodes.remove(&key);
                }



            }
            }

       
        while !json_value.is_null() {
            process_nodes(&mut egraph, &mut vars, &mut json_value);
        
            // 获取剩余节点的数量并打印
            if let Some(nodes) = json_value.get("nodes").and_then(Value::as_object) {
                let remaining_nodes = nodes.len();
       //         println!("Remaining nodes: {}", remaining_nodes);
                
                if remaining_nodes == 0 {
                    break;
                }
            }
        }

        println!("input node{}", egraph.total_size());
        println!("input class{}", egraph.number_of_classes());
    Ok(())
}