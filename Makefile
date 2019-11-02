RELEASE_DIR=release

LINUX=x86_64-unknown-linux-gnu
WINDOWS=x86_64-pc-windows-gnu

all: linux windows

linux:
	cargo build --release --target ${LINUX}
	strip target/${LINUX}/release/2a-emulator
	mkdir -p ${RELEASE_DIR}
	cd ${RELEASE_DIR}
	zip -r9 ${RELEASE_DIR}/2a-emulator-${LINUX}.zip \
		target/${LINUX}/release/2a-emulator

windows:
	cargo build --release --target ${WINDOWS}
	strip target/${WINDOWS}/release/2a-emulator.exe
	mkdir -p ${RELEASE_DIR}
	cd ${RELEASE_DIR}
	zip -r9 ${RELEASE_DIR}/2a-emulator-${WINDOWS}.zip \
		target/${WINDOWS}/release/2a-emulator.exe
