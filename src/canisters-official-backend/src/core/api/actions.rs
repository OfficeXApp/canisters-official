// src/core/api/actions.rs
use std::result::Result;
use crate::{core::{state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata}, types::{DriveFullFilePath, FileID, FolderID, PathTranslationResponse, ShareTrackID, ShareTrackResourceID}}, drives::{state::state::{update_external_id_mapping, DRIVE_ID, OWNER_ID, URL_ENDPOINT}, types::{ExternalID, ExternalPayload}}, permissions::types::{DirectoryPermissionType, PermissionGranteeID}, webhooks::types::{WebhookAltIndexID, WebhookEventLabel}}, types::{ICPPrincipalString, IDPrefix, PublicKeyICP, UserID}}, debug_log, rest::{directory::types::{CreateFileResponse, CreateFolderResponse, DeleteFileResponse, DeleteFolderResponse, DirectoryAction, DirectoryActionEnum, DirectoryActionPayload, DirectoryActionResult, DirectoryResourceID, GetFileResponse, GetFolderResponse, UpdateFileResponse}, webhooks::types::{DirectoryWebhookData, FileWebhookData, FolderWebhookData, ShareTrackingWebhookData}}};
use super::{drive::drive::{copy_file, copy_folder, create_file, create_folder, delete_file, delete_folder, get_file_by_id, get_folder_by_id, move_file, move_folder, rename_file, rename_folder, restore_from_trash}, internals::drive_internals::{get_destination_folder, translate_path_to_id}, permissions::{self, directory::{check_directory_permissions, derive_directory_breadcrumbs, preview_directory_permissions}}, uuid::{decode_share_track_hash, generate_share_track_hash, ShareTrackHash}, webhooks::directory::{fire_directory_webhook, get_active_file_webhooks, get_active_folder_webhooks}};


#[derive(Debug, Clone)]
pub struct DirectoryActionErrorInfo {
    pub code: i32,
    pub message: String,
}

pub async fn pipe_action(action: DirectoryAction, user_id: UserID) -> Result<DirectoryActionResult, DirectoryActionErrorInfo> {
    
    if let Err(validation_error) = action.validate_body() {
        return Err(DirectoryActionErrorInfo {
            code: 400,
            message: format!("Validation error: {} - {}", validation_error.field, validation_error.message),
        });
    }
    
    match action.action {
        DirectoryActionEnum::GetFile => {
            match action.payload {
                DirectoryActionPayload::GetFile(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }

                    // First try to get file_id either from resource_id or resource_path
                    let file_id = payload.id;

                    // Get webhooks for both event types and combine them
                    let webhooks_file = get_active_file_webhooks(&file_id, WebhookEventLabel::FileViewed);
                    let webhooks_subfile = get_active_file_webhooks(&file_id, WebhookEventLabel::SubfileViewed);
                  
                    // Get file metadata to use for permission check
                    let file = match get_file_by_id(file_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("File not found: {}", e),
                        }),
                    };

                    // Check if user has View permission on the file
                    let resource_id = DirectoryResourceID::File(file_id.clone());
                    let user_permissions = check_directory_permissions(
                        resource_id.clone(),
                        PermissionGranteeID::User(user_id.clone())
                    ).await;

                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());
        
                    // User needs at least View permission to get file details
                    if !is_owner && !user_permissions.contains(&DirectoryPermissionType::View) {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to view this file".to_string(), 
                        });
                    }

                    // let your_permissions = preview_directory_permissions(&resource_id, &user_id);

                    let before_snap_file = DirectoryWebhookData::File(FileWebhookData {
                        file: Some(file.clone()),
                    }); 

                    fire_directory_webhook(
                        WebhookEventLabel::FileViewed,
                        webhooks_file,
                        Some(before_snap_file.clone()),
                        Some(before_snap_file.clone()),
                        Some("File viewed".to_string()),
                    );
                    fire_directory_webhook(
                        WebhookEventLabel::SubfileViewed,
                        webhooks_subfile,
                        Some(before_snap_file.clone()),
                        Some(before_snap_file),
                        Some("Subfile viewed".to_string()),
                    );


                    let mut share_tracking_origin_id = ShareTrackID(String::new());
                    let mut share_tracking_origin_user = UserID(String::new());
                    if let Some(share_track_hash) = &payload.share_track_hash {
                        if !share_track_hash.is_empty() {
                            let share_track_hash = ShareTrackHash(share_track_hash.clone());
                            let (share_track_id, from_user_id) = decode_share_track_hash(&share_track_hash);
                            share_tracking_origin_id = share_track_id;
                            share_tracking_origin_user = from_user_id;
                        }
                    }
                    // generate_share_track_hash 
                    let (my_share_track_id, my_share_track_hash) = generate_share_track_hash(&user_id);
                    let webhooks_file_shared = get_active_file_webhooks(&file_id, WebhookEventLabel::FileShared);
                    let webhooks_subfile_shared = get_active_file_webhooks(&file_id, WebhookEventLabel::SubfileShared);
                    let share_tracking_payload = ShareTrackingWebhookData {
                        id: my_share_track_id.clone(),
                        hash: my_share_track_hash.clone(),
                        origin_id: Some(share_tracking_origin_id),
                        origin_hash: Some(ShareTrackHash(payload.share_track_hash.unwrap_or(String::new()))),
                        from_user: Some(share_tracking_origin_user.clone()),
                        to_user: Some(user_id.clone()),
                        resource_id: ShareTrackResourceID::File(file_id.clone()),
                        resource_name: file.name.clone(),
                        drive_id: DRIVE_ID.with(|id| id.clone()),
                        timestamp_ms: ic_cdk::api::time() / 1_000_000,
                        host_url: URL_ENDPOINT.with(|url| url.borrow().get().clone()),
                        metadata: None
                    };
                    fire_directory_webhook(
                        WebhookEventLabel::FileShared,
                        webhooks_file_shared,
                        None,
                        Some(DirectoryWebhookData::ShareTracking(share_tracking_payload.clone())),
                        Some("Tracked file share".to_string()),
                    );
                    fire_directory_webhook(
                        WebhookEventLabel::SubfileShared,
                        webhooks_subfile_shared,
                        None,
                        Some(DirectoryWebhookData::ShareTracking(share_tracking_payload)),
                        Some("Tracked subfile share".to_string()),
                    );

                    let breadcrumbs = derive_directory_breadcrumbs(
                        resource_id,
                        user_id.clone()
                    ).await;

                    // If we get here, user is authorized - return the file metadata
                    Ok(DirectoryActionResult::GetFile(GetFileResponse {
                        file: file.cast_fe(&user_id).await,
                        breadcrumbs
                    }))
                },
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for GET_FILE action".to_string(),
                }),
            }
        }
        
        DirectoryActionEnum::GetFolder => {
            match action.payload {
                DirectoryActionPayload::GetFolder(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }

                    // Get folder_id from either resource_id or resource_path
                    let folder_id = payload.id;
        
                    // Get folder metadata
                    let folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Folder not found: {}", e),
                        }),
                    };

                    // Get webhooks for both event types and combine them
                    let webhooks_folder = get_active_folder_webhooks(&folder_id, WebhookEventLabel::FolderViewed);
                    let webhooks_subfolder = get_active_folder_webhooks(&folder_id, WebhookEventLabel::SubfolderViewed);

                    // Check if user has View permission on the folder
                    let resource_id = DirectoryResourceID::Folder(folder_id.clone());
                    let user_permissions = check_directory_permissions(
                        resource_id.clone(),
                        PermissionGranteeID::User(user_id.clone())
                    ).await;

                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());
        
                    if !is_owner && !user_permissions.contains(&DirectoryPermissionType::View) {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to view this folder".to_string(),
                        });
                    }

                    // let your_permissions = preview_directory_permissions(&resource_id, &user_id);
                                        
                    let before_snap_folder = DirectoryWebhookData::Folder(FolderWebhookData {
                        folder: Some(folder.clone()),
                    });    
                    

                    fire_directory_webhook(
                        WebhookEventLabel::FolderViewed,
                        webhooks_folder,
                        Some(before_snap_folder.clone()),
                        Some(before_snap_folder.clone()),
                        Some("Folder viewed".to_string()),
                    );
                    fire_directory_webhook(
                        WebhookEventLabel::SubfolderViewed,
                        webhooks_subfolder,
                        Some(before_snap_folder.clone()),
                        Some(before_snap_folder),
                        Some("Subfolder viewed".to_string()),
                    );


                    let mut share_tracking_origin_id = ShareTrackID(String::new());
                    let mut share_tracking_origin_user = UserID(String::new());
                    if let Some(share_track_hash) = &payload.share_track_hash {
                        if !share_track_hash.is_empty() {
                            let share_track_hash = ShareTrackHash(share_track_hash.clone());
                            let (share_track_id, from_user_id) = decode_share_track_hash(&share_track_hash);
                            share_tracking_origin_id = share_track_id;
                            share_tracking_origin_user = from_user_id;
                        }
                    }
                    // generate_share_track_hash 
                    let (my_share_track_id, my_share_track_hash) = generate_share_track_hash(&user_id);
                    let webhooks_folder_shared = get_active_folder_webhooks(&folder_id, WebhookEventLabel::FolderShared);
                    let webhooks_subfolder_shared = get_active_folder_webhooks(&folder_id, WebhookEventLabel::SubfolderShared);
                    let share_tracking_payload = ShareTrackingWebhookData {
                        id: my_share_track_id.clone(),
                        hash: my_share_track_hash.clone(),
                        origin_id: Some(share_tracking_origin_id),
                        origin_hash: Some(ShareTrackHash(payload.share_track_hash.unwrap_or(String::new()))),
                        from_user: Some(share_tracking_origin_user.clone()),
                        to_user: Some(user_id.clone()),
                        resource_id: ShareTrackResourceID::Folder(folder_id.clone()),
                        resource_name: folder.name.clone(),
                        drive_id: DRIVE_ID.with(|id| id.clone()),
                        timestamp_ms: ic_cdk::api::time() / 1_000_000,
                        host_url: URL_ENDPOINT.with(|url| url.borrow().get().clone()),
                        metadata: None
                    };
                    fire_directory_webhook(
                        WebhookEventLabel::FolderShared,
                        webhooks_folder_shared,
                        None,
                        Some(DirectoryWebhookData::ShareTracking(share_tracking_payload.clone())),
                        Some("Tracked folder share".to_string()),
                    );
                    fire_directory_webhook(
                        WebhookEventLabel::SubfolderShared,
                        webhooks_subfolder_shared,
                        None,
                        Some(DirectoryWebhookData::ShareTracking(share_tracking_payload)),
                        Some("Tracked subfolder share".to_string()),
                    );


                    let breadcrumbs = derive_directory_breadcrumbs(
                        resource_id,
                        user_id.clone()
                    ).await;

        
                    Ok(DirectoryActionResult::GetFolder(GetFolderResponse {
                        folder: folder.clone().cast_fe(&user_id).await,
                        breadcrumbs
                    }))
                },
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for GET_FOLDER action".to_string(),
                }),
            }
        },
        
        DirectoryActionEnum::CreateFile => {
            match action.payload {
                DirectoryActionPayload::CreateFile(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }

                    // Get parent folder ID where the file will be created
                    let parent_folder_id = payload.parent_folder_uuid;

                    // Get webhooks for both event types and combine them
                    let webhooks_file = get_active_file_webhooks(&FileID(WebhookAltIndexID::file_created_slug().to_string()), WebhookEventLabel::FileCreated);
                    let webhooks_subfile = get_active_folder_webhooks(&parent_folder_id, WebhookEventLabel::SubfileCreated);

                    // Check if user has Upload, Edit, or Manage permission on the parent folder
                    let parent_resource_id = DirectoryResourceID::Folder(parent_folder_id.clone());
                    let user_permissions = check_directory_permissions(
                        parent_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;

                    // Check if theres an existing file and whether user has EDIT permission
                    // if no file found, it should just continue with code execution
                    let mut has_edit_permission = false;
                    if let Ok(existing_file) = get_file_by_id(FileID(format!("{:?}", payload.id))) {
                        let user_permissions = check_directory_permissions(
                            DirectoryResourceID::File(existing_file.id.clone()),
                            PermissionGranteeID::User(user_id.clone())
                        ).await;
                        has_edit_permission = user_permissions.contains(&DirectoryPermissionType::Edit);
                    }

                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());
        
                    if !is_owner && !user_permissions.contains(&DirectoryPermissionType::Upload) && 
                       !user_permissions.contains(&DirectoryPermissionType::Edit) &&
                       !user_permissions.contains(&DirectoryPermissionType::Manage) && !has_edit_permission {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to create files in this folder".to_string(),
                        });
                    }
        
                    // Get the destination folder metadata
                    let parent_folder = match get_folder_by_id(parent_folder_id.clone()) {
                        Ok(folder) => folder,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Parent folder not found: {}", e),
                        }),
                    };
        
                    // Construct the full file path
                    let full_directory_path = format!("{}{}", parent_folder.full_directory_path.0, payload.name);

        
                    // Create file using the drive API
                    match create_file(
                        payload.id,
                        full_directory_path,
                        payload.disk_id,
                        user_id.clone(),
                        payload.file_size,
                        payload.expires_at.unwrap_or(-1),
                        String::new(), // Empty canister ID to use current canister
                        payload.file_conflict_resolution,
                        Some(payload.has_sovereign_permissions.unwrap_or(false)),
                        payload.shortcut_to,
                        Some(ExternalID(payload.external_id.unwrap_or("".to_string()))),
                        Some(ExternalPayload(payload.external_payload.unwrap_or("".to_string()))),
                        payload.raw_url,
                        payload.notes,
                    ) {
                        Ok((file_metadata, upload_response)) => {

                            let after_snap_file = DirectoryWebhookData::File(FileWebhookData {
                                file: Some(file_metadata.clone()),
                            });

                            fire_directory_webhook(
                                WebhookEventLabel::FileCreated,
                                webhooks_file,
                                None,
                                Some(after_snap_file.clone()),
                                Some("File created".to_string()),
                            );
                            fire_directory_webhook(
                                WebhookEventLabel::SubfileCreated,
                                webhooks_subfile,
                                None,
                                Some(after_snap_file),
                                Some("Subfile created".to_string()),
                            );

                            Ok(DirectoryActionResult::CreateFile(CreateFileResponse {
                                file: file_metadata.cast_fe(&user_id).await,
                                upload: Some(upload_response),
                                notes: Some("File created successfully".to_string()),
                            }))
                        },
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to create file: {}", e),
                        })
                    }
                },
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for CREATE_FILE action".to_string(),
                })
            }
        },
         
        DirectoryActionEnum::CreateFolder => {
            match action.payload {
                DirectoryActionPayload::CreateFolder(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }

                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());

                    // Get parent folder ID where the new folder will be created
                    let parent_folder_id = payload.parent_folder_uuid;

                    // Get webhooks for both event types and combine them
                    let webhooks_folder = get_active_folder_webhooks(&FolderID(WebhookAltIndexID::folder_created_slug().to_string()), WebhookEventLabel::FolderCreated);
                    let webhooks_subfolder = get_active_folder_webhooks(&parent_folder_id, WebhookEventLabel::SubfolderCreated);

        
                    // Check if user has Upload, Edit, or Manage permission on the parent folder
                    let parent_resource_id = DirectoryResourceID::Folder(parent_folder_id.clone());
                    let user_permissions = check_directory_permissions(
                        parent_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;
        
                    if !is_owner && 
                       !user_permissions.contains(&DirectoryPermissionType::Upload) && 
                       !user_permissions.contains(&DirectoryPermissionType::Edit) &&
                       !user_permissions.contains(&DirectoryPermissionType::Manage) {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to create folders here".to_string(),
                        });
                    }
        
                    // Get the parent folder metadata
                    let parent_folder = match get_folder_by_id(parent_folder_id.clone()) {
                        Ok(folder) => folder,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Parent folder not found: {}", e),
                        }),
                    };
        
                    // Construct the full folder path
                    let full_directory_path = DriveFullFilePath(format!("{}{}/", parent_folder.full_directory_path.0, payload.name));
        
                    // Create folder using the drive API
                    match create_folder(
                        payload.id,
                        full_directory_path,
                        payload.disk_id,
                        user_id.clone(),
                        payload.expires_at.unwrap_or(-1),
                        String::new(), // Empty canister ID to use current canister
                        payload.file_conflict_resolution,
                        Some(payload.has_sovereign_permissions.unwrap_or(false)),
                        payload.shortcut_to,
                        Some(ExternalID(payload.external_id.unwrap_or("".to_string()))),
                        Some(ExternalPayload(payload.external_payload.unwrap_or("".to_string()))),
                        payload.notes
                    ) {
                        Ok(folder) => {
                            let after_snap_folder = DirectoryWebhookData::Folder(FolderWebhookData {
                                folder: Some(folder.clone()),
                            });

                            fire_directory_webhook(
                                WebhookEventLabel::FolderCreated,
                                webhooks_folder,
                                None,
                                Some(after_snap_folder.clone()),
                                Some("Folder created".to_string()),
                            );
                            fire_directory_webhook(
                                WebhookEventLabel::SubfolderCreated,
                                webhooks_subfolder,
                                None,
                                Some(after_snap_folder),
                                Some("Subfolder created".to_string()),
                            );

                            Ok(DirectoryActionResult::CreateFolder(CreateFolderResponse {
                                notes: Some("Folder created successfully".to_string()),
                                folder: folder.cast_fe(&user_id).await,
                            }))
                        },
                        Err(e) => match e.as_str() {
                            "Folder already exists" => Err(DirectoryActionErrorInfo {
                                code: 409,
                                message: "A folder with this name already exists".to_string(),
                            }),
                            _ => Err(DirectoryActionErrorInfo {
                                code: 500,
                                message: format!("Failed to create folder: {}", e),
                            })
                        }
                    }
                },
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for CREATE_FOLDER action".to_string(),
                })
            }
        },
        
        DirectoryActionEnum::UpdateFile => {
            match action.payload {
                DirectoryActionPayload::UpdateFile(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }

                    // Get file ID from either resource_id or resource_path
                    let file_id = payload.id;
        
                    // Get current file metadata
                    let file = match get_file_by_id(file_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("File not found: {}", e),
                        }),
                    };

                    // Get webhooks for both event types and combine them
                    let webhooks_file = get_active_file_webhooks(&file_id, WebhookEventLabel::FileUpdated);
                    let webhooks_subfile = get_active_file_webhooks(&file_id, WebhookEventLabel::SubfileUpdated);
        
                    let before_snap_file = DirectoryWebhookData::File(FileWebhookData {
                        file: Some(file.clone()),
                    });

                    // Get parent folder permissions
                    let parent_folder_id = file.parent_folder_uuid.clone();
                    let parent_resource_id = DirectoryResourceID::Folder(parent_folder_id);
                    let user_permissions = check_directory_permissions(
                        parent_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;

                    // Check permissions:
                    // 1. User is creator AND still has upload/edit/manage permissions on parent folder, OR
                    // 2. User has Edit or Manage permissions
                    let is_creator_with_upload = file.created_by == user_id && 
                        (user_permissions.contains(&DirectoryPermissionType::Upload) ||
                        user_permissions.contains(&DirectoryPermissionType::Edit) ||
                        user_permissions.contains(&DirectoryPermissionType::Manage));

                    let has_edit_permission = user_permissions.contains(&DirectoryPermissionType::Edit) ||
                                            user_permissions.contains(&DirectoryPermissionType::Manage);

                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());

                    if !is_owner && !is_creator_with_upload && !has_edit_permission {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to edit this file".to_string(),
                        });
                    }

        
                    // Handle name update separately since it requires path updates
                    if let Some(new_name) = payload.name {
                        if new_name != file.name {
                            match rename_file(file_id.clone(), new_name) {
                                Ok(_) => (),
                                Err(e) => return Err(DirectoryActionErrorInfo {
                                    code: 500,
                                    message: format!("Failed to rename file: {}", e),
                                })
                            }
                        }
                    }
        
                    // Update other metadata fields directly
                    file_uuid_to_metadata.with_mut(|map| {
                        if let Some(mut file) = map.get(&file_id) {
                            if let Some(labels) = payload.labels {
                                file.labels = labels;
                            }
                            if let Some(expires_at) = payload.expires_at {
                                file.expires_at = expires_at;
                            }
                            if let Some(upload_status) = payload.upload_status {
                                file.upload_status = upload_status;
                            }
                            file.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                            file.last_updated_by = user_id.clone();
                            
                            // Check external_payload size before creating
                            if let Some(ref external_payload) = payload.external_payload {
                                file.external_payload = Some(ExternalPayload(external_payload.clone()));
                            }

                            if payload.external_id.is_some() {
                                let new_external_id = Some(ExternalID(payload.external_id.unwrap_or("".to_string())));
                                update_external_id_mapping(
                                    file.external_id.clone(),
                                    new_external_id.clone(),
                                    Some(file.id.clone().to_string())
                                );
                                file.external_id = new_external_id;
                            }

                            // notes
                            if let Some(notes) = payload.notes {
                                file.notes = Some(notes);
                            }
                            
                            // Insert the modified record back into the map
                            map.insert(file_id.clone(), file);
                        }
                    });

        
                    // Get updated metadata to return
                    match get_file_by_id(file_id) {
                        Ok(updated_file) => {
                            let after_snap_file = DirectoryWebhookData::File(FileWebhookData {
                                file: Some(updated_file.clone()),
                            });
                            fire_directory_webhook(
                                WebhookEventLabel::FileUpdated,
                                webhooks_file,
                                Some(before_snap_file.clone()),
                                Some(after_snap_file.clone()),
                                Some("File updated".to_string()),
                            );
                            fire_directory_webhook(
                                WebhookEventLabel::SubfileUpdated,
                                webhooks_subfile,
                                Some(before_snap_file.clone()),
                                Some(after_snap_file),
                                Some("Subfile updated".to_string()),
                            );
                            let updated_file_fe = updated_file.cast_fe(&user_id).await;

                            Ok(DirectoryActionResult::UpdateFile(UpdateFileResponse {
                                file: updated_file_fe,
                                upload: None,
                                notes: Some("".to_string()),
                            }))
                        },
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to get updated file metadata: {}", e),
                        })
                    }
                },
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for UPDATE_FILE action".to_string(),
                })
            }
        },
        
        DirectoryActionEnum::UpdateFolder => {
            match action.payload {
                DirectoryActionPayload::UpdateFolder(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }

                    // Get folder ID from either resource_id or resource_path
                    let folder_id = payload.id;
        
                    // Get current folder metadata
                    let folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Folder not found: {}", e),
                        }),
                    };

                    // Get webhooks for both event types and combine them
                    let webhooks_folder = get_active_folder_webhooks(&folder_id, WebhookEventLabel::FolderUpdated);
                    let webhooks_subfolder = get_active_folder_webhooks(&folder_id, WebhookEventLabel::SubfolderUpdated);
                    let before_snap_folder = DirectoryWebhookData::Folder(FolderWebhookData {
                        folder: Some(folder.clone()),
                    });    

                    // Get parent folder permissions
                    let parent_resource_id = if let Some(parent_id) = folder.parent_folder_uuid.clone() {
                        DirectoryResourceID::Folder(parent_id)
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "Cannot edit root folder".to_string(),
                        });
                    };

                    let user_permissions = check_directory_permissions(
                        parent_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;

                    // Check permissions:
                    // 1. User is creator AND still has upload/edit/manage permissions on parent folder, OR
                    // 2. User has Edit or Manage permissions
                    let is_creator_with_upload = folder.created_by == user_id && 
                        (user_permissions.contains(&DirectoryPermissionType::Upload) ||
                        user_permissions.contains(&DirectoryPermissionType::Edit) ||
                        user_permissions.contains(&DirectoryPermissionType::Manage));

                    let has_edit_permission = user_permissions.contains(&DirectoryPermissionType::Edit) ||
                                            user_permissions.contains(&DirectoryPermissionType::Manage);

                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());

                    if !is_owner && !is_creator_with_upload && !has_edit_permission {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to edit this folder".to_string(),
                        });
                    }
        
                    // Handle name update separately since it requires path updates
                    if let Some(new_name) = payload.name {
                        if new_name != folder.name {
                            match rename_folder(folder_id.clone(), new_name) {
                                Ok(_) => (),
                                Err(e) => return Err(DirectoryActionErrorInfo {
                                    code: 500,
                                    message: format!("Failed to rename folder: {}", e),
                                })
                            }
                        }
                    }
        
                    // Update other metadata fields directly
                    folder_uuid_to_metadata.with_mut(|map| {
                        if let Some(mut folder) = map.get(&folder_id) {
                            if let Some(labels) = payload.labels {
                                folder.labels = labels;
                            }
                            if let Some(expires_at) = payload.expires_at {
                                folder.expires_at = expires_at;
                            }
                            folder.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                            folder.last_updated_by = user_id.clone();

                            // Check external_payload size before creating
                            if let Some(ref external_payload) = payload.external_payload {
                                folder.external_payload = Some(ExternalPayload(external_payload.clone()));
                            }
                            
                            if payload.external_id.is_some() {
                                let new_external_id = Some(ExternalID(payload.external_id.unwrap_or("".to_string())));
                                update_external_id_mapping(
                                    folder.external_id.clone(),
                                    new_external_id.clone(),
                                    Some(folder.id.clone().to_string())
                                );
                                folder.external_id = new_external_id;
                            }

                            // notes
                            if let Some(notes) = payload.notes {
                                folder.notes = Some(notes);
                            }
                            
                            // Insert the modified record back into the map
                            map.insert(folder_id.clone(), folder);
                        }
                    });
        
                    // Get updated metadata to return
                    match get_folder_by_id(folder_id.clone()) {
                        Ok(updated_folder) => {
                            let after_snap_folder = DirectoryWebhookData::Folder(FolderWebhookData {
                                folder: Some(updated_folder.clone()),
                            });    
                            fire_directory_webhook(
                                WebhookEventLabel::FolderUpdated,
                                webhooks_folder,
                                Some(before_snap_folder.clone()),
                                Some(after_snap_folder.clone()),
                                Some("Folder updated".to_string()),
                            );
                            fire_directory_webhook(
                                WebhookEventLabel::SubfolderUpdated,
                                webhooks_subfolder,
                                Some(before_snap_folder.clone()),
                                Some(after_snap_folder),
                                Some("Subfolder updated".to_string()),
                            );
                            Ok(DirectoryActionResult::UpdateFolder(updated_folder.cast_fe(&user_id).await))
                        },
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to get updated folder metadata: {}", e),
                        })
                    }
                },
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for UPDATE_FOLDER action".to_string(),
                })
            }
        },
        
        DirectoryActionEnum::DeleteFile => {
            match action.payload {
                DirectoryActionPayload::DeleteFile(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }

                    // Get file ID from either resource_id or resource_path
                    let file_id = payload.id;
        
                    // Get file metadata
                    let file = match get_file_by_id(file_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("File not found: {}", e),
                        }),
                    };
                    
                    // Get webhooks for both event types and combine them
                    let webhooks_file = get_active_file_webhooks(&file_id, WebhookEventLabel::FileDeleted);
                    let webhooks_subfile = get_active_file_webhooks(&file_id, WebhookEventLabel::SubfileDeleted);
        
                    let before_snap_file = DirectoryWebhookData::File(FileWebhookData {
                        file: Some(file.clone()),
                    });
        
                    // Get parent folder for permission check if user is creator
                    let parent_folder_id = file.parent_folder_uuid.clone();
                    let resource_id = DirectoryResourceID::Folder(parent_folder_id);
                    let user_permissions = check_directory_permissions(
                        resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;
        
                    // Check permissions:
                    // 1. User is creator AND still has upload permissions on parent folder, OR
                    // 2. User has Delete or Manage permissions
                    let is_creator_with_upload = file.created_by == user_id && 
                        (user_permissions.contains(&DirectoryPermissionType::Upload) ||
                         user_permissions.contains(&DirectoryPermissionType::Edit) ||
                         user_permissions.contains(&DirectoryPermissionType::Manage));
        
                    let has_delete_permission = user_permissions.contains(&DirectoryPermissionType::Delete) ||
                                              user_permissions.contains(&DirectoryPermissionType::Manage);
        
                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());

                    if !is_owner && !is_creator_with_upload && !has_delete_permission {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to delete this file".to_string(),
                        });
                    }
        
                    // Perform deletion
                    match delete_file(&file_id, payload.permanent) {
                        Ok(path_to_trash) => {
                            fire_directory_webhook(
                                WebhookEventLabel::FileDeleted,
                                webhooks_file,
                                Some(before_snap_file.clone()),
                                None,
                                Some("File deleted".to_string()),
                            );
                            fire_directory_webhook(
                                WebhookEventLabel::SubfileDeleted,
                                webhooks_subfile,
                                Some(before_snap_file.clone()),
                                None,
                                Some("Subfile deleted".to_string()),
                            );
                            Ok(DirectoryActionResult::DeleteFile(DeleteFileResponse {
                                file_id,
                                path_to_trash
                            })
                        )
                    },
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to delete file: {}", e),
                        })
                    }
                },
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for DELETE_FILE action".to_string(),
                })
            }
        },
        
        DirectoryActionEnum::DeleteFolder => {
            match action.payload {
                DirectoryActionPayload::DeleteFolder(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }

                    // Get folder ID from either resource_id or resource_path
                    let folder_id = payload.id;
        
                    // Get folder metadata
                    let folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Folder not found: {}", e),
                        }),
                    };

                    // Get webhooks for both event types and combine them
                    let webhooks_folder = get_active_folder_webhooks(&folder_id, WebhookEventLabel::FolderDeleted);
                    let webhooks_subfolder = get_active_folder_webhooks(&folder_id, WebhookEventLabel::SubfolderDeleted);
                    let before_snap_folder = DirectoryWebhookData::Folder(FolderWebhookData {
                        folder: Some(folder.clone()),
                    });    

        
                    // Get parent folder for permission check if user is creator
                    let parent_resource_id = if let Some(parent_id) = folder.parent_folder_uuid.clone() {
                        DirectoryResourceID::Folder(parent_id)
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "Cannot delete root folder".to_string(),
                        });
                    };
        
                    let user_permissions = check_directory_permissions(
                        parent_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;
        
                    // Check permissions:
                    // 1. User is creator AND still has upload permissions on parent folder, OR
                    // 2. User has Delete or Manage permissions
                    let is_creator_with_upload = folder.created_by == user_id && 
                        (user_permissions.contains(&DirectoryPermissionType::Upload) ||
                         user_permissions.contains(&DirectoryPermissionType::Edit) ||
                         user_permissions.contains(&DirectoryPermissionType::Manage));
        
                    let has_delete_permission = user_permissions.contains(&DirectoryPermissionType::Delete) ||
                                              user_permissions.contains(&DirectoryPermissionType::Manage);
                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());

                    if !is_owner && !is_creator_with_upload && !has_delete_permission {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to delete this folder".to_string(),
                        });
                    }
        
                    // Initialize vectors to collect deleted items
                    let mut deleted_files = Vec::with_capacity(2000);
                    let mut deleted_folders = Vec::with_capacity(2000);
        
                    // Perform deletion with collection vectors
                    match delete_folder(&folder_id, &mut deleted_folders, &mut deleted_files, payload.permanent) {
                        Ok(path_to_trash) => {

                            fire_directory_webhook(
                                WebhookEventLabel::FolderDeleted,
                                webhooks_folder,
                                Some(before_snap_folder.clone()),
                                None,
                                Some("Folder deleted".to_string()),
                            );
                            fire_directory_webhook(
                                WebhookEventLabel::SubfolderDeleted,
                                webhooks_subfolder,
                                Some(before_snap_folder.clone()),
                                None,
                                Some("Subfolder deleted".to_string()),
                            );

                            Ok(DirectoryActionResult::DeleteFolder(DeleteFolderResponse {
                                folder_id,
                                path_to_trash,
                                deleted_files: Some(deleted_files),
                                deleted_folders: Some(deleted_folders),
                            }))
                        },
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to delete folder: {}", e),
                        })
                    }
                },
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for DELETE_FOLDER action".to_string(),
                })
            }
        },
        
        DirectoryActionEnum::CopyFile => {
            match action.payload {
                DirectoryActionPayload::CopyFile(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }


                    // Get source file ID
                    let file_id = payload.id;
        
                    // Get source file metadata
                    let source_file = match get_file_by_id(file_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Source file not found: {}", e),
                        }),
                    };

                    // Get webhooks for both event types and combine them
                    let webhooks_file = get_active_file_webhooks(&FileID(WebhookAltIndexID::file_created_slug().to_string()), WebhookEventLabel::FileCreated);
                    let before_snap_file = DirectoryWebhookData::File(FileWebhookData {
                        file: Some(source_file.clone()),
                    });
        
                    // Check if user has View permission on source file
                    let source_resource_id = DirectoryResourceID::File(file_id.clone());
                    let user_permissions = check_directory_permissions(
                        source_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;
                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());
        
                    if !user_permissions.contains(&DirectoryPermissionType::View) && !is_owner {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to view this file".to_string(),
                        });
                    }

        
                    // Get destination folder metadata
                    let destination_folder = match get_destination_folder(
                        payload.destination_folder_id.clone(),
                        payload.destination_folder_path.clone(),
                        source_file.disk_id.clone(),
                        source_file.disk_type.clone(),
                        user_id.clone(),
                        source_file.drive_id.clone(),
                    ) {
                        Ok(folder) => folder,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Destination folder not found: {}", e),
                        }),
                    };
                    let webhooks_subfolder = get_active_folder_webhooks(&destination_folder.id, WebhookEventLabel::SubfileCreated);
        
                    // Check if user has Upload/Edit/Manage permission on destination folder
                    let dest_resource_id = DirectoryResourceID::Folder(destination_folder.id.clone());
                    let dest_permissions = check_directory_permissions(
                        dest_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;
        

                    if !is_owner && !dest_permissions.contains(&DirectoryPermissionType::Upload) &&
                       !dest_permissions.contains(&DirectoryPermissionType::Edit) &&
                       !dest_permissions.contains(&DirectoryPermissionType::Manage) {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to copy files to this folder".to_string(),
                        });
                    }
        
                    // Perform the copy operation
                    match copy_file(&file_id, &destination_folder, payload.file_conflict_resolution, payload.new_copy_id) {
                        Ok(file) => {
                            let after_snap_file = DirectoryWebhookData::File(FileWebhookData {
                                file: Some(file.clone()),
                            });
                            fire_directory_webhook(
                                WebhookEventLabel::FileCreated,
                                webhooks_file,
                                Some(before_snap_file.clone()),
                                Some(after_snap_file.clone()),
                                Some("Copy File".to_string()),
                            );
                            fire_directory_webhook(
                                WebhookEventLabel::SubfileCreated,
                                webhooks_subfolder,
                                Some(before_snap_file.clone()),
                                Some(after_snap_file),
                                Some("Copy File".to_string()),
                            );
                            Ok(DirectoryActionResult::CopyFile(file.cast_fe(&user_id).await))
                        },
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to copy file: {}", e),
                        }),
                    }
                },
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for COPY_FILE action".to_string(),
                }),
            }
        },
        
        DirectoryActionEnum::CopyFolder => {
            match action.payload {
                DirectoryActionPayload::CopyFolder(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }

                    // Get source folder ID
                    let folder_id = payload.id;
        
                    // Get source folder metadata
                    let source_folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Source folder not found: {}", e),
                        }),
                    };

                    // Get webhooks for both event types and combine them
                    let webhooks_folder = get_active_folder_webhooks(&&FolderID(WebhookAltIndexID::folder_created_slug().to_string()), WebhookEventLabel::FolderCreated);
                    let before_snap_folder = DirectoryWebhookData::Folder(FolderWebhookData {
                        folder: Some(source_folder.clone()),
                    });
        
                    // Check if user has View permission on source folder
                    let source_resource_id = DirectoryResourceID::Folder(folder_id.clone());
                    let user_permissions = check_directory_permissions(
                        source_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;
                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());
        
                    if !user_permissions.contains(&DirectoryPermissionType::View) && !is_owner {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to view this folder".to_string(),
                        });
                    }
        
                    // Get destination folder metadata
                    let destination_folder = match get_destination_folder(
                        payload.destination_folder_id.clone(),
                        payload.destination_folder_path.clone(),
                        source_folder.disk_id.clone(),
                        source_folder.disk_type.clone(),
                        user_id.clone(),
                        source_folder.drive_id.clone(),
                    ) {
                        Ok(folder) => folder,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Destination folder not found: {}", e),
                        }),
                    };
        
                    let webhooks_subfolder = get_active_folder_webhooks(&destination_folder.id, WebhookEventLabel::SubfolderCreated);
        
                    // Check if user has Upload/Edit/Manage permission on destination folder
                    let dest_resource_id = DirectoryResourceID::Folder(destination_folder.id.clone());
                    let dest_permissions = check_directory_permissions(
                        dest_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;

                    if !is_owner && !dest_permissions.contains(&DirectoryPermissionType::Upload) &&
                       !dest_permissions.contains(&DirectoryPermissionType::Edit) &&
                       !dest_permissions.contains(&DirectoryPermissionType::Manage) {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to copy folders to this location".to_string(),
                        });
                    }
        
                    // Perform the copy operation
                    match copy_folder(&folder_id, &destination_folder, payload.file_conflict_resolution, payload.new_copy_id) {
                        Ok(folder) => {
                            let after_snap_folder = DirectoryWebhookData::Folder(FolderWebhookData {
                                folder: Some(folder.clone()),
                            });
                            fire_directory_webhook(
                                WebhookEventLabel::FolderCreated,
                                webhooks_folder,
                                Some(before_snap_folder.clone()),
                                Some(after_snap_folder.clone()),
                                Some("Copy Folder".to_string()),
                            );
                            fire_directory_webhook(
                                WebhookEventLabel::SubfolderCreated,
                                webhooks_subfolder,
                                Some(before_snap_folder.clone()),
                                Some(after_snap_folder),
                                Some("Copy Folder".to_string()),
                            );
                            Ok(DirectoryActionResult::CopyFolder(folder.cast_fe(&user_id).await))
                        },
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to copy folder: {}", e),
                        }),
                    }
                },
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for COPY_FOLDER action".to_string(),
                }),
            }
        },

        DirectoryActionEnum::MoveFile => {
            match action.payload {
                DirectoryActionPayload::MoveFile(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }


                    // Get the file ID from either resource_id or resource_path
                    let file_id = payload.id;
        
                    // Get file metadata
                    let file = match get_file_by_id(file_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("File not found: {}", e),
                        }),
                    };


                    // Get webhooks for both event types and combine them
                    let webhooks_file = get_active_file_webhooks(&FileID(WebhookAltIndexID::file_created_slug().to_string()), WebhookEventLabel::FileCreated);
                    
                    let before_snap_file = DirectoryWebhookData::File(FileWebhookData {
                        file: Some(file.clone()),
                    });
        
                    // Check source file permissions
                    let source_resource_id = DirectoryResourceID::File(file_id.clone());
                    let source_permissions = check_directory_permissions(
                        source_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;
        
                    // Check if user has permission to move the file from source
                    let is_creator_with_upload = file.created_by == user_id && 
                        (source_permissions.contains(&DirectoryPermissionType::Upload) ||
                         source_permissions.contains(&DirectoryPermissionType::Edit) ||
                         source_permissions.contains(&DirectoryPermissionType::Manage));
        
                    let has_move_permission = source_permissions.contains(&DirectoryPermissionType::Edit) ||
                                            source_permissions.contains(&DirectoryPermissionType::Manage);
                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());

                    if !is_owner && !is_creator_with_upload && !has_move_permission {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to move this file from its current location".to_string(),
                        });
                    }
        
                    // Get destination folder
                    let destination_folder = match get_destination_folder(
                        payload.destination_folder_id,
                        payload.destination_folder_path,
                        file.disk_id,
                        file.disk_type,
                        user_id.clone(),
                        file.drive_id.clone()
                    ) {
                        Ok(folder) => folder,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Destination folder not found: {}", e),
                        }),
                    };
                    let webhooks_subfolder = get_active_folder_webhooks(&destination_folder.id, WebhookEventLabel::SubfileCreated);
        
        
                    // Check destination folder permissions
                    let dest_resource_id = DirectoryResourceID::Folder(destination_folder.id.clone());
                    let dest_permissions = check_directory_permissions(
                        dest_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;
        
                    if !dest_permissions.contains(&DirectoryPermissionType::Upload) && 
                       !dest_permissions.contains(&DirectoryPermissionType::Edit) &&
                       !dest_permissions.contains(&DirectoryPermissionType::Manage) && !is_owner {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to move files to the destination folder".to_string(),
                        });
                    }
        
                    match move_file(&file_id, &destination_folder, payload.file_conflict_resolution) {
                        Ok(file) => {
                            let after_snap_file = DirectoryWebhookData::File(FileWebhookData {
                                file: Some(file.clone()),
                            });
                            fire_directory_webhook(
                                WebhookEventLabel::FileCreated,
                                webhooks_file,
                                Some(before_snap_file.clone()),
                                Some(after_snap_file.clone()),
                                Some("Moved File".to_string()),
                            );
                            fire_directory_webhook(
                                WebhookEventLabel::SubfileCreated,
                                webhooks_subfolder,
                                Some(before_snap_file.clone()),
                                Some(after_snap_file),
                                Some("Moved File".to_string()),
                            );
                            Ok(DirectoryActionResult::MoveFile(file.cast_fe(&user_id).await))
                        },
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to move file: {}", e),
                        }),
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for MOVE_FILE action".to_string(),
                })
            }
        },
        
        DirectoryActionEnum::MoveFolder => {
            match action.payload {
                DirectoryActionPayload::MoveFolder(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }
                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());

                    // Get the folder ID from either resource_id or resource_path
                    let folder_id = payload.id;
        
                    // Get folder metadata
                    let folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Folder not found: {}", e),
                        }),
                    };
                    // Get webhooks for both event types and combine them
                    let webhooks_folder = get_active_folder_webhooks(&&FolderID(WebhookAltIndexID::folder_created_slug().to_string()), WebhookEventLabel::FolderCreated);
                    let before_snap_folder = DirectoryWebhookData::Folder(FolderWebhookData {
                        folder: Some(folder.clone()),
                    });
        
                    // Prevent moving root folder
                    if folder.parent_folder_uuid.is_none() {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "Cannot move root folder".to_string(),
                        });
                    }
        
                    // Check source folder permissions
                    let source_resource_id = DirectoryResourceID::Folder(folder_id.clone());
                    let source_permissions = check_directory_permissions(
                        source_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;
        
                    // Check if user has permission to move the folder from source
                    let is_creator_with_upload = folder.created_by == user_id && 
                        (source_permissions.contains(&DirectoryPermissionType::Upload) ||
                         source_permissions.contains(&DirectoryPermissionType::Edit) ||
                         source_permissions.contains(&DirectoryPermissionType::Manage));
        
                    let has_move_permission = source_permissions.contains(&DirectoryPermissionType::Edit) ||
                                            source_permissions.contains(&DirectoryPermissionType::Manage);
        
                    if !is_creator_with_upload && !has_move_permission && !is_owner {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to move this folder from its current location".to_string(),
                        });
                    }
        
                    // Get destination folder
                    let destination_folder = match get_destination_folder(
                        payload.destination_folder_id,
                        payload.destination_folder_path,
                        folder.disk_id,
                        folder.disk_type,
                        user_id.clone(),
                        folder.drive_id.clone()
                    ) {
                        Ok(folder) => folder,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Destination folder not found: {}", e),
                        }),
                    };
        
                    let webhooks_subfolder = get_active_folder_webhooks(&destination_folder.id, WebhookEventLabel::SubfolderCreated);
        
                    // Check destination folder permissions
                    let dest_resource_id = DirectoryResourceID::Folder(destination_folder.id.clone());
                    let dest_permissions = check_directory_permissions(
                        dest_resource_id,
                        PermissionGranteeID::User(user_id.clone())
                    ).await;

                    let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());
        
                    if !is_owner && !dest_permissions.contains(&DirectoryPermissionType::Upload) && 
                       !dest_permissions.contains(&DirectoryPermissionType::Edit) &&
                       !dest_permissions.contains(&DirectoryPermissionType::Manage) {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to move folders to the destination folder".to_string(),
                        });
                    }
        
                    match move_folder(&folder_id, &destination_folder, payload.file_conflict_resolution) {
                        Ok(folder) => {
                            let after_snap_folder = DirectoryWebhookData::Folder(FolderWebhookData {
                                folder: Some(folder.clone()),
                            });
                            fire_directory_webhook(
                                WebhookEventLabel::FolderCreated,
                                webhooks_folder,
                                Some(before_snap_folder.clone()),
                                Some(after_snap_folder.clone()),
                                Some("Move Folder".to_string()),
                            );
                            fire_directory_webhook(
                                WebhookEventLabel::SubfolderCreated,
                                webhooks_subfolder,
                                Some(before_snap_folder.clone()),
                                Some(after_snap_folder),
                                Some("Move Folder".to_string()),
                            );
                            Ok(DirectoryActionResult::MoveFolder(folder.cast_fe(&user_id).await))
                        },
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to move folder: {}", e),
                        }),
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for MOVE_FOLDER action".to_string(),
                })
            }
        },

        DirectoryActionEnum::RestoreTrash => {
            match action.payload {
                DirectoryActionPayload::RestoreTrash(payload) => {

                    // validate payload
                    if let Err(validation_error) = payload.validate_body() {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: format!("Validation error: {}", validation_error.message),
                        });
                    }

                    let resource_id = payload.clone().id;
        
                    // First check if it's a folder
                    let folder_id = if resource_id.to_string().starts_with(IDPrefix::Folder.as_str()) {
                        // Extract the ID portion by stripping the prefix
                        Some(FolderID(resource_id.to_string()))
                    } else {
                        None
                    };
                    let webhooks_restore_trash = get_active_folder_webhooks(&&FolderID(WebhookAltIndexID::restore_trash_slug().to_string()), WebhookEventLabel::DriveRestoreTrash);
        
                    if let Some(folder_id) = folder_id {
                        // Get folder metadata
                        let folder = folder_uuid_to_metadata
                            .get(&folder_id)
                            .ok_or_else(|| DirectoryActionErrorInfo {
                                code: 404,
                                message: "Folder not found".to_string(),
                            })?;

                        // Verify folder is actually in trash
                        if folder.restore_trash_prior_folder_uuid.is_none() {
                            return Err(DirectoryActionErrorInfo {
                                code: 400,
                                message: "Folder is not in trash".to_string(),
                            });
                        }
                        
                        let before_snap = DirectoryWebhookData::Folder(FolderWebhookData {
                            folder: Some(folder.clone()),
                        });
        
                        // Check permissions on the folder itself
                        let folder_resource_id = DirectoryResourceID::Folder(folder_id.clone());
                        let folder_permissions = check_directory_permissions(
                            folder_resource_id,
                            PermissionGranteeID::User(user_id.clone())
                        ).await;
        
                        // User needs Edit/Manage permission OR be creator with Upload permission to restore
                        let is_creator_with_upload = folder.created_by == user_id && 
                            folder_permissions.contains(&DirectoryPermissionType::Upload);
                        let has_restore_permission = folder_permissions.contains(&DirectoryPermissionType::Edit) ||
                                                  folder_permissions.contains(&DirectoryPermissionType::Manage);
        
                        let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());

                        if !is_owner && !is_creator_with_upload && !has_restore_permission {
                            return Err(DirectoryActionErrorInfo {
                                code: 403,
                                message: "You don't have permission to restore this folder".to_string(),
                            });
                        }
        
                        match restore_from_trash(&folder_id.to_string(), &payload) {
                            Ok(result) => {
                                // query the folder again to get the updated metadata
                                let after_snap = match get_folder_by_id(folder_id.clone()) {
                                    Ok(updated_folder) => {
                                        DirectoryWebhookData::Folder(FolderWebhookData {
                                            folder: Some(updated_folder.clone()),
                                        })
                                    },
                                    Err(e) => return Err(DirectoryActionErrorInfo {
                                        code: 500,
                                        message: format!("Failed to get updated folder metadata: {}", e),
                                    })
                                };
                                // fire the webhooks
                                fire_directory_webhook(
                                    WebhookEventLabel::DriveRestoreTrash,
                                    webhooks_restore_trash,
                                    Some(before_snap.clone()),
                                    Some(after_snap.clone()),
                                    Some("Folder restored from trash".to_string()),
                                );
                                Ok(result)
                            },
                            Err(e) => Err(DirectoryActionErrorInfo {
                                code: 500,
                                message: format!("Failed to restore folder from trash: {}", e),
                            })
                        }
                    } else {
                        // Try as a file
                        let file_id = if resource_id.to_string().starts_with(IDPrefix::File.as_str()) {
                            // Extract the ID portion by stripping the prefix
                            Some(FileID(resource_id.to_string()))
                        } else {
                            None
                        };

                        if let Some(file_id) = file_id {
                            // Get file metadata
                            let file = file_uuid_to_metadata
                                .get(&file_id)
                                .ok_or_else(|| DirectoryActionErrorInfo {
                                    code: 404,
                                    message: "File not found".to_string(),
                                })?;
            
                            // Verify file is actually in trash
                            if file.restore_trash_prior_folder_uuid.is_none() {
                                return Err(DirectoryActionErrorInfo {
                                    code: 400,
                                    message: "File is not in trash".to_string(),
                                });
                            }

                            let before_snap = DirectoryWebhookData::File(FileWebhookData {
                                file: Some(file.clone()),
                            });
            
                            // Check permissions on the file itself
                            let file_resource_id = DirectoryResourceID::File(file_id.clone());
                            let file_permissions = check_directory_permissions(
                                file_resource_id,
                                PermissionGranteeID::User(user_id.clone())
                            ).await;
            
                            // User needs Edit/Manage permission OR be creator with Upload permission to restore
                            let is_creator_with_upload = file.created_by == user_id && 
                                file_permissions.contains(&DirectoryPermissionType::Upload);
                            let has_restore_permission = file_permissions.contains(&DirectoryPermissionType::Edit) ||
                                                    file_permissions.contains(&DirectoryPermissionType::Manage);
                            let is_owner = OWNER_ID.with(|owner_id| user_id == *owner_id.borrow().get());

                            if !is_owner && !is_creator_with_upload && !has_restore_permission {
                                return Err(DirectoryActionErrorInfo {
                                    code: 403,
                                    message: "You don't have permission to restore this file".to_string(),
                                });
                            }
            
                            match restore_from_trash(&file_id.to_string(), &payload) {
                                Ok(result) => {
                                    // query the file again to get the updated metadata
                                    let after_snap = match get_file_by_id(file_id.clone()) {
                                        Ok(updated_file) => {
                                            DirectoryWebhookData::File(FileWebhookData {
                                                file: Some(updated_file.clone()),
                                            })
                                        },
                                        Err(e) => return Err(DirectoryActionErrorInfo {
                                            code: 500,
                                            message: format!("Failed to get updated file metadata: {}", e),
                                        })
                                    };
                                    // fire the webhooks
                                    fire_directory_webhook(
                                        WebhookEventLabel::DriveRestoreTrash,
                                        webhooks_restore_trash,
                                        Some(before_snap.clone()),
                                        Some(after_snap.clone()),
                                        Some("File restored from trash".to_string()),
                                    );
                                    Ok(result)
                                },
                                Err(e) => Err(DirectoryActionErrorInfo {
                                    code: 500,
                                    message: format!("Failed to restore file from trash: {}", e),
                                })
                            }
                        } else {
                            Err(DirectoryActionErrorInfo {
                                code: 400,
                                message: "Invalid resource ID".to_string(),
                            })
                        }
        
                        
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for RESTORE_TRASH action".to_string(),
                })
            }
        }
    }
}