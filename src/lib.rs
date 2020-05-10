//!

#![deny(
    // missing_docs,
    bare_trait_objects,
    missing_copy_implementations,
    single_use_lifetimes,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results,
    variant_size_differences,
    unsafe_code,
    trivial_casts,
    // missing_debug_implementations,
//     // 把所有warnings级别的改为deny
    warnings,
    clippy::all,
    clippy::correctness,
    clippy::restriction,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::cargo,
    // clippy::nursery ,
    clippy::pedantic
)]
#![allow(
    unused_imports,
    clippy::missing_inline_in_public_items,
    clippy::missing_errors_doc,
    unused_variables,
    clippy::module_name_repetitions,
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::wildcard_enum_match_arm,
    clippy::as_conversions,
    clippy::similar_names,
    clippy::dbg_macro,
    clippy::default_trait_access,
    //
    clippy::pub_enum_variant_names,
    clippy::multiple_crate_versions,
    clippy::doc_markdown,
    clippy::result_unwrap_used, //
    clippy::option_unwrap_used, //
    clippy::cast_possible_truncation, //
    clippy::integer_arithmetic, //
    clippy::get_unwrap, //
)]

// #[macro_use]
// extern crate anyhow;

mod client;
mod connection;
pub mod error;
mod protocol;
mod stream;

pub type Result<T> = std::result::Result<T, error::MemcachedError>;
pub use client::{connectable::Connectable, Client};

/// Create a memcached client instance and connect to memcached server.
/// 默认连接池个数为1
///
/// Example
///
/// ```rust
/// let client = memcached::connect("memcache://127.0.0.1:12345").unwrap();
/// ```
pub fn connect(url: &str) -> Result<Client> {
    Client::connect(url)
}

/// 创建一个client，可以指定多个url，连接池大小，key hash连接池的函数
///
/// Example
///
/// ```rust
/// let client = memcached::Client::connects_with(vec!["memcache://127.0.0.1:12345".to_owned()], 2, |s|1).unwrap();
/// ```
pub fn connects_withconnects_with(
    urls: Vec<String>,
    pool_size: u64,
    hash_function: fn(&str) -> usize,
) -> Result<Client> {
    Client::connects_with(urls, pool_size, hash_function)
}