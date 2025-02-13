# TODO TRIAGE

## Sprints Roadmap

1. Implement permissions (including deterministics principal ids)
2. Implement webhooks
3. Implement replayability
4. Cleanup & testing backend
5. Refactor frontend & implement torrents for browser-cache/local-ssd sharing

## Urgent Next

- [🔵] Refactor all ID generation to use prefix, and handle all multi-type IDs with conversion
- [🔵] Implement permissions for directory
- [🔵] Allow BLS signature or EDSCA signature as "native api key" with time window (solves issue of cold start no api_keys). Also requires frontend implementation for convinence.
- [ ] Implement permissions for system (disks, teams, drives, contacts)
- [ ] Handle cosmic teams in permissions, remember TeamID is `TeamID_123--DriveID_abc`. Might need a route to allow 3rd party checks if member is in team
- [ ] Write the `permissions` REST routes (https://youtu.be/5GG-VUvruzE?si=lEC0epAhFlD9-2Bp&t=1165)
- [ ] Auth check `permissions` on all REST routes

## Near Future

- [ ] Connect relevant REST routes with relevant `webhook` firing
- [ ] Consider optimistic frontend UI (we should probably use Tanstack Query for React as it handles it for us)
- [ ] Implement proxied aws/storj where users simply send ETH/SOL to us and we provide storage (might be a scope API key for S3?)
- [ ] Consider migrating internal state to `ic-stable-structures` for easy upgradeability, otherwise need to implement pre/post upgrade hooks
- [ ] Consider audit trailing events for replayability (on directory actions but also permissions and such)
- [ ] Consider whether to add hashed cosmic id into the url. eg. `/drive/{urlencoded_cosmic_id}/directory/list`. generate the id with `base64.urlsafe_b64encode("MYADDRESS::MYIP".encode()).decode().rstrip('=')`
- [ ] Investigate web2/web3 use of auth signatures as API Keys, will it work? how to prevent spoofing?

## Priority Backlog

- [ ] Add pub(crate) to rust repo in post-retro
- [ ] Add validation to `contact.icp_principal` and `contact.evm_public_address`
- [ ] Refactor list pagniation to use single cursor instead of cursor_up and cursor_down, since direction tells us where to go
- [ ] Refactor list to apply filter on all appropriate route items
- [ ] Refactor all CRUD creates/updates/deletes to accept array to support bulk operations (hard to change API spec after)
- [ ] Implement file/folder tags
- [ ] Consider the danger of UserID values that dont comply with ICP Principals ThresholdBLS and how it would work in non-canister envs such as NodeJS and ClientJS. where are all the touchpoints? especially future signature proofs --> we dont know all the touchpoints yet as we are still making on the fly decisions. but the encryption method itself would be the same in NodeJS as we can just run the same code
- [ ] Consider whether we need historical action logs and replayability (eg. who did what when, can we "rollback"?)
- [ ] Test whether the s3/storj copy operation works (does raw_storage actually get duplicated?)
- [ ] Implement browser-cache raw file storage --> no raw_url as it lives in browser cache, only way to access is via p2p webrtc which is a non-persistent link or via torrent link
- [ ] Implement local-ssd raw file storage --> no raw_url as it lives in local SSD, only way to access is via p2p webrtc which is a non-persistent link or via torrent link
- [ ] Figure out best way to elegantly handle in-canister vs off-canister raw file storage (potentially also `disks` logic holding auth creds)

## Backlog

- [ ] Decide if we want to add a route `GET /contacts/get/principal/{icp_principal}` alongside `GET /contacts/get/id/{user_id}` (smells since breaks pattern of routes. maybe should be url param like `GET /contacts/get/{user_id}?icp_principal={icp_principal}`)
- [ ] Figure out how to handle CRUD of canisters when someone shares a file with you and you accept. Also how to share a file with an anon person? --> current theory, we can generate an API key for them and set the API key user_id to a hardcoded non-principal value. then the receivers client ui can generate an icp principal and tell our canister which then updates the API key's user_id
- [ ] Unit Tests (https://www.startearly.ai/)
- [ ] Implement deterministic canister public keys so that we can set a public icp principal without spending gas or wifi (this is moreso for NodeJS)

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
- [x] Migrate & refactor core drive code --> in front of every POST /directory/action:getFile we need to generate a new raw_url (and potentially also track access_tokens for reuse, at least for public)
- [x] Implement in-canister raw file storage, perhaps we should try a pure asset container? --> has raw_url but only if we add asset-canister functionality --> in the end we settled for in-canister filestorage via persistent chunked binary data. while the data is stored & streamed onchain, its slow expensive and cant be accessed via plain url string, must be via webapp to render.
- [x] Refactor how recycling bin works.
- [x] Handle directory file & folder actions via `/directory/action`. Note that copy & move operations are also in actions but could be recursively long if many subfolders/subfiles. Do not allow copy/move between disks and max 20 actions in a batch.
- [x] Implement multi-disk storage
- [x] Implement aws s3 storage --> has raw_url but we should be generating on-the-fly urls with temp access token each time. also implement copy/move operations (dont need to wait for ACL since generating access token logic is same)
- [x] Implement web3storj storage --> has raw_url but we should be generating on-the-fly urls with temp access token each time. also implement copy/move operations (dont need to wait for ACL since generating access token logic is same)
- [x] Add compatibility for storjweb3 alongside aws s3. test with private acl and upload.
- [x] Write the `directory` REST routes and particularly the file action logic
