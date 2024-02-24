use core::panic;

use super::*;
use coin_cbc::{Col, Model, Sense};
use indexmap::IndexSet;

const INITIALISE_WITH_BOTTOM_UP: bool = false;

struct ClassVars {
    active: Col,
    nodes: Vec<Col>,
}

pub struct CbcExtractor;

impl Extractor for CbcExtractor {
    fn extract(&self, egraph: &EGraph, roots: &[ClassId]) -> ExtractionResult {
        let mut model = Model::default();

        let true_literal = model.add_binary();
        model.set_col_lower(true_literal, 1.0);

        let vars: IndexMap<ClassId, ClassVars> = egraph
            .classes()
            .values()
            .map(|class| {
                let cvars = ClassVars {
                    active: if roots.contains(&class.id) {
                        // Roots must be active.
                        true_literal
                    } else {
                        model.add_binary()
                    },
                    nodes: class.nodes.iter().map(|_| model.add_binary()).collect(),
                };
                (class.id.clone(), cvars)
            })
            .collect();

        for (class_id, class) in &vars {
            // class active == some node active
            // sum(for node_active in class) == class_active
            let row = model.add_row();
            model.set_row_equal(row, 0.0);
            model.set_weight(row, class.active, -1.0);
            for &node_active in &class.nodes {
                model.set_weight(row, node_active, 1.0);
            }

            let childrens_classes_var = |nid: NodeId| {
                egraph[&nid]
                    .children
                    .iter()
                    .map(|n| egraph[n].eclass.clone())
                    .map(|n| vars[&n].active)
                    .collect::<IndexSet<_>>()
            };

            let mut intersection: IndexSet<Col> =
                childrens_classes_var(egraph[class_id].nodes[0].clone());

            for node in &egraph[class_id].nodes[1..] {
                intersection = intersection
                    .intersection(&childrens_classes_var(node.clone()))
                    .cloned()
                    .collect();
            }

            // A class being active implies that all in the intersection
            // of it's children are too.
            for c in &intersection {
                let row = model.add_row();
                model.set_row_upper(row, 0.0);
                model.set_weight(row, class.active, 1.0);
                model.set_weight(row, *c, -1.0);
            }

            for (node_id, &node_active) in egraph[class_id].nodes.iter().zip(&class.nodes) {
                for child_active in childrens_classes_var(node_id.clone()) {
                    // node active implies child active, encoded as:
                    //   node_active <= child_active
                    //   node_active - child_active <= 0
                    if !intersection.contains(&child_active) {
                        let row = model.add_row();
                        model.set_row_upper(row, 0.0);
                        model.set_weight(row, node_active, 1.0);
                        model.set_weight(row, child_active, -1.0);
                    }
                }
            }
        }

        model.set_obj_sense(Sense::Minimize);
        for class in egraph.classes().values() {
            let min_cost = class
                .nodes
                .iter()
                .map(|n_id| egraph[n_id].cost)
                .min()
                .unwrap_or(Cost::default())
                .into_inner();

            // Most helpful when the members of the class all have the same cost.
            // For example if the members' costs are [1,1,1], three terms get
            // replaced by one in the objective function.
            if min_cost != 0.0 {
                model.set_obj_coeff(vars[&class.id].active, min_cost);
            }

            for (node_id, &node_active) in class.nodes.iter().zip(&vars[&class.id].nodes) {
                let node = &egraph[node_id];
                let node_cost = node.cost.into_inner() - min_cost;
                assert!(node_cost >= 0.0);

                if node_cost != 0.0 {
                    model.set_obj_coeff(node_active, node_cost);
                }
            }
        }

        // set initial solution based on bottom up extractor
        if INITIALISE_WITH_BOTTOM_UP {
            let initial_result = super::bottom_up::BottomUpExtractor.extract(egraph, roots);
            for (class, class_vars) in egraph.classes().values().zip(vars.values()) {
                if let Some(node_id) = initial_result.choices.get(&class.id) {
                    model.set_col_initial_solution(class_vars.active, 1.0);
                    for col in &class_vars.nodes {
                        model.set_col_initial_solution(*col, 0.0);
                    }
                    let node_idx = class.nodes.iter().position(|n| n == node_id).unwrap();
                    model.set_col_initial_solution(class_vars.nodes[node_idx], 1.0);
                } else {
                    model.set_col_initial_solution(class_vars.active, 0.0);
                }
            }
        }

        let mut banned_cycles: IndexSet<(ClassId, usize)> = Default::default();
        find_cycles(egraph, |id, i| {
            banned_cycles.insert((id, i));
        });
        for (class_id, class_vars) in &vars {
            for (i, &node_active) in class_vars.nodes.iter().enumerate() {
                if banned_cycles.contains(&(class_id.clone(), i)) {
                    model.set_col_upper(node_active, 0.0);
                    model.set_col_lower(node_active, 0.0);
                }
            }
        }
        log::info!("@blocked {}", banned_cycles.len());

        let solution = model.solve();
        log::info!(
            "CBC status {:?}, {:?}, obj = {}",
            solution.raw().status(),
            solution.raw().secondary_status(),
            solution.raw().obj_value(),
        );

        let mut result = ExtractionResult::default();

        for (id, var) in &vars {
            let active = solution.col(var.active) > 0.0;
            if active {
                let node_idx = var
                    .nodes
                    .iter()
                    .position(|&n| solution.col(n) > 0.0)
                    .unwrap();
                let node_id = egraph[id].nodes[node_idx].clone();
                result.choose(id.clone(), node_id);
            }
        }

        let cycles = result.find_cycles(egraph, roots);
        assert!(cycles.is_empty());
        return result;
    }
}

// from @khaki3
// fixes bug in egg 0.9.4's version
// https://github.com/egraphs-good/egg/issues/207#issuecomment-1264737441
fn find_cycles(egraph: &EGraph, mut f: impl FnMut(ClassId, usize)) {
    let mut pending: IndexMap<ClassId, Vec<(ClassId, usize)>> = IndexMap::default();

    let mut order: IndexMap<ClassId, usize> = IndexMap::default();

    let mut memo: IndexMap<(ClassId, usize), bool> = IndexMap::default();

    let mut stack: Vec<(ClassId, usize)> = vec![];

    let n2c = |nid: &NodeId| egraph.nid_to_cid(nid);

    for class in egraph.classes().values() {
        let id = &class.id;
        for (i, node_id) in egraph[id].nodes.iter().enumerate() {
            let node = &egraph[node_id];
            for child in &node.children {
                let child = n2c(child).clone();
                pending
                    .entry(child)
                    .or_insert_with(Vec::new)
                    .push((id.clone(), i));
            }

            if node.is_leaf() {
                stack.push((id.clone(), i));
            }
        }
    }

    let mut count = 0;

    while let Some((id, i)) = stack.pop() {
        if memo.get(&(id.clone(), i)).is_some() {
            continue;
        }

        let node_id = &egraph[&id].nodes[i];
        let node = &egraph[node_id];
        let mut update = false;

        if node.is_leaf() {
            update = true;
        } else if node.children.iter().all(|x| order.get(n2c(x)).is_some()) {
            if let Some(ord) = order.get(&id) {
                update = node.children.iter().all(|x| &order[n2c(x)] < ord);
                if !update {
                    memo.insert((id, i), false);
                    continue;
                }
            } else {
                update = true;
            }
        }

        if update {
            if order.get(&id).is_none() {
                if egraph[node_id].is_leaf() {
                    order.insert(id.clone(), 0);
                } else {
                    order.insert(id.clone(), count);
                    count += 1;
                }
            }
            memo.insert((id.clone(), i), true);
            if let Some(mut v) = pending.remove(&id) {
                stack.append(&mut v);
                stack.sort();
                stack.dedup();
            };
        }
    }

    for class in egraph.classes().values() {
        let id = &class.id;
        for (i, node) in class.nodes.iter().enumerate() {
            if let Some(true) = memo.get(&(id.clone(), i)) {
                continue;
            }
            assert!(!egraph[node].is_leaf());
            f(id.clone(), i);
        }
    }
    assert!(pending.is_empty());
}
