fmt:
	cargo clippy --fix --allow-dirty

lint:
	cargo clippy

build:
	docker build . -t soulgarden/swup:http2-0.0.9 --platform linux/amd64
	docker push soulgarden/swup:http2-0.0.9
