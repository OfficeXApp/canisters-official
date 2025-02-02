# TODO TRIAGE

## Urgent Next

- [ ] Write the `webhooks` REST routes & define the possible events for listening
- [ ] Remove the `__type` from upserts. Just check if `req.body.id` is present to determine create vs edit
- [ ] Write the `contacts` REST routes
- [ ] Connect relevant `contacts` REST routes with relevant `webhook` firing

## Near Future

- [ ] Write the `drive-info` REST routes
- [ ] Write the `teams` REST routes
- [ ] Write the `canisters` REST routes
- [ ] Write the `disks` REST routes
- [ ] Write the `directory` REST routes
- [ ] Write the `permissions` REST routes
- [ ] Auth check `permissions` on all REST routes

## Backlog

- [ ] Allow BLS signature or EDSCA signature as "native api key" with time window (solves issue of cold start no api-keys). Also requires frontend implementation for convinence.
- [ ] Unit Tests

## Completed

- [x] Setup boilerplate
- [x] Write base `api-keys` REST routes
- [x] Find, debug & fix developer gotchas related to ICP HTTP Canister development
