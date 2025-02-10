.PHONY: clean all build pre-build post-build
all: clean pre-build build post-build

pre-build:
    mkdir -p bin

build:
    cargo build --jobs 3

clean:
    cargo clean

post-build:
    cp -f target/debug/*.so bin
    cp -f target/debug/quanta bin