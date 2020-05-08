use std::{collections::BTreeSet, mem};

use pothole::{BlockIndex, Effect, Pothole};

use super::{NetworkMessage, WorldHandle};

/// A dummy transaction type
pub type Transaction = String;

/// A dummy block type, containing dummy transactions
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    transactions: Vec<Transaction>,
}

/// A dummy NodeId - a static string
pub type NodeId = &'static str;

/// A mock Node type: representing a node in the network running a Pothole instance
pub struct Node {
    #[allow(unused)]
    our_id: NodeId,
    other_nodes: BTreeSet<NodeId>,
    pothole: Pothole<Block>,
    world: WorldHandle,
    transaction_buffer: BTreeSet<Transaction>,
}

impl Node {
    /// Creates a new Node with a given ID and set of peers.
    pub fn new(our_id: NodeId, mut all_ids: BTreeSet<NodeId>, world: WorldHandle) -> Self {
        let (pothole, effects) = Pothole::new(&our_id, &all_ids);
        let _ = all_ids.remove(&our_id);
        let mut node = Self {
            our_id,
            other_nodes: all_ids,
            pothole,
            world,
            transaction_buffer: Default::default(),
        };
        node.handle_effects(effects);
        node
    }

    /// Handles a single effect returned from the Pothole instance. Returns all the effects created
    /// as a result.
    fn handle_pothole_effect(&mut self, effect: Effect<Block>) -> Vec<Effect<Block>> {
        match effect {
            Effect::ScheduleTimer(timer_id, instant) => {
                self.world.schedule_timer(timer_id, instant);
                vec![]
            }
            Effect::RequestBlock => {
                let transactions = mem::take(&mut self.transaction_buffer);
                if !transactions.is_empty() {
                    self.pothole.propose_block(Block {
                        transactions: transactions.into_iter().collect::<Vec<_>>(),
                    })
                } else {
                    vec![]
                }
            }
            Effect::FinalizedBlock(index, block) => {
                // remove finalized transactions from buffer
                for transaction in &block.transactions {
                    self.transaction_buffer.remove(transaction);
                }
                for node_id in &self.other_nodes {
                    self.world.send_message(
                        *node_id,
                        NetworkMessage::NewFinalizedBlock(index, block.clone()),
                    );
                }
                vec![]
            }
        }
    }

    /// Handles a set of effects returned from the Pothole instance.
    fn handle_effects(&mut self, mut effects: Vec<Effect<Block>>) {
        loop {
            effects = effects
                .into_iter()
                .flat_map(|effect| self.handle_pothole_effect(effect))
                .collect::<Vec<_>>();
            if effects.is_empty() {
                break;
            }
        }
    }

    /// Handles an incoming network message.
    fn handle_message(&mut self, _sender: NodeId, message: NetworkMessage) -> Vec<Effect<Block>> {
        match message {
            NetworkMessage::NewTransaction(transaction) => {
                self.transaction_buffer.insert(transaction);
                vec![]
            }
            NetworkMessage::NewFinalizedBlock(index, block) => {
                self.pothole.handle_new_block(index, block)
            }
        }
    }

    /// Proposes a new transaction to be included in a future block.
    pub fn propose_transaction(&mut self, transaction: Transaction) {
        self.transaction_buffer.insert(transaction.clone());
        for node in &self.other_nodes {
            self.world
                .send_message(*node, NetworkMessage::NewTransaction(transaction.clone()));
        }
    }

    /// Takes a simulated step - processes all the events that happened since the last step (which
    /// can include timer events and incoming network messages).
    pub fn step(&mut self) {
        let timers = self.world.fire_timers();
        let mut effects: Vec<_> = timers
            .into_iter()
            .flat_map(|timer_id| self.pothole.handle_timer(timer_id))
            .collect();

        while let Some(msg) = self.world.recv_message() {
            effects.extend(self.handle_message(msg.sender, msg.message));
        }

        self.handle_effects(effects);
    }

    /// Returns an iterator over the blocks that reached consensus.
    pub fn consensused_blocks(&self) -> impl Iterator<Item = (&BlockIndex, &Block)> {
        self.pothole.blocks_iterator()
    }

    /// Returns whether this node still has some transactions that have been proposed, but not
    /// included in a finalized block.
    pub fn has_pending_transactions(&self) -> bool {
        !self.transaction_buffer.is_empty()
    }
}
