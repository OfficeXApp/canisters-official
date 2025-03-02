# TODO TRIAGE

## Sprints Roadmap

3. Cleanup & testing backend
4. Refactor frontend & implement torrents for browser-cache/local-ssd sharing

## Urgent Next

- [ ] Refactor unify Errors
- [ ] Regenerate proper REST API docs

## Awkward Urgent

- [ ] Refactor list pagniation to use single cursor instead of cursor_up and cursor_down, since direction tells us where to go
- [ ] Refactor list to apply filter on all appropriate route items, including tags

## Near Future

- [ ] Setup factory to spawn Drive canisters with owner set
- [ ] Test out webapp http server
- [ ] Test out webapp torrenting
- [ ] Consider optimistic frontend UI (we should probably use Tanstack Query for React as it handles it for us)
- [ ] Refactor frontend (or consider how to enable AI rest calls)
- [ ] Consider how to obfuscate ancestor folders in url route (eg. show folder_uuid in the url instead of full path)

## Priority Backlog

- [ ] Test whether the s3/storj copy operation works (does raw_storage actually get duplicated?)
- [ ] Implement browser-cache raw file storage --> no raw_url as it lives in browser cache, only way to access is via p2p webrtc which is a non-persistent link or via torrent link
- [ ] Implement local-ssd raw file storage --> no raw_url as it lives in local SSD, only way to access is via p2p webrtc which is a non-persistent link or via torrent link

## Backlog

- [ ] Figure out how to cleanly update past Drive canisters spawned from factory
- [ ] Consider migrating internal state to `ic-stable-structures` for easy upgradeability, otherwise need to implement pre/post upgrade hooks
- [ ] Migrate S3 secret key storage to safer VET keys https://x.com/DFINITYDev/status/1893198318781513878
- [ ] Implement proxied aws/storj where users simply send ETH/SOL to us and we provide storage (might be a scope API key for S3?)
- [ ] Implement recent files/folders queue (should this be a frontend only distinction instead of a tag per user, `Recent::{UserID}`?) --> this should be frontend only because removing a recent tag is pain in ass
- [ ] Paywalls
- [ ] Should we allow "network visualization" where we give frontend a JSON graph of what a user has access to? their teams, etc
- [ ] Audit broken atomic transactions on route, if throw an error we should undo mutations
- [ ] Unit Tests (https://www.startearly.ai/)
- [ ] Implement deterministic canister public keys so that we can set a public icp principal without spending gas or wifi (this is moreso for NodeJS)
- [ ] Implement signed signatures in canister-to-canister REST calls (that icp canister can create same signature as frontend for signing). should use same signing pattern as frontend js but doesnt have to, just add new AuthTypeEnum
- [ ] Video preview slides? (this would probably have to be a limited feature or post upload job)
- [ ] Standardized Error codes? Useful for internationalization and 3rd party developers --> i18n might not need error codes if we allow frontend clients to handle translation of errors, but having standardized error codes is definately helpful for 3rd party developers

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
- [x] Consider whether we need to decouple IDs from ICP public address, and instead let it be uuid and have `Contact.icp_principal` and `Contact.external_id` and save for `Drive.icp_principal` and `Drive.external_id`. --> Yes we decoupled it. There is no `Contact.external_id` as all communication must use PublicKeyICP
- [x] Migrate & refactor core drive code --> in front of every POST /directory/action:getFile we need to generate a new raw_url (and potentially also track access_tokens for reuse, at least for public)
- [x] Implement in-canister raw file storage, perhaps we should try a pure asset container? --> has raw_url but only if we add asset-canister functionality --> in the end we settled for in-canister filestorage via persistent chunked binary data. while the data is stored & streamed onchain, its slow expensive and cant be accessed via plain url string, must be via webapp to render.
- [x] Refactor how recycling bin works.
- [x] Handle directory file & folder actions via `/directory/action`. Note that copy & move operations are also in actions but could be recursively long if many subfolders/subfiles. Do not allow copy/move between disks and max 20 actions in a batch.
- [x] Implement multi-disk storage
- [x] Implement aws s3 storage --> has raw_url but we should be generating on-the-fly urls with temp access token each time. also implement copy/move operations (dont need to wait for ACL since generating access token logic is same)
- [x] Implement web3storj storage --> has raw_url but we should be generating on-the-fly urls with temp access token each time. also implement copy/move operations (dont need to wait for ACL since generating access token logic is same)
- [x] Add compatibility for storjweb3 alongside aws s3. test with private acl and upload.
- [x] Figure out best way to elegantly handle in-canister vs off-canister raw file storage (potentially also `disks` logic holding auth creds)
- [x] Write the `directory` REST routes and particularly the file action logic
- [x] Refactor all ID generation to use prefix, and handle all multi-type IDs with conversion
- [x] Implement routes `permissions/directory/*`
- [x] Implement routes `/permissions/system/*`
- [x] Auth check `permissions` on all REST CRUD
- [x] Implement meta-permission to allow teams/users to edit ALL permission records `SystemTableEnum.Permissions`
- [x] Include permissions in the response body of GET system records and `directory/action`
- [x] Add deferred join team links, with ICP signature as proof of user icp principal
- [x] Handle cosmic teams in permissions, remember TeamID is `TeamID_123--DriveID_abc`. Might need a route to allow 3rd party checks if member is in team
- [x] Connect relevant REST routes with relevant `webhook` firing
- [x] Implement external share tracking via webhooks
- [x] Implement directory webhook permissions `DirectoryPermissionType::Webhooks`
- [x] Implement replayability
- [x] Consider audit trailing events for replayability (on directory actions but also permissions and such)
- [x] Consider whether we need historical action logs and replayability (eg. who did what when, can we "rollback"?)
- [x] Figure out how to handle CRUD of canisters when someone shares a file with you and you accept. Also how to share a file with an anon person? --> current theory, we can generate an API key for them and set the API key user_id to a hardcoded non-principal value. then the receivers client ui can generate an icp principal and tell our canister which then updates the API key's user_id --> solution, the placeholder share links
- [x] Allow ICP signature or EDSCA signature as "native api key" with time window (solves issue of cold start no api_keys). Also requires frontend implementation for convinence. Use the function `src/core/state/types.rs::parse_auth_header_value`
- [x] Investigate web2/web3 use of auth signatures as API Keys, will it work? how to prevent spoofing?
- [x] Consider the danger of UserID values that dont comply with ICP Principals and how it would work in non-canister envs such as NodeJS and ClientJS. where are all the touchpoints? especially future signature proofs --> we dont know all the touchpoints yet as we are still making on the fly decisions. but the encryption method itself would be the same in NodeJS as we can just run the same code
- [x] Update the deferred placeholder team invites & permissions, with cryptographic proofs of public address ownership --> unncessary as we might actually _want_ to allow delegated placeholder redemption
- [x] Implement fuzzy string search to files with re-indexing & update search directory route -> use crate rust-fuzzy-search and minimize search space by searching within a folder
- [x] Implement universal tagging of files/folders/contacts/drives/disks/teams/webhooks & update search directory route -> keep a hashtable of tags Hashtable<TagString, Vec<ResourceID>>, update FileMetadata.tags/FolderMetadata.tags = Vec<TagString>, and should be able to set tags on create too, with auto ignore invalid tags or unauth tags --> also we dont allow creating resources with tags pre-set for simplicity purposes. if you want to add tags to a new resource, it must be done after create via the dedicated tag route
- [x] Add support for protected tags so that users cant masquerade attack tag groups (do we need to setup permissions for this?) --> we likely need to refactor tags from its simple strings to a full crud with acl, because we want to let team leaders manage tags too
- [x] Implement tag deletion
- [x] Add webhooks on tags
- [x] Implement file/folder tags
- [x] Fix missing table permissions on resources
- [x] Upgrade tag permissions with metadata of tag prefix (allow users to write on specific prefix tag strings)
- [x] Upgrade directory/system permissions to allow CRUD on tag prefixes also
- [x] Add system resource wide "external_id" & "external_metadata" to all tables, and a new hashtable to track external_id to internal id (maybe even a route for it). note we need to remember to call
- [x] Consider whether to add api_version & canister_id into the url to support multi-tenant backends, primarily in nodejs. eg. `/v1/{drive_id}/rest_of_route`
- [x] Ability to change drive owners (this can be a single REST route with 2-step process, where admin simply calls function twice with same new owner_id. a local state can be used to track 1st "placeholder" of who and timestamp, and 2nd call only works if after 24 hours or something)
- [x] Add validation to `contact.icp_principal` and `contact.evm_public_address`
- [x] Add route body validation to all routes (eg. similar to external_payload max 8kb, nicknames max 64 chars, etc)
- [x] Review & standardize backend routes and their ingress/egress shapes to be a unified clean
