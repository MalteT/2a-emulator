RELEASE_DIR=release

CURRENT := $(shell clang -dumpmachine)

current:
	cargo build --release

package-only:
	rm -rf ./packages
	mkdir packages
	zip -9jr packages/2a-emulator-${CURRENT}.zip ./target/release/2a-emulator

package: current package-only


.PHONY: package
