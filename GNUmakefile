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
	for file in target/debug/lib*.so; do \
		if [ -f "$$file" ]; then \
			newname=$$(basename "$$file" | sed 's/^lib//'); \
			cp -f "$$file" "bin/$$newname"; \
		fi \
	done
	cp -rf target/debug/quanta bin/

post-release:
	for file in target/release/lib*.so; do \
		if [ -f "$$file" ]; then \
			newname=$$(basename "$$file" | sed 's/^lib//'); \
			cp -f "$$file" "bin/$$newname"; \
		fi \
	done
	cp -rf target/release/quanta bin/

dev: debug post-debug
pub: release post-release
