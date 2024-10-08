// use std::{fs, path::PathBuf};

// // Returns the absolute path for the given relative path
// fn get_absolute_path(path: &str) -> PathBuf {
//     fs::canonicalize(path).expect("Failed to get absolute path")
// }

// // Function to return the absolute path for TEMP_DIR
// pub fn temp_dir() -> PathBuf {
//     get_absolute_path("file/temp_assets")
// }

use std::{
    fs,
    path::{Path, PathBuf},
};

// Returns the absolute path for the given relative path, creating the directory if it doesn't exist
fn get_absolute_path(path: &str) -> PathBuf {
    let path = Path::new(path);
    if !path.exists() {
        fs::create_dir_all(path).expect("Failed to create directory");
    }
    fs::canonicalize(path).expect("Failed to get absolute path")
}

// Function to return the absolute path for TEMP_DIR
pub fn temp_dir() -> PathBuf {
    get_absolute_path("../file/temp_assets")
}
// pref json
pub fn common_path() -> PathBuf {
    get_absolute_path("../common")
}
