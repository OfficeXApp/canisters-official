# TODO TRIAGE

## Urgent Next

- [ ] Migrate & refactor the `directory` REST routes
- [ ] Add pub(crate) to rust repo
- [ ] Consider audit trailing events for replayability (on directory actions but also permissions and such)
- [ ] Write the `directory` REST routes, including adding new one `POST /directory/path-to-id` that given full_url_path returns folder_id or file_id
- [ ] Consider whether to add hashed cosmic id into the url. eg. `/drive/{urlencoded_cosmic_id}/directory/list`. generate the id with `base64.urlsafe_b64encode("MYADDRESS::MYIP".encode()).decode().rstrip('=')`
- [ ] Investigate web2/web3 use of auth signatures as API Keys, will it work? how to prevent spoofing?

## Near Future

- [ ] Connect relevant REST routes with relevant `webhook` firing
- [ ] Handle cosmic teams in permissions, remember TeamID is `TeamID_123--DriveID_abc`. Might need a route to allow 3rd party checks if member is in team
- [ ] Write the `permissions` REST routes (https://youtu.be/5GG-VUvruzE?si=lEC0epAhFlD9-2Bp&t=1165)
- [ ] Auth check `permissions` on all REST routes
- [ ] Consider optimistic frontend UI (we should probably use Tanstack Query for React as it handles it for us)

## Priority Backlog

- [ ] Add validation to `contact.icp_principal` and `contact.evm_public_address`
- [ ] Refactor list pagniation to use single cursor instead of cursor_up and cursor_down, since direction tells us where to go
- [ ] Refactor list to apply filter on all appropriate route items
- [ ] Refactor all CRUD creates/updates/deletes to accept array to support bulk operations (hard to change API spec after)
- [ ] Consider the danger of UserID values that dont comply with ICP Principals ThresholdBLS and how it would work in non-canister envs such as NodeJS and ClientJS. where are all the touchpoints? especially future signature proofs --> we dont know all the touchpoints yet as we are still making on the fly decisions. but the encryption method itself would be the same in NodeJS as we can just run the same code
- [ ] Consider whether we need historical action logs and replayability (eg. who did what when, can we "rollback"?)

## Backlog

- [ ] Decide if we want to add a route `GET /contacts/get/principal/{icp_principal}` alongside `GET /contacts/get/id/{user_id}` (smells since breaks pattern of routes. maybe should be url param like `GET /contacts/get/{user_id}?icp_principal={icp_principal}`)
- [ ] Allow BLS signature or EDSCA signature as "native api key" with time window (solves issue of cold start no api_keys). Also requires frontend implementation for convinence.
- [ ] Figure out how to handle CRUD of canisters when someone shares a file with you and you accept. Also how to share a file with an anon person? --> current theory, we can generate an API key for them and set the API key user_id to a hardcoded non-principal value. then the receivers client ui can generate an icp principal and tell our canister which then updates the API key's user_id
- [ ] Unit Tests (https://www.startearly.ai/)

## Completed

- [x] Setup boilerplate
- [x] Write base `api_keys` REST routes
- [x] Find, debug & fix developer gotchas related to ICP HTTP Canister development
- [x] Write the `webhooks` REST routes & define the possible events for listening
- [x] Remove the `__type` from upserts. Just check if `req.body.id` is present to determine create vs edit
- [x] Write the `contacts` REST routes
- [x] Write the `teams` REST routes
- [x] Write the `team_invites` REST routes
- [x] Write the `drives` REST routes
- [x] Write the `disks` REST routes
- [x] Consider whether we need to decouple IDs from BLS public address, and instead let it be uuid and have `Contact.icp_principal` and `Contact.external_id` and save for `Drive.icp_principal` and `Drive.external_id`. --> Yes we decoupled it. There is no `Contact.external_id` as all communication must use PublicKeyBLS
