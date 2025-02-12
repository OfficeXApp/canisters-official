# Gotchas

## Webhooks

Webhooks are stored in 3 hashtables in `src/core/state/webhooks/state.rs`:

```rs
thread_local! {
    // users pass in api key value, we O(1) lookup the api key id + O(1) lookup the api key
    pub static WEBHOOKS_BY_ALT_INDEX_HASHTABLE: RefCell<HashMap<WebhookAltIndexID, WebhookID>> = RefCell::new(HashMap::new());
    // default is to use the api key id to lookup the api key
    pub static WEBHOOKS_BY_ID_HASHTABLE: RefCell<HashMap<WebhookID, Webhook>> = RefCell::new(HashMap::new());
    // track in hashtable users list of ApiKeyIDs
    pub static WEBHOOKS_BY_TIME_LIST: RefCell<Vec<WebhookID>> = RefCell::new(Vec::new());
}
```

- `WEBHOOKS_BY_TIME_LIST` is a chronological append list of WebhookID, oldest first, latest appended. We use this for listing webhooks and pagination.
- `WEBHOOKS_BY_ID_HASHTABLE` contains the full Webhook data, searchable in O(n) time by its WebhookID
- `WEBHOOKS_BY_ALT_INDEX_HASHTABLE` is a quick reference key:value so that we can quickly find any relevant webhooks for a particular event. Depending on the event, the alt_index could be a different ID string.

This is what a webhook looks like:

```rs
pub struct Webhook {
    pub id: WebhookID,
    pub url: String,
    pub alt_index: WebhookAltIndexID,
    pub event: WebhookEventLabel,
    pub signature: String,
    pub description: String,
    pub active: bool,
}
```

And here are the possible set of events and their corresponding alt_index patterns

- `file.*` -> `{FileID}`
- `folder.*` -> `{FolderUUID}`
- `folder.file.*` -> `{FolderID_files}`
- `team.invite.*` -> `{TeamID}_{InviteID}`
- `drive.*` -> `""` empty string

So when a CRUD event happens on a file, we can take the file.id and use it to query against `WEBHOOKS_BY_ALT_INDEX_HASHTABLE` to quickly check in O(1) time if theres a relevant webhook. The limitation is there can only be 1 webhook per resource.

## Default Canister Memory

By default, a drive on ICP mainnet can use the canister itself for filestorage. While convinient, its also slow (10x slower, takes several mins to upload a 60mb video) and expensive ($5/gb/month). However we must still offer it as an option. For this we use the `/directory/raw_upload/*` and `/directory/raw_download/*` routes.

We batch upload files in chunks and canister server reconstructs writing to stable memory continously (hence slow). When we download, we can either:

1. Download by building the file in browser memory as chunks are downloaded. Convinent but memory inefficient, large files can cause lag. Use this for under <100mb.
2. Download by write-streaming to local computer filesystem which solves the memory efficiency issues, but requires browser permission to access local file directory. Use this for large files >100mb.

The best place to store files is not the canister, its 3rd party integrations like S3, Storj, or local SSD.

## Copy/Move Files Across Disks

OfficeX does not support copy/move between disks (users must manual download/reupload when moving between disks). while s3/storj allow for transfering files to other buckets via REST API, its not always possible on other destination disks like localSSD or cross platform s3 to storj. they would require manual download/reupload. for simplicity, we require all cross-disk transfers to require manual download/reupload. this may change in future with advanced functionality.

## CORS

When adding AWS S3 Buckets disks, users are responsible for enabling cors! In the AWS Console GUI it can be done by navigating to `AWS S3 > YourBucket > Permissions > Cross-Origin resource sharing (CORS)` and pasting this permissive cors policy:

```json
[
  {
    "AllowedHeaders": ["*"],
    "AllowedMethods": ["GET", "POST", "PUT"],
    "AllowedOrigins": ["*"],
    "ExposeHeaders": []
  }
]
```
