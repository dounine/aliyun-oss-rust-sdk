use std::collections::HashMap;
use chrono::{DateTime, Utc};
use reqwest::header::HeaderMap;
use crate::debug;

#[derive(Debug)]
pub struct ObjectMetadata {
    pub(crate) metadata: HashMap<String, String>,
}

unsafe impl Send for ObjectMetadata {}

unsafe impl Sync for ObjectMetadata {}

impl ObjectMetadata {
    pub fn new(headers: &HeaderMap) -> ObjectMetadata {
        let mut metadata = HashMap::new();
        let mut user_metadata = HashMap::new();

        for (key, value) in headers.iter() {
            let key = key.as_str().to_string().to_lowercase();
            let value = value.to_str().unwrap().to_string();
            if key.starts_with("x-oss-meta-") {
                user_metadata.insert(key[11..].to_string(), value);
            } else if key == "etag" {
                let value = value.trim_matches('"').to_string();
                metadata.insert("etag".to_string(), value);
            } else {
                metadata.insert(key, value);
            }
        }

        ObjectMetadata {
            metadata,
        }
    }

    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        let val = self.metadata.get("last-modified");
        if val.is_none() {
            debug!("Can't find <last-modified>.");
            return None;
        }
        let result = chrono::DateTime::parse_from_rfc2822(val.unwrap()).map(|dt| dt.with_timezone(&chrono::Utc));
        return match result {
            Ok(date) => {
                Some(date)
            }
            Err(e) => {
                debug!("Last modified parsed failed.{}", e);
                None
            }
        };
    }

    pub fn expiration_time(&self) -> Option<DateTime<Utc>> {
        let val = self.metadata.get("x-oss-expiration");
        if val.is_none() {
            debug!("Can't find <x-oss-expiration>.");
            return None;
        }
        let result = chrono::DateTime::parse_from_rfc2822(val.unwrap()).map(|dt| dt.with_timezone(&Utc));
        return match result {
            Ok(date) => {
                Some(date)
            }
            Err(e) => {
                debug!("Expiration time parsed failed.{}", e);
                None
            }
        };
    }
    pub fn content_md5(&self) -> Option<String> {
        self.metadata.get("content-md5").map(|s| s.to_string())
    }
    pub fn etag(&self) -> Option<String> {
        self.metadata.get("etag").map(|s| s.to_string())
    }
    pub fn content_length(&self) -> Option<String> {
        self.metadata.get("content-length").map(|s| s.to_string())
    }
    pub fn content_type(&self) -> Option<String> {
        self.metadata.get("content-type").map(|s| s.to_string())
    }
    pub fn content_encoding(&self) -> Option<String> {
        self.metadata.get("content-encoding").map(|s| s.to_string())
    }
    pub fn content_disposition(&self) -> Option<String> {
        self.metadata.get("content-disposition").map(|s| s.to_string())
    }
    pub fn cache_control(&self) -> Option<String> {
        self.metadata.get("cache-control").map(|s| s.to_string())
    }
    pub fn crc64(&self) -> Option<String> {
        self.metadata.get("x-oss-hash-crc64ecma").map(|s| s.to_string())
    }
    pub fn server_side_encryption(&self) -> Option<String> {
        self.metadata.get("x-oss-server-side-encryption").map(|s| s.to_string())
    }
    pub fn object_type(&self) -> Option<String> {
        self.metadata.get("x-oss-object-type").map(|s| s.to_string())
    }
}