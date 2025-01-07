.PHONY: clean all build pre-build post-build
all: clean pre-build build post-build

pre-build:
    mkdir -p bin

build:
    cargo build

clean:
    cargo clean

post-build:
    cp -f target/debug/deps/*.so bin
    cp -f target/debug/deps/quanta bin