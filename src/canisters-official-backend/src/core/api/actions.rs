// src/core/api/actions.rs
use std::result::Result;

use crate::rest::directory::types::{DirectoryAction, DirectoryActionEnum, DirectoryActionPayload, DirectoryActionResult};


#[derive(Debug, Clone)]
pub struct DirectoryActionErrorInfo {
    pub code: i32,
    pub message: String,
}

pub fn pipe_action(action: DirectoryAction) -> Result<DirectoryActionResult, DirectoryActionErrorInfo> {
    match action.action {
        DirectoryActionEnum::GetFile => {
            match action.payload {
                DirectoryActionPayload::GetFile(_payload) => {
                    // Implementation for getting file
                    todo!("Implement get file")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
                    message: "Invalid payload for GET_FILE action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::GetFolder => {
            match action.payload {
                DirectoryActionPayload::GetFolder(_payload) => {
                    // Implementation for getting folder
                    todo!("Implement get folder")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
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
                    // Implementation for updating file
                    todo!("Implement update file")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
                    message: "Invalid payload for UPDATE_FILE action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::UpdateFolder => {
            match action.payload {
                DirectoryActionPayload::UpdateFolder(payload) => {
                    // Implementation for updating folder
                    todo!("Implement update folder")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
                    message: "Invalid payload for UPDATE_FOLDER action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::DeleteFile => {
            match action.payload {
                DirectoryActionPayload::DeleteFile(payload) => {
                    // Implementation for deleting file
                    todo!("Implement delete file")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
                    message: "Invalid payload for DELETE_FILE action".to_string()
                })
            }
        }
        
        DirectoryActionEnum::DeleteFolder => {
            match action.payload {
                DirectoryActionPayload::DeleteFolder(payload) => {
                    // Implementation for deleting folder
                    todo!("Implement delete folder")
                }
                _ => Err(DirectoryActionErrorInfo {
                    code: 500,
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