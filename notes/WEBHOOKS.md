# Webhooks

Webhooks on OfficeX Drive events let us integrate into systems as a primitive.
Here are some example events to receieve webhook alerts on, along with their alt_index and filters.

Quick Notes:

- `alt_index` is a resource_id that differs per event type. it might be a FileID or a FolderID, etc
- `filters` are not supported yet (responsibility is delegated to receiving webhook)

```js
const webhook_events = [
  { label: "file.viewed", alt_index: `${FileID}` },
  { label: "file.created", alt_index: "FILE_CREATED" },
  { label: "file.updated", alt_index: `${FileID}` },
  { label: "file.deleted", alt_index: `${FileID}` },
  { label: "file.shared", alt_index: `${FileID}` },
  { label: "folder.viewed", alt_index: `${FolderID}` },
  { label: "folder.created", alt_index: "FOLDER_CREATED" },
  { label: "folder.updated", alt_index: `${FolderID}` },
  { label: "folder.deleted", alt_index: `${FolderID}` },
  { label: "folder.shared", alt_index: `${FolderID}` },
  {
    label: "folder.file.created",
    alt_index: `${FolderID}`,
    filters: "{ recursion_depth: number }",
  },
  {
    label: "folder.file.updated",
    alt_index: `${FolderID}`,
    filters: "{ recursion_depth: number }",
  },
  {
    label: "folder.file.deleted",
    alt_index: `${FolderID}`,
    filters: "{ recursion_depth: number }",
  },
  {
    label: "folder.file.shared",
    alt_index: `${FolderID}`,
    filters: "{ recursion_depth: number }",
  },
  { label: "team.invite.created", alt_index: `${TeamID}` },
  { label: "team.invite.updated", alt_index: `${TeamID}` },
  { label: "___.___", alt_index: "___ID" },
  { label: "___.___", alt_index: "___ID" },
];
```
