RELEASE_DIR=release

CURRENT := $(shell clang -dumpmachine)

current:
	cargo build --release
	rm -rf ./packages
	mkdir packages
	zip -9jr packages/2a-emulator-${CURRENT}.zip ./target/release/2a-emulator
