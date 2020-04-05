extern crate openssl;  // fix linkage on musl
#[macro_use]
extern crate diesel;

pub mod db;
pub mod plan;
pub mod proto;
pub mod settings;
