# Webhooks

Webhooks on OfficeX Drive events let us integrate into systems as a primitive.
Here are some example events to receieve webhook alerts on, along with their alt_index and filters.

Quick Notes:

- `alt_index` is a resource_id that differs per event type. it might be a FileID or a FolderID, etc
- `filters` are not supported yet (responsibility is delegated to receiving webhook)

```js
/**
 * Webhook events configuration for OfficeX Drive
 * Maps to Rust WebhookEventLabel enum
 *
 * alt_index patterns:
 * - Specific resource IDs: ${FileID}, ${FolderID}, ${TeamID}
 * - Constants: FILE_CREATED, FOLDER_CREATED, RESTORE_TRASH
 */
const webhook_events = [
  // File events
  { label: "file.viewed", alt_index: "${FileID}" },
  { label: "file.created", alt_index: "FILE_CREATED" },
  { label: "file.updated", alt_index: "${FileID}" },
  { label: "file.deleted", alt_index: "${FileID}" },
  { label: "file.shared", alt_index: "${FileID}" },
  { label: "file.accepted", alt_index: "${FileID}" },

  // Folder events
  { label: "folder.viewed", alt_index: "${FolderID}" },
  { label: "folder.created", alt_index: "FOLDER_CREATED" },
  { label: "folder.updated", alt_index: "${FolderID}" },
  { label: "folder.deleted", alt_index: "${FolderID}" },
  { label: "folder.shared", alt_index: "${FolderID}" },
  { label: "folder.accepted", alt_index: "${FolderID}" },

  // Subfile events
  { label: "subfile.viewed", alt_index: "${FolderID}" },
  { label: "subfile.created", alt_index: "${FolderID}" },
  { label: "subfile.updated", alt_index: "${FolderID}" },
  { label: "subfile.deleted", alt_index: "${FolderID}" },
  { label: "subfile.shared", alt_index: "${FolderID}" },
  { label: "subfile.accepted", alt_index: "${FolderID}" },

  // Subfolder events
  { label: "subfolder.viewed", alt_index: "${FolderID}" },
  { label: "subfolder.created", alt_index: "${FolderID}" },
  { label: "subfolder.updated", alt_index: "${FolderID}" },
  { label: "subfolder.deleted", alt_index: "${FolderID}" },
  { label: "subfolder.shared", alt_index: "${FolderID}" },
  { label: "subfolder.accepted", alt_index: "${FolderID}" },

  // Team events
  { label: "team.invite.created", alt_index: "${TeamID}" },
  { label: "team.invite.updated", alt_index: "${TeamID}" },

  // Drive events
  { label: "drive.gas_low", alt_index: "${DriveID}" },
  { label: "drive.sync_completed", alt_index: "${DriveID}" },
  { label: "drive.restore_trash", alt_index: "RESTORE_TRASH" },
];
```

Users can only apply webhooks if they have permission on directory via `DirectoryPermissionType::Webhooks`
