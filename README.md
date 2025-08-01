# OfficeX Drive Canisters

Powers the cloud smart contract of https://drive.officex.app

![developer-docs](https://github.com/user-attachments/assets/057afc6d-da2f-4750-80c0-5b590bed47de)

```sh
$ rustc --version
rustc 1.86.0 (05f9846f8 2025-03-31)
```

## Get Started

- `NOTES.md` for developer quickstart
- `TODO.md` for roadmap triage
- View developer docs at https://dev.officex.app

### First time install:

```sh
# install dfx
$ sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)" # https://internetcomputer.org/docs/building-apps/getting-started/install

# install nodejs
$ brew install node # https://nodejs.org/en/download

# install rust
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # https://www.rust-lang.org/tools/install

# add rust wasm target
$ rustup target add wasm32-unknown-unknown

# run dfx locally
$ dfx start --clean
```

### First Time Running

```sh
# create, build & deploy canisters locally
$ dfx canister create canisters-official-backend && dfx canister create canisters-factory-backend && dfx build && dfx deploy canisters-official-backend --argument "(opt record { owner = \"$(dfx identity get-principal)\" })" && dfx deploy canisters-factory-backend --argument "(opt record { owner = \"$(dfx identity get-principal)\" })"

# add gas to factory (get the factory_canister_id from prev step localhost:8000/...?id=factory_canister_id)
$ dfx canister deposit-cycles 20000000000000 <factory_canister_id>
```

### Quick Reference

Dev single line restart:

```sh
$ dfx canister create canisters-official-backend && dfx canister create canisters-factory-backend && dfx build && dfx deploy canisters-official-backend --argument "(opt record { owner = \"$(dfx identity get-principal)\" })" && dfx deploy canisters-factory-backend --argument "(opt record { owner = \"$(dfx identity get-principal)\" })"
```

or standalone factory in dev:

```sh
$ dfx canister create canisters-factory-backend && dfx build && dfx deploy canisters-factory-backend --argument "(opt record { owner = \"$(dfx identity get-principal)\" })"
```

or upgrade specific canister in dev:

```sh
$ dfx build canisters-official-backend && dfx canister install canisters-official-backend --mode upgrade --argument "(opt record { owner = \"$(dfx identity get-principal)\" })" --wasm target/wasm32-unknown-unknown/release/canisters_official_backend.wasm
```

or standalone factory in prod:

```sh
$ dfx canister create canisters-factory-backend && dfx build && dfx deploy --network ic canisters-factory-backend --argument "(opt record { owner = \"$(dfx identity get-principal)\" })"
```

or standalone backend in prod:

```sh
$ dfx canister --network ic create canisters-official-backend && dfx build && dfx deploy --network ic canisters-official-backend --argument "(opt record { owner = \"$(dfx identity get-principal)\" })"
```

or update backend in prod:

```sh
$ dfx build && dfx canister --network ic install <canister_id> --mode upgrade --argument "(opt record { owner = \"$(dfx identity get-principal)\" })" --wasm target/wasm32-unknown-unknown/release/canisters_official_backend.wasm
```

or update factory in prod:

```sh
$ dfx build && dfx canister --network ic install <canister_id> --mode upgrade --argument "(opt record { owner = \"$(dfx identity get-principal)\" })" --wasm target/wasm32-unknown-unknown/release/canisters_factory_backend.wasm
```

deposit gas cycles into dev canister

```sh
$ dfx canister deposit-cycles 20000000000000 <canister_id>
```

From clean start:

```sh
$ dfx start --clean
```

Then deploy canisters:

```sh
$ dfx canister create canisters-official-frontend
$ dfx canister create canisters-official-backend
$ dfx canister create canisters-factory-backend
$ dfx build
$ dfx deploy canisters-official-backend --argument "(opt record { owner = \"$(dfx identity get-principal)\" })"
$ dfx deploy canisters-factory-backend --argument "(opt record { owner = \"$(dfx identity get-principal)\" })"

# Optional: Then we can get the admin api key (this redeem can only happen once)
# You can also do this via REST call elsewhere
$ curl -X POST "http://$(dfx canister id canisters-official-backend).localhost:$(dfx info webserver-port)/v1/default/organization/redeem_spawn" \
  -H "Content-Type: application/json" \
  --data '{"redeem_code": "DEFAULT_SPAWN_REDEEM_CODE"}'

# Which responds something like below. use the admin_login_password. if you are on localhost, update "src/lib.rs" variable LOCAL_DEV_MODE = true

# {"ok":{"data":{"drive_id":"DriveID_bkyz2-fmaaa-aaaaa-qaaaq-cai","endpoint":"https://bkyz2-fmaaa-aaaaa-qaaaq-cai.icp0.io","api_key":"eyJhdXRoX3R5cGUiOiJBUElfX0tFWSIsInZhbHVlIjoiOTg4N2FhYzFhYjZkOGE5OGMyYmYwY2RkNzA2YmU4MTY4MjIwZjc3NmUwYzU1ODdlNDU5N2ExMTM1ZjRiZGNiYiJ9","note":"","admin_login_password":"DriveID_bkyz2-fmaaa-aaaaa-qaaaq-cai:eyJhdXRoX3R5cGUiOiJBUElfX0tFWSIsInZhbHVlIjoiOTg4N2FhYzFhYjZkOGE5OGMyYmYwY2RkNzA2YmU4MTY4MjIwZjc3NmUwYzU1ODdlNDU5N2ExMTM1ZjRiZGNiYiJ9@https://bkyz2-fmaaa-aaaaa-qaaaq-cai.icp0.io"}}}
```

Then test:

```
curl -s \ "http://$(dfx canister id canisters-official-backend).localhost:$(dfx info webserver-port)/todos" \ --resolve "$(dfx canister id canisters-official-backend).localhost:$(dfx info webserver-port):127.0.0.1"
```

```sh
# create
$ dfx canister create canisters-official-backend --no-wallet
$ dfx canister create canisters-official-backend-2 --no-wallet
# deploy
$ dfx deploy canisters-official-backend
$ dfx deploy canisters-official-backend-2
```

## Prod Deploy

```sh
# check ICP balance
$ dfx ledger --network ic balance
# check cycles balancedfx ledger --network ic balance
$ dfx wallet --network ic balance
# get account_id for sending ICP tokens to (eg account_id="a641efb49f6febc41a84b7442770619b46693718db210889cefd6750848b2a36")
$ dfx ledger account-id
# get wallet_id to convert ICP to cycles (eg wallet_id="q7b2w-ziaaa-aaaak-afrba-cai")
$ dfx identity get-wallet --network ic
# top up wallet_id with 0.5 ICP from account_id
$ dfx ledger top-up --network ic --amount 0.5 <wallet_id>

# deploy factory (eg canister_id="lfp6f-3iaaa-aaaak-apcgq-cai")
$ dfx build && dfx deploy canisters-factory-backend --network ic --argument "(opt record { owner = \"$(dfx identity get-principal)\" })"

# deposit 4T cycles into canister
$ dfx canister deposit-cycles --network ic 4000000000000 <canister_id>

# check status of deployed caniter
$ dfx canister --network ic status <canister_id>
$ dfx canister --network ic logs <canister_id>

# add custom controller
$ dfx canister update-settings --add-controller <controller_principal_id>  --all <canister_id>
```

Deleting canister to get cycles back:

```sh
$ dfx canister --network ic stop <canister_id>
$ dfx canister --network ic delete <canister_id>
```
