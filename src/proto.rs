#![allow(clippy::all)]

pub mod data;
pub mod import;
pub mod scraping;
pub mod uuid;

mod google;

pub mod ext {
    use std::convert::TryFrom;

    impl super::uuid::Uuid {
        pub fn new() -> Self {
            let uuid = uuid::Uuid::new_v4();
            let mut buf = vec![];
            buf.extend(uuid.as_bytes());
            super::uuid::Uuid { uuid: buf }
        }

        pub fn as_slice(&self) -> &[u8] {
            &self.uuid
        }
    }

    impl TryFrom<&[u8]> for super::uuid::Uuid {
        type Error = String;

        fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
            const BYTES_LEN: usize = 16;

            if value.len() != BYTES_LEN && !value.is_empty() {
                return Err("uuid slice has wrong size".to_owned());
            }

            Ok(super::uuid::Uuid {
                uuid: Vec::from(value)
            })
        }
    }
}
