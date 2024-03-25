use std::collections::HashMap;
use chrono::{DateTime, ParseError, Utc};
use reqwest::header::HeaderMap;
use crate::debug;

pub struct ObjectMetadata {
    pub(crate) metadata: HashMap<String, String>,
    pub(crate) user_metadata: HashMap<String, String>,
}

impl ObjectMetadata {
    pub fn new(headers: &HeaderMap) -> ObjectMetadata {
        let mut metadata = HashMap::new();
        let mut user_metadata = HashMap::new();

        for (key, value) in headers.iter() {
            let key = key.as_str().to_string();
            let value = value.to_str().unwrap().to_string();
            if key.starts_with("x-oss-meta-") {
                user_metadata.insert(key[11..].to_string(), value);
            } else if key == "ETag" {
                metadata.insert("ETag".to_string(), value.trim_matches('"').to_string());
            } else {
                metadata.insert(key, value);
            }
        }

        ObjectMetadata {
            metadata,
            user_metadata,
        }
    }

    pub fn get_last_modified(&self) -> Option<DateTime<Utc>> {
        let val = self.metadata.get("Last-Modified");
        if val.is_none() {
            debug!("Can't find <Last-Modified>.");
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

    pub fn get_expiration_time(&self) -> Option<DateTime<Utc>> {
        let val = self.metadata.get("x-oss-expiration");
        if val.is_none() {
            debug!("Can't find <x-oss-expiration>.");
            return None;
        }
        let result = chrono::DateTime::parse_from_rfc2822(val.unwrap()).map(|dt| dt.with_timezone(&chrono::Utc));
        return match result {
            Ok(date) => {
                Some(date)
            }
            Err(e) => {
                debug!("Expiration time parsed failed.{}", e);
            }
        };
    }
    pub fn set_expiration_time(&mut self, time: DateTime<Utc>) {
        self.metadata.insert("x-oss-expiration".to_string(), format!("{}", time));
    }

    pub fn get_content_md5(&self) -> Option<String> {
        self.metadata.get("Content-MD5").map(|s| s.to_string())
    }
    pub fn set_content_md5<S: AsRef<str>>(&mut self, md5: S) {
        self.metadata.insert("Content-MD5".to_string(), md5);
    }

    pub fn get_etag(&self) -> Option<String> {
        self.metadata.get("ETag").map(|s| s.to_string())
    }
    pub fn set_etag<S: AsRef<str>>(&mut self, etag: S) {
        self.metadata.insert("ETag".to_string(), etag);
    }

    pub fn get_content_length(&self) -> Option<String> {
        self.metadata.get("Content-Length").map(|s| s.to_string())
    }
    pub fn set_content_length(&mut self, length: u64) {
        self.metadata.insert("Content-Length".to_string(), length.to_string());
    }

    pub fn get_content_type(&self) -> Option<String> {
        self.metadata.get("Content-Type").map(|s| s.to_string())
    }
    pub fn set_content_type<S: AsRef<str>>(&mut self, content_type: S) {
        if content_type.as_ref().is_empty() {
            return;
        }
        self.metadata.insert("Content-Type".to_string(), content_type);
    }

    pub fn get_content_encoding(&self) -> Option<String> {
        self.metadata.get("Content-Encoding").map(|s| s.to_string())
    }
    pub fn set_content_encoding<S: AsRef<str>>(&mut self, content_encoding: S) {
        self.metadata.insert("Content-Encoding".to_string(), content_encoding);
    }

    pub fn get_content_disposition(&self) -> Option<String> {
        self.metadata.get("Content-Disposition").map(|s| s.to_string())
    }
    pub fn set_content_disposition<S: AsRef<str>>(&mut self, content_disposition: S) {
        self.metadata.insert("Content-Disposition".to_string(), content_disposition);
    }

    pub fn get_cache_control(&self) -> Option<String> {
        self.metadata.get("Cache-Control").map(|s| s.to_string())
    }
    pub fn set_cache_control<S: AsRef<str>>(&mut self, cache_control: S) {
        self.metadata.insert("Cache-Control".to_string(), cache_control);
    }

    pub fn get_crc64(&self) -> Option<String> {
        self.metadata.get("x-oss-hash-crc64ecma").map(|s| s.to_string())
    }
    pub fn set_crc64<S: AsRef<str>>(&mut self, crc64: S) {
        self.metadata.insert("x-oss-hash-crc64ecma".to_string(), crc64);
    }

    pub fn server_side_encryption(&self) -> Option<String> {
        self.metadata.get("x-oss-server-side-encryption").map(|s| s.to_string())
    }
    pub fn set_server_side_encryption<S: AsRef<str>>(&mut self, server_side_encryption: S) {
        self.metadata.insert("x-oss-server-side-encryption".to_string(), server_side_encryption);
    }

    pub fn object_type(&self) -> Option<String> {
        self.metadata.get("x-oss-object-type").map(|s| s.to_string())
    }
}