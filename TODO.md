# TODO TRIAGE

## Urgent Next

- [ ] Add validation to `contact.icp_principal` and `contact.evm_public_address`
- [ ] Refactor list pagniation to use single cursor instead of cursor_up and cursor_down, since direction tells us where to go
- [ ] Refactor all CRUD creates/updates/deletes to accept array to support bulk operations (hard to change API spec after)

## Near Future

- [ ] Write the `drive_info` REST routes
- [ ] Write the `canisters` REST routes
- [ ] Write the `disks` REST routes
- [ ] Write the `directory` REST routes
- [ ] Connect relevant REST routes with relevant `webhook` firing
- [ ] Handle cosmic teams in permissions, remember TeamID is `TeamID_123--CanisterID_abc`. Might need a route to allow 3rd party checks if member is in team
- [ ] Write the `permissions` REST routes
- [ ] Auth check `permissions` on all REST routes

## Backlog

- [ ] Decide if we want to add a route `GET /contacts/get/principal/{icp_principal}` alongside `GET /contacts/get/id/{user_id}` (smells since breaks pattern of routes. maybe should be url param like `GET /contacts/get/{user_id}?icp_principal={icp_principal}`)
- [ ] Allow BLS signature or EDSCA signature as "native api key" with time window (solves issue of cold start no api_keys). Also requires frontend implementation for convinence.
- [ ] Unit Tests

## Completed

- [x] Setup boilerplate
- [x] Write base `api_keys` REST routes
- [x] Find, debug & fix developer gotchas related to ICP HTTP Canister development
- [x] Write the `webhooks` REST routes & define the possible events for listening
- [x] Remove the `__type` from upserts. Just check if `req.body.id` is present to determine create vs edit
- [x] Write the `contacts` REST routes
- [x] Write the `teams` REST routes
- [x] Write the `team_invites` REST routes
