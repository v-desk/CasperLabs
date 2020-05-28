use std::mem;

// TODO: temporary type, probably will get replaced with something with more structure
#[derive(Debug, Clone, PartialEq)]
pub struct Deploy(Vec<u8>);

#[derive(Debug, Clone, Default)]
pub struct DeployBuffer {
    collected_deploys: Vec<Deploy>,
}

impl DeployBuffer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_deploy(&mut self, deploy: Deploy) {
        self.collected_deploys.push(deploy);
    }

    pub fn remaining_deploys(&mut self) -> Vec<Deploy> {
        mem::take(&mut self.collected_deploys)
    }
}

#[cfg(test)]
mod tests {
    use super::{Deploy, DeployBuffer};

    #[test]
    fn add_and_take_deploys() {
        let mut buffer = DeployBuffer::new();

        assert!(buffer.remaining_deploys().is_empty());

        buffer.add_deploy(Deploy(vec![1]));
        buffer.add_deploy(Deploy(vec![2]));

        assert_eq!(
            buffer.remaining_deploys(),
            vec![Deploy(vec![1]), Deploy(vec![2])]
        );

        assert!(buffer.remaining_deploys().is_empty());
    }
}
