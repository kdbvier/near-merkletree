use airdrop_merkle_tree_near_rs::hash::MerkleTreeHash;
use airdrop_merkle_tree_near_rs::proof::MerkleTreeProof;
use airdrop_merkle_tree_near_rs::root;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::env::predecessor_account_id;
use near_sdk::{env, near_bindgen, require, AccountId, CryptoHash, PanicOnDefault, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AirdropContract {
    pub owner: AccountId,
    pub claimed: LookupMap<AccountId, bool>,
    pub root_hash: MerkleTreeHash,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct AirdropInfo {
    address: AccountId,
    amount: String
}

#[near_bindgen]
impl AirdropContract {
    #[init]
    pub fn new(root_hash: CryptoHash) -> Self {
        require!(!env::state_exists(), "AirdropContract: already initialized");

        AirdropContract {
            owner: env::predecessor_account_id(),
            claimed: LookupMap::new(b"claimed".to_vec()),
            root_hash,
        }
    }

    #[private]
    fn verify(&self, account_id: AccountId, amount: String, proof: MerkleTreeProof) -> bool {
        let airdrop_info = AirdropInfo {
            amount,
            address: account_id
        };
        root::verify(
            &self.root_hash,
            &airdrop_info.try_to_vec().unwrap(),
            &proof,
            None,
        )
    }

    pub fn can_claim(&self, account_id: AccountId, amount: String, proof: MerkleTreeProof) -> bool {
        if self.claimed.contains_key(&account_id) && self.claimed.get(&account_id).unwrap() {
            false
        } else {
            self.verify(account_id, amount, proof)
        }
    }

    pub fn claim(&mut self, amount: String, proof: MerkleTreeProof) -> Promise {
        require!(
            self.can_claim(predecessor_account_id(), amount.clone(), proof),
            "AirdropContract: can't claim"
        );

        Promise::new(predecessor_account_id()).transfer(amount.parse::<u128>().unwrap())
    }
    pub fn change_root_hash(&mut self, root_hash: CryptoHash) {
        assert!(self.owner == env::predecessor_account_id(), "Only owner");
        self.root_hash = root_hash;
    }
    pub fn get_root_hash(&self) -> CryptoHash {
        self.root_hash
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    use airdrop_merkle_tree_near_rs::MerkleTree;
    use near_sdk::test_utils::accounts;
    use near_sdk::{AccountId, ONE_NEAR};
    use serde_json;
    
    #[test]
    pub fn can_claim_ok() {
        let file = fs::File::open("doc.json").expect("file");
        let json: serde_json::Value = serde_json::from_reader(file).expect("string json");
        let mut items:Vec<AirdropInfo> = Vec::new();

        for item in json.as_array().unwrap() {
                let object_item = item.as_object().unwrap();
                items.push(AirdropInfo {address: AccountId::try_from(object_item.get("address").unwrap().as_str().unwrap().to_string()).unwrap(), amount: object_item.get("amount").unwrap().as_str().unwrap().to_string()});
        }
        let mut items_as_vec: Vec<Vec<u8>> = Vec::new();

        for item in &items {
            items_as_vec.push(item.try_to_vec().unwrap());
        }
        let merkle_tree = MerkleTree::create(&items_as_vec, None);

        let airdrop_contract = AirdropContract::new(merkle_tree.root_hash);
        // println!("root: {:?}, proof: {:?}", merkle_tree.root_hash, merkle_tree.get_proof(1));
        println!("item_try_to_vec: {:?}", items_as_vec[1]);
        for i in 0..items.len() {
            assert!(airdrop_contract.can_claim(
                items[i].address.clone(),
                items[i].amount.clone(),
                merkle_tree.get_proof(i)
            ));

        }
    }

    // #[test]
    // pub fn can_claim_wrong_amount() {
    //     let mut items: Vec<(AccountId, u128)> = Vec::new();

    //     for i in 0..6 {
    //         items.push((accounts(i), ONE_NEAR));
    //     }

    //     let mut items_as_vec: Vec<Vec<u8>> = Vec::new();

    //     for item in &items {
    //         items_as_vec.push(item.try_to_vec().unwrap());
    //     }

    //     let merkle_tree = MerkleTree::create(&items_as_vec, None);

    //     let airdrop_contract = AirdropContract::new(merkle_tree.root_hash);

    //     for i in 0..items.len() {
    //         assert!(!airdrop_contract.can_claim(
    //             items[i].0.clone(),
    //             items[i].1 / 2,
    //             merkle_tree.get_proof(i)
    //         ));
    //     }
    // }

    // #[test]
    // pub fn can_claim_wrong_proof() {
    //     let mut items: Vec<(AccountId, u128)> = Vec::new();

    //     for i in 0..6 {
    //         items.push((accounts(i), ONE_NEAR));
    //     }

    //     let mut items_as_vec: Vec<Vec<u8>> = Vec::new();

    //     for item in &items {
    //         items_as_vec.push(item.try_to_vec().unwrap());
    //     }

    //     let merkle_tree = MerkleTree::create(&items_as_vec, None);

    //     let airdrop_contract = AirdropContract::new(merkle_tree.root_hash);

    //     assert!(!airdrop_contract.can_claim(
    //         items[0].0.clone(),
    //         items[0].1,
    //         merkle_tree.get_proof(1)
    //     ));
    // }
}
