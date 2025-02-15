.PHONY: clean all debug pre-build post-debug, post-release, dev, pub

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
    xcopy /Y target\debug\*.dll bin
    copy /Y target\debug\quanta.exe bin

post-release:
    xcopy /Y target\release\*.dll bin
    copy /Y target\release\quanta.exe bin

dev: debug post-debug
pub: release post-release
