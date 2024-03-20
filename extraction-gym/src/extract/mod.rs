use indexmap::IndexMap;
use rustc_hash::FxHashMap;
use std::collections::HashMap;

pub use crate::*;

pub mod bottom_up;
pub mod faster_bottom_up;
pub mod faster_greedy_dag;
pub mod global_greedy_dag;
pub mod greedy_dag;
use serde::{Deserialize, Serialize};
use rustc_hash::FxHashSet;
use std::collections::HashSet;
use rand::Rng;
#[cfg(feature = "ilp-cbc")]
pub mod ilp_cbc;

pub trait Extractor: Sync {
    fn extract(&self, egraph: &EGraph, roots: &[ClassId]) -> ExtractionResult;

    fn boxed(self) -> Box<dyn Extractor>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

pub trait MapGet<K, V> {
    fn get(&self, key: &K) -> Option<&V>;
}

impl<K, V> MapGet<K, V> for HashMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    fn get(&self, key: &K) -> Option<&V> {
        HashMap::get(self, key)
    }
}

impl<K, V> MapGet<K, V> for FxHashMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    fn get(&self, key: &K) -> Option<&V> {
        FxHashMap::get(self, key)
    }
}

impl<K, V> MapGet<K, V> for IndexMap<K, V>
where
    K: Eq + std::hash::Hash,
{
    fn get(&self, key: &K) -> Option<&V> {
        IndexMap::get(self, key)
    }
}

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct ExtractionResult {
    pub choices: IndexMap<ClassId, NodeId>,
    
}

pub struct Cost_extract {
   
}


#[derive(Clone, Copy)]
enum Status {
    Doing,
    Done,
}

impl ExtractionResult {
    pub fn choose(&mut self, class_id: ClassId, node_id: NodeId) {
        self.choices.insert(class_id, node_id);
    }

    pub fn find_cycles(&self, egraph: &EGraph, roots: &[ClassId]) -> Vec<ClassId> {
        // let mut status = vec![Status::Todo; egraph.classes().len()];
        let mut status = IndexMap::<ClassId, Status>::default();
        let mut cycles = vec![];
        for root in roots {
            // let root_index = egraph.classes().get_index_of(root).unwrap();
            self.cycle_dfs(egraph, root, &mut status, &mut cycles)
        }
        cycles
    }

    fn cycle_dfs(
        &self,
        egraph: &EGraph,
        class_id: &ClassId,
        status: &mut IndexMap<ClassId, Status>,
        cycles: &mut Vec<ClassId>,
    ) {
        match status.get(class_id).cloned() {
            Some(Status::Done) => (),
            Some(Status::Doing) => cycles.push(class_id.clone()),
            None => {
                status.insert(class_id.clone(), Status::Doing);
                //print!("class id {}\n",class_id);
                let node_id = &self.choices[class_id];
                let node = &egraph[node_id];
                for child in &node.children {
                    let child_cid = egraph.nid_to_cid(child);
                    self.cycle_dfs(egraph, child_cid, status, cycles)
                }
                status.insert(class_id.clone(), Status::Done);
            }
        }
    }

    pub fn tree_cost(&self, egraph: &EGraph, roots: &[ClassId]) -> Cost {
        let node_roots = roots
            .iter()
            .map(|cid| self.choices[cid].clone())
            .collect::<Vec<NodeId>>();
        self.tree_cost_rec(egraph, &node_roots, &mut HashMap::new())
    }

    fn tree_cost_rec(
        &self,
        egraph: &EGraph,
        roots: &[NodeId],
        memo: &mut HashMap<NodeId, Cost>,
    ) -> Cost {
        let mut cost = Cost::default();
        for root in roots {
            if let Some(c) = memo.get(root) {
                cost += *c;
                continue;
            }
            let class = egraph.nid_to_cid(root);
            let node = &egraph[&self.choices[class]];
            let inner = node.cost + self.tree_cost_rec(egraph, &node.children, memo);
            memo.insert(root.clone(), inner);
            cost += inner;
        }
        cost
    }

    // this will loop if there are cycles
    pub fn dag_cost(&self, egraph: &EGraph, roots: &[ClassId]) -> Cost {
        let mut costs: IndexMap<ClassId, Cost> = IndexMap::new();
        let mut todo: Vec<ClassId> = roots.to_vec();
        while let Some(cid) = todo.pop() {
            let node_id = &self.choices[&cid];
            let node = &egraph[node_id];
            if costs.insert(cid.clone(), node.cost).is_some() {
                continue;
            }
            for child in &node.children {
                todo.push(egraph.nid_to_cid(child).clone());
            }
        }
        costs.values().sum()
    }

    pub fn dag_cost_with_extraction_result(
        &self,
        egraph: &EGraph,
        roots: &[ClassId],
    ) -> (Cost, ExtractionResult) {
        let mut costs: IndexMap<ClassId, Cost> = IndexMap::new();
        let mut todo: Vec<ClassId> = roots.to_vec();
        let mut extraction_result = ExtractionResult {
            choices: IndexMap::new(),
        };

        while let Some(cid) = todo.pop() {
            let node_id = &self.choices[&cid];
            let node = &egraph[node_id];
            if costs.insert(cid.clone(), node.cost).is_some() {
                continue;
            }
            extraction_result
                .choices
                .insert(cid.clone(), node_id.clone());
            for child in &node.children {
                todo.push(egraph.nid_to_cid(child).clone());
            }
        }

        let total_cost = costs.values().sum();

        (total_cost, extraction_result)
    }

    pub fn node_sum_cost<M>(&self, egraph: &EGraph, node: &Node, costs: &M) -> Cost
    where
        M: MapGet<ClassId, Cost>,
    {
        node.cost
            + node
                .children
                .iter()
                .map(|n| {
                    let cid = egraph.nid_to_cid(n);
                    costs.get(cid).unwrap_or(&INFINITY)
                })
                .sum::<Cost>()
    }

    pub fn node_depth_cost<M>(&self, egraph: &EGraph, node: &Node, costs: &M) -> Cost
where
    M: MapGet<ClassId, Cost>,
{
    let child_max_cost = node
        .children
        .iter()
        .map(|n| {
            let cid = egraph.nid_to_cid(n);
            costs.get(cid).unwrap_or(&INFINITY)
        })
        .max()
        .copied()
        .unwrap_or(Cost::default());

    node.cost+child_max_cost
}



pub fn record_costs_random(&self, num_runs: u32, random_ratio: f64,egraph: &EGraph,dag_cost_with_extraction_result:&ExtractionResult)  {
    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);

    for num in 0..num_runs {
        let mut result: FxHashMap<ClassId, NodeId> = FxHashMap::default();
        let mut selected_ids: FxHashSet<ClassId> = HashSet::default(); // 用于跟踪已选择的节点 Id
        for classid in dag_cost_with_extraction_result.choices.keys() {
            let class =egraph.classes().get(classid).unwrap();
            let nodes=class.nodes.clone();
            let mut rng = rand::thread_rng();
        //   if input_vec_id.contains(id) {
        //     let value = format!("{}.{}", id, index);
        //     let value1 = &eclass.nodes[*index];
        //     result.insert(id.to_string(), value);
        //     result1.insert(id.to_string(), value1.clone());
        //     selected_ids.insert(*id);}

            if !selected_ids.contains(&classid) && rng.gen::<f64>() <= random_ratio && nodes.len() > 1 {
                let random_index = rng.gen_range(0..nodes.len());
                result.insert(classid.clone(), class.nodes[random_index].clone());
            
                // Add the selected node ID to the set
                selected_ids.insert(classid.clone());
            }
             else {
            result.insert(class.id.clone(), dag_cost_with_extraction_result.choices[classid].clone());
        }
    }
        
        let filename = format!("result{}.json", num);
        let path = format!("random_result/{}", filename);

        // Create directory if it doesn't exist
        if let Err(err) = fs::create_dir_all("random_result") {
            eprintln!("Failed to create directory: {}", err);
            continue; // Skip current iteration if directory creation fails
        }

        if let Ok(mut file) = File::create(path) {
            let json_dag_result =  to_string_pretty(&result).unwrap() ;
                let _ = write!(file, "{}", json_dag_result);

        }

}









}
}
