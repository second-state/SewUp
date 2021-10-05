use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("This handler is for chairman only")]
    ChairmanOnly,
    #[error("Voter `{0}` already exist")]
    VoterExist(String),
    #[error("Voter `{0}` address is incorrect")]
    VoterAddressIncorrect(String),
    #[error("You are lack right to vote")]
    LackRightToVote,
    #[error("You already vote")]
    AlreadyVote,
    #[error("Proposal `{0}` is not exist")]
    ProposalNonExist(usize),
    #[error("Someone still not vote yet")]
    StillVoting,
}
