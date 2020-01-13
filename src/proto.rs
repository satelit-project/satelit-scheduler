#![allow(clippy::all)]

pub mod data;
pub mod import;
pub mod scraping;
pub mod uuid;

mod google;

pub mod ext {
    use std::{
        convert::TryFrom,
        fmt::{self, Display, Formatter},
    };

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

    impl Display for super::uuid::Uuid {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            const BYTES_IDX: [usize; 6] = [0, 4, 6, 8, 10, 16];
            const HYPHEN_IDX: [usize; 4] = [8, 13, 18, 23];
            const ALPHABET: [u8; 16] = [
                b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd',
                b'e', b'f',
            ];

            let bytes = self.uuid.as_slice();
            let mut buffer = [0; 36];
            for group in 0..5 {
                for i in BYTES_IDX[group]..BYTES_IDX[group + 1] {
                    let byte = bytes[i];
                    let iout = group + 2 * i;
                    buffer[iout] = ALPHABET[(byte >> 4) as usize];
                    buffer[iout + 1] = ALPHABET[(byte & 0b1111) as usize];
                }

                if group != 4 {
                    buffer[HYPHEN_IDX[group]] = b'-';
                }
            }

            match std::str::from_utf8_mut(&mut buffer) {
                Ok(formatted) => write!(f, "{}", formatted),
                Err(_) => write!(f, "{}", "non-uuid"),
            }
        }
    }
}
