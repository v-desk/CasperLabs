use std::collections::BTreeMap;

/// An identifier for the block position in the chain.
pub type BlockIndex = u64;

/// A simple structure to contain a linear progression of blocks (allowing for sparse population,
/// i.e. empty spaces left between blocks)
pub struct Chain<B> {
    blocks: BTreeMap<BlockIndex, B>,
    next_block: BlockIndex,
}

impl<B> Default for Chain<B> {
    fn default() -> Self {
        Self {
            blocks: Default::default(),
            next_block: 0,
        }
    }
}

impl<B> Chain<B> {
    /// Appends a new block right after the last one out of all currently held.
    pub fn append(&mut self, block: B) -> BlockIndex {
        self.blocks.insert(self.next_block, block);
        let result = self.next_block;
        self.next_block += 1;
        result
    }

    /// Inserts a new block at a given index. Returns the block that was already at this index, if
    /// any.
    pub fn insert(&mut self, index: BlockIndex, block: B) -> Option<B> {
        let result = self.blocks.insert(index, block);
        if index >= self.next_block {
            self.next_block = index + 1;
        }
        result
    }

    /// Returns the reference to the last known block.
    pub fn get_last_block(&self) -> Option<&B> {
        self.blocks.get(&(self.next_block - 1))
    }

    /// Gets the block at a given index.
    pub fn get_block(&self, index: BlockIndex) -> Option<&B> {
        self.blocks.get(&index)
    }

    /// Returns the current length of the chain (if there are empty spaces in the middle, they are
    /// treated as if there are actual blocks there, but we just don't know them yet)
    pub fn num_blocks(&self) -> usize {
        self.next_block as usize
    }

    /// Returns an iterator over all the blocks along with their indices
    pub fn blocks_iterator(&self) -> impl Iterator<Item = (&BlockIndex, &B)> {
        self.blocks.iter()
    }
}
