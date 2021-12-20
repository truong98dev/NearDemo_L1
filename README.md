 Make own token

### Deploy Contract
```sh
cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./res/
near dev-deploy --wasmFile res/make_own_token.wasm 
export $ID
```

### Init contract (sale duration 10 minutes, price in near)
```sh
near call $ID new '{"token_name": "TEST",  "token_symbol": "TEST",  "svg_icon": "E",  "token_decimals":  24, "total_supply": 1000000000000, "sale_duration": 600000000000, "tokennomic": [{"account_id": "ceo_token.testnet", "percent_of_token": 30}, {"account_id": "cto_token.testnet", "percent_of_token": 30}], "price_type": {"type": "FixedPrice", "near": 1}}' --accountId $ID
```sh
### Distribute token to Founder
```sh
near call $ID distribute_tokens '{}' --accountId $ID --depositYocto 1
```
### Get distribute status
```sh
near call $ID distributed_status '{}' --accountId $ID
```

### Get remaining token
```sh
near call $ID remaining_tokens '{}' --accountId $ID
```

### Buy token (buy 2 times)
```sh
near call $ID deposit_for_sale '{}' --accountId buyer_token.testnet --deposit 10
near call $ID deposit_for_sale '{}' --accountId buyer_token.testnet --deposit 10
```

### Get my tokens
```sh
near call $ID my_tokens '{"account_id": "buyer_token.testnet"}' --accountId buyer_token.testnet
```

### Distribute tokens for buyers 
```sh
near call $ID distribute_tokens_to_buyers '{}' --accountId $ID --depositYocto 1
```

### Check tokens balance in buyers wallet
