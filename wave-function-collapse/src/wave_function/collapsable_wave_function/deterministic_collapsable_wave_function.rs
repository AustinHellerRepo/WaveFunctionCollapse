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
    fn revert_existing_neighbor_masks(&mut self) {
        let wrapped_current_collapsable_node = self.collapsable_nodes.get_mut(self.current_collapsable_node_index).expect("The collapsable node should exist at this index.");
        let current_collapsable_node = wrapped_current_collapsable_node.borrow();
        if let Some(current_possible_state) = current_collapsable_node.node_state_indexed_view.get() {
            // if there is a mask_per_neighbor for this node's current state
            if let Some(mask_per_neighbor) = current_collapsable_node.mask_per_neighbor_per_state.get(current_possible_state) {
                for neighbor_node_id in current_collapsable_node.neighbor_node_ids.iter() {
                    if mask_per_neighbor.contains_key(neighbor_node_id) {
                        let wrapped_neighbor_collapsable_node = self.collapsable_node_per_id.get(neighbor_node_id).unwrap();
                        let mut neighbor_collapsable_node = wrapped_neighbor_collapsable_node.borrow_mut();
                        debug!("reversing mask for {:?} when in revert_existing_neighbor_masks", neighbor_node_id);
                        neighbor_collapsable_node.reverse_mask();  // each node contain a memo structure such that the masks are not needed to revert to the previous state since subtractions happen in reverse order anyway
                    }
                }
            }
        }
    }
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
                        neighbor_collapsable_node.add_mask(mask);
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