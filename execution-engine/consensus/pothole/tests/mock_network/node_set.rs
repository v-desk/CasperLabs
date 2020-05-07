use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::time::Duration;

use super::{Node, NodeId, Transaction, World, WorldHandle};

pub struct NodeSet {
    world: Rc<RefCell<World>>,
    nodes: BTreeMap<NodeId, Node>,
}

impl NodeSet {
    pub fn new(nodes: &[NodeId]) -> Self {
        let ids: BTreeSet<_> = nodes.into_iter().cloned().collect();
        let world = Rc::new(RefCell::new(World::new()));
        Self {
            nodes: nodes
                .into_iter()
                .map(|id| {
                    (
                        id.clone(),
                        Node::new(
                            id.clone(),
                            ids.clone(),
                            WorldHandle::new(world.clone(), id.clone()),
                        ),
                    )
                })
                .collect(),
            world,
        }
    }

    pub fn step(&mut self) {
        let world_ref = self.world.borrow();
        let queue_empty = world_ref.is_queue_empty();
        let dur_to_timer = world_ref.time_to_earliest_timer();
        drop(world_ref); // explicit drop to avoid issues with RefCell

        if queue_empty {
            // if there are no messages, advance time so that some timer fires and the nodes will do something
            if let Some(duration) = dur_to_timer {
                self.world.borrow_mut().advance_time(duration);
            }
        } else {
            self.world
                .borrow_mut()
                .advance_time(Duration::from_millis(250));
        }

        for (_, node) in &mut self.nodes {
            node.step();
        }
    }

    pub fn propose_transaction(&mut self, node: NodeId, transaction: Transaction) {
        if let Some(node) = self.nodes.get_mut(&node) {
            node.propose_transaction(transaction);
        }
    }

    pub fn nodes(&self) -> &BTreeMap<NodeId, Node> {
        &self.nodes
    }

    pub fn busy(&self) -> bool {
        !self.world.borrow().is_queue_empty()
            || self
                .nodes
                .iter()
                .any(|(_, node)| node.has_pending_transactions())
    }
}
