use serde::{Deserialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Write, Read};
use std::env;

// Define Data Structures
#[derive(Deserialize, Debug)]
struct Node {
    op: String,
    children: Vec<String>,
    eclass: String, // Add this to handle the 'eclass' field in JSON
    cost: f64,      // Add this to handle the 'cost' field in JSON
}

#[derive(Deserialize, Debug)]
struct Graph {
    nodes: HashMap<String, Node>,
    root_eclasses: Vec<String>,
}


// Parse json
fn parse_json(json_str: &str) -> Graph {
    serde_json::from_str(json_str).expect("JSON was not well-formatted")
}

// Convert DAG to eqn with proper hierarchical representation
fn dag_to_equations(
    nodes: &HashMap<String, Node>,
    node_id: &str,
    visited: &mut HashMap<String, String>,
    visit_count: &mut HashMap<String, usize>,
) -> String {
    *visit_count.entry(node_id.to_string()).or_insert(0) += 1;

    // if let Some(expr) = visited.get(node_id) {
    //     return expr.clone(); // Return the expression if already recorded.
    // }
    if visited.contains_key(node_id) {
        return format!("new_n_{}", node_id); // Return the modified key value if already recorded.
    }

    let node = nodes.get(node_id).unwrap();
  //  println!("Node ID: {:?}", node_id); // 添加这行
    let expression = match node.op.as_str() {
        "&" => {
            let operands: Vec<String> = node.children.iter().map(|child_id| {
                dag_to_equations(nodes, child_id, visited, visit_count)
            }).collect();
            operands.join(" & ")
        }
        _ => {
            let operands: Vec<String> = node.children.iter().map(|child_id| {
                dag_to_equations(nodes, child_id, visited, visit_count)
            }).collect();
            if operands.is_empty() {
                node.op.clone() // No children means it's a variable or a constant
            } else if operands.len() == 1 {
                format!("{}({})", node.op, operands[0]) // Unary operation
            } else {
                format!("({} {} {})", operands[0], node.op, operands[1]) // Binary operation
            }
        }
    };

    // Only record the expression if the node has been visited more than once.
    
    if visit_count[node_id] > 1 && (expression.contains(" ") || expression.contains("(")) {
    //if visit_count[node_id] > 1  {
        visited.insert(node_id.to_string(), expression.clone());
    }

    expression
}

// Function to format Synopsys for a single equation
fn format_synopsys_single(equation: &str) -> Vec<String> {
    equation.split('&').map(|part| part.trim().to_string()).collect()
}

 // Function to write Synopsys format to a file
 fn write_to_file(
    variables: &Vec<String>,
    parts: Vec<String>,
    file_name: &str,
    f_prefix: &str,
    visited: HashMap<String, String>,
) {
    let mut file = File::create(file_name).expect("Unable to create file");

    writeln!(file, "INORDER = {};", variables.join(" ")).expect("Unable to write to file");
    
    let mut f_counter: usize = 0;
    for part in parts {
        let f_number = format!("{:02}", f_counter);
        writeln!(file, "{}{} = {};", f_prefix, f_number, part).expect("Unable to write to file");
        f_counter += 1;
    }

    writeln!(file, "OUTORDER = {};", (0..f_counter)
        .map(|i| format!("{}{:02}", f_prefix, i))
        .collect::<Vec<String>>()
        .join(" ")).expect("Unable to write to file");

    for (node_id, expr) in visited.iter() {
        writeln!(file, "new_n_{} = {};", node_id, expr).expect("Unable to write to file");
    }

  //  println!("Equation written to {}", file_name);
}


fn main() {
    // Read the file path from the command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: <program> <path_to_json_file>");
        std::process::exit(1);
    }
    let file_path = &args[1];

    // Open the file and read the JSON content
    let mut file = File::open(file_path).expect("Unable to open file");
    let mut json_str = String::new();
    file.read_to_string(&mut json_str).expect("Unable to read file");

    // Parse the JSON string
    let graph = parse_json(&json_str);

    // Determine the nodes that are not children of any other nodes
    // Use the root_eclasses as root nodes
    let root_nodes = graph.root_eclasses;

    // Print the root nodes
   // println!("Root nodes: {:?}", root_nodes);

    // Process each identified root node
    for (i, root) in root_nodes.iter().enumerate() {
        let mut visited = HashMap::new();
        let mut visit_count = HashMap::new();
        let equation = dag_to_equations(&graph.nodes, root, &mut visited, &mut visit_count);
        //println!("Equation: {}", equation);
        // if the length of visited is 0, not printing the visited nodes
        if visited.len() != 0 {
        //    println!("Visit count: {:?}", visit_count);
        //    println!("Visited nodes: {:?}", visited);
        }
        
        // for (node_id, expr) in visited.iter() {
        //     println!("{}: {}", node_id, expr);
        // }
        //println!("{}", equation);
        //println!("Visited nodes: {:?}", visited);
      //  println!("Circuit {}: ", i + 1);

        // Extract unique variables
        let mut variables = vec![];
        for node in graph.nodes.values() {
            if node.children.is_empty() && !variables.contains(&node.op) && node.op != "1"  && node.op != "0" {
                variables.push(node.op.clone());
            }
        }

        // Format each equation
        let parts = format_synopsys_single(&equation);

        // Print Synopsys format for each Circuit
        // println!("INORDER = {};", variables.join(" "));
        // println!("OUTORDER = {};", (1..=parts.len()).map(|i| format!("F_{}", i)).collect::<Vec<String>>().join(" "));
        // for (i, part) in parts.iter().enumerate() {
        //     println!("F_{} = {};", i + 1, part);
        // }
        // println!();

        // Format and write each Circuit to a file
        write_to_file(&variables, parts, &format!("circuit{}.eqn", i), "po", visited);
    }
}