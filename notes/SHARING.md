# Share Tracking

We are considering two approaches to share tracking - internal vs external.
We should use external strategy since its so much simpler.

## Tracked Internally

Internal Share Tracking is expensive and higher risk.

Share tracking is enabled at the direct file or ancestor folder level up to 20 folders up. It still respects sovereign permissions.

```ts
FileMetadata.enable_share_tracking = bool;
FolderMetadata.enable_share_tracking = bool;
```

Share tracking is managed via a Radix-tree which lets us quickly query sharing by ResourceID (fileID or folderID). The node ID is a concat of resource_id and referrer share_id, so for example:

```ts
const radix_node_id = `${FileUUID}_${refererShareTrackID}` // refererShareID may also be blank
const radix_node = (radix_node_id, ShareTrackID[])
```

This lets us easily query all shares related to a file or a referers share, in a sorted scope. Finding matches can return multiple shareID results.

Then we manage a seperate hashtable for the actual shareIDs themselves.

```ts
type ShareTrackID = string;
// we can construct the radix_node_id from the resource_id and origin
interface ShareTrack {
  id: ShareTrackID;
  origin?: ShareTrackID;
  from?: UserID;
  to?: UserID;
  resource_id: FileUUID | FolderUUID;
  timestamp_ms: number;
  metadata?: String; // metadata can contain utm params and other data
}
const hashtable_share_tracks = Record<ShareID, ShareTrack>;
```

To accompany the share tracking, we also need REST rest route to clear up old space. For this we would use the delete permissions of folders and files. It should only be possible via `radix_node_id` where passing in a substring will delete all records in the radix tree underneath it. Note that deletions on the radix tree will also delete ShareTrackID records in the hashtable which can cause missing data (for example, a deleted ShareTrackID still is the origin for another ShareTrack record). Our code must be able to safely handle such missing cases.

We also need a REST route for querying the breadth first or depth first graph of share tracking. Together the REST route looks something like this:

```js
POST /analytics/sharing/delete
body = { prefix: "radix_node_id" }

POST /analytics/sharing/graph
body = { prefix: "radix_node_id", strategy: 'DFS' | 'BFS', depth: number }
```

## Tracked Externally

External share tracking is simpler and offloads complexity to webhooks. Essentially we treat the act of creating and accepting shares as webhook events with their own unique payloads. Then all complexity around share tracking is offloaded to whatever system the user prefers. (We should recommend a default analytics plugin, something that works with visualization of graph relationships)

With external share tracking, we dont need to add a new `enable_share_tracking` boolean to the file/folder metadata. We can simply rely on webhooks. In which case we want to:

1. Treat every view as a share (generate a new ShareTrackID on the fly)
2. Treat every view as an accept (use the ShareTrackID sent from client)

We can still use similar data structures:

```ts
type ShareTrackID = string;
// we can construct the radix_node_id from the resource_id and origin
interface ShareTrack {
  id: ShareTrackID;
  hash: ShareTrackHash;
  origin_id?: ShareTrackID;
  origin_hash?: ShareTrackHash;
  from_user?: UserID;
  to_user?: UserID;
  resource_id: FileUUID | FolderUUID;
  resource_name: String;
  canister_id: CanisterID;
  timestamp_ms: number;
  url_endpoint: String; // url of the canister
  metadata?: String; // metadata can contain utm params and other data
}
```

Note this also means users can only get analytics if they have directory permission `DirectoryPermissionType::Webhooks`

Since we wont track any share data in the hashtables, we must embed the necessary info in the sharetrack id itself. in which case our ID should be a btoa hash with the referring userID.

```js
const shareTrackID = generate_unique_id(IDPrefix::ShareTrackID, "");
const shareTrackHash = btoa({
  id: shareTrackID,
  from_user: UserID,
});
```

The shareTrackHash is what gets appended to url params.
