use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::{cell::RefCell, rc::Rc};
use std::hash::Hash;
use bitvec::vec::BitVec;

use super::collapsable_wave_function::{CollapsableNode, CollapsableWaveFunction, CollapsedNodeState, CollapsedWaveFunction};

// This keeps track of the tree traversal of the collapsable nodes
struct VisitedCollapsableNode<'a> {
    node_id: &'a str,
    // None: we have not traversed to a neighbor yet, the last valid visited node will never have a need for a non-None neighbor_index
    // Some(_): the next node (that we just popped back from) was at this index
    neighbor_index: Option<usize>,
}

pub struct SequentialAdjacentCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    collapsable_nodes_length: usize,
    node_state_type: PhantomData<TNodeState>,
    current_visited_collapsable_nodes: Vec<VisitedCollapsableNode<'a>>,
    current_visited_collapsable_node_ids: HashSet<&'a str>,
    is_able_to_move_to_next_collapsable_node: bool,
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> SequentialAdjacentCollapsableWaveFunction<'a, TNodeState> {
    fn try_increment_current_collapsable_node_state(&mut self) -> CollapsedNodeState<TNodeState> {
        let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(self.current_visited_collapsable_nodes.last().unwrap().node_id).unwrap();
        let mut current_collapsable_node = wrapped_current_collapsable_node.borrow_mut();

        let is_successful = current_collapsable_node.node_state_indexed_view.try_move_next();
        if is_successful {
            current_collapsable_node.current_chosen_from_sort_index = Some(self.current_visited_collapsable_nodes.len() - 1);
            CollapsedNodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: Some((*current_collapsable_node.node_state_indexed_view.get().unwrap()).clone())
            }
        }
        else {
            current_collapsable_node.current_chosen_from_sort_index = None;
            CollapsedNodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: None
            }
        }
    }
    fn try_alter_reference_to_current_collapsable_node_mask(&mut self) -> bool {
        let mut is_successful: bool = true;
        let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(self.current_visited_collapsable_nodes.last().unwrap().node_id).unwrap();
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        if let Some(current_possible_state) = current_collapsable_node.node_state_indexed_view.get() {
            let neighbor_node_ids: &Vec<&str> = &current_collapsable_node.neighbor_node_ids;
            let mask_per_neighbor_per_state: &HashMap<&TNodeState, HashMap<&str, BitVec>> = &current_collapsable_node.mask_per_neighbor_per_state;
            if let Some(mask_per_neighbor) = mask_per_neighbor_per_state.get(current_possible_state) {
                let mut traversed_neighbor_node_ids: Vec<&str> = Vec::new();
                for neighbor_node_id in neighbor_node_ids.iter() {
                    if mask_per_neighbor.contains_key(neighbor_node_id) {
                        let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                        let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                        //debug!("looking for mask from parent {:?} to child {:?}.", current_collapsable_node.id, neighbor_node_id);
                        //debug!("mask_per_neighbor: {:?}", mask_per_neighbor);
                        let mask = mask_per_neighbor.get(neighbor_node_id).unwrap();
                        neighbor_collapsable_node.forward_mask(mask);
                        debug!("adding mask to {:?} when in try_alter_reference_to_current_collapsable_node_mask", neighbor_node_id);
                        traversed_neighbor_node_ids.push(neighbor_node_id);
                        if neighbor_collapsable_node.is_fully_restricted() {
                            is_successful = false;
                            break;
                        }
                    }
                }
                if !is_successful {
                    // revert all of the traversed neighbors
                    for neighbor_node_id in traversed_neighbor_node_ids.iter() {
                        let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                        let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                        debug!("reversing mask for {:?} when in try_alter_reference_to_current_collapsable_node_mask", neighbor_node_id);
                        neighbor_collapsable_node.reverse_mask();
                    }
                }
            }
        }
        is_successful
    }
    // select a next neighbor for the current leaf node
    fn move_to_next_collapsable_node(&mut self) {

        if self.current_visited_collapsable_nodes.len() == self.collapsable_nodes_length {
            // if we've already reached the very end of the first solution, we're done
            self.is_able_to_move_to_next_collapsable_node = false;
        }
        else {
            'try_to_visit_next_node: {
                let mut visited_node_index = self.current_visited_collapsable_nodes.len() - 1;
                while visited_node_index > 0 {
                    // look for a next valid neighbor index
                    let visited_collapsable_node_id = self.current_visited_collapsable_nodes[visited_node_index].node_id;
                    let wrapped_collapsable_node = self.collapsable_node_per_id.get(visited_collapsable_node_id).unwrap();
                    let collapsable_node = wrapped_collapsable_node.borrow();
                    let next_neighbor_index = if let Some(current_neighbor_index) = self.current_visited_collapsable_nodes[visited_node_index].neighbor_index {
                        current_neighbor_index + 1
                    }
                    else {
                        0
                    };

                    while next_neighbor_index < collapsable_node.neighbor_node_ids.len() {
                        let neighbor_node_id = collapsable_node.neighbor_node_ids[next_neighbor_index];

                        // if this neighbor to the current leaf node has not been visited, we should visit it next
                        if self.current_visited_collapsable_node_ids.insert(neighbor_node_id) {
                            self.current_visited_collapsable_nodes.push(VisitedCollapsableNode {
                                node_id: &neighbor_node_id,
                                neighbor_index: None,
                            });
                            break 'try_to_visit_next_node;
                        }
                    }

                    // we failed to find the next neighbor of the leaf node, so backtrack and try again on the previously visited node
                    visited_node_index -= 1;
                }

                // we've visited all of the nodes and could not find a valid neighbor or next node to move to
                self.is_able_to_move_to_next_collapsable_node = false;
            }
        }
    }
    fn is_fully_collapsed(&self) -> bool {
        self.current_visited_collapsable_nodes.len() == self.collapsable_nodes_length && !self.is_able_to_move_to_next_collapsable_node
    }
    fn try_move_to_previous_collapsable_node_neighbor(&mut self) {

        {
            let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(self.current_visited_collapsable_nodes.last().unwrap().node_id).unwrap();
            let mut current_collapsable_node = wrapped_current_collapsable_node.borrow_mut();

            // reset the node state index for the current node
            current_collapsable_node.node_state_indexed_view.reset();
            // reset chosen index within collapsable node
            current_collapsable_node.current_chosen_from_sort_index = None;
        }
        
        // move to the previously chosen node
        if !self.current_visited_collapsable_nodes.is_empty() {
            let leaf_visited_collapsable_node = self.current_visited_collapsable_nodes.pop().unwrap();
            self.current_visited_collapsable_node_ids.remove(leaf_visited_collapsable_node.node_id);

            // revert the masks of the new current collapsable node prior to the next state change/increment
            {
                let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(leaf_visited_collapsable_node.node_id).unwrap();
                let current_collapsable_node = wrapped_current_collapsable_node.borrow_mut();

                let neighbor_node_ids: &Vec<&str>;
                if let Some(current_collapsable_node_state) = current_collapsable_node.node_state_indexed_view.get() {
                    neighbor_node_ids = &current_collapsable_node.neighbor_node_ids;
                    if let Some(mask_per_neighbor) = current_collapsable_node.mask_per_neighbor_per_state.get(current_collapsable_node_state) {
                        for neighbor_node_id in neighbor_node_ids.iter() {
                            if mask_per_neighbor.contains_key(neighbor_node_id) {
                                let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                                let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                                debug!("reversing mask for {:?} when in try_move_to_previous_collapsable_node_neighbor", neighbor_node_id);
                                neighbor_collapsable_node.reverse_mask();
                            }
                        }
                    }
                }
            }
        }
            
    }
    fn is_fully_reset(&self) -> bool {
        if !self.current_visited_collapsable_nodes.is_empty() {
            return false;
        }
        let wrapped_current_collapsable_node = self.collapsable_node_per_id.get(self.current_visited_collapsable_nodes.last().unwrap().node_id).unwrap();
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        return current_collapsable_node.current_chosen_from_sort_index.is_none();
    }
    fn get_collapsed_wave_function(&self) -> CollapsedWaveFunction<TNodeState> {
        let mut node_state_per_node: HashMap<String, TNodeState> = HashMap::new();
        for wrapped_collapsable_node in self.collapsable_nodes.iter() {
            let collapsable_node = wrapped_collapsable_node.borrow();
            let node_state: TNodeState = (*collapsable_node.node_state_indexed_view.get().unwrap()).clone();
            let node: String = String::from(collapsable_node.id);
            debug!("established node {node} in state {:?}.", node_state);
            node_state_per_node.insert(node, node_state);
        }
        CollapsedWaveFunction {
            node_state_per_node_id: node_state_per_node
        }
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for SequentialAdjacentCollapsableWaveFunction<'a, TNodeState> {
    fn new(collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>, collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>, _random_instance: Rc<RefCell<fastrand::Rng>>) -> Self {
        let collapsable_nodes_length: usize = collapsable_nodes.len();

        SequentialAdjacentCollapsableWaveFunction {
            collapsable_nodes,
            collapsable_node_per_id,
            collapsable_nodes_length,
            current_visited_collapsable_nodes: Vec::new(),
            current_visited_collapsable_node_ids: HashSet::new(),
            node_state_type: PhantomData,
            is_able_to_move_to_next_collapsable_node: true,
        }
    }
    fn collapse_into_steps(&'a mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {

        let mut collapsed_node_states: Vec<CollapsedNodeState<TNodeState>> = Vec::new();

        let mut is_unable_to_collapse = false;
        debug!("starting while loop");
        while !is_unable_to_collapse && !self.is_fully_collapsed() {
            debug!("incrementing node state");
            // the current collapsable node is either in a None state or is in a successful Some state but my neighbors are not aware
            let collapsed_node_state = self.try_increment_current_collapsable_node_state();
            // this will be None if the current collapsable node did not have another unmasked state that it could increment to
            let is_successful: bool = collapsed_node_state.node_state_id.is_some();
            collapsed_node_states.push(collapsed_node_state);

            debug!("stored node state");
            if is_successful {
                debug!("incremented node state: {:?}", collapsed_node_states.last());
                if self.try_alter_reference_to_current_collapsable_node_mask() {
                    debug!("altered reference and all neighbors have at least one valid state");
                    self.move_to_next_collapsable_node(); // this has the potential to move outside of the bounds and put the collapsable wave function in a state of being fully collapsed
                    debug!("moved to next collapsable node");
                    if !self.is_fully_collapsed() {
                        debug!("not yet fully collapsed");
                        //collapsable_wave_function.sort_collapsable_nodes();
                        //debug!("sorted nodes");
                    }
                }
                else {
                    debug!("at least one neighbor is fully restricted");
                }
            }
            else {
                debug!("failed to incremented node");
                self.try_move_to_previous_collapsable_node_neighbor();

                if self.is_fully_reset() {
                    debug!("moved back to first node and reset it");
                    is_unable_to_collapse = true;
                }
                else {
                    debug!("moved back to previous neighbor");
                    //collapsable_wave_function.alter_reference_to_current_collapsable_node_mask();
                    //debug!("stored uncollapsed_wave_function state");
                }
            }
        }
        debug!("finished while loop");

        Ok(collapsed_node_states)
    }

    fn collapse(&'a mut self) -> Result<CollapsedWaveFunction<TNodeState>, String> {

        // while not yet discovered that the wave function is uncollapsable and not yet fully collapsed
        //      try to increment the state of the current node forward
        //      if it was possible to increment the state because the neighbor nodes are not restricting that state
        //          try to inform the neighbor nodes of their new restrictions based on the current node's state
        //          if all neighbors have at least one valid state that they could be
        //              move the pointer to the next uncollapsed node
        //      else
        //          revert to the previous node so that it can try a different state since this is a dead end
        //          if we ended up back at the root node and it has also been fully reset
        //              this fully explored wave function is discovered to be uncollapsable

        let mut is_unable_to_collapse = false;
        debug!("starting while loop");
        while !is_unable_to_collapse && !self.is_fully_collapsed() {
            debug!("incrementing node state");
            let is_increment_successful = self.try_increment_current_collapsable_node_state().node_state_id.is_some();
            if is_increment_successful {
                debug!("incremented node state");
                if self.try_alter_reference_to_current_collapsable_node_mask() {
                    debug!("altered reference and all neighbors have at least one valid state");
                    self.move_to_next_collapsable_node();
                    debug!("moved to next collapsable node");
                }
                else {
                    debug!("at least one neighbor is fully restricted");
                }
            }
            else {
                debug!("failed to incremented node");
                self.try_move_to_previous_collapsable_node_neighbor();
                if self.is_fully_reset() {
                    debug!("moved back to first node");
                    is_unable_to_collapse = true;
                }
                else {
                    debug!("moved back to previous neighbor");
                }
            }
        }
        debug!("finished while loop");

        if is_unable_to_collapse {
            Err(String::from("Cannot collapse wave function."))
        }
        else {
            let collapsed_wave_function = self.get_collapsed_wave_function();
            Ok(collapsed_wave_function)
        }
    }
}