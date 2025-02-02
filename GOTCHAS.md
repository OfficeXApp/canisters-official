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
- `folder.*` -> `{FolderID}`
- `folder.file.*` -> `{FolderID_files}`
- `team.invite.*` -> `{TeamID}_{InviteID}`
- `drive.*` -> `""` empty string

So when a CRUD event happens on a file, we can take the file.id and use it to query against `WEBHOOKS_BY_ALT_INDEX_HASHTABLE` to quickly check in O(1) time if theres a relevant webhook. The limitation is there can only be 1 webhook per resource.
