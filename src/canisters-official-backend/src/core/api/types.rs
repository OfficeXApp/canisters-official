#[derive(Debug)]
pub enum DirectoryError {
    FolderNotFound(String),
    // Add other error types as needed
}

#[derive(Debug)]
pub enum DirectoryIDError {
    InvalidPrefix,
    MalformedID,
    UnknownType,
}