use pothole::BlockIndex;

use super::{Block, Transaction};

pub enum NetworkMessage {
    NewTransaction(Transaction),
    NewFinalizedBlock(BlockIndex, Block),
}
