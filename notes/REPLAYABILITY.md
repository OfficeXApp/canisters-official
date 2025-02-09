# Replayability

P2P filesharing systems should have a replayable state architecture based on ordered diffs. This is crucial for:

- Offline-First: Users can make changes locally and sync when back online
- Conflict Resolution: When multiple peers modify the same file/folder, we can replay both change sequences to determine the correct final state
- Versioning: Any past state can be reconstructed by replaying diffs up to a specific timestamp
- Audit Trail: Every change is recorded with timestamps and actors, providing accountability
- Data Recovery: Corrupted states can be rebuilt by replaying the verified diff sequence

The system stores each change as a diff with a unique timestamp ID, making it easy to sync, merge, and reconstruct states across the P2P network while maintaining consistency.

```js
type Diff = {
  id: number, // Unix timestamp in ms as unique ID
  path: string[], // Path to changed value
  from: any, // Previous value
  to: any, // New value
  meta?: {
    actor: string, // Who made the change
    memo: string, // Why/what changed
  },
};

// Example usage:
const state = { id: "my-count", data: { counter: 0 } };

const diffs: Diff[] = [
  {
    id: Date.now(), // e.g. 1707175440000
    path: ["data", "counter"],
    from: 0,
    to: 1,
    meta: {
      actor: "user123",
      memo: "Increment counter",
    },
  },
];

function replay(initial: any, diffs: Diff[]): any {
  return diffs
    .sort((a, b) => a.id - b.id) // Ensure chronological order
    .reduce((state, diff) => {
      return immer(state, (draft) => {
        let target = draft;
        for (let i = 0; i < diff.path.length - 1; i++) {
          target = target[diff.path[i]];
        }
        target[diff.path[diff.path.length - 1]] = diff.to;
      });
    }, initial);
}
```
