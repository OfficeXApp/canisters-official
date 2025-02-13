// src/core/api/actions.rs
use std::result::Result;

use crate::{core::{state::{directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata}, types::{DriveFullFilePath, FileUUID, FolderUUID, PathTranslationResponse}}, permissions::types::{DirectoryGranteeID, DirectoryPermissionType}}, types::{ICPPrincipalString, PublicKeyICP, UserID}}, debug_log, rest::directory::types::{CreateFileResponse, DeleteFileResponse, DeleteFolderResponse, DirectoryAction, DirectoryActionEnum, DirectoryActionPayload, DirectoryActionResult, DirectoryResourceID}};

use super::{drive::drive::{copy_file, copy_folder, create_file, create_folder, delete_file, delete_folder, get_file_by_id, get_folder_by_id, move_file, move_folder, rename_file, rename_folder, restore_from_trash}, internals::drive_internals::{check_directory_permissions, get_destination_folder, translate_path_to_id}};


#[derive(Debug, Clone)]
pub struct DirectoryActionErrorInfo {
    pub code: i32,
    pub message: String,
}

pub fn pipe_action(action: DirectoryAction, user_id: UserID) -> Result<DirectoryActionResult, DirectoryActionErrorInfo> {
    match action.action {
        DirectoryActionEnum::GetFile => {
            match action.payload {
                DirectoryActionPayload::GetFile(_) => {
                    // First try to get file_id either from resource_id or resource_path
                    let file_id = if let Some(id) = action.target.resource_id {
                        match id {
                            DirectoryResourceID::File(file_id) => file_id,
                            DirectoryResourceID::Folder(_) => return Err(DirectoryActionErrorInfo {
                                code: 400,
                                message: "Expected file ID but got folder ID".to_string(),
                            }),
                        }
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.file {
                            Some(file) => file.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "File not found at specified path".to_string(),
                            }),
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400, 
                            message: "Neither resource_id nor resource_path provided".to_string(),
                        });
                    };
        
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
                        resource_id,
                        DirectoryGranteeID::User(user_id.clone())
                    );
        
                    // User needs at least View permission to get file details
                    if !user_permissions.contains(&DirectoryPermissionType::View) {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to view this file".to_string(), 
                        });
                    }
        
                    // If we get here, user is authorized - return the file metadata
                    Ok(DirectoryActionResult::GetFile(file))
                },
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for GET_FILE action".to_string(),
                }),
            }
        }
        
        DirectoryActionEnum::GetFolder => {
            match action.payload {
                DirectoryActionPayload::GetFolder(_) => {
                    // Get folder_id from either resource_id or resource_path
                    let folder_id = if let Some(id) = action.target.resource_id {
                        match id {
                            DirectoryResourceID::Folder(folder_id) => folder_id,
                            DirectoryResourceID::File(_) => return Err(DirectoryActionErrorInfo {
                                code: 400,
                                message: "Expected folder ID but got file ID".to_string(),
                            }),
                        }
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.folder {
                            Some(folder) => folder.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Folder not found at specified path".to_string(),
                            }),
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided".to_string(),
                        });
                    };
        
                    // Get folder metadata
                    let folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Folder not found: {}", e),
                        }),
                    };
        
                    // Check if user has View permission on the folder
                    let resource_id = DirectoryResourceID::Folder(folder_id.clone());
                    let user_permissions = check_directory_permissions(
                        resource_id,
                        DirectoryGranteeID::User(user_id.clone())
                    );
        
                    if !user_permissions.contains(&DirectoryPermissionType::View) {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to view this folder".to_string(),
                        });
                    }
        
                    Ok(DirectoryActionResult::GetFolder(folder))
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
                    // Get parent folder ID where the file will be created
                    let parent_folder_id = if let Some(id) = action.target.resource_id {
                        match &id {
                            DirectoryResourceID::Folder(folder_id) => folder_id.clone(),
                            DirectoryResourceID::File(_) => return Err(DirectoryActionErrorInfo {
                                code: 400,
                                message: "Expected folder ID but got file ID for parent".to_string(),
                            }),
                        }
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.folder {
                            Some(folder) => folder.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Parent folder not found at specified path".to_string(),
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided for parent folder".to_string(),
                        });
                    };
        
                    // Check if user has Upload, Edit, or Manage permission on the parent folder
                    let parent_resource_id = DirectoryResourceID::Folder(parent_folder_id.clone());
                    let user_permissions = check_directory_permissions(
                        parent_resource_id,
                        DirectoryGranteeID::User(user_id.clone())
                    );
        
                    if !user_permissions.contains(&DirectoryPermissionType::Upload) && 
                       !user_permissions.contains(&DirectoryPermissionType::Edit) &&
                       !user_permissions.contains(&DirectoryPermissionType::Manage) {
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
                    let full_file_path = format!("{}{}", parent_folder.full_folder_path.0, payload.name);
        
                    // Create file using the drive API
                    match create_file(
                        full_file_path,
                        payload.disk_id,
                        user_id.clone(),
                        payload.file_size,
                        payload.expires_at.unwrap_or(-1),
                        String::new(), // Empty canister ID to use current canister
                        payload.file_conflict_resolution,
                        Some(payload.has_sovereign_permissions.unwrap_or(false))
                    ) {
                        Ok((file_metadata, upload_response)) => {
                            Ok(DirectoryActionResult::CreateFile(CreateFileResponse {
                                file: file_metadata,
                                upload: upload_response,
                                notes: "File created successfully".to_string(),
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
                    // Get parent folder ID where the new folder will be created
                    let parent_folder_id = if let Some(id) = action.target.resource_id {
                        match &id {
                            DirectoryResourceID::Folder(folder_id) => folder_id.clone(),
                            DirectoryResourceID::File(_) => return Err(DirectoryActionErrorInfo {
                                code: 400,
                                message: "Expected folder ID but got file ID for parent".to_string(),
                            }),
                        }
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.folder {
                            Some(folder) => folder.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Parent folder not found at specified path".to_string(),
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided for parent folder".to_string(),
                        });
                    };
        
                    // Check if user has Upload, Edit, or Manage permission on the parent folder
                    let parent_resource_id = DirectoryResourceID::Folder(parent_folder_id.clone());
                    let user_permissions = check_directory_permissions(
                        parent_resource_id,
                        DirectoryGranteeID::User(user_id.clone())
                    );
        
                    if !user_permissions.contains(&DirectoryPermissionType::Upload) && 
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
                    let full_folder_path = DriveFullFilePath(format!("{}{}/", parent_folder.full_folder_path.0, payload.name));
        
                    // Create folder using the drive API
                    match create_folder(
                        full_folder_path,
                        payload.disk_id,
                        user_id.clone(),
                        payload.expires_at.unwrap_or(-1),
                        String::new(), // Empty canister ID to use current canister
                        payload.file_conflict_resolution,
                        Some(payload.has_sovereign_permissions.unwrap_or(false))
                    ) {
                        Ok(folder) => Ok(DirectoryActionResult::CreateFolder(folder)),
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
                    // Get file ID from either resource_id or resource_path
                    let file_id = if let Some(id) = action.target.resource_id {
                        match &id {
                            DirectoryResourceID::File(file_id) => file_id.clone(),
                            DirectoryResourceID::Folder(_) => return Err(DirectoryActionErrorInfo {
                                code: 400,
                                message: "Expected file ID but got folder ID".to_string(),
                            }),
                        }
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.file {
                            Some(file) => file.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "File not found at specified path".to_string(),
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided".to_string(),
                        });
                    };
        
                    // Get current file metadata
                    let file = match get_file_by_id(file_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("File not found: {}", e),
                        }),
                    };
        
                    // Get parent folder permissions
                    let parent_folder_id = file.folder_uuid.clone();
                    let parent_resource_id = DirectoryResourceID::Folder(parent_folder_id);
                    let user_permissions = check_directory_permissions(
                        parent_resource_id,
                        DirectoryGranteeID::User(user_id.clone())
                    );

                    // Check permissions:
                    // 1. User is creator AND still has upload/edit/manage permissions on parent folder, OR
                    // 2. User has Edit or Manage permissions
                    let is_creator_with_upload = file.created_by == user_id && 
                        (user_permissions.contains(&DirectoryPermissionType::Upload) ||
                        user_permissions.contains(&DirectoryPermissionType::Edit) ||
                        user_permissions.contains(&DirectoryPermissionType::Manage));

                    let has_edit_permission = user_permissions.contains(&DirectoryPermissionType::Edit) ||
                                            user_permissions.contains(&DirectoryPermissionType::Manage);

                    if !is_creator_with_upload && !has_edit_permission {
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
                        if let Some(file) = map.get_mut(&file_id) {
                            if let Some(tags) = payload.tags {
                                file.tags = tags;
                            }
                            if let Some(raw_url) = payload.raw_url {
                                file.raw_url = raw_url;
                            }
                            if let Some(expires_at) = payload.expires_at {
                                file.expires_at = expires_at;
                            }
                            file.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                            file.last_updated_by = user_id.clone();
                        }
                    });
        
                    // Get updated metadata to return
                    match get_file_by_id(file_id) {
                        Ok(updated_file) => Ok(DirectoryActionResult::UpdateFile(updated_file)),
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
                    // Get folder ID from either resource_id or resource_path
                    let folder_id = if let Some(id) = action.target.resource_id {
                        match &id {
                            DirectoryResourceID::Folder(folder_id) => folder_id.clone(),
                            DirectoryResourceID::File(_) => return Err(DirectoryActionErrorInfo {
                                code: 400,
                                message: "Expected folder ID but got file ID".to_string(),
                            }),
                        }
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.folder {
                            Some(folder) => folder.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Folder not found at specified path".to_string(),
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided".to_string(),
                        });
                    };
        
                    // Get current folder metadata
                    let folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Folder not found: {}", e),
                        }),
                    };
        
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
                        DirectoryGranteeID::User(user_id.clone())
                    );

                    // Check permissions:
                    // 1. User is creator AND still has upload/edit/manage permissions on parent folder, OR
                    // 2. User has Edit or Manage permissions
                    let is_creator_with_upload = folder.created_by == user_id && 
                        (user_permissions.contains(&DirectoryPermissionType::Upload) ||
                        user_permissions.contains(&DirectoryPermissionType::Edit) ||
                        user_permissions.contains(&DirectoryPermissionType::Manage));

                    let has_edit_permission = user_permissions.contains(&DirectoryPermissionType::Edit) ||
                                            user_permissions.contains(&DirectoryPermissionType::Manage);

                    if !is_creator_with_upload && !has_edit_permission {
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
                        if let Some(folder) = map.get_mut(&folder_id) {
                            if let Some(tags) = payload.tags {
                                folder.tags = tags;
                            }
                            if let Some(expires_at) = payload.expires_at {
                                folder.expires_at = expires_at;
                            }
                            folder.last_updated_date_ms = ic_cdk::api::time() / 1_000_000;
                            folder.last_updated_by = user_id.clone();
                        }
                    });
        
                    // Get updated metadata to return
                    match get_folder_by_id(folder_id) {
                        Ok(updated_folder) => Ok(DirectoryActionResult::UpdateFolder(updated_folder)),
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
                    // Get file ID from either resource_id or resource_path
                    let file_id = if let Some(id) = action.target.resource_id {
                        match &id {
                            DirectoryResourceID::File(file_id) => file_id.clone(),
                            DirectoryResourceID::Folder(_) => return Err(DirectoryActionErrorInfo {
                                code: 400,
                                message: "Expected file ID but got folder ID".to_string(),
                            }),
                        }
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.file {
                            Some(file) => file.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "File not found at specified path".to_string(),
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided".to_string(),
                        });
                    };
        
                    // Get file metadata
                    let file = match get_file_by_id(file_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("File not found: {}", e),
                        }),
                    };
        
                    // Get parent folder for permission check if user is creator
                    let parent_folder_id = file.folder_uuid.clone();
                    let resource_id = DirectoryResourceID::Folder(parent_folder_id);
                    let user_permissions = check_directory_permissions(
                        resource_id,
                        DirectoryGranteeID::User(user_id.clone())
                    );
        
                    // Check permissions:
                    // 1. User is creator AND still has upload permissions on parent folder, OR
                    // 2. User has Delete or Manage permissions
                    let is_creator_with_upload = file.created_by == user_id && 
                        (user_permissions.contains(&DirectoryPermissionType::Upload) ||
                         user_permissions.contains(&DirectoryPermissionType::Edit) ||
                         user_permissions.contains(&DirectoryPermissionType::Manage));
        
                    let has_delete_permission = user_permissions.contains(&DirectoryPermissionType::Delete) ||
                                              user_permissions.contains(&DirectoryPermissionType::Manage);
        
                    if !is_creator_with_upload && !has_delete_permission {
                        return Err(DirectoryActionErrorInfo {
                            code: 403,
                            message: "You don't have permission to delete this file".to_string(),
                        });
                    }
        
                    // Perform deletion
                    match delete_file(&file_id, payload.permanent) {
                        Ok(path_to_trash) => Ok(DirectoryActionResult::DeleteFile(DeleteFileResponse {
                            file_id,
                            path_to_trash
                        })),
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
                    // Get folder ID from either resource_id or resource_path
                    let folder_id = if let Some(id) = action.target.resource_id {
                        match &id {
                            DirectoryResourceID::Folder(folder_id) => folder_id.clone(),
                            DirectoryResourceID::File(_) => return Err(DirectoryActionErrorInfo {
                                code: 400,
                                message: "Expected folder ID but got file ID".to_string(),
                            }),
                        }
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.folder {
                            Some(folder) => folder.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Folder not found at specified path".to_string(),
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided".to_string(),
                        });
                    };
        
                    // Get folder metadata
                    let folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Folder not found: {}", e),
                        }),
                    };
        
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
                        DirectoryGranteeID::User(user_id.clone())
                    );
        
                    // Check permissions:
                    // 1. User is creator AND still has upload permissions on parent folder, OR
                    // 2. User has Delete or Manage permissions
                    let is_creator_with_upload = folder.created_by == user_id && 
                        (user_permissions.contains(&DirectoryPermissionType::Upload) ||
                         user_permissions.contains(&DirectoryPermissionType::Edit) ||
                         user_permissions.contains(&DirectoryPermissionType::Manage));
        
                    let has_delete_permission = user_permissions.contains(&DirectoryPermissionType::Delete) ||
                                              user_permissions.contains(&DirectoryPermissionType::Manage);
        
                    if !is_creator_with_upload && !has_delete_permission {
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
                        Ok(path_to_trash) => Ok(DirectoryActionResult::DeleteFolder(DeleteFolderResponse {
                            folder_id,
                            path_to_trash,
                            deleted_files: Some(deleted_files),
                            deleted_folders: Some(deleted_folders),
                        })),
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
                    // Get the file ID from either resource_id or resource_path
                    let file_id = if let Some(id) = action.target.resource_id {
                        FileUUID(id.to_string())
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.file {
                            Some(file) => file.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Source file not found at specified path".to_string()
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided for source file".to_string()
                        });
                    };

                    // get the file from directory state
                    let preexisting_file = file_uuid_to_metadata.get(&file_id).unwrap();
        
                    // Get destination folder
                    let destination_folder = match get_destination_folder(
                        payload.destination_folder_id,
                        payload.destination_folder_path,
                        preexisting_file.disk_id,
                        user_id,
                        preexisting_file.canister_id.to_string()
                    ) {
                        Ok(folder) => folder,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Destination folder not found: {}", e)
                        })
                    };
        
                    match copy_file(&file_id, &destination_folder, payload.file_conflict_resolution) {
                        Ok(file) => Ok(DirectoryActionResult::CopyFile(file)),
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to copy file: {}", e)
                        })
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for COPY_FILE action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::CopyFolder => {
            match action.payload {
                DirectoryActionPayload::CopyFolder(payload) => {
                    // Get the folder ID from either resource_id or resource_path
                    let folder_id = if let Some(id) = action.target.resource_id {
                        FolderUUID(id.to_string())
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.folder {
                            Some(folder) => folder.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Source folder not found at specified path".to_string()
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided for source folder".to_string()
                        });
                    };

                    let preexisting_folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Folder not found: {}", e)
                        })
                    };
        
                    // Get destination folder
                    let destination_folder = match get_destination_folder(
                        payload.destination_folder_id,
                        payload.destination_folder_path,
                        preexisting_folder.disk_id,
                        user_id,
                        preexisting_folder.canister_id.to_string()
                    ) {
                        Ok(folder) => folder,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Destination folder not found: {}", e)
                        })
                    };
        
                    match copy_folder(&folder_id, &destination_folder, payload.file_conflict_resolution) {
                        Ok(folder) => Ok(DirectoryActionResult::CopyFolder(folder)),
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to copy folder: {}", e)
                        })
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for COPY_FOLDER action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::MoveFile => {
            match action.payload {
                DirectoryActionPayload::MoveFile(payload) => {
                    // Get the file ID from either resource_id or resource_path
                    let file_id = if let Some(id) = action.target.resource_id {
                        FileUUID(id.to_string())
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.file {
                            Some(file) => file.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Source file not found at specified path".to_string()
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided for source file".to_string()
                        });
                    };

                    let preexisting_file = file_uuid_to_metadata.get(&file_id).unwrap();
        
                    // Get destination folder
                    let destination_folder = match get_destination_folder(
                        payload.destination_folder_id,
                        payload.destination_folder_path,
                        preexisting_file.disk_id,
                        user_id,
                        preexisting_file.canister_id.to_string()
                    ) {
                        Ok(folder) => folder,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Destination folder not found: {}", e)
                        })
                    };
        
                    match move_file(&file_id, &destination_folder, payload.file_conflict_resolution) {
                        Ok(file) => Ok(DirectoryActionResult::MoveFile(file)),
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to move file: {}", e)
                        })
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for MOVE_FILE action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::MoveFolder => {
            match action.payload {
                DirectoryActionPayload::MoveFolder(payload) => {
                    // Get the folder ID from either resource_id or resource_path
                    let folder_id = if let Some(id) = action.target.resource_id {
                        FolderUUID(id.to_string())
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.folder {
                            Some(folder) => folder.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Source folder not found at specified path".to_string()
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided for source folder".to_string()
                        });
                    };

                    // get the folder metadata
                    let preexisting_folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Folder not found: {}", e)
                        })
                    };
        
                    // Get destination folder
                    let destination_folder = match get_destination_folder(
                        payload.destination_folder_id,
                        payload.destination_folder_path,
                        preexisting_folder.disk_id,
                        user_id,
                        preexisting_folder.canister_id.to_string()
                    ) {
                        Ok(folder) => folder,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Destination folder not found: {}", e)
                        })
                    };
        
                    match move_folder(&folder_id, &destination_folder, payload.file_conflict_resolution) {
                        Ok(folder) => Ok(DirectoryActionResult::MoveFolder(folder)),
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to move folder: {}", e)
                        })
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for MOVE_FOLDER action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::RestoreTrash => {
            match action.payload {
                DirectoryActionPayload::RestoreTrash(payload) => {
                    let resource_id = action.target.resource_id.ok_or_else(|| DirectoryActionErrorInfo {
                        code: 400,
                        message: "Resource ID is required for restore operation".to_string()
                    })?;
        
                    match restore_from_trash(&resource_id.to_string(), &payload) {
                        Ok(result) => Ok(result),
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to restore from trash: {}", e)
                        })
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for RESTORE_TRASH action".to_string()
                })
            }
        }
    }
}