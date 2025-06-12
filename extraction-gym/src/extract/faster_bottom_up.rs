use super::*;

use rand::prelude::*;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::error::Error;
use std::env;
use std::process;
use tokio::runtime::Runtime;
use std::time::Instant;
//use abc::Abc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fs;
use std::future::Future;
use std::io::Write;
use tempfile::NamedTempFile;
use rand::distributions::WeightedIndex;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tonic::Request;
use vectorservice::vector_service_client::VectorServiceClient;
use vectorservice::CircuitFilesRequest;
pub static VERILOG_COUNTER: AtomicUsize = AtomicUsize::new(0);
use crate::extract::lib::Abc;
use crate::extract::circuit_conversion::process_circuit_conversion;

pub mod vectorservice {
    tonic::include_proto!("vectorservice");
}


pub struct FasterBottomUpExtractorRandom; // extraction method based on random extraction

pub struct FasterBottomUpExtractorRandomIncremental;
pub struct FasterBottomUpFastSimulatedAnnealingExtractorParallel {
    base: Arc<dyn Extractor + Send + Sync>,
}

impl FasterBottomUpFastSimulatedAnnealingExtractorParallel {
    pub fn new<E>(base: E) -> Self
    where
        E: Extractor + Send + Sync + 'static,
    {
        Self { base: Arc::new(base) }
    }
}




// ========================================== SA - Main Entrance ==========================================
impl Extractor for FasterBottomUpFastSimulatedAnnealingExtractorParallel {
    fn extract_par(
        &self,
        egraph: &EGraph,
        _roots: &[ClassId],
        cost_function: &str,
        random_prob: f64,
        num_samples: u32,
    ) -> ExtractionResult {
        let sat_path  = "input/rewritten_egraph_with_weight_cost_serd.json";
        let eqn_path  = "../e-rewriter/circuit0_opt.eqn";
        let sat_json  = std::fs::read_to_string(sat_path).expect("read sat graph");
        let cost_fn_arc: Arc<str> = Arc::from(cost_function);

        let init_extractor: Arc<dyn Extractor + Send + Sync> =
            Arc::new(FasterBottomUpExtractorRandom {});

        let init_vec = generate_base_solution_parallel(
            init_extractor,
            Arc::new(egraph.clone()),
            Arc::from(egraph.root_eclasses.clone()),
            cost_fn_arc.clone(),
            random_prob,
            num_samples,
            None,
        );

        let mut cand = evaluate_candidates_parallel(init_vec, &sat_json, eqn_path)
            .expect("ABC failed on init");

        let mid_idx = cand.len() / 2;
        let (mut cur_res, _cur_json, mut cur_cost, mut cur_v) = cand.remove(mid_idx);
        let mut best_res  = cur_res.clone();
        let mut best_cost = cur_cost;
        let mut best_v    = cur_v.clone();
        let base_cost     = cur_cost;

        let mut rng              = thread_rng();
        let p0          : f64 = 0.95;
        let mut temp0: f64 = -cur_cost / p0.ln();
        let final_temp  : f64 = 1e-1;
        let total_iters : usize = 6;
        let high_temp_iters: usize = 3;

        let temp_mid = temp0 * 0.5; 
        let cooling_high = (temp_mid / temp0).powf(1.0 / high_temp_iters as f64);
        let cooling_low  = (final_temp / temp_mid)
                .powf(1.0 / (total_iters - high_temp_iters) as f64);


            println!(
                "Fast-SA start initial solution for simulated annealing: base = {:.3}, T0 = {:.3}, cool_rate high= {:.4},cool_rate low= {:.4}",
                base_cost, temp0, cooling_high,cooling_low
            );

        for iter in 0..total_iters {
            let neighbour_extractor = self.base.clone();
            let neigh_vec = generate_base_solution_parallel(
                neighbour_extractor,
                Arc::new(egraph.clone()),
                Arc::from(egraph.root_eclasses.clone()),
                cost_fn_arc.clone(),
                random_prob,
                num_samples,
                Some(Arc::new(cur_res.clone())),
            );

            let cand_list = evaluate_candidates_parallel(neigh_vec, &sat_json, eqn_path)
                .expect("ABC failed");

            let (sel_res, _sel_json, sel_cost, sel_v) = if iter < high_temp_iters {
                let min_cost = cand_list.first().unwrap().2;
                let weights: Vec<f64> = cand_list
                    .iter()
                    .map(|(_, _, c, _)| (-(c - min_cost) / temp0).exp())
                    .collect();
                let dist = WeightedIndex::new(&weights).unwrap();
                let idx  = dist.sample(&mut rng);
                cand_list[idx].clone()
            } else {
                cand_list.first().unwrap().clone()
            };
            let cost_diff = sel_cost - cur_cost;
            let accept = if iter < high_temp_iters {
                cost_diff <= 0.0 || rng.gen::<f64>() < (-cost_diff / temp0).exp()
            } else {
                cost_diff <= 0.0
            };

            if accept {
                cur_res  = sel_res;
                cur_cost = sel_cost;
                cur_v    = sel_v;         
            }
            if cur_cost < best_cost {
                best_cost = cur_cost;
                best_res  = cur_res.clone();
                best_v    = cur_v.clone(); 
            }

            if iter < high_temp_iters {
                temp0 *= cooling_high;
            } else {
                temp0 *= cooling_low;
            }
            println!(
                "iter {}/{} | T={:.3} | cur={:.3} | best={:.3} | sel={:.3} | {}",
                iter + 1,
                total_iters,
                temp0,
                cur_cost,
                best_cost,
                sel_cost,
                if accept { "accept" } else { "reject" },
            );
            
        }

        println!(
            "Initial solution = {:.3}, best = {:.3}, \x1b[31m(↓{:.2}%)\x1b[0m",
            base_cost,
            best_cost,
            100.0 * (base_cost - best_cost) / base_cost.max(1.0)
        );

        clean_tmp_verilog(&best_v).expect("cleanup");
        best_res
    }
}


// ==========================================  SA - initial solution parrallel ==========================================
pub fn generate_base_solution_parallel(
    extractor: Arc<dyn Extractor + Send + Sync>,
    egraph: Arc<EGraph>,
    root_eclasses: Arc<[ClassId]>,
    cost_function: Arc<str>,
    random_prob: f64,
    num_samples: u32,
    initial_result: Option<Arc<ExtractionResult>>, 
) -> Vec<ExtractionResult> {
    let (result_sender, result_receiver) = channel();

    run_extract_result_parallel(
        extractor,
        egraph.clone(),
        root_eclasses.clone(),
        cost_function.clone(),
        random_prob,
        num_samples,
        result_sender,
        initial_result,
    );

    let mut extraction_results = Vec::new();
    loop {
        match result_receiver.recv() {
            Ok(extraction_result) => {
                extraction_results.push(extraction_result);
            }
            Err(_) => break,
        }
    }
    

    extraction_results
}
// ========================================== Extractor Interface ==========================================
// Extractor interface for extracting solutions
// ========================================== Random Faster Bottom Up Extractor Interface ==========================================



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

            if  prev_cost ==&INFINITY  {
                result.choose(class_id.clone(), node_id.clone());
                costs.insert(class_id.clone(), cost);
                analysis_pending.extend(parents[class_id].iter().cloned());
            }else if (cost < *prev_cost) {
                result.choose(class_id.clone(), node_id.clone());
                costs.insert(class_id.clone(), cost);
                analysis_pending.extend(parents[class_id].iter().cloned());
            }
            else if(cost == *prev_cost)&&random_value>=k {
                    result.choose(class_id.clone(), node_id.clone());
                    costs.insert(class_id.clone(), cost);
                    analysis_pending.extend(parents[class_id].iter().cloned());
            }
            
        }

        result
    }
}

// ========================================== SA -Internal Incremental Extractor Interface ==========================================

impl Extractor for FasterBottomUpExtractorRandomIncremental {
    fn extract_incremental(
        &self,
        egraph: &EGraph,
        _roots: &[ClassId],
        cost_fn: &str,
        random_prob: f64,
        prev: &ExtractionResult,
    ) -> ExtractionResult {
        let k = random_prob;
        let mut rng = rand::thread_rng();

        let mut parents: IndexMap<ClassId, Vec<NodeId>> =
            egraph.classes().keys().map(|cid| (cid.clone(), Vec::new())).collect();

        for class in egraph.classes().values() {
            for nid in &class.nodes {
                for ch in &egraph[nid].children {
                    parents[egraph.nid_to_cid(ch)].push(nid.clone());
                }
            }
        }
        let mut result = prev.clone();
        let mut costs: FxHashMap<ClassId, Cost> =
            FxHashMap::with_capacity_and_hasher(egraph.classes().len(), Default::default());

        for (cid, nid) in &prev.choices {
            let node = &egraph[nid];
            let c = match cost_fn {
                "node_sum_cost"   => result.node_sum_cost(egraph, node, &costs),
                "node_depth_cost" => result.node_depth_cost(egraph, node, &costs),
                _                 => unreachable!("unknown cost fn {cost_fn}"),
            };
            costs.insert(cid.clone(), c);
        }

        let mut queue = UniqueQueue::<NodeId>::default();
        let sample = ((costs.len() as f64) * 0.10).max(1.0) as usize;

        for (cid, _) in prev.choices.iter().choose_multiple(&mut rng, sample) {
            costs.remove(cid);                    
            for p in &parents[cid] {              
                queue.insert(p.clone());
            }
        }

        while let Some(nid) = queue.pop() {
            let cid  = egraph.nid_to_cid(&nid);
            let node = &egraph[&nid];            
            if node.children.iter().any(|c| !costs.contains_key(egraph.nid_to_cid(c))) {
                continue;
            }

            let new_cost = match cost_fn {
                "node_sum_cost"   => result.node_sum_cost(egraph, node, &costs),
                "node_depth_cost" => result.node_depth_cost(egraph, node, &costs),
                _                 => unreachable!(),
            };
            let old_cost = costs.get(cid).copied().unwrap_or(INFINITY);

            if old_cost == INFINITY
                || new_cost < old_cost
                || (new_cost == old_cost && rng.gen::<f64>() >= k)
            {
                result.choose(cid.clone(), nid.clone());
                costs.insert(cid.clone(), new_cost);
                queue.extend(parents[cid].iter().cloned());
            }
        }

        result
    }
}






pub fn run_extract_result_parallel(
    extractor: Arc<dyn Extractor + Send + Sync>,
    egraph: Arc<EGraph>,
    roots: Arc<[ClassId]>,
    cost_function: Arc<str>,
    k: f64,
    num_samples: u32,
    result_channel: Sender<ExtractionResult>,
    initial_result: Option<Arc<ExtractionResult>>,
) {

    let pool = ThreadPoolBuilder::new()
        .num_threads(64)
        .build()
        .expect("failed to build pool");

    for _ in 0..num_samples {
        let extractor      = Arc::clone(&extractor);
        let egraph         = Arc::clone(&egraph);
        let roots          = Arc::clone(&roots);
        let cost_function  = Arc::clone(&cost_function);
        let result_channel = result_channel.clone();
        let init_clone     = initial_result.clone(); 

        pool.spawn(move || {
      
            let mut result = if let Some(init_arc) = init_clone {
                extractor.extract_incremental(
                    &egraph,
                    &roots,
                    &cost_function,
                    k,
                    init_arc.as_ref(), 
                )
            } else {
                extractor.extract(&egraph, &roots, &cost_function, k)
            };

      
            let (_dag_cost, dag_cost_extraction_result) =
                result.calculate_dag_cost_with_extraction_result(&egraph, &egraph.root_eclasses);

            match to_string_pretty(&dag_cost_extraction_result) {
                Ok(json) => result.dag_cost_json = Some(json),
                Err(e)   => {
                    eprintln!("failed to serialize DAG cost extraction result: {e}");
                    result.dag_cost_json = None;
                }
            }

        
            if let Err(e) = result_channel.send(result) {
                eprintln!("failed to send extraction result: {e}");
            }
        });
    }


}


pub fn run_random_based_extraction(
    extractor: Arc<dyn Extractor + Send + Sync>,
    egraph: Arc<EGraph>,
    root_eclasses: Arc<[ClassId]>,
    cost_function: Arc<str>,
    random_prob: f64,
    num_samples: u32,
    modified_name_for_dag_cost: &str,
) {
    let (result_sender, result_receiver) = channel();

    run_extract_result_parallel(
        extractor,
        egraph.clone(),
        root_eclasses.clone(),
        cost_function.clone(),
        random_prob,
        num_samples,
        result_sender,
        None
    );

    let mut extraction_results: Vec<ExtractionResult> = Vec::new();
    loop {
        match result_receiver.recv() {
            Ok(extraction_result) => {
                extraction_results.push(extraction_result);
            }
            Err(_) => break,
        }
    }

    let modified_name_for_dag_cost = modify_filename(
        modified_name_for_dag_cost,
        "out_dag_json/",
        "random_out_dag_json/",
    );

    for (i, extraction_result) in extraction_results.iter().enumerate() {
        let (dag_cost, dag_cost_extraction_result_depth) = extraction_result
            .calculate_dag_cost_with_extraction_result(&egraph, &root_eclasses);
        let dag_cost_file_name = modify_filename(
            &modified_name_for_dag_cost,
            ".json",
            &format!("_{}.json", i),
        );
        write_json_result(&dag_cost_file_name, &dag_cost_extraction_result_depth);
    }
}






// ========================== Helper Functions For SA-based faster bottom-up ==========================
// Calculate ABC cost for a given solution
// ========================== Helper Functions For SA-based faster bottom-up ==========================



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












// ========================================== Helper Functions and data structure ==========================================
fn clean_tmp_verilog(keep: &str) -> std::io::Result<()> {
    for entry in std::fs::read_dir(".")? {
        let entry     = entry?;
        let file_name = entry.file_name();
        let name      = file_name.to_string_lossy();

        if name.starts_with("tmp_") && name.ends_with(".v") && name != keep {
            std::fs::remove_file(entry.path())?;
        }
    }
    Ok(())
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