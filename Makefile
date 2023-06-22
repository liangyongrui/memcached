test:
	cargo fmt --check
	cargo clippy

benchmark:
	RUST_LOG=debug cargo test --test benchmark -- --nocapture
