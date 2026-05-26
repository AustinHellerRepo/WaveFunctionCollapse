use std::collections::HashMap;
use std::hash::Hash;
use bitvec::vec::BitVec;
use crate::wave_function::indexed_view::IndexedViewMaskState;
use super::collapsable_wave_function::{CollapsableNode, CollapsedNodeState, CollapsedWaveFunction, CollapsableWaveFunction};

pub struct AccommodatingSequentialCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    collapsable_nodes: Vec<CollapsableNode<'a, TNodeState>>,
    spread_node_indices: Vec<usize>,
    spread_node_indices_length: usize,
    spread_node_indices_index: usize,
    impacted_node_indices: BitVec,
    impacted_node_count: usize,
    stash_per_neighbor_node_index: HashMap<usize, IndexedViewMaskState>,
    original_node_state_per_node_index: HashMap<usize, usize>,
    current_neighbor_node_indices: Vec<usize>,
    great_neighbor_node_indices_per_neighbor_node_index: HashMap<usize, Vec<usize>>,
    nongreat_neighbor_node_indices_per_neighbor_node_index: HashMap<usize, Vec<usize>>,
    current_neighbor_node_indices_index: usize,
    current_neighbor_node_indices_length: usize,
    is_current_neighbor_node_cycle_required: bool,
    is_current_node_neighbors_collapse_possible: bool,
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

fn collect_masks_for_state_index_for_targets<T: Eq + Hash + Clone + std::fmt::Debug + Ord>(
    nodes: &[CollapsableNode<'_, T>],
    node_index: usize,
    state_index: usize,
    target_indices: &[usize],
) -> Vec<(usize, BitVec)> {
    let node = &nodes[node_index];
    let mut m = Vec::new();
    if state_index < node.masks_by_state_index.len() {
        let mask_map = &node.masks_by_state_index[state_index];
        for &target in target_indices {
            if let Some(mask) = mask_map.get(&target) {
                m.push((target, mask.clone()));
            }
        }
    }
    m
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> AccommodatingSequentialCollapsableWaveFunction<'a, TNodeState> {
    fn initialize_nodes(&mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {
        let mut initial = Vec::new();
        for (idx, node) in self.collapsable_nodes.iter_mut().enumerate() {
            if !node.node_state_indexed_view.try_move_next() {
                return Err(String::from("Cannot collapse wave function."));
            }
            self.spread_node_indices.push(idx);
            let state = node.node_state_indexed_view.get().unwrap();
            initial.push(CollapsedNodeState {
                node_id: String::from(node.id),
                node_state_id: Some((*state).clone()),
            });
        }
        self.spread_node_indices_length = self.spread_node_indices.len();

        // Collect all masks, then apply
        let mut all_masks: Vec<(usize, BitVec)> = Vec::new();
        for idx in 0..self.collapsable_nodes.len() {
            all_masks.extend(collect_masks_from_node(&self.collapsable_nodes, idx));
        }
        for (target, mask) in all_masks {
            self.collapsable_nodes[target].add_mask(&mask);
        }
        Ok(initial)
    }

    fn is_fully_collapsed(&mut self) -> bool {
        for node in &mut self.collapsable_nodes {
            if node.is_fully_restricted() {
                return false;
            }
        }
        true
    }

    fn prepare_nodes_for_iteration(&mut self) {
        self.spread_node_indices_index = 0;
        self.random_instance.shuffle(&mut self.spread_node_indices);
        for i in 0..self.impacted_node_count {
            self.impacted_node_indices.set(i, false);
        }
    }

    fn is_done_spreading_nodes(&self) -> bool {
        self.spread_node_indices_index == self.spread_node_indices_length
    }

    fn is_current_node_in_conflict(&self) -> bool {
        let ci = self.spread_node_indices[self.spread_node_indices_index];
        let node = &self.collapsable_nodes[ci];
        let mut conflict = node.node_state_indexed_view.is_current_state_restricted();
        if !conflict {
            for &ni in &node.neighbor_node_indices {
                if self.collapsable_nodes[ni].node_state_indexed_view.is_current_state_restricted() {
                    conflict = true;
                    break;
                }
            }
        }
        if !conflict {
            for &pi in &node.parent_neighbor_node_indices {
                if self.collapsable_nodes[pi].node_state_indexed_view.is_current_state_restricted() {
                    conflict = true;
                    break;
                }
            }
        }
        if self.impacted_node_indices.get(ci).map_or(false, |v| *v) {
            conflict = false;
        } else {
            for &pi in &node.parent_neighbor_node_indices {
                if self.impacted_node_indices.get(pi).map_or(false, |v| *v) {
                    conflict = false;
                    break;
                }
            }
            if !conflict {
                for &ni in &node.neighbor_node_indices {
                    if self.impacted_node_indices.get(ni).map_or(false, |v| *v) {
                        conflict = false;
                        break;
                    }
                }
            }
        }
        conflict
    }

    fn prepare_current_node_neighbors(&mut self) {
        let ci = self.spread_node_indices[self.spread_node_indices_index];
        let current = &self.collapsable_nodes[ci];
        self.current_neighbor_node_indices.clear();
        self.current_neighbor_node_indices.extend(&current.neighbor_node_indices);
        self.current_neighbor_node_indices.extend(&current.parent_neighbor_node_indices);
        self.current_neighbor_node_indices.sort();
        self.current_neighbor_node_indices.dedup();

        // Collect all neighbor indices first (owned copies)
        let neighbor_indices: Vec<usize> = self.current_neighbor_node_indices.clone();
        let ci_val = ci;

        // Subtract current node's masks from neighbors
        let ci_state_index = self.collapsable_nodes[ci].node_state_indexed_view.get_index().unwrap();
        let subtract_masks = collect_masks_for_state_index(&self.collapsable_nodes, ci, ci_state_index);
        for (target, mask) in subtract_masks {
            self.collapsable_nodes[target].subtract_mask(&mask);
        }

        // Collect states and remove neighbor masks
        for &ni in &neighbor_indices {
            let ni_state_index = self.collapsable_nodes[ni].node_state_indexed_view.get_index().unwrap();
            self.original_node_state_per_node_index.insert(ni, ni_state_index);
            let masks = collect_masks_for_state_index(&self.collapsable_nodes, ni, ni_state_index);
            for (target, mask) in masks {
                self.collapsable_nodes[target].subtract_mask(&mask);
            }
        }

        // Stash masks
        for &ni in &neighbor_indices {
            let mask_state = self.collapsable_nodes[ni].node_state_indexed_view.stash_mask_state();
            self.stash_per_neighbor_node_index.insert(ni, mask_state);
        }

        // Add current node masks
        let add_masks = collect_masks_for_state_index(&self.collapsable_nodes, ci, ci_state_index);
        for (target, mask) in add_masks {
            self.collapsable_nodes[target].add_mask(&mask);
        }

        // Shuffle
        self.random_instance.shuffle(&mut self.current_neighbor_node_indices);

        // Great / nongreat
        self.great_neighbor_node_indices_per_neighbor_node_index.clear();
        self.nongreat_neighbor_node_indices_per_neighbor_node_index.clear();
        for &ni in &self.current_neighbor_node_indices {
            let neighbor = &self.collapsable_nodes[ni];
            let mut possible: Vec<usize> = Vec::new();
            possible.extend(&neighbor.neighbor_node_indices);
            possible.extend(&neighbor.parent_neighbor_node_indices);
            possible.sort();
            possible.dedup();
            let mut great = Vec::new();
            let mut nongreat = Vec::new();
            for &p in &possible {
                if p == ci_val || self.current_neighbor_node_indices.contains(&p) {
                    great.push(p);
                } else {
                    nongreat.push(p);
                }
            }
            self.great_neighbor_node_indices_per_neighbor_node_index.insert(ni, great);
            self.nongreat_neighbor_node_indices_per_neighbor_node_index.insert(ni, nongreat);
        }

        self.current_neighbor_node_indices_index = 0;
        self.current_neighbor_node_indices_length = self.current_neighbor_node_indices.len();
        self.is_current_neighbor_node_cycle_required = false;
        self.is_current_node_neighbors_collapse_possible = true;
    }

    fn is_current_node_neighbors_collapsed(&self) -> bool {
        !(self.current_neighbor_node_indices_index < self.current_neighbor_node_indices_length && self.is_current_node_neighbors_collapse_possible)
    }

    fn is_current_node_neighbor_state_change_required(&self) -> bool {
        if self.is_current_neighbor_node_cycle_required {
            return true;
        }
        let ni = self.current_neighbor_node_indices[self.current_neighbor_node_indices_index];
        self.collapsable_nodes[ni].node_state_indexed_view.is_current_state_restricted()
    }

    fn change_state_of_current_node_neighbor(&mut self) -> Vec<CollapsedNodeState<TNodeState>> {
        let mut changed = Vec::new();
        self.is_current_neighbor_node_cycle_required = false;

        let neighbor_index = self.current_neighbor_node_indices[self.current_neighbor_node_indices_index];
        let original_state_index = self.original_node_state_per_node_index[&neighbor_index];
        let original_state = *self.collapsable_nodes[neighbor_index].node_state_indexed_view.get_state_by_index(original_state_index).unwrap();

        let is_successful = self.collapsable_nodes[neighbor_index].node_state_indexed_view.try_move_next_cycle(&original_state);
        let new_state_index = self.collapsable_nodes[neighbor_index].node_state_indexed_view.get_index().unwrap();

        changed.push(CollapsedNodeState {
            node_id: String::from(self.collapsable_nodes[neighbor_index].id),
            node_state_id: Some((*self.collapsable_nodes[neighbor_index].node_state_indexed_view.get().unwrap()).clone()),
        });

        if is_successful {
            let great_indices = self.great_neighbor_node_indices_per_neighbor_node_index.get(&neighbor_index).cloned().unwrap_or_default();
            let candidate_masks = collect_masks_for_state_index_for_targets(&self.collapsable_nodes, neighbor_index, new_state_index, &great_indices);

            let mut masked: Vec<usize> = Vec::new();
            let mut rollback = false;
            for (gidx, mask) in &candidate_masks {
                let restrictive = self.collapsable_nodes[*gidx].node_state_indexed_view.is_mask_restrictive_to_current_state(mask);
                if !restrictive {
                    self.collapsable_nodes[*gidx].add_mask(mask);
                    masked.push(*gidx);
                } else {
                    rollback = true;
                    break;
                }
            }
            if rollback {
                for &gidx in &masked {
                    let mask = candidate_masks.iter().find(|(i, _)| *i == gidx).unwrap().1.clone();
                    self.collapsable_nodes[gidx].subtract_mask(&mask);
                }
                self.is_current_neighbor_node_cycle_required = true;
            } else {
                self.current_neighbor_node_indices_index += 1;
            }
        } else {
            if self.current_neighbor_node_indices_index == 0 {
                self.is_current_node_neighbors_collapse_possible = false;
            } else {
                self.current_neighbor_node_indices_index -= 1;
                self.is_current_neighbor_node_cycle_required = true;

                let prev_index = self.current_neighbor_node_indices[self.current_neighbor_node_indices_index];
                let prev_state_index = self.collapsable_nodes[prev_index].node_state_indexed_view.get_index().unwrap();
                let great_indices = self.great_neighbor_node_indices_per_neighbor_node_index.get(&prev_index).cloned().unwrap_or_default();
                let masks = collect_masks_for_state_index_for_targets(&self.collapsable_nodes, prev_index, prev_state_index, &great_indices);
                for (gidx, mask) in masks {
                    self.collapsable_nodes[gidx].subtract_mask(&mask);
                }
            }
        }
        changed
    }

    fn allow_current_node_neighbor_to_maintain_state(&mut self) {
        let neighbor_index = self.current_neighbor_node_indices[self.current_neighbor_node_indices_index];
        let great_indices = self.great_neighbor_node_indices_per_neighbor_node_index.get(&neighbor_index).cloned().unwrap_or_default();
        let node_state_index = self.collapsable_nodes[neighbor_index].node_state_indexed_view.get_index().unwrap();

        let candidate_masks = collect_masks_for_state_index_for_targets(&self.collapsable_nodes, neighbor_index, node_state_index, &great_indices);

        let mut masked: Vec<usize> = Vec::new();
        let mut rollback = false;
        for (gidx, mask) in &candidate_masks {
            let restrictive = self.collapsable_nodes[*gidx].node_state_indexed_view.is_mask_restrictive_to_current_state(mask);
            if !restrictive {
                self.collapsable_nodes[*gidx].add_mask(mask);
                masked.push(*gidx);
            } else {
                rollback = true;
                break;
            }
        }
        if rollback {
            for &gidx in &masked {
                let mask = candidate_masks.iter().find(|(i, _)| *i == gidx).unwrap().1.clone();
                self.collapsable_nodes[gidx].subtract_mask(&mask);
            }
            self.is_current_neighbor_node_cycle_required = true;
        } else {
            self.current_neighbor_node_indices_index += 1;
        }
    }

    fn cleanup_current_node_neighbors(&mut self) {
        if self.current_neighbor_node_indices_index == self.current_neighbor_node_indices_length {
            let ci = self.spread_node_indices[self.spread_node_indices_index];
            self.impacted_node_indices.set(ci, true);
            for &ni in &self.current_neighbor_node_indices {
                self.impacted_node_indices.set(ni, true);
            }

            let neighbor_indices: Vec<usize> = self.current_neighbor_node_indices.clone();
            for &ni in &neighbor_indices {
                let ni_state_index = self.collapsable_nodes[ni].node_state_indexed_view.get_index().unwrap();
                let nongreat = self.nongreat_neighbor_node_indices_per_neighbor_node_index.get(&ni).cloned().unwrap_or_default();
                let masks = collect_masks_for_state_index_for_targets(&self.collapsable_nodes, ni, ni_state_index, &nongreat);
                for (target, mask) in masks {
                    self.collapsable_nodes[target].add_mask(&mask);
                }
            }
        } else {
            let neighbor_indices: Vec<usize> = self.current_neighbor_node_indices.clone();
            for &ni in &neighbor_indices {
                let masks = collect_masks_from_node(&self.collapsable_nodes, ni);
                for (target, mask) in masks {
                    self.collapsable_nodes[target].add_mask(&mask);
                }
            }
        }

        for (&ni, mask_state) in self.stash_per_neighbor_node_index.iter_mut() {
            self.collapsable_nodes[ni].node_state_indexed_view.unstash_mask_state(mask_state);
        }
        self.original_node_state_per_node_index.clear();
        self.great_neighbor_node_indices_per_neighbor_node_index.clear();
        self.nongreat_neighbor_node_indices_per_neighbor_node_index.clear();
        self.current_neighbor_node_indices.clear();
    }

    fn move_to_next_node(&mut self) {
        self.spread_node_indices_index += 1;
    }

    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction<TNodeState> {
        let mut node_state_per_node_id = HashMap::new();
        for node in &self.collapsable_nodes {
            let state: TNodeState = (*node.node_state_indexed_view.get().unwrap()).clone();
            node_state_per_node_id.insert(String::from(node.id), state);
        }
        CollapsedWaveFunction { node_state_per_node_id }
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for AccommodatingSequentialCollapsableWaveFunction<'a, TNodeState> {
    fn new(
        collapsable_nodes: Vec<CollapsableNode<'a, TNodeState>>,
        random_instance: fastrand::Rng,
    ) -> Self {
        let node_count = collapsable_nodes.len();
        AccommodatingSequentialCollapsableWaveFunction {
            collapsable_nodes,
            spread_node_indices: Vec::new(),
            spread_node_indices_length: 0,
            spread_node_indices_index: 0,
            impacted_node_indices: bitvec::bitvec![0; node_count],
            impacted_node_count: node_count,
            stash_per_neighbor_node_index: HashMap::new(),
            original_node_state_per_node_index: HashMap::new(),
            current_neighbor_node_indices: Vec::new(),
            great_neighbor_node_indices_per_neighbor_node_index: HashMap::new(),
            nongreat_neighbor_node_indices_per_neighbor_node_index: HashMap::new(),
            current_neighbor_node_indices_index: 0,
            current_neighbor_node_indices_length: 0,
            is_current_neighbor_node_cycle_required: false,
            is_current_node_neighbors_collapse_possible: true,
            random_instance,
        }
    }

    fn collapse(&mut self) -> Result<CollapsedWaveFunction<TNodeState>, String> {
        let mut iterations_total: u32 = 0;
        let init = self.initialize_nodes();
        if init.is_err() {
            return Err(init.err().unwrap());
        }
        while !self.is_fully_collapsed() {
            self.prepare_nodes_for_iteration();
            while !self.is_done_spreading_nodes() {
                if self.is_current_node_in_conflict() {
                    self.prepare_current_node_neighbors();
                    while !self.is_current_node_neighbors_collapsed() {
                        if self.is_current_node_neighbor_state_change_required() {
                            self.change_state_of_current_node_neighbor();
                        } else {
                            self.allow_current_node_neighbor_to_maintain_state();
                        }
                    }
                    self.cleanup_current_node_neighbors();
                }
                self.move_to_next_node();
            }
            iterations_total += 1;
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
            while !self.is_done_spreading_nodes() {
                if self.is_current_node_in_conflict() {
                    self.prepare_current_node_neighbors();
                    while !self.is_current_node_neighbors_collapsed() {
                        if self.is_current_node_neighbor_state_change_required() {
                            collapsed_node_states.extend(self.change_state_of_current_node_neighbor());
                        } else {
                            self.allow_current_node_neighbor_to_maintain_state();
                        }
                    }
                    self.cleanup_current_node_neighbors();
                }
                self.move_to_next_node();
            }
        }
        Ok(collapsed_node_states)
    }
}
