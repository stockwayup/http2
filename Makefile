fmt:
	cargo fmt --all

lint:
	cargo clippy --fix --allow-dirty

build:
	docker build . -t soulgarden/swup:http2-0.0.15 --platform linux/amd64
	docker push soulgarden/swup:http2-0.0.15
