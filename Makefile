default: build_web

WEB_OUT=out/lib

dev_flags=
ifdef DEV
	dev_flags=--all-features
endif

build_web: clean_web
	cargo build --target wasm32-unknown-unknown -p valor-web --release ${dev_flags}
	@mkdir -p out/lib
	wasm-bindgen target/wasm32-unknown-unknown/release/valor_web.wasm \
		--target no-modules \
		--no-typescript --out-name valor --out-dir ${WEB_OUT}
	@echo 'wasm_bindgen();' >> ${WEB_OUT}/valor.js

clean_web:
	@rm -rf ${WEB_OUT}

