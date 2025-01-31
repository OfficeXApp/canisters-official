# Developer Notes

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
