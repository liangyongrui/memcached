//!

// #![deny(
//     missing_docs,
//     bare_trait_objects,
//     missing_copy_implementations,
//     single_use_lifetimes,
//     trivial_numeric_casts,
//     unreachable_pub,
//     unused_extern_crates,
//     unused_import_braces,
//     unused_qualifications,
//     unused_results,
//     variant_size_differences,
//     unsafe_code,
//     trivial_casts,
//     // missing_debug_implementations,
//     // 把所有warnings级别的改为deny
//     warnings,
//     clippy::all,
//     clippy::correctness,
//     clippy::restriction,
//     clippy::style,
//     clippy::complexity,
//     clippy::perf,
//     // clippy::cargo,
//     clippy::nursery ,
//     clippy::pedantic
// )]
// #![allow(
//     dead_code,
//     unused_imports,
//     clippy::missing_inline_in_public_items,
//     clippy::missing_errors_doc,
//     unused_variables,
//     clippy::module_name_repetitions,
//     clippy::missing_docs_in_private_items,
//     clippy::implicit_return,
//     clippy::wildcard_enum_match_arm,
//     clippy::as_conversions,
//     clippy::similar_names,
//     clippy::dbg_macro,
//     clippy::default_trait_access
// )]

#[macro_use]
extern crate anyhow;

mod client;
mod connection;
mod protocol;
mod stream;

pub use anyhow::Result;
pub use client::{connectable::Connectable, Client};
/// Create a memcached client instance and connect to memcached server.
pub async fn connect(url: &str) -> Result<Client> {
    Client::connect(url.to_owned())
}

#[cfg(test)]
mod tests {

    pub use anyhow::Result;
    #[async_std::test]
    async fn it_works() {
        let client = super::connect("memcache://127.0.0.1:12345").await.unwrap();
        let version = client.version().await.unwrap();
        client.set("123", "456", 100).await.unwrap();
        let get = client.get("123").await.unwrap();
        dbg!(version, get);
    }
}
