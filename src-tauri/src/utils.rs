use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;

pub fn get_data_path() -> PathBuf {
    let dirs = ProjectDirs::from("com", "InteliAgente", "AppInteliAgente").unwrap();
    PathBuf::from(dirs.data_local_dir())
}

pub fn get_rec_path() -> PathBuf {
    let mut path = get_data_path();
    path.push("recordings");
    path
}