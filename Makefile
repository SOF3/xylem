README.md: src/lib.rs
	grep '^//\!' src/lib.rs \
		| cut -c5- \
		| sed -e 's/```ignore/```rust/g' \
		> README.md
