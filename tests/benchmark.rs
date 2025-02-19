//! Test speed of main operations.
//! Run: cargo test --test benchmark -- --nocapture

use memcached::Client;
use std::time::Instant;

const LOOPS: u64 = 100;

fn connect() -> Client {
    env_logger::try_init().expect("Could not initialize logging");
    memcached::connect("memcache://127.0.0.1:12345").expect("Could not connect to memcached")
}

#[async_std::test]
async fn test_single_read() {
    let cache_key: &str = "test_single_read";
    let client = connect();

    client
        .set(cache_key, 1, 0)
        .await
        .expect("Error setting cache value");

    let start = Instant::now();

    for _n in 1..=LOOPS {
        let _value: u64 = client
            .get(cache_key)
            .await
            .expect("Error reading u64")
            .expect("Empty value read from cache");
    }

    let dur = start.elapsed().as_millis() as f64 / LOOPS as f64;
    println!("Single key read takes {dur} ms avg.");
}
