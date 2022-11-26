use std::collections::VecDeque;
use std::marker::PhantomData;
use std::{cell::RefCell, rc::Rc, collections::HashMap};
use std::hash::Hash;
use bitvec::vec::BitVec;
use super::collapsable_wave_function::{CollapsableWaveFunction, CollapsableNode, CollapsedNodeState, UncollapsedWaveFunction, CollapsedWaveFunction};

pub struct RandomCollapsableWaveFunction<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> {
    // represents a wave function with all of the necessary steps to collapse
    accommodate_node_ids: VecDeque<&'a str>,
    node_state_type: PhantomData<TNodeState>
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> RandomCollapsableWaveFunction<'a, TNodeState> {
    
}

impl<'a, TNodeState: Eq + Hash + Clone + std::fmt::Debug + Ord> CollapsableWaveFunction<'a, TNodeState> for RandomCollapsableWaveFunction<'a, TNodeState> {
    #[time_graph::instrument]
    fn new(collapsable_nodes: Vec<Rc<RefCell<CollapsableNode<'a, TNodeState>>>>, collapsable_node_per_id: HashMap<&'a str, Rc<RefCell<CollapsableNode<'a, TNodeState>>>>) -> Self {

        let mut accommodate_node_ids: VecDeque<&'a str> = VecDeque::new();
        for collapsable_node in collapsable_nodes.iter() {
            let collapsable_node_id: &str = collapsable_node.borrow().id;
            accommodate_node_ids.push_back(collapsable_node_id);
        }

        let mut collapsable_wave_function = RandomCollapsableWaveFunction {
            accommodate_node_ids: accommodate_node_ids,
            node_state_type: PhantomData
        };

        collapsable_wave_function
    }
    fn collapse(&'a mut self) -> Result<CollapsedWaveFunction<TNodeState>, String> {
        todo!()

        // initialize each collapsable node to its first (random) state
        // while at least one collapsable node is fully restricted
        //      while not every collapsable node has been considered for accommodation
        //          accommodate the next collapsable node that (1) has its current state restricted and (2) is not a neighbor of a previously accommodated node
        //      
        // NOTE: this could cause an infinite loop for the AB<-->CD unit test
    }
    fn collapse_into_steps(&'a mut self) -> Result<Vec<CollapsedNodeState<TNodeState>>, String> {
        todo!()
    }
}