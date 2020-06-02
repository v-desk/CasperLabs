use std::collections::{HashMap, HashSet};

// TODO: temporary type, probably will get replaced with something with more structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Deploy(Vec<u8>);

/// TODO: also temporary, will be defined somewhere else
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockHash([u8; 32]);

#[derive(Debug, Clone, Default)]
pub struct DeployBuffer {
    collected_deploys: HashSet<Deploy>,
    processed: HashMap<BlockHash, HashSet<Deploy>>,
    finalized: HashMap<BlockHash, HashSet<Deploy>>,
}

impl DeployBuffer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_deploy(&mut self, deploy: Deploy) {
        // TBD: do we add deploys that already are in `processed` or `finalized`?
        self.collected_deploys.insert(deploy);
    }

    pub fn remaining_deploys(&mut self, blocks: &HashSet<BlockHash>) -> HashSet<Deploy> {
        // deploys_to_return = all deploys in collected_deploys that aren't in processed or
        // finalized blocks from the set `blocks`
        let deploys_to_return =
            blocks
                .iter()
                .fold(self.collected_deploys.clone(), |mut set, block_hash| {
                    let empty = HashSet::new();
                    let included_deploys = self.processed.get(block_hash).unwrap_or(&empty)
                        | self.finalized.get(block_hash).unwrap_or(&empty);
                    set.retain(|deploy| !included_deploys.contains(deploy));
                    set
                });
        self.collected_deploys
            .retain(|deploy| !deploys_to_return.contains(deploy));
        deploys_to_return
    }

    pub fn added_block(&mut self, block: BlockHash, deploys: HashSet<Deploy>) {
        self.collected_deploys
            .retain(|deploy| !deploys.contains(deploy));
        self.processed.insert(block, deploys);
    }

    pub fn finalized_block(&mut self, block: BlockHash) {
        if let Some(deploys) = self.processed.remove(&block) {
            self.finalized.insert(block, deploys);
        } else {
            panic!("finalized block that hasn't been processed!");
        }
    }

    pub fn orphaned_block(&mut self, block: BlockHash) {
        if let Some(deploys) = self.processed.remove(&block) {
            self.collected_deploys.extend(deploys);
        } else {
            panic!("orphaned block that hasn't been processed!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockHash, Deploy, DeployBuffer};
    use std::collections::HashSet;

    #[test]
    fn add_and_take_deploys() {
        let no_blocks = HashSet::new();
        let mut buffer = DeployBuffer::new();
        let deploy1 = Deploy(vec![1]);
        let deploy2 = Deploy(vec![2]);
        let deploy3 = Deploy(vec![3]);
        let deploy4 = Deploy(vec![4]);

        assert!(buffer.remaining_deploys(&no_blocks).is_empty());

        // add two deploys
        buffer.add_deploy(deploy1.clone());
        buffer.add_deploy(deploy2.clone());

        // take the deploys out
        let deploys = buffer.remaining_deploys(&no_blocks);

        assert_eq!(deploys.len(), 2);
        assert!(deploys.contains(&deploy1));
        assert!(deploys.contains(&deploy2));

        assert!(buffer.remaining_deploys(&no_blocks).is_empty());

        // the two deploys will be included in block 1
        let block_hash1 = BlockHash([0; 32]);
        buffer.added_block(block_hash1, deploys);

        let mut blocks = HashSet::new();
        blocks.insert(block_hash1);

        assert!(buffer.remaining_deploys(&blocks).is_empty());

        // try adding the same deploy again
        buffer.add_deploy(deploy2);

        // it shouldn't be returned if we include block 1 in the past blocks
        assert!(buffer.remaining_deploys(&blocks).is_empty());

        // finalize the block
        buffer.finalized_block(block_hash1);

        // add more deploys
        buffer.add_deploy(deploy3.clone());
        buffer.add_deploy(deploy4.clone());

        let deploys = buffer.remaining_deploys(&blocks);

        assert_eq!(deploys.len(), 2);
        assert!(deploys.contains(&deploy3));
        assert!(deploys.contains(&deploy4));
    }
}
