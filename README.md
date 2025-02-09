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
$ dfx canister create canisters-official-backend
$ dfx build
$ dfx deploy
```

Then test:

```
curl -s \ "http://$(dfx canister id canisters-official-backend).localhost:$(dfx info webserver-port)/todos" \ --resolve "$(dfx canister id canisters-official-backend).localhost:$(dfx info webserver-port):127.0.0.1"
```
