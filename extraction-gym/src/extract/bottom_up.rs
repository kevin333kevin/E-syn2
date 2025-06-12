use super::*;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use rustc_hash::FxHashMap;
use std::collections::VecDeque;
//use rand::Rng;
use rand::prelude::IteratorRandom;

pub struct BottomUpExtractor;
impl Extractor for BottomUpExtractor {
    fn extract(&self, egraph: &EGraph, _roots: &[ClassId], cost_function: &str, random_prob: f64) -> ExtractionResult {
        let mut result = ExtractionResult::default();
        let mut costs = FxHashMap::<ClassId, Cost>::with_capacity_and_hasher(
            egraph.classes().len(),
            Default::default(),
        );
        let mut did_something = false;

        let use_bfs = true;
        // Perform topological sort
        let sorted_classes = if use_bfs {
            topological_sort_bfs(egraph)
        } else {
            topological_sort_dfs(egraph)
        };
        println!("Topological sort completed. Number of classes: {}", sorted_classes.len());


        loop {
            for class in egraph.classes().values() {
                for node in &class.nodes {
                    let cost = result.node_sum_cost(egraph, &egraph[node], &costs);
                    if &cost < costs.get(&class.id).unwrap_or(&INFINITY) {
                        result.choose(class.id.clone(), node.clone());
                        costs.insert(class.id.clone(), cost);
                        did_something = true;
                    }
                }
            }

            //println!("Costs: {:?}", costs);

            if did_something {
                did_something = false;
            } else {
                break;
            }
        }

        result
    }
}
