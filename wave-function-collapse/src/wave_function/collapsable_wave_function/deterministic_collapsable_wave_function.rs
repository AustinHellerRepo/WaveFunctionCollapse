use std::marker::PhantomData;
use std::{cell::RefCell, rc::Rc, collections::HashMap};
use std::hash::Hash;
use bitvec::vec::BitVec;
use super::collapsable_wave_function::{CollapsableWaveFunction, CollapsableNode, CollapsedNodeState, UncollapsedWaveFunction, CollapsedWaveFunction};

pub struct DeterministicCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    // represents a wave function with all of the necessary steps to collapse
    collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>,
    collapsable_nodes_length: usize,
    current_collapsable_node_index: usize,
    node_state_type: PhantomData<TNodeState>
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> DeterministicCollapsableWaveFunction<'a, TNodeState> {
    #[time_graph::instrument]
    fn try_increment_current_collapsable_node_state(&mut self) -> CollapsedNodeState<TNodeState> {
        let wrapped_current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
        let mut current_collapsable_node = wrapped_current_collapsable_node.borrow_mut();

        let is_successful = current_collapsable_node.node_state_indexed_view.try_move_next();
        let collapsed_node_state: CollapsedNodeState<TNodeState>;
        if is_successful {
            current_collapsable_node.current_chosen_from_sort_index = Some(self.current_collapsable_node_index);
            collapsed_node_state = CollapsedNodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: Some((*current_collapsable_node.node_state_indexed_view.get().unwrap()).clone())
            };
        }
        else {
            current_collapsable_node.current_chosen_from_sort_index = None;
            collapsed_node_state = CollapsedNodeState {
                node_id: String::from(current_collapsable_node.id),
                node_state_id: None
            };
        }

        collapsed_node_state
    }
    #[time_graph::instrument]
    fn try_alter_reference_to_current_collapsable_node_mask(&mut self) -> bool {
        let mut is_successful: bool = true;
        let wrapped_current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.");
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
    #[time_graph::instrument]
    fn move_to_next_collapsable_node(&mut self) {
        let wrapped_current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
        let current_node_id: &str = wrapped_current_collapsable_node.borrow().id;
        let current_collapsable_node_index: &usize = &self.current_collapsable_node_index;
        debug!("moving from {current_node_id} at index {current_collapsable_node_index}");

        self.current_collapsable_node_index += 1;

        let next_collapsable_node_index: &usize = &self.current_collapsable_node_index;
        if self.current_collapsable_node_index == self.collapsable_nodes_length {
            debug!("moved outside of bounds at index {next_collapsable_node_index}");
        }
        else {
            let wrapped_current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
            let next_node_id: &str = wrapped_current_collapsable_node.borrow().id;
            debug!("moved to {next_node_id} at index {next_collapsable_node_index}");
        }
    }
    #[time_graph::instrument]
    fn is_fully_collapsed(&self) -> bool {
        self.current_collapsable_node_index == self.collapsable_nodes_length
    }
    #[time_graph::instrument]
    fn try_move_to_previous_collapsable_node_neighbor(&mut self) {

        {
            let wrapped_current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.");
            let mut current_collapsable_node = wrapped_current_collapsable_node.borrow_mut();

            // reset the node state index for the current node
            current_collapsable_node.node_state_indexed_view.reset();
            // reset chosen index within collapsable node
            current_collapsable_node.current_chosen_from_sort_index = None;
        }
        
        // move to the previously chosen node
        if self.current_collapsable_node_index != 0 {
            self.current_collapsable_node_index -= 1;

            // revert the masks of the new current collapsable node prior to the next state change/increment
            {
                let wrapped_current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.");
                let mut current_collapsable_node = wrapped_current_collapsable_node.borrow_mut();

                let neighbor_node_ids: &Vec<&str>;
                if let Some(current_possible_state) = current_collapsable_node.node_state_indexed_view.get() {
                    neighbor_node_ids = &current_collapsable_node.neighbor_node_ids;
                    let current_collapsable_node_state = current_collapsable_node.node_state_indexed_view.get().unwrap();
                    if current_collapsable_node.mask_per_neighbor_per_state.contains_key(current_collapsable_node_state) {
                        let mask_per_neighbor = current_collapsable_node.mask_per_neighbor_per_state.get(current_collapsable_node_state).unwrap();
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
    #[time_graph::instrument]
    fn is_fully_reset(&self) -> bool {
        let wrapped_current_collapsable_node = self.collapsable_nodes.get(self.current_collapsable_node_index).unwrap();
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        self.current_collapsable_node_index == 0 && current_collapsable_node.current_chosen_from_sort_index.is_none()
    }
    fn get_uncollapsed_wave_function(&self) -> UncollapsedWaveFunction<TNodeState> {
        let mut node_state_per_node: HashMap<String, Option<TNodeState>> = HashMap::new();
        for wrapped_collapsable_node in self.collapsable_nodes.iter() {
            let collapsable_node = wrapped_collapsable_node.borrow();
            let node_state_id_option: Option<TNodeState>;
            if let Some(node_state_id) = collapsable_node.node_state_indexed_view.get() {
                node_state_id_option = Some((*node_state_id).clone());
            }
            else {
                node_state_id_option = None;
            }
            let node: String = String::from(collapsable_node.id);
            debug!("established node {node} in state {:?}.", node_state_id_option);
            node_state_per_node.insert(node, node_state_id_option);
        }
        UncollapsedWaveFunction {
            node_state_per_node: node_state_per_node
        }
    }
    #[time_graph::instrument]
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
            node_state_per_node: node_state_per_node
        }
    }
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for DeterministicCollapsableWaveFunction<'a, TNodeState> {
    #[time_graph::instrument]
    fn new(collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>, collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>) -> Self {
        let collapsable_nodes_length: usize = collapsable_nodes.len();

        let mut collapsable_wave_function = DeterministicCollapsableWaveFunction {
            collapsable_nodes: collapsable_nodes,
            collapsable_node_per_id: collapsable_node_per_id,
            collapsable_nodes_length: collapsable_nodes_length,
            current_collapsable_node_index: 0,
            node_state_type: PhantomData
        };

        collapsable_wave_function
    }
    #[time_graph::instrument]
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

    #[time_graph::instrument]
    fn collapse(&'a mut self) -> Result<CollapsedWaveFunction<TNodeState>, String> {

        // TODO use the provided cells in cell_per_neighbor_node_id_per_node_id during construction of CollapsableNode

        // set sort necessary
        // set error message as None
        // while
        //          no error message
        //          and
        //          the collapsable node index is less than the total number of collapsable nodes
        //
        // REMOVE      if current collapsable node has Some state id index
        // REMOVE          inform neighbors that this state id is now available again (if applicable)
        //
        //      try to increment the current collapsable node state id index (maybe just going from None to Some(0))
        //
        //      if succeeded to increment
        //          alter reference to mask (via orient function)
        //          if not at least one neighbor no longer having any valid states (or better said if all neighbors have at least one valid state)
        //              increment current collapsable node index
        //              if node index is not outside of the bounds
        //                  sort by (1) chosen from sorted collapsable nodes vector index (in order to maintain the chosen order) and then (2) least possible states being first (in order to adjust the next possible nodes to pick the most restricted nodes first)
        //      else (then we need to try a different state for the most recent parent that has the current node as a neighbor)
        //          set is neighbor found to false
        //          cache the current collapsable node id
        //          while
        //                  not yet errored
        //                  and
        //                  not found neighbor
        //
        //              if current collapsable node index is the first node (then the nodes have been exhausted)
        //                  set error message
        //              else
        //                  set current collapsable node's state id index to None (via reset function)
        //                  decrement current collapsale node index
        //                  if one of the newly current collapsable node's neighbors is the original collapsable node
        //                      set found neighbor to true

        let mut is_unable_to_collapse = false;
        debug!("starting while loop");
        while !is_unable_to_collapse && !self.is_fully_collapsed() {
            debug!("incrementing node state");
            let is_increment_successful: bool;
            is_increment_successful = self.try_increment_current_collapsable_node_state().node_state_id.is_some();
            if is_increment_successful {
                debug!("incremented node state");
                if self.try_alter_reference_to_current_collapsable_node_mask() {
                    debug!("altered reference and all neighbors have at least one valid state");
                    self.move_to_next_collapsable_node();
                    debug!("moved to next collapsable node");
                    if !self.is_fully_collapsed() {
                        debug!("not yet fully collapsed");
                        /*time_graph::spanned!("sort_collapsable_nodes (during)", {
                            collapsable_wave_function.sort_collapsable_nodes();
                        });
                        debug!("sorted nodes");*/
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