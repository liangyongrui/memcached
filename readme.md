<h1 align="center">Memcached</h1>
<div align="center">
 <strong>
   Async memcached client built on Rust and <a href="https://github.com/async-rs/async-std">Async-std</a>
 </strong>
</div>
<br />
<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/memcached">
    <img src="https://img.shields.io/crates/v/memcached.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/memcached">
    <img src="https://img.shields.io/crates/d/memcached.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/memcached">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <a href="https://docs.rs/memcached">
  <!-- ci -->
    <img src="https://github.com/liangyongrui/memcached/workflows/Rust/badge.svg"
      alt="ci" />
  </a>
</div>

<br/>

## Features

This project is still under development. The following features with the check marks are supported.

If you are concerned about an unimplemented feature, please tell me and I will finish writing it ASAP.

- [x] Client Supported Method
  - [x] add
  - [x] append
  - [x] cas
  - [x] decrement
  - [x] delete
  - [x] flush
  - [x] flush_with_delay
  - [x] get
  - [x] gets
  - [x] increment
  - [x] prepend
  - [x] replace
  - [x] set
  - [x] stats
  - [x] touch
  - [x] version
- [x] Supported protocols
  - [x] Binary protocol
  - [ ] ASCII protocol
- [x] All memcached supported connections
  - [x] TCP connection
  - [ ] [TLS connection](https://crates.io/crates/memcachd)
  - [ ] UDP connection
  - [ ] UNIX Domain socket connection
- [x] Encodings
  - [x] Support [Serde](https://github.com/serde-rs/serde)
- [x] Memcached cluster support with custom key hash algorithm

## Basic usage

Your Cargo.toml could look like this:

```toml
[dependencies]
async-std = { version = "1", features = ["attributes"] }
memcached = "*"
```

And then the code:

```rust
let client = memcached::connect("memcache://127.0.0.1:12345")?;
client.set("abc", "hello", 100).await?;
let t: Option<String> = client.get("abc").await?;
assert_eq!(t, Some("hello".to_owned()));
```

For more usage, see [doc](https://docs.rs/memcached), each method of client has example.

## FAQ

### Should I use this in production?

Better not.

This project needs a lot of details to complete. But if you want, you can try it.

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions
