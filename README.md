# Make own token

### Deploy Contract
```sh
cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/*.wasm ./res/
near dev-deploy --wasmFile res/make_own_token.wasm 
export $ID
```

### Init contract (sale duration 10 minutes)
```sh
near call $ID new '{ \
    "token_name": "TEST", \
    "token_symbol": "TEST", \
    "svg_icon": "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E", \
    "token_decimals": 24, \
    "total_supply": 1000000000000000, \
    "sale_duration": 600000000000, \
    "tokennomic": ["{"account_id": "CEO_token.testnet", "percent_of_token": 30}", "{"account_id": "CTO_token.testnet", "percent_of_token": 30}"], \
    "price_type": "{"type": "FixedPrice", "near": "100000000000000000"}"}' --accountId $ID
```

### Distribute token to Founder
```sh
near call $ID distribute_tokens \
    '{}' \
    --accountId $ID
```
### Get distribute status
```sh
near call $ID distributed_status '{}' --accountId $ID
```

### Buy token 
```sh
near call $ID deposit_for_sale '{}' --accountId $ID --deposit 1
```

### Get my tokens
```sh
near call $ID my_tokens '{}' --accountId $ID
```

### Distribute tokens for buyers 
```sh
near call $ID distribute_tokens_to_buyers '{}' --accountId $ID
```

### Check tokens balance in buyers wallet 
