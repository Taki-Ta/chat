use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFile {
    pub ext: String,
    pub hash: String,
}

#[allow(unused)]
impl ChatFile {
    pub fn new(filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ext: filename.split('.').last().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn path(&self, base_url: &Path) -> PathBuf {
        base_url.join(self.hash_to_path())
    }

    pub fn url(&self, ws_id: i64) -> String {
        format!("{ws_id}/{}", self.hash_to_path())
    }

    pub fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}.{}", part1, part2, part3, self.ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_path_should_work() {
        let cf = ChatFile {
            ext: "txt".to_string(),
            hash: "1234567890abcdef".to_string(),
        };
        let path = PathBuf::from("/tmp");
        assert_eq!(cf.path(&path), PathBuf::from("/tmp/123/456/7890abcdef.txt"));
    }
    #[test]
    fn file_url_should_work() {
        let cf = ChatFile {
            ext: "txt".to_string(),
            hash: "1234567890abcdef".to_string(),
        };
        assert_eq!(cf.url(1), "1/123/456/7890abcdef.txt");
    }
}
