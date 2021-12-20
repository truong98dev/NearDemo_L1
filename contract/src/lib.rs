use std::convert::TryFrom;
use std::thread::AccessError;

use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::json_types::{WrappedBalance, WrappedDuration, ValidAccountId, U128};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap, UnorderedSet};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PromiseOrValue, Timestamp, Duration, Gas, Promise, PromiseResult, PanicOnDefault};

near_sdk::setup_alloc!();

use crate::internal::*;

mod internal;

const ONE_NEAR: Balance = 1_000_000_000_000_000_000_000_000;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate="near_sdk::serde")]
#[serde(tag="type")]
pub enum TokenPriceType {
    FixedPrice { near: u8 },
    DynamicPrice { ratio: f64 }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
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
    tokennomic: UnorderedSet<ShareHolder>,
    distributed_status: bool,
    start_time: Duration,
    sale_duration: Duration,

    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    whitelist_map: UnorderedMap<AccountId, Balance>
}

impl Default for Contract {
    fn default() -> Self {
     env::panic(b"Contract should be initialized before usage");}
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
        sale_duration: Duration,
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

        let _owner_id = owner_id.clone();
        let mut this = Self {
            owner_id: _owner_id,
            distributed_status: false,
            start_time: env::block_timestamp(), 
            price_type,
            tokennomic: UnorderedSet::new(b"t".to_vec()),
            sale_duration,
            token: FungibleToken::new(b"f".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            whitelist_map: UnorderedMap::new(b"w".to_vec())
        };

        for item in &tokennomic {
            this.tokennomic.insert(item);
        }

        this.token.internal_register_account(owner_id.as_ref());
        this.token.internal_deposit(owner_id.as_ref(), total_supply.into());
        this
    }

    #[payable]
    pub fn distribute_tokens(
        &mut self
    ) {
        self.assert_owner();

        for item in self.tokennomic.iter() {
            let _internal_item = item.clone();
// self.token.storage_deposit(Some(item.account_id), Some(true));
            self.token.internal_register_account(item.account_id.as_ref());
            
            let num_tokens = item.percent_of_token as u128 * self.token.total_supply / 100;
            self.token.ft_transfer(_internal_item.account_id, U128::try_from(num_tokens).unwrap(),  None);
        }

        self.distributed_status = true;
    }

    pub fn distributed_status(&self) -> bool {
        return self.distributed_status;
    }

    pub fn remaining_tokens(&self) -> Balance {
        self.token.total_supply * self.percent_for_sale() as u128 - self.sold_tokens()  
    }

    pub fn sold_tokens(&self) -> Balance {
       let total: Balance =  self.whitelist_map
                                .iter()
                                .map(|(_, balance)| balance)
                                .sum();
        total
    }

    pub fn tokennomic(&self) -> Vec<ShareHolder> {
        self.tokennomic.to_vec()
    }

    pub fn percent_for_sale(&self) -> f64 {
        let share_holder_total_percent: f64 = self.tokennomic
            .iter()
            .map(|item| item.percent_of_token)
            .sum();
        100 as f64 - share_holder_total_percent 
    }

    pub fn get_token_price(&self) -> Balance {
        return 0
    }

    pub fn whitelist(&self) -> Vec<(AccountId, Balance)> {
        self.whitelist_map
            .to_vec()
    }
    
    pub fn price(&self) -> Balance {
        match self.price_type {
            TokenPriceType::FixedPrice { near } => (near as u128 * ONE_NEAR),
            //Will add algorithm later
            TokenPriceType::DynamicPrice { ratio } => return ONE_NEAR
        }
    }

    pub fn my_tokens(
        &self,
        account_id: ValidAccountId
    ) -> Balance {
        self.whitelist_map
            .get(&account_id.to_string())
            .unwrap_or(0)
    }

    #[payable]
    pub fn deposit_for_sale(&mut self) -> bool {
        let curr_time_stamp = env::block_timestamp();
        assert!(
            curr_time_stamp >= self.start_time && curr_time_stamp <= self.start_time + self.sale_duration,
            "Not time for sale"
        );

        let amount = env::attached_deposit();

        let num_tokens = amount / self.price();
        let buyer = env::signer_account_id();

        let current_tokens = self.whitelist_map.get(&buyer).unwrap_or(0);
        self.whitelist_map.insert(&buyer, &(current_tokens + num_tokens));
        true        
    }
   
    #[payable] 
    pub fn distribute_tokens_to_buyers(
        &mut self
    ) -> bool {
        self.assert_owner();
        
        let curr_time_stamp = env::block_timestamp();
        assert!(
            curr_time_stamp > self.start_time + self.sale_duration,
            "Not time for distribute"
        );
        
        for (_account, _balance) in self.whitelist_map.iter() {
// self.token.storage_deposit(Some(ValidAccountId::try_from(_account.clone()).unwrap()), Some(false));

            self.token.internal_register_account(&_account);
            self.token.ft_transfer(ValidAccountId::try_from(_account).unwrap(), U128::try_from(_balance).unwrap(), None);
        }
        true
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
