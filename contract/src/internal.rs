use crate::*;

#[near_bindgen]
impl Contract {
    pub fn assert_owner(&self) {
        assert_eq!(
            &self.owner_id,
            &ValidAccountId::try_from(env::predecessor_account_id().clone()).unwrap(),
            "Only contract owner can call this function"
        );
    }
}
