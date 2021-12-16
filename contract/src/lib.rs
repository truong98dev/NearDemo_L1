use std::convert::TryFrom;
use std::thread::AccessError;

use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::json_types::{WrappedBalance, WrappedDuration, ValidAccountId, U128};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap, Vector};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PromiseOrValue, Timestamp, Duration, Gas, Promise, PromiseResult, PanicOnDefault};

near_sdk::setup_alloc!();

use crate::internal::*;

mod internal;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate="near_sdk::serde")]
#[serde(tag="type")]
pub enum TokenPriceType {
    FixedPrice { near: Balance },
    DynamicPrice { ratio: f64 }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate="near_sdk::serde")]
pub struct ShareHolder {
    account_id: ValidAccountId,
    percent_of_token: f64
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Contract {
    owner_id: ValidAccountId,
    price_type: TokenPriceType,
    tokennomic: Vector<ShareHolder>,
    sale_duration: WrappedDuration,

    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    whitelist_map: UnorderedMap<AccountId, Balance>
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        token_name: String,
        token_symbol: String,
        svg_icon: String,
        token_decimals: u8,
        total_supply: Balance,
        sale_duration: WrappedDuration,
        tokennomic: Vec<ShareHolder>,
        price_type: TokenPriceType 
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        
        let owner_id = ValidAccountId::try_from(env::predecessor_account_id().clone()).unwrap();

        let metadata = FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: token_name.to_string(),
            symbol: token_symbol.to_string(),
            icon: Some(svg_icon.to_string()),
            reference: None,
            reference_hash: None,
            decimals: token_decimals
        };

        let mut this = Self {
            owner_id,
            price_type,
            tokennomic: Vector::new(b"t".to_vec()),
            sale_duration,
            token: FungibleToken::new(b"f".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            whitelist_map: UnorderedMap::new(b"w".to_vec())
        };

        for item in &tokennomic {
            this.tokennomic.push(item);
        }

        this.token.internal_register_account(owner_id.as_ref());
        this.token.internal_deposit(owner_id.as_ref(), total_supply.into());
        this
    }

    pub fn remaining_tokens(&self) -> Balance {
        return 0
    }

    pub fn tokennomic(&self) -> Vec<ShareHolder> {
        self.tokennomic.to_vec()
    }

    pub fn get_tokens_for_sale_percent(&self) -> u64 {
        let share_holder_total_percent: f64 = self.tokennomic
            .map(item.percent)
            .sum();
        (100 - share_holder_total_percent) / 100 * self.token.total_supply
    }

    pub fn get_token_price(&self) -> Balance {
        return 0
    }

    pub fn get_remaining_tokens(&self) -> Balance {
        return 0
    }

    pub fn whitelist(&self) -> Vec<(AccountId, Balance)> {
        self.whitelist_map
            .collect()
    }

  fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
