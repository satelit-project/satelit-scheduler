#![allow(clippy::all)]

pub mod data;
pub mod import;
pub mod scraping;
pub mod uuid;

// MARK: uuid::Uuid

pub mod ext {
    use std::{convert::TryFrom, fmt};

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
                uuid: Vec::from(value),
            })
        }
    }

    impl fmt::Display for super::uuid::Uuid {
        #[allow(clippy::needless_range_loop)]
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            const HEX: [u8; 16] = [
                b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd',
                b'e', b'f',
            ];
            const BYTE_POS: [usize; 6] = [0, 4, 6, 8, 10, 16];
            const HYPHEN_POS: [usize; 4] = [8, 13, 18, 23];

            let mut buf = [0u8; 36];
            let bytes = self.uuid.as_slice();
            for group in 0..5 {
                for idx in BYTE_POS[group]..BYTE_POS[group + 1] {
                    let b = bytes[idx];
                    let out_idx = group + 2 * idx;
                    buf[out_idx] = HEX[(b >> 4) as usize];
                    buf[out_idx + 1] = HEX[(b & 0b1111) as usize];
                }

                if group != 4 {
                    buf[HYPHEN_POS[group]] = b'-';
                }
            }

            match std::str::from_utf8_mut(&mut buf) {
                Ok(hex) => hex.fmt(f),
                Err(_) => Err(fmt::Error),
            }
        }
    }
}
