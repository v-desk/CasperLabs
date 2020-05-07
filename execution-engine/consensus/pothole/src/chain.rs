use std::collections::BTreeMap;

pub type BlockIndex = u64;

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
    pub fn append(&mut self, block: B) -> BlockIndex {
        self.blocks.insert(self.next_block, block);
        let result = self.next_block;
        self.next_block += 1;
        result
    }

    pub fn insert(&mut self, index: BlockIndex, block: B) -> Option<B> {
        let result = self.blocks.insert(index, block);
        if index >= self.next_block {
            self.next_block = index + 1;
        }
        result
    }

    pub fn get_last_block(&self) -> Option<&B> {
        self.blocks.get(&(self.next_block - 1))
    }

    pub fn get_block(&self, index: BlockIndex) -> Option<&B> {
        self.blocks.get(&index)
    }

    pub fn num_blocks(&self) -> usize {
        self.next_block as usize
    }

    pub fn blocks_iterator(&self) -> impl Iterator<Item = (&BlockIndex, &B)> {
        self.blocks.iter()
    }
}
