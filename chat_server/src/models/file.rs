use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
    pub ws_id: u64,
    pub ext: String,
    pub hash: String,
}

#[allow(unused)]
impl ChatFile {
    pub fn new(ws_id: u64, filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ws_id,
            ext: filename.split('.').last().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn path(&self, base_url: &Path) -> PathBuf {
        base_url.join(self.hash_to_path())
    }

    pub fn url(&self) -> String {
        format!("/files/{}", self.hash_to_path())
    }

    pub fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}/{}.{}", self.ws_id, part1, part2, part3, self.ext)
    }
}

impl FromStr for ChatFile {
    type Err = AppError;

    // convert /files/s/339/807/e635afbeab088ce33206fdf4223a6bb156.png to ChatFile
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix("/files/") else {
            return Err(AppError::ChatFileError(format!(
                "Invalid chat file path: {}",
                s
            )));
        };

        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 4 {
            return Err(AppError::ChatFileError(format!(
                "File path {} does not valid",
                s
            )));
        }

        let Ok(ws_id) = parts[0].parse::<u64>() else {
            return Err(AppError::ChatFileError(format!(
                "Invalid workspace id: {}",
                parts[1]
            )));
        };

        let Some((part3, ext)) = parts[3].split_once('.') else {
            return Err(AppError::ChatFileError(format!(
                "Invalid file name: {}",
                parts[3]
            )));
        };

        let hash = format!("{}{}{}", parts[1], parts[2], part3);
        Ok(Self {
            ws_id,
            ext: ext.to_string(),
            hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_path_should_work() {
        let cf = ChatFile {
            ws_id: 1,
            ext: "txt".to_string(),
            hash: "1234567890abcdef".to_string(),
        };
        let path = PathBuf::from("/tmp");
        assert_eq!(
            cf.path(&path),
            PathBuf::from("/tmp/1/123/456/7890abcdef.txt")
        );
    }
    #[test]
    fn file_url_should_work() {
        let cf = ChatFile {
            ws_id: 1,
            ext: "txt".to_string(),
            hash: "1234567890abcdef".to_string(),
        };
        assert_eq!(cf.url(), "/files/1/123/456/7890abcdef.txt");
    }
}
