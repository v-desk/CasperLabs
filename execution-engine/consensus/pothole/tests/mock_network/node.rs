use std::collections::BTreeSet;
use std::mem;

use pothole::Block as BlockTrait;
use pothole::{BlockIndex, Effect, Pothole};

use super::{NetworkMessage, WorldHandle};

pub type Transaction = String;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    transactions: Vec<Transaction>,
}

impl BlockTrait for Block {}

pub type NodeId = &'static str;

pub struct Node {
    #[allow(unused)]
    our_id: NodeId,
    other_nodes: BTreeSet<NodeId>,
    pothole: Pothole<Block>,
    world: WorldHandle,
    transaction_buffer: BTreeSet<Transaction>,
}

impl Node {
    pub fn new(our_id: NodeId, mut all_ids: BTreeSet<NodeId>, world: WorldHandle) -> Self {
        let dictator = Some(&our_id) == all_ids.iter().next();
        let _ = all_ids.remove(&our_id);
        let (pothole, effects) = Pothole::new(dictator);
        let mut node = Self {
            our_id,
            other_nodes: all_ids,
            pothole: pothole,
            world,
            transaction_buffer: Default::default(),
        };
        node.handle_effects(effects);
        node
    }

    fn handle_pothole_effect(&mut self, effect: Effect<Block>) -> Vec<Effect<Block>> {
        match effect {
            Effect::ScheduleTimer(timer_id, instant) => {
                self.world.schedule_timer(timer_id, instant);
                vec![]
            }
            Effect::RequestBlock => {
                let transactions = mem::replace(&mut self.transaction_buffer, Default::default());
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

    pub fn propose_transaction(&mut self, transaction: Transaction) {
        self.transaction_buffer.insert(transaction.clone());
        for node in &self.other_nodes {
            self.world
                .send_message(*node, NetworkMessage::NewTransaction(transaction.clone()));
        }
    }

    /// Returns whether any actions were taken this step
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

    pub fn consensused_blocks(&self) -> impl Iterator<Item = (&BlockIndex, &Block)> {
        self.pothole.blocks_iterator()
    }

    pub fn has_pending_transactions(&self) -> bool {
        !self.transaction_buffer.is_empty()
    }
}
