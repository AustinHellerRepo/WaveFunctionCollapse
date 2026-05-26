use std::collections::HashMap;
use std::hash::Hash;
use bitvec::vec::BitVec;
use super::collapsable_wave_function::{CollapsableWaveFunction, CollapsableNode, CollapsedNodeState, CollapsedWaveFunction};

pub struct AccommodatingCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    collapsable_nodes: Vec<CollapsableNode<'a, TNodeState>>,
    accommodate_node_indices: Vec<usize>,
    accommodate_node_indices_length: usize,
    accommodate_node_indices_index: usize,
    accommodated_total: usize,
    impacted_node_indices: BitVec,
    impacted_node_count: usize,
    random_instance: fastrand::Rng,
}

fn collect_masks_from_node<T: Eq + Hash + Clone + std::fmt::Debug + Ord>(
    nodes: &[CollapsableNode<'_, T>],
    node_index: usize,
) -> Vec<(usize, BitVec)> {
    let node = &nodes[node_index];
    let mut m = Vec::new();
    if let Some(state_index) = node.node_state_indexed_view.get_index() {
        if state_index < node.masks_by_state_index.len() {
            let mask_map = &node.masks_by_state_index[state_index];
            for &neighbor_index in &node.neighbor_node_indices {
                if let Some(mask) = mask_map.get(&neighbor_index) {
                    m.push((neighbor_index, mask.clone()));
                }
            }
        }
    }
    m
}

fn collect_masks_for_state_index<T: Eq + Hash + Clone + std::fmt::Debug + Ord>(
    nodes: &[CollapsableNode<'_, T>],
    node_index: usize,
    state_index: usize,
) -> Vec<(usize, BitVec)> {
    let node = &nodes[node_index];
    let mut m = Vec::new();
    if state_index < node.masks_by_state_index.len() {
        let mask_map = &node.masks_by_state_index[state_index];
        for &neighbor_index in &node.neighbor_node_indices {
            if let Some(mask) = mask_map.get(&neighbor_index) {
                m.push((neighbor_index, mask.clone()));
            }
        }
    }
    m
}

/// Check if the parent at parent_index, when in the given state, would restrict the current node
/// Returns the mask if one exists, None otherwise
fn get_restricting_mask_from_parent_to_current<T: Eq + Hash + Clone + std::fmt::Debug + Ord>(
    nodes: &[CollapsableNode<'_, T>],
    parent_index: usize,
    current_index: usize,
    parent_state_index: usize,
) -> Option<BitVec> {
    let parent = &nodes[parent_index];
    if parent_state_index < parent.masks_by_state_index.len() {
        parent.masks_by_state_index[parent_state_index].get(&current_index).cloned()
    } else {
        None
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> AccommodatingCollapsableWaveFunction<'a, TNodeState> {
    fn initialize_nodes(&mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {
        let mut initial = Vec::new();
        for (idx, node) in self.collapsable_nodes.iter_mut().enumerate() {
            if !node.node_state_indexed_view.try_move_next() {
                return Err(String::from("Cannot collapse wave function."));
            }
            self.accommodate_node_indices.push(idx);
            let state = node.node_state_indexed_view.get().unwrap();
            initial.push(CollapsedNodeState {
                node_id: String::from(node.id),
                node_state_id: Some((*state).clone()),
            });
        }
        self.accommodate_node_indices_length = self.accommodate_node_indices.len();
        self.accommodated_total = self.accommodate_node_indices_length;

        let mut all_masks = Vec::new();
        for idx in 0..self.collapsable_nodes.len() {
            all_masks.extend(collect_masks_from_node(&self.collapsable_nodes, idx));
        }
        for (target, mask) in all_masks {
            self.collapsable_nodes[target].add_mask(&mask);
        }
        Ok(initial)
    }

    fn is_fully_collapsed(&self) -> bool {
        self.accommodated_total == 0
    }

    fn prepare_nodes_for_iteration(&mut self) {
        self.accommodate_node_indices_index = 0;
        self.random_instance.shuffle(&mut self.accommodate_node_indices);
        self.accommodated_total = 0;
        for i in 0..self.impacted_node_count {
            self.impacted_node_indices.set(i, false);
        }
    }

    fn is_done_accommodating_nodes(&self) -> bool {
        self.accommodate_node_indices_index == self.accommodate_node_indices_length
    }

    fn is_current_node_in_conflict(&mut self) -> bool {
        let ci = self.accommodate_node_indices[self.accommodate_node_indices_index];
        let node = &self.collapsable_nodes[ci];
        let mut conflict = node.node_state_indexed_view.is_current_state_restricted();

        if self.impacted_node_indices.get(ci).map_or(false, |v| *v) {
            conflict = false;
        } else {
            for &pi in &node.parent_neighbor_node_indices {
                if self.impacted_node_indices.get(pi).map_or(false, |v| *v) {
                    conflict = false;
                    break;
                }
            }
        }

        if !conflict {
            self.accommodate_node_indices_index += 1;
        }
        conflict
    }

    fn accommodate_current_node(&mut self) -> Vec<CollapsedNodeState<TNodeState>> {
        let mut changed_states = Vec::new();
        let mut state_changes: Vec<(usize, usize, usize, String)> = Vec::new();

        let current_index = self.accommodate_node_indices[self.accommodate_node_indices_index];
        self.impacted_node_indices.set(current_index, true);

        let parent_indices: Vec<usize> = self.collapsable_nodes[current_index].parent_neighbor_node_indices.clone();

        for &parent_index in &parent_indices {
            self.impacted_node_indices.set(parent_index, true);

            let original_state_index = self.collapsable_nodes[parent_index].node_state_indexed_view.get_index().unwrap();
            let parent_id = String::from(self.collapsable_nodes[parent_index].id);

            let mut current_state_index = original_state_index;
            let mut is_restrictive = true;

            while is_restrictive {
                let restricting_mask = get_restricting_mask_from_parent_to_current(
                    &self.collapsable_nodes,
                    parent_index,
                    current_index,
                    current_state_index,
                );

                let is_this_restrictive = match restricting_mask {
                    Some(ref mask) => {
                        let cn = &self.collapsable_nodes[current_index];
                        cn.node_state_indexed_view.is_mask_restrictive_to_current_state(mask)
                    }
                    None => false,
                };

                if !is_this_restrictive {
                    is_restrictive = false;
                    // Move parent to current_state_index by iterating
                    {
                        let parent = &mut self.collapsable_nodes[parent_index];
                        loop {
                            let ci = parent.node_state_indexed_view.get_index().unwrap();
                            if ci == current_state_index {
                                break;
                            }
                            parent.node_state_indexed_view.move_next();
                        }
                    }

                    if current_state_index != original_state_index {
                        changed_states.push(CollapsedNodeState {
                            node_id: parent_id.clone(),
                            node_state_id: Some((*self.collapsable_nodes[parent_index].node_state_indexed_view.get().unwrap()).clone()),
                        });
                        state_changes.push((parent_index, original_state_index, current_state_index, parent_id.clone()));
                    }
                } else {
                    {
                        let parent = &mut self.collapsable_nodes[parent_index];
                        parent.node_state_indexed_view.move_next();
                    }
                    let next_state_index = self.collapsable_nodes[parent_index].node_state_indexed_view.get_index().unwrap();
                    if next_state_index == original_state_index {
                        break;
                    }
                    current_state_index = next_state_index;
                }
            }
        }

        // Apply mask changes
        for (parent_index, original_state_index, current_state_index, _id) in state_changes {
            let subtract = collect_masks_for_state_index(&self.collapsable_nodes, parent_index, original_state_index);
            for (target, mask) in subtract {
                self.collapsable_nodes[target].subtract_mask(&mask);
            }
            let add = collect_masks_for_state_index(&self.collapsable_nodes, parent_index, current_state_index);
            for (target, mask) in add {
                self.collapsable_nodes[target].add_mask(&mask);
            }
        }

        self.accommodated_total += 1;
        changed_states
    }

    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction<TNodeState> {
        let mut m = HashMap::new();
        for node in &self.collapsable_nodes {
            let state: TNodeState = (*node.node_state_indexed_view.get().unwrap()).clone();
            m.insert(String::from(node.id), state);
        }
        CollapsedWaveFunction { node_state_per_node_id: m }
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for AccommodatingCollapsableWaveFunction<'a, TNodeState> {
    fn new(
        collapsable_nodes: Vec<CollapsableNode<'a, TNodeState>>,
        random_instance: fastrand::Rng,
    ) -> Self {
        let node_count = collapsable_nodes.len();
        AccommodatingCollapsableWaveFunction {
            collapsable_nodes,
            accommodate_node_indices: Vec::new(),
            accommodate_node_indices_length: 0,
            accommodate_node_indices_index: 0,
            accommodated_total: 0,
            impacted_node_indices: bitvec::bitvec![0; node_count],
            impacted_node_count: node_count,
            random_instance,
        }
    }

    fn collapse(&mut self) -> Result<CollapsedWaveFunction<TNodeState>, String> {
        let init = self.initialize_nodes();
        if init.is_err() {
            return Err(init.err().unwrap());
        }
        let mut iterations_total: u32 = 0;
        while !self.is_fully_collapsed() {
            self.prepare_nodes_for_iteration();
            while !self.is_done_accommodating_nodes() {
                if self.is_current_node_in_conflict() {
                    self.accommodate_current_node();
                }
                iterations_total += 1;
            }
        }
        debug!("fully collapsed after {:?} iterations", iterations_total);
        Ok(self.get_collapsed_wave_function())
    }

    fn collapse_into_steps(&mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {
        let mut collapsed_node_states = Vec::new();
        let init = self.initialize_nodes();
        if init.is_err() {
            return Err(init.err().unwrap());
        }
        collapsed_node_states.extend(init.unwrap());
        while !self.is_fully_collapsed() {
            self.prepare_nodes_for_iteration();
            while !self.is_done_accommodating_nodes() {
                if self.is_current_node_in_conflict() {
                    collapsed_node_states.extend(self.accommodate_current_node());
                }
            }
        }
        Ok(collapsed_node_states)
    }
}
