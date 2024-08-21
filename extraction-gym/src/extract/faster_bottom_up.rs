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

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tonic::Request;
use vectorservice::vector_service_client::VectorServiceClient;
use vectorservice::CircuitFilesRequest;

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
pub struct FasterBottomUpExtractor; // baseline faster bottom-up extractor
pub struct FasterBottomUpExtractorGRPC; // extraction method based on faster bottom-up
pub struct FasterBottomUpExtractorRandom; // extraction method based on random extraction
pub struct FasterBottomUpSimulatedAnnealingExtractor; // extraction method based on SA

// ========================================== Neighbor Solution Type ==========================================
// Neighbor solution type for generating neighbor solution
// ========================================== Neighbor Solution Type ==========================================
enum NeighborSolutionType {
    Naive,
    CycleCheck,
    RandomExtraction,
    IntermediatePropagation,
    PermutatePendingNodes,
    RandomSkipInf,
    SmartIntermediatePropagation,
}

fn generate_neighbor_solution(
    current: &ExtractionResult,
    egraph: &EGraph,
    solution_type: NeighborSolutionType,
    sample_size: usize,
    rng: &mut impl Rng,
    cost_function: &str,
    random_prob: f64,
    temperature: f64,
    initial_temp: f64,
) -> ExtractionResult {
    match solution_type {
        NeighborSolutionType::Naive => {
            generate_neighbor_solution_naive(current, egraph, sample_size, rng)
        }
        NeighborSolutionType::CycleCheck => {
            generate_neighbor_solution_with_cycle_check(current, egraph, sample_size, rng)
        }
        NeighborSolutionType::RandomExtraction => {
            generate_neighbor_solution_random_extraction(
                current,
                egraph,
                cost_function,
                random_prob,
                rng,
            )
        }
        NeighborSolutionType::RandomSkipInf => {
            generate_neighbor_solution_skip_inf(
                current,
                egraph,
                cost_function,
                random_prob,
                rng,
            )
        }
        NeighborSolutionType::SmartIntermediatePropagation => {
            generate_neighbor_solution_smart_intermediate_propagation(
                current,
                egraph,
                cost_function,
                random_prob,
                rng,
                temperature,
                initial_temp,
            )
        }
        NeighborSolutionType::IntermediatePropagation => {
            generate_neighbor_solution_intermediate_propagation(
                current,
                egraph,
                cost_function,
                random_prob,
                rng,
            )
        }
        NeighborSolutionType::PermutatePendingNodes => {
            generate_neighbor_solution_permutate_pending_nodes(
                current,
                egraph,
                cost_function,
                random_prob,
                rng,
            )
        }
    }
}

// ========================================== Initial Solution Type ==========================================
// Initial solution type for generating initial solution
// ========================================== Initial Solution Type ==========================================
enum InitialSolutionType {
    Base,
    Random,
}

fn generate_initial_solution(
    egraph: &EGraph,
    solution_type: InitialSolutionType,
    cost_function: &str,
    //rng: &mut impl Rng,
) -> ExtractionResult {
    match solution_type {
        InitialSolutionType::Base => generate_base_solution(egraph, cost_function),
        InitialSolutionType::Random => generate_random_solution(egraph),
    }
}

// ========================================== Extractor Interface ==========================================
// Extractor interface for extracting solutions
// ========================================== Extractor Interface ==========================================
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

            // feed input saturated graph and extracted e-graph to process json
            let saturated_graph_path = "input/rewritten_egraph_with_weight_cost_serd.json";
            let prefix_mapping_path = "../e-rewriter/circuit0_opt.eqn";
            let mode = "large";

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
        let prefix_mapping_path = "../e-rewriter/circuit0_opt.eqn";

        let saturated_graph_json = fs::read_to_string(saturated_graph_path).unwrap_or_else(|e| {
            eprintln!("Failed to read saturated graph file: {}", e);
            String::new()
        });

        // Generate base solution using faster bottom-up
        let mut base_result =
            generate_initial_solution(egraph, InitialSolutionType::Base, cost_function);
        update_json_buffers_in_result(&mut base_result, egraph);
        let base_abc_cost = calculate_abc_cost_or_dump(
            &base_result,
            &saturated_graph_json,
            &prefix_mapping_path,
            false,
        );

        // Generate random initial solution for SA
        let mut current_result =
            generate_initial_solution(egraph, InitialSolutionType::Base, cost_function); // TODO: adjust here
        update_json_buffers_in_result(&mut current_result, egraph);
        let mut current_abc_cost = calculate_abc_cost_or_dump(
            &current_result,
            &saturated_graph_json,
            &prefix_mapping_path,
            false,
        );

        let initial_temp: f64 = 100.0;
        let cooling_rate: f64 = 0.7;
        let mut temperature: f64 = initial_temp;
        let sample_size = (egraph.classes().len() as f64 * 0.3).max(1.0) as usize;
        let iterations_per_temp = 2;
        let min_temperature: f64 = 0.1;
        let verbose = true;

        // Calculate total iterations using logarithms
        //let total_iterations = ((initial_temp / min_temperature).ln() / cooling_rate.ln()).ceil() as u64 * iterations_per_temp as u64;

        // let total interation = ceil( (log(min_temp) - log(initial_temp))/log(cooling_rate) ) * iterations_per_temp
        let total_iterations = ((min_temperature.ln() - initial_temp.ln()) / cooling_rate.ln())
            .ceil() as u64
            * iterations_per_temp as u64;
        let mut best_result = current_result.clone();
        let mut best_abc_cost = current_abc_cost;

        let m = MultiProgress::new();
        let pb = m.add(ProgressBar::new(total_iterations));
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        let panel = m.add(ProgressBar::new(1));
        panel.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {wide_msg}")
                .unwrap(),
        );

        println!("========== Starting Simulated Annealing ==========");
        panel.set_message(format!(
            "Base solution ABC cost: {:.6}. Initial solution's ABC cost: {:.6}",
            base_abc_cost, current_abc_cost
        ));

        let mut iteration_count = 0;

        while temperature > min_temperature {
            for _ in 0..iterations_per_temp {
                let mut new_result = generate_neighbor_solution(
                    &current_result,
                    egraph,
                    NeighborSolutionType::IntermediatePropagation,
                    sample_size,
                    &mut rng,
                    cost_function,
                    0.5,
                    temperature,
                    initial_temp,
                );
                update_json_buffers_in_result(&mut new_result, egraph);
                let new_abc_cost = calculate_abc_cost_or_dump(
                    &new_result,
                    &saturated_graph_json,
                    &prefix_mapping_path,
                    false,
                );

                let cost_change = new_abc_cost - current_abc_cost;

                if verbose {
                    panel.set_message(format!(
                        "Temp: {:.2}\nCurrent ABC cost: {:.6}\nNew ABC cost: {:.6}\nChange: {:.6}",
                        temperature, current_abc_cost, new_abc_cost, cost_change
                    ));
                }

                if cost_change <= 0.0 || rng.gen::<f64>() < (-cost_change / temperature).exp() {
                    current_result = new_result;
                    current_abc_cost = new_abc_cost;

                    if current_abc_cost < best_abc_cost {
                        best_result = current_result.clone();
                        best_abc_cost = current_abc_cost;
                        panel.println(format!(
                            "New best solution found! Cost: {:.6}",
                            best_abc_cost
                        ));
                    }
                }
                iteration_count += 1;
                pb.set_position(iteration_count);
            }

            temperature *= cooling_rate;
        }

        pb.finish_with_message("Simulated Annealing Complete");
        panel.finish_with_message(format!(
            "SA-final ABC cost: {:.6}\nBase solution ABC cost: {:.6}",
            best_abc_cost, base_abc_cost
        ));

        // Compare SA-final with base solution
        if best_abc_cost <= base_abc_cost {
            println!("SA-final solution is better. Returning SA-final.");
            // save the best result to file
            _ = calculate_abc_cost_or_dump(
                &best_result,
                &saturated_graph_json,
                &prefix_mapping_path,
                true,
            );
            best_result
        } else {
            println!("Base solution is better. Returning base solution.");
            // save the base result to file
            _ = calculate_abc_cost_or_dump(
                &base_result,
                &saturated_graph_json,
                &prefix_mapping_path,
                true,
            );
            base_result
        }
    }
}

// ========================== Helper Functions For SA-based faster bottom-up ==========================
// Save best result to file
// ========================== Helper Functions For SA-based faster bottom-up ==========================

// fn save_best_result_to_file(
//     pass;
// }

// ========================== Helper Functions For SA-based faster bottom-up ==========================
// Generate initial or base solution for SA
// ========================== Helper Functions For SA-based faster bottom-up ==========================

// Generate random initial solution for SA
fn generate_random_solution(egraph: &EGraph) -> ExtractionResult {
    let mut rng = thread_rng();
    let mut result = ExtractionResult::default();

    for class in egraph.classes().values() {
        if let Some(random_node) = class.nodes.choose(&mut rng) {
            result.choose(class.id.clone(), random_node.clone());
        }
    }

    result
}

// Generate base solution for Simulated Annealing

fn generate_base_solution(egraph: &EGraph, cost_function: &str) -> ExtractionResult {
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

// easy one

fn generate_neighbor_solution_naive(
    current: &ExtractionResult,
    egraph: &EGraph,
    sample_size: usize,
    rng: &mut impl Rng,
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

//with cycle check

fn generate_neighbor_solution_with_cycle_check(
    current: &ExtractionResult,
    egraph: &EGraph,
    sample_size: usize,
    rng: &mut impl Rng,
) -> ExtractionResult {
    let mut new_result = current.clone();
    let sampled_classes: Vec<_> = egraph.classes().values().choose_multiple(rng, sample_size);

    let mut proposed_changes = Vec::new();

    for class in sampled_classes {
        if let Some(neighbor_node) = class.nodes.choose(rng) {
            proposed_changes.push((class.id.clone(), neighbor_node.clone()));
        }
    }

    // Apply changes sequentially and check for cycles
    let mut temp_result = new_result.clone();

    for (class_id, node_id) in proposed_changes {
        temp_result.choose(class_id.clone(), node_id.clone());
        let cycles = temp_result.find_cycles(egraph, &egraph.root_eclasses);

        if cycles.is_empty() {
            new_result.choose(class_id, node_id);
        } else {
            // Revert the change if it introduces a cycle
            temp_result = new_result.clone();
        }
    }

    new_result
}

// idea from random extraction

fn generate_neighbor_solution_random_extraction(
    current: &ExtractionResult,
    egraph: &EGraph,
    cost_function: &str,
    random_prob: f64,
    rng: &mut impl Rng,
) -> ExtractionResult {
    // map each ClassId to its parent nodeID
    let mut parents = IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);
    let mut analysis_pending = UniqueQueue::default(); // a queue of nodes to be analyzed

    for class in egraph.classes().values() {
        parents.insert(class.id.clone(), Vec::new());
    }

    // build the parent-child relationships
    for class in egraph.classes().values() {
        for node in &class.nodes {
            for c in &egraph[node].children {
                parents[n2c(c)].push(node.clone());
            }
            if egraph[node].is_leaf() {
                analysis_pending.insert(node.clone()); // add leaf nodes to the analysis queue
            }
        }
    }

    // initialize the result and costs
    let mut result = current.clone();
    let mut costs = FxHashMap::<ClassId, Cost>::with_capacity_and_hasher(
        egraph.classes().len(),
        Default::default(),
    );

    // propagate changes from leaves to roots
    while let Some(node_id) = analysis_pending.pop() {
        let class_id = n2c(&node_id);
        let node = &egraph[&node_id];
        let prev_cost = costs.get(class_id).unwrap_or(&INFINITY);

        // generate the cost of the current e-node
        let cost = match cost_function {
            "node_sum_cost" => result.node_sum_cost(egraph, node, &costs),
            "node_depth_cost" => result.node_depth_cost(egraph, node, &costs),
            _ => panic!("Unknown cost function: {}", cost_function),
        };
        let random_value: f64 = rng.gen();

        if prev_cost == &INFINITY || (random_value >= random_prob && cost <= prev_cost * 1.0) {
            result.choose(class_id.clone(), node_id.clone());
            costs.insert(class_id.clone(), cost);
            analysis_pending.extend(parents[class_id].iter().cloned());
        }
    }

    result
}

// add randomization when previous cost is infinity

fn generate_neighbor_solution_skip_inf(
    current: &ExtractionResult,
    egraph: &EGraph,
    cost_function: &str,
    random_prob: f64,
    rng: &mut impl Rng,
) -> ExtractionResult {
    // map each ClassId to its parent nodeID
    let mut parents = IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);
    let mut analysis_pending = UniqueQueue::default(); // a queue of nodes to be analyzed

    for class in egraph.classes().values() {
        parents.insert(class.id.clone(), Vec::new());
    }

    // build the parent-child relationships
    for class in egraph.classes().values() {
        for node in &class.nodes {
            for c in &egraph[node].children {
                parents[n2c(c)].push(node.clone());
            }
            if egraph[node].is_leaf() {
                analysis_pending.insert(node.clone()); // add leaf nodes to the analysis queue
            }
        }
    }

    // initialize the result and costs
    let mut result = current.clone();
    let mut costs = FxHashMap::<ClassId, Cost>::with_capacity_and_hasher(
        egraph.classes().len(),
        Default::default(),
    );

    let mut selected_any = false;
    let mut extended_nodes = Vec::new();

    // propagate changes from leaves to roots
    while let Some(node_id) = analysis_pending.pop() {
        let class_id = n2c(&node_id);
        let node = &egraph[&node_id];
        let prev_cost = costs.get(class_id).unwrap_or(&INFINITY);

        // generate the cost of the current e-node
        let cost = match cost_function {
            "node_sum_cost" => result.node_sum_cost(egraph, node, &costs),
            "node_depth_cost" => result.node_depth_cost(egraph, node, &costs),
            _ => panic!("Unknown cost function: {}", cost_function),
        };
        let random_value: f64 = rng.gen();

        if random_value >= random_prob && cost <= prev_cost * 1.0 {
            result.choose(class_id.clone(), node_id.clone());
            costs.insert(class_id.clone(), cost);
            selected_any = true;
            extended_nodes.extend(parents[class_id].iter().cloned());
        }
    }

    // If no node was selected, choose one randomly from the extended nodes
    if !selected_any && !extended_nodes.is_empty() {
        let random_node = extended_nodes.choose(rng).unwrap();
        let class_id = n2c(random_node);
        result.choose(class_id.clone(), random_node.clone());
    }

    // Add all extended nodes to the analysis pending queue
    analysis_pending.extend(extended_nodes.iter().cloned());

    result
}

// permutate the pending nodes to generate a neighbor solution

fn generate_neighbor_solution_permutate_pending_nodes(
    current: &ExtractionResult,
    egraph: &EGraph,
    cost_function: &str,
    random_prob: f64,
    rng: &mut impl Rng,
) -> ExtractionResult {
    let mut result = current.clone();
    let mut costs = FxHashMap::<ClassId, Cost>::with_capacity_and_hasher(
        egraph.classes().len(),
        Default::default(),
    );

    // Build parent relationships
    let mut parents = IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);

    for class in egraph.classes().values() {
        parents.insert(class.id.clone(), Vec::new());
        for node in &class.nodes {
            for c in &egraph[node].children {
                parents
                    .entry(n2c(c).clone())
                    .or_default()
                    .push(node.clone());
            }
        }
    }

    // Initialize the analysis queue with leaf nodes
    let mut analysis_pending = UniqueQueue::default();
    for class in egraph.classes().values() {
        for node in &class.nodes {
            if egraph[node].is_leaf() {
                analysis_pending.insert(node.clone());
            }
        }
    }

    while let Some(node_id) = analysis_pending.pop() {
        let class_id = n2c(&node_id);
        let node = &egraph[&node_id];
        let prev_cost = costs.get(class_id).unwrap_or(&INFINITY);

        // Calculate the cost of the current node
        let cost = match cost_function {
            "node_sum_cost" => result.node_sum_cost(egraph, node, &costs),
            "node_depth_cost" => result.node_depth_cost(egraph, node, &costs),
            _ => panic!("Unknown cost function: {}", cost_function),
        };

        let random_value: f64 = rng.gen();

        // Decide whether to update the current choice
        if prev_cost == &INFINITY || (random_value >= random_prob && cost < *prev_cost) {
            result.choose(class_id.clone(), node_id.clone());
            costs.insert(class_id.clone(), cost);

            // Permutate and add parents to the analysis queue
            if let Some(parent_nodes) = parents.get(class_id) {
                let mut shuffled_parents = parent_nodes.clone();
                shuffled_parents.shuffle(rng);
                for parent_id in shuffled_parents {
                    analysis_pending.insert(parent_id);
                }
            }
        } else {
            // If we don't update, still add parents to ensure full exploration
            if let Some(parent_nodes) = parents.get(class_id) {
                for parent_id in parent_nodes {
                    analysis_pending.insert(parent_id.clone());
                }
            }
        }
    }

    result
}

// with intermediate node changes propagations

fn generate_neighbor_solution_intermediate_propagation(
    current: &ExtractionResult,
    egraph: &EGraph,
    cost_function: &str,
    random_prob: f64,
    rng: &mut impl Rng,
) -> ExtractionResult {
    let mut result = current.clone();
    let mut costs = FxHashMap::<ClassId, Cost>::with_capacity_and_hasher(
        egraph.classes().len(),
        Default::default(),
    );

    // Build parent relationships
    let mut parents = IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);

    for class in egraph.classes().values() {
        parents.insert(class.id.clone(), Vec::new());
        for node in &class.nodes {
            for c in &egraph[node].children {
                parents
                    .entry(n2c(c).clone())
                    .or_default()
                    .push(node.clone());
            }
        }
    }

    // Choose a random intermediate node to start from
    let start_class = egraph.classes().values().choose(rng).unwrap();
    let start_node = start_class.nodes.choose(rng).unwrap();

    // Initialize the analysis queue with the chosen node
    let mut analysis_pending = UniqueQueue::default();
    analysis_pending.insert(start_node.clone());

    // Propagate changes from the intermediate node to the root
    while let Some(node_id) = analysis_pending.pop() {
        let class_id = n2c(&node_id);
        let node = &egraph[&node_id];
        let prev_cost = costs.get(class_id).unwrap_or(&INFINITY);

        // Calculate the cost of the current node
        let cost = match cost_function {
            "node_sum_cost" => result.node_sum_cost(egraph, node, &costs),
            "node_depth_cost" => result.node_depth_cost(egraph, node, &costs),
            _ => panic!("Unknown cost function: {}", cost_function),
        };

        let random_value: f64 = rng.gen();

        // Decide whether to update the current choice
        if prev_cost == &INFINITY || (random_value >= random_prob && cost < *prev_cost) {
            result.choose(class_id.clone(), node_id.clone());
            costs.insert(class_id.clone(), cost);

            // Propagate changes to parents
            if let Some(parent_nodes) = parents.get(class_id) {
                for parent_id in parent_nodes {
                    analysis_pending.insert(parent_id.clone());
                }
            }
        }
    }

    result
}

fn generate_neighbor_solution_smart_intermediate_propagation(
    current: &ExtractionResult,
    egraph: &EGraph,
    cost_function: &str,
    random_prob: f64,
    rng: &mut impl Rng,
    temperature: f64,
    initial_temp: f64,
) -> ExtractionResult {
    let mut result = current.clone();
    let mut costs = FxHashMap::<ClassId, Cost>::with_capacity_and_hasher(
        egraph.classes().len(),
        Default::default(),
    );

    // Build parent and child relationships
    let mut parents = IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
    let mut children = IndexMap::<ClassId, Vec<NodeId>>::with_capacity(egraph.classes().len());
    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);

    for class in egraph.classes().values() {
        parents.insert(class.id.clone(), Vec::new());
        children.insert(class.id.clone(), Vec::new());
        for node in &class.nodes {
            for c in &egraph[node].children {
                parents
                    .entry(n2c(c).clone())
                    .or_default()
                    .push(node.clone());
                children
                    .entry(class.id.clone())
                    .or_default()
                    .push(c.clone());
            }
        }
    }

    // Calculate node depths
    let mut node_depths = FxHashMap::<ClassId, usize>::default();
    let mut queue = VecDeque::new();
    for (class_id, child_nodes) in &children {
        if child_nodes.is_empty() {
            queue.push_back((class_id.clone(), 0));
        }
    }
    while let Some((class_id, depth)) = queue.pop_front() {
        if node_depths.insert(class_id.clone(), depth).is_none() {
            for parent in &parents[&class_id] {
                queue.push_back((n2c(parent).clone(), depth + 1));
            }
        }
    }

    // Choose a starting node based on temperature
    let temp_ratio = temperature / initial_temp;
    let start_class = if rng.gen::<f64>() < temp_ratio {
        // Choose a node closer to the root when temperature is high
        egraph.classes().values()
            .max_by_key(|class| node_depths.get(&class.id).unwrap_or(&0))
            .unwrap()
    } else {
        // Choose a node closer to the leaves when temperature is low
        egraph.classes().values()
            .min_by_key(|class| node_depths.get(&class.id).unwrap_or(&0))
            .unwrap()
    };
    let start_node = start_class.nodes.choose(rng).unwrap();

    // Initialize the analysis queue with the chosen node
    let mut analysis_pending = UniqueQueue::default();
    analysis_pending.insert(start_node.clone());

    // Propagate changes from the intermediate node
    while let Some(node_id) = analysis_pending.pop() {
        let class_id = n2c(&node_id);
        let node = &egraph[&node_id];
        let prev_cost = costs.get(class_id).unwrap_or(&INFINITY);

        // Calculate the cost of the current node
        let cost = match cost_function {
            "node_sum_cost" => result.node_sum_cost(egraph, node, &costs),
            "node_depth_cost" => result.node_depth_cost(egraph, node, &costs),
            _ => panic!("Unknown cost function: {}", cost_function),
        };

        let random_value: f64 = rng.gen();

        // Decide whether to update the current choice
        if prev_cost == &INFINITY || (random_value >= random_prob && cost < *prev_cost) {
            result.choose(class_id.clone(), node_id.clone());
            costs.insert(class_id.clone(), cost);

            // Propagate changes to parents and children
            if let Some(parent_nodes) = parents.get(class_id) {
                for parent_id in parent_nodes {
                    analysis_pending.insert(parent_id.clone());
                }
            }
            if let Some(child_nodes) = children.get(class_id) {
                for child_id in child_nodes {
                    analysis_pending.insert(child_id.clone());
                }
            }
        }
    }

    result
}

// ========================== Helper Functions For SA-based faster bottom-up ==========================
// Calculate ABC cost for a given solution
// ========================== Helper Functions For SA-based faster bottom-up ==========================

fn calculate_abc_cost_or_dump(
    result: &ExtractionResult,
    saturated_graph_json: &str,
    prefix_mapping_path: &str,
    dump_to_file: bool,
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
    // dump file mode
    if dump_to_file {
        if let Err(e) = std::fs::write("src/extract/tmp/output.eqn", &eqn_content) {
            eprintln!("Error writing to file: {}", e);
            // Handle the error appropriately
        }
        return f64::INFINITY;
    // abc cost calculation mode
    } else {
        match call_abc(&eqn_content) {
            Ok(delay) => delay as f64,
            Err(e) => {
                eprintln!("Error in ABC processing: {}", e);
                f64::INFINITY
            }
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

// ========================================== Helper Functions For SA-based faster bottom-up ==========================================
// Send circuit files to server
// ========================================== Helper Functions For SA-based faster bottom-up ==========================================
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

// ========================================== Helper Functions For SA-based faster bottom-up ==========================================
// Call ABC
// ========================================== Helper Functions For SA-based faster bottom-up ==========================================

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
    abc.execute_command(&format!("read_lib ../abc/asap7_clean.lib"));
    //println!("Performing structural hashing...");
    abc.execute_command(&format!("strash"));
    //println!("Performing dump the edgelist...");
    abc.execute_command(&format!("&get; &edgelist  -F src/extract/tmp/opt_1.el -f src/extract/tmp/opt-feats.csv -c src/extract/tmp/opt_1.json; &put"));
    abc.execute_command(&format!("dch"));
    //println!("Performing technology mapping...");
    abc.execute_command(&format!("map"));
    //println!("Performing post-processing...(topo; gate sizing)");
    abc.execute_command(&format!("topo"));
    abc.execute_command(&format!("upsize"));
    abc.execute_command(&format!("dnsize"));

    //println!("Executing stime command...");
    let stime_output = abc.execute_command_with_output(&format!("stime -d"));

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
