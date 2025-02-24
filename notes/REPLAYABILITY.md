# Replayability

## ToDo

- [🔵] Add decorator `SerdeDiff` to all types in state
- [🔵] Add `snap_prestate` and `snap_poststate` to all routes
- [ ] Implement the checksum validation on diffs
- [ ] Implement the apply-diff on state (enables rollback) and its state representations
- [ ] Implement undo/redo/rollback in routes

## About

P2P filesharing systems should have a replayable state architecture based on ordered diffs. This is crucial for:

- Offline-First: Users can make changes locally and sync when back online
- Conflict Resolution: When multiple peers modify the same file/folder, we can replay both change sequences to determine the correct final state
- Versioning: Any past state can be reconstructed by replaying diffs up to a specific timestamp
- Audit Trail: Every change is recorded with timestamps and actors, providing accountability
- Data Recovery: Corrupted states can be rebuilt by replaying the verified diff sequence

The system stores each change as a diff with a unique timestamp ID, making it easy to sync, merge, and reconstruct states across the P2P network while maintaining consistency.

## Implementation

We can use rust crate `serde-diff` to handle diffing two objects. It will efficiently handle the diffing process and we get bidirectional replayability out of the box. It handles large arrays well.

We can apply atomic transactions on a batch of state changes, as we dont need such granular changes. This would mean we need a tracer_id/or/atomic_txid to represent a state change (which rolls up multiple state changes into one final diff).

If the state gets very large, like 10k records which is 20mb state that needs to be cloned, it will cost about ~$0.01 for every diff operation. mainly due to the cloning of the large state. if 100k records this cost can increase 10x, resulting in quite expensive audit logging. especially if its doing 1000 operations per day, can cost $100/day. however for most retail users they would only have 10k records x 10 operations per day, for a cost of $0.1 per day.

However note that users can just disable audit logs altogether and the cost becomes $0.

## Usage

To enable replayability, simply add a webhook for event type `drive.state_diffs` and you'll receive the diff payload to your webhook.

However we also want to store the last 100 diffs in-canister as well, just for coverage of the most common undo/redo actions.

The REST routes would look like:

```txt
POST /drive/replay
body = {
  command: 'undo' | 'redo' | 'rollback',
  payload: Option<StateDiffWebhookData>
}

returns 'timestamp_ms' // this would be timestamp_ms of the current drive state
```
