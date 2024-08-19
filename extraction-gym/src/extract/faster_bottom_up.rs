use super::*;
use crate::extract::circuit_conversion::process_circuit_conversion;
use rand::prelude::*;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use std::env;
use std::process;
use tokio::runtime::Runtime;

//use abc::Abc;

use crate::extract::lib::Abc;

use std::fs;
use std::future::Future;
use std::io::Write;
use tempfile::NamedTempFile;

use tonic::Request;
use vectorservice::vector_service_client::VectorServiceClient;
use vectorservice::CircuitFilesRequest;

async fn send_circuit_files_to_server(
    el_content: &str,
    csv_content: &str,
    json_content: &str,
) -> Result<f64, Box<dyn std::error::Error>> {
    let mut client = VectorServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(CircuitFilesRequest {
        el_content: el_content.to_string(),
        csv_content: csv_content.to_string(),
        json_content: json_content.to_string(),
    });

    let response = client.process_circuit_files(request).await?;
    Ok(response.into_inner().delay)
}

pub mod vectorservice {
    tonic::include_proto!("vectorservice");
}

fn call_abc(eqn_content: &str) -> Result<f32, Box<dyn std::error::Error>> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(eqn_content.as_bytes())?;
    let temp_path = temp_file.path().to_str().unwrap();

    let mut abc = Abc::new();

    //println!("Reading equation file...");
    abc.execute_command(&format!("read_eqn {}", temp_path));
    //println!("Reading library...");
    abc.execute_command("read_lib ../abc/asap7_clean.lib");
    //println!("Performing structural hashing...");
    abc.execute_command("strash");
    //println!("Performing dump the edgelist...");
    abc.execute_command("&get; &edgelist  -F src/extract/tmp/opt_1.el -f src/extract/tmp/opt-feats.csv -c src/extract/tmp/opt_1.json; &put");
    //println!("Performing technology mapping...");
    abc.execute_command("map");
    //println!("Performing post-processing...(topo; gate sizing)");
    abc.execute_command("topo");
    abc.execute_command("upsize");
    abc.execute_command("dnsize");

    //println!("Executing stime command...");
    let stime_output = abc.execute_command_with_output("stime -d");

    if let Some(delay) = parse_delay(&stime_output) {
        let delay_ns = delay / 1000.0;
        //println!("Delay in nanoseconds: {} ns", delay_ns);
        Ok(delay)
    } else {
        Err("Failed to parse delay value".into())
    }
}

fn parse_delay(output: &str) -> Option<f32> {
    for line in output.lines() {
        if line.contains("Delay") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                return parts[1].parse::<f32>().ok();
            }
        }
    }
    None
}

/// A faster bottom up extractor inspired by the faster-greedy-dag extractor.
/// It should return an extraction result with the same cost as the bottom-up extractor.
///
/// Bottom-up extraction works by iteratively computing the current best cost of each
/// node in the e-graph based on the current best costs of its children.
/// Extraction terminates when our estimates of the best cost for each node
/// reach a fixed point.
/// The baseline bottom-up implementation visits every node during each iteration
/// of the fixed point.
/// This algorithm instead only visits the nodes whose current cost estimate may change:
/// it does this by tracking parent-child relationships and storing relevant nodes
/// in a work list (UniqueQueue).
pub struct FasterBottomUpExtractor;
pub struct FasterBottomUpExtractorGRPC;
pub struct FasterBottomUpExtractorRandom;
pub struct FasterBottomUpSimulatedAnnealingExtractor;

impl Extractor for FasterBottomUpExtractor {
    fn extract(
        &self,
        egraph: &EGraph,
        _roots: &[ClassId],
        cost_function: &str,
        random_prob: f64,
    ) -> ExtractionResult {
        let mut parents = IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
        let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);
        let mut analysis_pending = UniqueQueue::default();

        for class in egraph.classes().values() {
            parents.insert(class.id.clone(), Vec::new());
        }

        for class in egraph.classes().values() {
            for node in &class.nodes {
                for c in &egraph[node].children {
                    // compute parents of this enode
                    parents[n2c(c)].push(node.clone());
                }

                // start the analysis from leaves
                if egraph[node].is_leaf() {
                    analysis_pending.insert(node.clone());
                }
            }
        }

        let mut result = ExtractionResult::default();
        let mut costs = FxHashMap::<ClassId, Cost>::with_capacity_and_hasher(
            egraph.classes().len(),
            Default::default(),
        );

        while let Some(node_id) = analysis_pending.pop() {
            let class_id = n2c(&node_id);
            let node = &egraph[&node_id];
            let prev_cost = costs.get(class_id).unwrap_or(&INFINITY);
            let cost = match cost_function {
                "node_sum_cost" => result.node_sum_cost(egraph, node, &costs),
                "node_depth_cost" => result.node_depth_cost(egraph, node, &costs),
                _ => panic!("Unknown cost function: {}", cost_function),
            };
            if cost < *prev_cost {
                result.choose(class_id.clone(), node_id.clone());
                costs.insert(class_id.clone(), cost);
                analysis_pending.extend(parents[class_id].iter().cloned());
            }
        }

        result
    }
}

impl Extractor for FasterBottomUpExtractorGRPC {
    fn extract(
        &self,
        egraph: &EGraph,
        roots: &[ClassId],
        cost_function: &str,
        random_prob: f64,
    ) -> ExtractionResult {
        // Create a new runtime for this extraction
        let rt = Runtime::new().unwrap();
        // Use the runtime to block on the async extraction
        rt.block_on(self.extract_async(egraph, roots, cost_function, random_prob))
    }
}

impl AsyncExtractor for FasterBottomUpExtractorGRPC {
    fn extract_async<'a>(
        &'a self,
        egraph: &'a EGraph,
        roots: &'a [ClassId],
        cost_function: &'a str,
        random_prob: f64,
    ) -> impl Future<Output = ExtractionResult> + Send + 'a {
        async move {
            let mut parents =
                IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
            let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);
            let mut analysis_pending = UniqueQueue::default();

            for class in egraph.classes().values() {
                parents.insert(class.id.clone(), Vec::new());
            }

            for class in egraph.classes().values() {
                for node in &class.nodes {
                    for c in &egraph[node].children {
                        // compute parents of this enode
                        parents[n2c(c)].push(node.clone());
                    }

                    // start the analysis from leaves
                    if egraph[node].is_leaf() {
                        analysis_pending.insert(node.clone());
                    }
                }
            }

            let mut result = ExtractionResult::default();
            let mut costs = FxHashMap::<ClassId, Cost>::with_capacity_and_hasher(
                egraph.classes().len(),
                Default::default(),
            );

            while let Some(node_id) = analysis_pending.pop() {
                let class_id = n2c(&node_id);
                let node = &egraph[&node_id];
                let prev_cost = costs.get(class_id).unwrap_or(&INFINITY);
                let cost = match cost_function {
                    "node_sum_cost" => result.node_sum_cost(egraph, node, &costs),
                    "node_depth_cost" => result.node_depth_cost(egraph, node, &costs),
                    _ => panic!("Unknown cost function: {}", cost_function),
                };
                if cost < *prev_cost {
                    result.choose(class_id.clone(), node_id.clone());
                    costs.insert(class_id.clone(), cost);
                    analysis_pending.extend(parents[class_id].iter().cloned());
                }
            }

            // Compute JSON buffers for tree cost and DAG cost extraction results
            let tree_cost_json = to_string_pretty(&result).unwrap();

            let (dag_cost, dag_cost_extraction_result) =
                result.calculate_dag_cost_with_extraction_result(&egraph, &egraph.root_eclasses);
            let dag_cost_json = to_string_pretty(&dag_cost_extraction_result).unwrap();

            // Store JSON buffers in the ExtractionResult
            result.tree_cost_json = Some(tree_cost_json);
            result.dag_cost_json = Some(dag_cost_json);

            // print the dag cost
            //print!("print from extractor: dag cost: {}\n", dag_cost);

            // use circuit convertor to conver the json -> processed json -> eqn -> abc rust binding to get the delay

            // first, feed input saturated graph and extracted e-graph to process json
            let saturated_graph_path = "input/rewritten_egraph_with_weight_cost_serd.json";
            let prefix_mapping_path = "../e-rewriter/circuit0.eqn";
            let mode = "small";

            let saturated_graph_json =
                fs::read_to_string(saturated_graph_path).unwrap_or_else(|e| {
                    eprintln!("Failed to read saturated graph file: {}", e);
                    String::new()
                });

            let eqn_content = match process_circuit_conversion(
                &result,
                &saturated_graph_json,
                &prefix_mapping_path,
                mode == "large",
            ) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error in circuit conversion: {}", e);
                    return Default::default(); // or handle the error appropriately
                }
            };

            if let Err(e) = std::fs::write("src/extract/tmp/output.eqn", &eqn_content) {
                eprintln!("Error writing to file: {}", e);
                // Handle the error appropriately
            }

            match call_abc(&eqn_content) {
                Ok(delay) => println!("Circuit delay: {} ns", delay),
                Err(e) => eprintln!("Error in ABC processing: {}", e),
            }

            let el_content =
                fs::read_to_string("src/extract/tmp/opt_1.el").expect("Failed to read el file");
            let csv_content = fs::read_to_string("src/extract/tmp/opt-feats.csv")
                .expect("Failed to read csv file");
            let json_content =
                fs::read_to_string("src/extract/tmp/opt_1.json").expect("Failed to read json file");

            // Call the gRPC client function
            // let delay = match send_circuit_files_to_server(&el_content, &csv_content, &json_content) {
            //     Ok(d) => d,
            //     Err(e) => {
            //         eprintln!("Error sending circuit files to server: {}", e);
            //         0.0 // Use a default value or handle the error as appropriate
            //     }
            // };
            let delay = match send_circuit_files_to_server(&el_content, &csv_content, &json_content)
                .await
            {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error sending circuit files to server: {}", e);
                    0.0 // Use a default value or handle the error as appropriate
                }
            };

            println!("Received delay from ML server: {} ns", delay);

            result
        }
    }
}

impl Extractor for FasterBottomUpExtractorRandom {
    fn extract(
        &self,
        egraph: &EGraph,
        _roots: &[ClassId],
        cost_function: &str,
        random_prob: f64,
    ) -> ExtractionResult {
        let k = random_prob;
        let mut parents = IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
        let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);
        let mut analysis_pending = UniqueQueue::default();

        for class in egraph.classes().values() {
            parents.insert(class.id.clone(), Vec::new());
        }

        for class in egraph.classes().values() {
            for node in &class.nodes {
                for c in &egraph[node].children {
                    // compute parents of this enode
                    parents[n2c(c)].push(node.clone());
                    //println!("Node: {:?}", node);
                }

                // start the analysis from leaves
                if egraph[node].is_leaf() {
                    analysis_pending.insert(node.clone());
                }
            }
        }

        let mut result = ExtractionResult::default();
        let mut costs = FxHashMap::<ClassId, Cost>::with_capacity_and_hasher(
            egraph.classes().len(),
            Default::default(),
        );
        let mut chosen_classes = HashSet::<ClassId>::new(); // 新增的 HashSet
        while let Some(node_id) = analysis_pending.pop() {
            let class_id = n2c(&node_id);
            let node = &egraph[&node_id];
            let prev_cost = costs.get(class_id).unwrap_or(&INFINITY);
            let cost = match cost_function {
                "node_sum_cost" => result.node_sum_cost(egraph, node, &costs),
                "node_depth_cost" => result.node_depth_cost(egraph, node, &costs),
                _ => panic!("Unknown cost function: {}", cost_function),
            };
            let mut rng = rand::thread_rng();
            let random_value: f64 = rng.gen();
            // if     (cost < *prev_cost) {
            //     result.choose(class_id.clone(), node_id.clone());
            //     costs.insert(class_id.clone(), cost);
            //     analysis_pending.extend(parents[class_id].iter().cloned());

            // }

            //version1
            // if ((random_value >= k) && (cost < *prev_cost)) || (*prev_cost == std::f64::INFINITY) {
            //     result.choose(class_id.clone(), node_id.clone());
            //     costs.insert(class_id.clone(), cost);
            //     analysis_pending.extend(parents[class_id].iter().cloned());
            // }

            //version2
            //  if      ((random_value >= k) &&(cost < *prev_cost)) {
            //     result.choose(class_id.clone(), node_id.clone());
            //     costs.insert(class_id.clone(), cost);
            //     analysis_pending.extend(parents[class_id].iter().cloned());
            //     chosen_classes.insert(class_id.clone());
            // }
            //     else if chosen_classes.contains(&class_id) {
            //     continue;}
            //     else{
            //         result.choose(class_id.clone(), node_id.clone());
            //         costs.insert(class_id.clone(), cost);
            //         analysis_pending.extend(parents[class_id].iter().cloned());
            //         chosen_classes.insert(class_id.clone());
            //     }

            //version3
            if prev_cost == &INFINITY && (cost < *prev_cost) {
                result.choose(class_id.clone(), node_id.clone());
                costs.insert(class_id.clone(), cost);
                analysis_pending.extend(parents[class_id].iter().cloned());
            } else if random_value >= k && (cost < *prev_cost) {
                result.choose(class_id.clone(), node_id.clone());
                costs.insert(class_id.clone(), cost);
                analysis_pending.extend(parents[class_id].iter().cloned());
            }
        }

        result
    }
}

impl Extractor for FasterBottomUpSimulatedAnnealingExtractor {
    fn extract(
        &self,
        egraph: &EGraph,
        _roots: &[ClassId],
        cost_function: &str,
        random_prob: f64,
    ) -> ExtractionResult {
        let mut rng = thread_rng();
        let saturated_graph_path = "input/rewritten_egraph_with_weight_cost_serd.json";
        let prefix_mapping_path = "../e-rewriter/circuit0.eqn";

        let saturated_graph_json = fs::read_to_string(saturated_graph_path).unwrap_or_else(|e| {
            eprintln!("Failed to read saturated graph file: {}", e);
            String::new()
        });

        let initial_temp = 100.0;
        let cooling_rate = 0.9;
        let mut temperature = initial_temp;
        let sample_size = (egraph.classes().len() as f64 * 0.3).max(1.0) as usize;
        let max_iterations = 100;
        let iterations_per_temp = 2;
        let min_temperature = 0.1;
        let verbose = true;

        // Generate initial solution
        let mut current_result = generate_initial_solution(egraph, cost_function);
        update_json_buffers_in_result(&mut current_result, egraph); // Add this line
        let mut current_abc_cost = calculate_abc_cost(&current_result, &saturated_graph_json, &prefix_mapping_path);
        let mut best_result = current_result.clone();
        let mut best_abc_cost = current_abc_cost;

        println!("========== Starting Simulated Annealing ==========");
        println!("Initial ABC cost: {:.6}", current_abc_cost);

        while temperature > min_temperature {
            for _ in 0..iterations_per_temp {
                let mut new_result = generate_neighbor_solution(&current_result, egraph, sample_size, &mut rng);
                update_json_buffers_in_result(&mut new_result, egraph); 
                let new_abc_cost = calculate_abc_cost(&new_result, &saturated_graph_json, &prefix_mapping_path);

                let cost_change = new_abc_cost - current_abc_cost;

                if verbose {
                    println!("Current temp: {:.2}; Current ABC cost: {:.6}; New ABC cost: {:.6}, Change: {:.6}", 
                             temperature, current_abc_cost, new_abc_cost, cost_change);
                }

                if cost_change <= 0.0 || rng.gen::<f64>() < (-cost_change / temperature).exp() {
                    current_result = new_result;
                    current_abc_cost = new_abc_cost;

                    if current_abc_cost < best_abc_cost {
                        best_result = current_result.clone();
                        best_abc_cost = current_abc_cost;
                        println!("New best solution found! Cost: {:.6}", best_abc_cost);
                    }
                }
            }

            temperature *= cooling_rate;
        }

        println!("========== Simulated Annealing Complete ==========");
        println!("Best ABC cost: {:.6}", best_abc_cost);

        // Compute final JSON buffers for the best result
        update_json_buffers_in_result(&mut best_result, egraph);

        best_result
    }
}

// ========================== Helper Functions For SA-based faster bottom-up ==========================
// Generate initial solution for Simulated Annealing 
// ========================== Helper Functions For SA-based faster bottom-up ==========================

fn generate_initial_solution(egraph: &EGraph, cost_function: &str) -> ExtractionResult {
    let mut parents = IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);
    let mut analysis_pending = UniqueQueue::default();

    for class in egraph.classes().values() {
        parents.insert(class.id.clone(), Vec::new());
    }

    for class in egraph.classes().values() {
        for node in &class.nodes {
            for c in &egraph[node].children {
                parents[n2c(c)].push(node.clone());
            }
            if egraph[node].is_leaf() {
                analysis_pending.insert(node.clone());
            }
        }
    }

    let mut result = ExtractionResult::default();
    let mut costs = FxHashMap::<ClassId, Cost>::with_capacity_and_hasher(
        egraph.classes().len(),
        Default::default(),
    );

    while let Some(node_id) = analysis_pending.pop() {
        let class_id = n2c(&node_id);
        let node = &egraph[&node_id];
        let prev_cost = costs.get(class_id).unwrap_or(&INFINITY);
        let cost = match cost_function {
            "node_sum_cost" => result.node_sum_cost(egraph, node, &costs),
            "node_depth_cost" => result.node_depth_cost(egraph, node, &costs),
            _ => panic!("Unknown cost function: {}", cost_function),
        };
        if cost < *prev_cost {
            result.choose(class_id.clone(), node_id.clone());
            costs.insert(class_id.clone(), cost);
            analysis_pending.extend(parents[class_id].iter().cloned());
        }
    }

    result
}

// ========================== Helper Functions For SA-based faster bottom-up ==========================
// Generate neighbor solution relate to domain structure
// ========================== Helper Functions For SA-based faster bottom-up ==========================

fn generate_neighbor_solution(
    current: &ExtractionResult, 
    egraph: &EGraph, 
    sample_size: usize, 
    rng: &mut impl Rng
) -> ExtractionResult {
    let mut new_result = current.clone();
    let sampled_classes: Vec<_> = egraph.classes().values().choose_multiple(rng, sample_size);

    for class in sampled_classes {
        if let Some(neighbor_node) = class.nodes.choose(rng) {
            new_result.choose(class.id.clone(), neighbor_node.clone());
        }
    }

    new_result
}

// ========================== Helper Functions For SA-based faster bottom-up ==========================
// Calculate ABC cost for a given solution
// ========================== Helper Functions For SA-based faster bottom-up ==========================

fn calculate_abc_cost(
    result: &ExtractionResult, 
    saturated_graph_json: &str, 
    prefix_mapping_path: &str
) -> f64 {
    let eqn_content = match process_circuit_conversion(
        result,
        saturated_graph_json,
        prefix_mapping_path,
        false,
    ) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error in circuit conversion: {}", e);
            return f64::INFINITY;
        }
    };

    match call_abc(&eqn_content) {
        Ok(delay) => delay as f64,
        Err(e) => {
            eprintln!("Error in ABC processing: {}", e);
            f64::INFINITY
        }
    }
}

// ========================== Helper Functions For SA-based faster bottom-up ==========================
// Update JSON buffers for a given solution
// ========================== Helper Functions For SA-based faster bottom-up ==========================

fn update_json_buffers_in_result(result: &mut ExtractionResult, egraph: &EGraph) {
    let tree_cost_json = to_string_pretty(&result).unwrap();
    let (dag_cost, dag_cost_extraction_result) =
        result.calculate_dag_cost_with_extraction_result(&egraph, &egraph.root_eclasses);
    let dag_cost_json = to_string_pretty(&dag_cost_extraction_result).unwrap();

    result.tree_cost_json = Some(tree_cost_json);
    result.dag_cost_json = Some(dag_cost_json);
}

/** A data structure to maintain a queue of unique elements.

Notably, insert/pop operations have O(1) expected amortized runtime complexity.

Thanks @Bastacyclop for the implementation!
*/
#[derive(Clone)]
#[cfg_attr(feature = "serde-1", derive(Serialize, Deserialize))]
pub(crate) struct UniqueQueue<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    set: FxHashSet<T>, // hashbrown::
    queue: std::collections::VecDeque<T>,
}

impl<T> Default for UniqueQueue<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn default() -> Self {
        UniqueQueue {
            set: Default::default(),
            queue: std::collections::VecDeque::new(),
        }
    }
}

impl<T> UniqueQueue<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn insert(&mut self, t: T) {
        if self.set.insert(t.clone()) {
            self.queue.push_back(t);
        }
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for t in iter.into_iter() {
            self.insert(t);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let res = self.queue.pop_front();
        res.as_ref().map(|t| self.set.remove(t));
        res
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        let r = self.queue.is_empty();
        debug_assert_eq!(r, self.set.is_empty());
        r
    }
}
