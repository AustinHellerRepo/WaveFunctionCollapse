// this abstraction is a web of nodes that have a center and specific states are expected to be closer to each other than further away
// you can imagine a game needing points of interest that are nearby each other - you would not want quest-adjacent locations to be physically distant

use std::{cell::RefCell, rc::Rc};
use std::hash::Hash;
use crate::wave_function::collapsable_wave_function::collapsable_wave_function::CollapsableWaveFunction;

trait PriorityWebNodeValue {
    fn get_proximity(other: &Self) -> f32 where Self: Sized;
}

struct PriorityWeb<TValue: PriorityWebNodeValue> {
    values: Vec<TValue>,
    edges: Vec<(usize, usize)>
}

impl<TValue: PriorityWebNodeValue> PriorityWeb<TValue> {
    pub fn new(values: Vec<TValue>, edges: Vec<(usize, usize)>) -> Self {
        Self {
            values,
            edges,
        }
    }
    pub fn get_collapsable_wave_function<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord, TCollapsableWaveFunction: CollapsableWaveFunction<'a, TNodeState>>(&self, random_instance: Rc<RefCell<fastrand::Rng>>) -> TCollapsableWaveFunction {
        let collapsable_nodes = todo!();
        let collapsable_node_per_id = todo!();
        return TCollapsableWaveFunction::new(collapsable_nodes, collapsable_node_per_id, random_instance);
    }
}