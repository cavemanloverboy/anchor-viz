.PHONY: build
build:
	cargo build --release

.PHONY: linux-mac
linux-mac:
	cargo build --release
	cp target/release/anchor-viz /usr/local/bin/anchorviz