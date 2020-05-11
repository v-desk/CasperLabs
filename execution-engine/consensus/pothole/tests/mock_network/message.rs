use pothole::BlockIndex;

use super::{Block, Transaction};

/// Enum representing possible network messages
#[derive(Debug)]
pub enum NetworkMessage {
    NewTransaction(Transaction),
    NewFinalizedBlock(BlockIndex, Block),
}
