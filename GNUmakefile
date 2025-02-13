.PHONY: clean all debug release pre-build post-debug, post-release, dev, pub

all: clean pre-build debug post-debug

pre-build:
    mkdir -p bin

debug:
    cargo build --jobs 3

release:
    cargo build --release --jobs 3

clean:
    cargo clean

post-debug:
	cp -rf target/debug/*.dll bin/
	cp -rf target/debug/quanta bin/

post-release:
	cp -rf target/release/*.dll bin/
	cp -rf target/release/quanta bin/

dev: debug post-debug
pub: release post-release
