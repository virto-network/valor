default: build_web

WEB_OUT=out

dev_flags=
ifdef DEV
	dev_flags=--all-features
endif

build_web: clean_web
	cargo build --target wasm32-unknown-unknown -p valor-web --release ${dev_flags}
	@mkdir -p out/lib
	wasm-bindgen target/wasm32-unknown-unknown/release/valor_web.wasm \
		--target no-modules --weak-refs \
		--no-typescript --out-name valor --out-dir ${WEB_OUT}/lib
	@echo 'wasm_bindgen();' >> ${WEB_OUT}/lib/valor.js
	@cp valor-web/sw.js  ${WEB_OUT}
	@cp valor-web/example.html  ${WEB_OUT}/index.html
clean_web:
	@rm -rf ${WEB_OUT}

