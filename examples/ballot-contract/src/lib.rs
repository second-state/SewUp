//! This is an example sewup contract for a simple voting scenario
//!
//! Where in,
//! the proposals and the chairman are set in constructor (setup once when the contract on chain)
//! only chairman can give the ballots to voters
//! the voter can vote the proposal once
//! everyone can check out the voting result after everyone voted
//!
use std::convert::TryInto;

use serde_derive::{Deserialize, Serialize};
use sewup::types::{Address, Raw};
use sewup_derive::{
    ewasm_call_only_by, ewasm_constructor, ewasm_fn, ewasm_fn_sig, ewasm_main, ewasm_test,
    SizedString, Value,
};

mod errors;

static CHAIRMAN: &str = "8663DBF0cC68AaF37fC8BA262F2df4c666a41993";

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Value)]
struct Voter {
    voted: bool,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Value)]
struct Proposal {
    name: SizedString!(50),
    vote_count: usize,
}

#[derive(Serialize, Deserialize)]
struct Input {
    proposal_id: usize,
}

#[ewasm_constructor]
fn constructor() {
    let mut storage =
        sewup::kv::Store::new().expect("there is no return for constructor currently");

    let voters_bucket = storage
        .bucket::<Address, Voter>("voters")
        .expect("there is no return for constructor currently");

    // TODO: make usize be compatiable with Key trait of KV
    /// use KV to storage array like data structure
    let mut proposals_bucket = storage
        .bucket::<Raw, Proposal>("proposals")
        .expect("there is no return for constructor currently");

    let proposals = ["carbon neutral in 2021", "safety with Rust in 2022"];

    for (idx, name) in proposals.iter().enumerate() {
        let name = sewup::types::SizedString::new(50).from_str(name).unwrap();
        proposals_bucket.set(
            Raw::from(idx),
            Proposal {
                name: name.into(),
                vote_count: 0,
            },
        );
    }

    storage.save(voters_bucket);
    storage.save(proposals_bucket);
    storage
        .commit()
        .expect("there is no return for constructor currently");
}

#[ewasm_fn]
fn give_right_to_vote(voter: String) -> anyhow::Result<sewup::primitives::EwasmAny> {
    ewasm_call_only_by!(CHAIRMAN);
    // or
    // ewasm_call_only_by!("8663DBF0cC68AaF37fC8BA262F2df4c666a41993");

    let mut storage = sewup::kv::Store::load(None)?;
    let mut voters_bucket = storage.bucket::<Address, Voter>("voters")?;
    let voter_address = Address::from_str(&voter)?;

    return if voters_bucket.get(voter_address.clone())?.is_some() {
        Err(errors::Error::VoterExist(voter).into())
    } else {
        voters_bucket.set(voter_address, Voter { voted: false });
        storage.save(voters_bucket);
        storage.commit()?;
        Ok(().into())
    };
}

#[ewasm_fn]
fn vote(input: Input) -> anyhow::Result<sewup::primitives::EwasmAny> {
    let caller_address = sewup::utils::caller();

    let mut storage = sewup::kv::Store::load(None)?;
    let mut voters_bucket = storage.bucket::<Address, Voter>("voters")?;
    let mut proposals_bucket = storage.bucket::<Raw, Proposal>("proposals")?;

    if let Some(mut voter) = voters_bucket.get(caller_address.clone())? {
        if voter.voted {
            return Err(errors::Error::AlreadyVote.into());
        } else {
            if let Some(mut proposal) = proposals_bucket.get(Raw::from(input.proposal_id))? {
                voter.voted = true;
                voters_bucket.set(caller_address, voter);

                proposal.vote_count += 1;
                proposals_bucket.set(Raw::from(input.proposal_id), proposal);

                storage.save(voters_bucket);
                storage.save(proposals_bucket);
                storage.commit()?;

                return Ok(().into());
            } else {
                return Err(errors::Error::ProposalNonExist(input.proposal_id).into());
            }
        }
    } else {
        return Err(errors::Error::LackRightToVote.into());
    }
}

#[ewasm_fn]
fn winning_proposals() -> anyhow::Result<sewup::primitives::EwasmAny> {
    let mut storage = sewup::kv::Store::load(None)?;
    let voters_bucket = storage.bucket::<Address, Voter>("voters")?;
    let proposals_bucket = storage.bucket::<Raw, Proposal>("proposals")?;
    for (_, voter) in voters_bucket.iter() {
        if !voter.voted {
            return Err(errors::Error::StillVoting.into());
        }
    }

    let mut highest_vote = 0;
    let mut highest_proposals: Vec<Proposal> = vec![];
    for (_, proposal) in proposals_bucket.iter() {
        if proposal.vote_count > highest_vote {
            highest_vote = proposal.vote_count;
            highest_proposals = vec![proposal];
        } else if proposal.vote_count == highest_vote {
            highest_proposals.push(proposal);
        }
    }
    return Ok(highest_proposals.into());
}

#[ewasm_main(auto)]
fn main() -> anyhow::Result<sewup::primitives::EwasmAny> {
    use sewup_derive::ewasm_input_from;

    let contract = sewup::primitives::Contract::new()?;
    return match contract.get_function_selector()? {
        ewasm_fn_sig!(give_right_to_vote) => ewasm_input_from!(contract move give_right_to_vote),
        ewasm_fn_sig!(vote) => ewasm_input_from!(contract move vote),
        ewasm_fn_sig!(winning_proposals) => winning_proposals(),
        _ => panic!("unknown handle"),
    };
}

#[ewasm_test]
mod tests {
    use super::*;
    use sewup_derive::{
        ewasm_assert_eq, ewasm_auto_assert_eq, ewasm_err_output, ewasm_output_from,
    };

    #[ewasm_test]
    fn test_give_right_to_vote() {
        ewasm_assert_eq!(
            give_right_to_vote("1cCA28600d7491365520B31b466f88647B9839eC"),
            ewasm_err_output!(sewup::errors::HandlerError::Unauthorized)
        );

        // TODO: handle input with primitive types, ex: usize
        let input = Input { proposal_id: 1 };
        ewasm_assert_eq!(
            vote(input) by "1cCA28600d7491365520B31b466f88647B9839eC",
            ewasm_err_output!(errors::Error::LackRightToVote)
        );

        ewasm_auto_assert_eq!(
            give_right_to_vote("1cCA28600d7491365520B31b466f88647B9839eC") by "8663DBF0cC68AaF37fC8BA262F2df4c666a41993",
            ()
        );

        ewasm_auto_assert_eq!(
            vote(input) by "1cCA28600d7491365520B31b466f88647B9839eC",
            ()
        );

        let name = sewup::types::SizedString::new(50)
            .from_str("safety with Rust in 2022")
            .unwrap();
        let proposal = Proposal {
            name: name.into(),
            vote_count: 1,
        };
        ewasm_auto_assert_eq!(winning_proposals(), vec![proposal]);
    }
}
