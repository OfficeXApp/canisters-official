# OfficeX Drive Canisters

Powers the cloud smart contract of https://drive.officex.app

![developer-docs](https://github.com/user-attachments/assets/057afc6d-da2f-4750-80c0-5b590bed47de)

### Get Started

- `NOTES.md` for developer quickstart
- `TODO.md` for roadmap triage
- View developer docs at https://dev.officex.app

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
