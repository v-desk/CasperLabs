use std::{fmt::Debug, hash::Hash};

/// A validator identifier.
pub trait ValidatorIdT: Eq + Ord + Clone + Debug + Hash {}
impl<VID> ValidatorIdT for VID where VID: Eq + Ord + Clone + Debug + Hash {}

/// The consensus value type, e.g. a list of transactions.
pub trait ConsensusValueT: Eq + Clone + Debug + Hash {}
impl<CV> ConsensusValueT for CV where CV: Eq + Clone + Debug + Hash {}

/// A hash, as an identifier for a block or vote.
pub trait HashT: Eq + Ord + Clone + Debug + Hash {}
impl<H> HashT for H where H: Eq + Ord + Clone + Debug + Hash {}

/// A validator's secret signing key.
pub trait ValidatorSecret: Debug {
    type Signature: Eq + Clone + Debug + Hash;

    fn sign(&self, data: &[u8]) -> Vec<u8>;
}

/// The collection of types the user can choose for cryptography, IDs, transactions, etc.
// TODO: The `Clone` trait bound makes `#[derive(Clone)]` work for `Block`...
pub trait Context: Clone + Debug {
    /// The consensus value type, e.g. a list of transactions.
    type ConsensusValue: ConsensusValueT;
    /// Unique identifiers for validators.
    type ValidatorId: ValidatorIdT;
    /// A validator's secret signing key.
    type ValidatorSecret: ValidatorSecret;
    /// Unique identifiers for votes.
    type VoteHash: HashT;
    /// The ID of a consensus protocol instance.
    type InstanceId: HashT;
}
