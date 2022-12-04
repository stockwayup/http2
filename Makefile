fmt:
	cargo fmt

build:
	docker build . -t soulgarden/swup:http2-0.0.5 --platform linux/amd64
	docker push soulgarden/swup:http2-0.0.5
