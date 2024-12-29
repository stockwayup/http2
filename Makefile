fmt:
	cargo fmt --all

lint:
	cargo clippy --fix --allow-dirty --allow-staged

build:
	docker build . -t soulgarden/swup:http2-0.1.0 --platform linux/amd64
	docker push soulgarden/swup:http2-0.1.0
