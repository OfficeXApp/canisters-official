// src/core/api/actions.rs
use std::result::Result;

use crate::{core::state::directory::{state::state::{file_uuid_to_metadata, folder_uuid_to_metadata}, types::{FileUUID, FolderUUID}}, rest::directory::types::{DeleteFileResponse, DeleteFolderResponse, DirectoryAction, DirectoryActionEnum, DirectoryActionPayload, DirectoryActionResult}};

use super::{drive::drive::{delete_file, delete_folder, get_file_by_id, get_folder_by_id, rename_file, rename_folder}, internals::drive_internals::translate_path_to_id};


#[derive(Debug, Clone)]
pub struct DirectoryActionErrorInfo {
    pub code: i32,
    pub message: String,
}

pub fn pipe_action(action: DirectoryAction) -> Result<DirectoryActionResult, DirectoryActionErrorInfo> {
    match action.action {
        DirectoryActionEnum::GetFile => {
            match action.payload {
                DirectoryActionPayload::GetFile(_) => {
                    // First try resource_id
                    if let Some(id) = action.target.resource_id {
                        match get_file_by_id(FileUUID(id)) {
                            Ok(file) => Ok(DirectoryActionResult::GetFile(file)),
                            Err(e) => Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: format!("File not found by ID: {}", e)
                            })
                        }
                    }
                    // Then try resource_path
                    else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.file {
                            Some(file) => Ok(DirectoryActionResult::GetFile(file)),
                            None => Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "File not found at specified path".to_string()
                            })
                        }
                    } else {
                        Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided".to_string()
                        })
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for GET_FILE action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::GetFolder => {
            match action.payload {
                DirectoryActionPayload::GetFolder(_) => {
                    // First try resource_id
                    if let Some(id) = action.target.resource_id {
                        match get_folder_by_id(FolderUUID(id)) {
                            Ok(folder) => Ok(DirectoryActionResult::GetFolder(folder)),
                            Err(e) => Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: format!("Folder not found by ID: {}", e)
                            })
                        }
                    }
                    // Then try resource_path
                    else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.folder {
                            Some(folder) => Ok(DirectoryActionResult::GetFolder(folder)),
                            None => Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Folder not found at specified path".to_string()
                            })
                        }
                    } else {
                        Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided".to_string()
                        })
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for GET_FOLDER action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::CreateFile => {
            match action.payload {
                DirectoryActionPayload::CreateFile(payload) => {
                    // Implementation for creating file
                    todo!("Implement create file")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
                    message: "Invalid payload for CREATE_FILE action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::CreateFolder => {
            match action.payload {
                DirectoryActionPayload::CreateFolder(payload) => {
                    // Implementation for creating folder
                    todo!("Implement create folder")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
                    message: "Invalid payload for CREATE_FOLDER action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::UpdateFile => {
            match action.payload {
                DirectoryActionPayload::UpdateFile(payload) => {
                    // Get the file ID from either resource_id or resource_path
                    let file_id = if let Some(id) = action.target.resource_id {
                        FileUUID(id)
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.file {
                            Some(file) => file.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "File not found at specified path".to_string()
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided".to_string()
                        });
                    };
        
                    // Get current file metadata
                    let file = match get_file_by_id(file_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("File not found: {}", e)
                        })
                    };
        
                    // Handle name update separately since it requires path updates
                    if let Some(new_name) = payload.name {
                        if new_name != file.name {
                            match rename_file(file_id.clone(), new_name) {
                                Ok(_) => (),
                                Err(e) => return Err(DirectoryActionErrorInfo {
                                    code: 500,
                                    message: format!("Failed to rename file: {}", e)
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
                            file.last_changed_unix_ms = ic_cdk::api::time() / 1_000_000;
                        }
                    });
        
                    // Get updated metadata to return
                    match get_file_by_id(file_id) {
                        Ok(updated_file) => Ok(DirectoryActionResult::UpdateFile(updated_file)),
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to get updated file metadata: {}", e)
                        })
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for UPDATE_FILE action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::UpdateFolder => {
            match action.payload {
                DirectoryActionPayload::UpdateFolder(payload) => {
                    // Get the folder ID from either resource_id or resource_path
                    let folder_id = if let Some(id) = action.target.resource_id {
                        FolderUUID(id)
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.folder {
                            Some(folder) => folder.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Folder not found at specified path".to_string()
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided".to_string()
                        });
                    };
        
                    // Get current folder metadata
                    let folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Folder not found: {}", e)
                        })
                    };
        
                    // Handle name update separately since it requires path updates
                    if let Some(new_name) = payload.name {
                        if new_name != folder.name {
                            match rename_folder(folder_id.clone(), new_name) {
                                Ok(_) => (),
                                Err(e) => return Err(DirectoryActionErrorInfo {
                                    code: 500,
                                    message: format!("Failed to rename folder: {}", e)
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
                            folder.last_changed_unix_ms = ic_cdk::api::time() / 1_000_000;
                        }
                    });
        
                    // Get updated metadata to return
                    match get_folder_by_id(folder_id) {
                        Ok(updated_folder) => Ok(DirectoryActionResult::UpdateFolder(updated_folder)),
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to get updated folder metadata: {}", e)
                        })
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for UPDATE_FOLDER action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::DeleteFile => {
            match action.payload {
                DirectoryActionPayload::DeleteFile(payload) => {
                    // Get the file first to ensure it exists and get its metadata
                    let file_id = if let Some(id) = action.target.resource_id {
                        FileUUID(id)
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.file {
                            Some(file) => file.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "File not found at specified path".to_string()
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided".to_string()
                        });
                    };

                    // Get file metadata before deletion
                    let file = match get_file_by_id(file_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("File not found: {}", e)
                        })
                    };

                    // Perform deletion
                    match delete_file(&file_id) {
                        Ok(_) => Ok(DirectoryActionResult::DeleteFile(DeleteFileResponse {
                            file_id,
                            full_path: file.full_file_path
                        })),
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to delete file: {}", e)
                        })
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for DELETE_FILE action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::DeleteFolder => {
            match action.payload {
                DirectoryActionPayload::DeleteFolder(payload) => {
                    // Get the folder first to ensure it exists and get its metadata
                    let folder_id = if let Some(id) = action.target.resource_id {
                        FolderUUID(id)
                    } else if let Some(path) = action.target.resource_path {
                        let translation = translate_path_to_id(path);
                        match translation.folder {
                            Some(folder) => folder.id,
                            None => return Err(DirectoryActionErrorInfo {
                                code: 404,
                                message: "Folder not found at specified path".to_string()
                            })
                        }
                    } else {
                        return Err(DirectoryActionErrorInfo {
                            code: 400,
                            message: "Neither resource_id nor resource_path provided".to_string()
                        });
                    };

                    // Get folder metadata before deletion
                    let folder = match get_folder_by_id(folder_id.clone()) {
                        Ok(f) => f,
                        Err(e) => return Err(DirectoryActionErrorInfo {
                            code: 404,
                            message: format!("Folder not found: {}", e)
                        })
                    };

                    // Initialize vectors to collect deleted items
                    let mut deleted_files = Vec::with_capacity(2000);
                    let mut deleted_folders = Vec::with_capacity(2000);

                    // Perform deletion with collection vectors
                    match delete_folder(&folder_id, &mut deleted_folders, &mut deleted_files) {
                        Ok(()) => Ok(DirectoryActionResult::DeleteFolder(DeleteFolderResponse {
                            folder_id,
                            full_path: folder.full_folder_path,
                            deleted_files: Some(deleted_files),
                            deleted_folders: Some(deleted_folders),
                        })),
                        Err(e) => Err(DirectoryActionErrorInfo {
                            code: 500,
                            message: format!("Failed to delete folder: {}", e)
                        })
                    }
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 400,
                    message: "Invalid payload for DELETE_FOLDER action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::CopyFile => {
            match action.payload {
                DirectoryActionPayload::CopyFile(payload) => {
                    // Implementation for copying file
                    todo!("Implement copy file")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
                    message: "Invalid payload for COPY_FILE action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::CopyFolder => {
            match action.payload {
                DirectoryActionPayload::CopyFolder(payload) => {
                    // Implementation for copying folder
                    todo!("Implement copy folder")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
                    message: "Invalid payload for COPY_FOLDER action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::MoveFile => {
            match action.payload {
                DirectoryActionPayload::MoveFile(payload) => {
                    // Implementation for moving file
                    todo!("Implement move file")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
                    message: "Invalid payload for MOVE_FILE action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::MoveFolder => {
            match action.payload {
                DirectoryActionPayload::MoveFolder(payload) => {
                    // Implementation for moving folder
                    todo!("Implement move folder")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
                    message: "Invalid payload for MOVE_FOLDER action".to_string()
                })
            }
        }
    }
}