# Webhooks

Webhooks on OfficeX Drive events let us integrate into systems as a primitive.
Here are some example events to receieve webhook alerts on, along with their alt_index and filters.

Quick Notes:

- `alt_index` is a resource_id that differs per event type. it might be a FileID or a FolderID, etc
- `filters` is a string that differs per event type. it might be filters on a path or a user, etc

```js
const webhook_events = [
  { label: "file.viewed", alt_index: `${FileID}`, filters: "by___" },
  { label: "file.created", alt_index: "FILE_CREATED", filters: "by___" },
  { label: "file.updated", alt_index: `${FileID}`, filters: "by___" },
  { label: "file.deleted", alt_index: `${FileID}`, filters: "by___" },
  { label: "file.shared", alt_index: `${FileID}`, filters: "by___" },
  { label: "folder.viewed", alt_index: `${FolderID}`, filters: "by___" },
  { label: "folder.created", alt_index: "FOLDER_CREATED", filters: "by___" },
  { label: "folder.updated", alt_index: `${FolderID}`, filters: "by___" },
  { label: "folder.deleted", alt_index: `${FolderID}`, filters: "by___" },
  { label: "folder.shared", alt_index: `${FolderID}`, filters: "by___" },
  { label: "folder.file.created", alt_index: `${FolderID}`, filters: "by___" },
  { label: "folder.file.updated", alt_index: `${FolderID}`, filters: "by___" },
  { label: "folder.file.deleted", alt_index: `${FolderID}`, filters: "by___" },
  { label: "folder.file.shared", alt_index: `${FolderID}`, filters: "by___" },
  { label: "team.invite.created", alt_index: `${TeamID}`, filters: "by___" },
  { label: "team.invite.updated", alt_index: `${TeamID}`, filters: "by___" },
  { label: "___.___", alt_index: "___ID", filters: "by___" },
  { label: "___.___", alt_index: "___ID", filters: "by___" },
];
```
