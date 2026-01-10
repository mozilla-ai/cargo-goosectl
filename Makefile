.PHONY: generate-schemas
generate-schemas:
	cargo run \
		--bin generate-json-schemas \
		--all-features
