FLAGS=--release
MODE=release
ifdef DEV
	FLAGS=--all-features
	MODE=debug
endif

OUT_DIR=out
PLUGINS=hello_plugin with_state
WASM=target/wasm32-unknown-unknown/$(MODE)/%.wasm
OUT_WASM=$(OUT_DIR)/%.js $(OUT_DIR)/plugins/%.js
NATIVE=target/$(MODE)/%
NATIVE_PLUGINS=target/$(MODE)/lib%.so

default: clean native web

native: runtime_native plugins_native

web: runtime_web plugins_web

$(OUT_DIR)/valor: $(OUT_DIR)/valor_bin ; @mv $< $@
runtime_native: $(OUT_DIR)/valor

$(OUT_DIR)/valor.js: $(OUT_DIR)/valor_web.js
	@mv $< $@; mv $(<D)/valor_*.wasm $(@D)/valor.wasm
runtime_web: $(OUT_DIR)/valor.js
	@echo "init('valor.wasm');" >> $<
	@cp $(basename $(<F))_web/*.{js,html} $(OUT_DIR)

plugins_native: clean_plugins $(PLUGINS:%=$(OUT_DIR)/plugins/%)

plugins_web: clean_plugins $(PLUGINS:%=$(OUT_DIR)/plugins/%.js)

$(OUT_DIR)/%: $(NATIVE)
	@mkdir -p $(@D); cp $^ $@

$(OUT_DIR)/plugins/%: $(NATIVE_PLUGINS)
	@mkdir -p $(@D); cp $^ $@

$(OUT_DIR)/%.js $(OUT_DIR)/plugins/%.js: $(WASM)
	wasm-bindgen $^ --target web --weak-refs --no-typescript --out-name $* --out-dir $(@D)

$(WASM):
	cargo build --target wasm32-unknown-unknown -p $* ${FLAGS}

$(NATIVE) $(NATIVE_PLUGINS):
	cargo build -p $* ${FLAGS}

clean:
	@rm -rf $(OUT_DIR)
	cargo clean

clean_plugins: ; @rm -rf $(OUT_DIR)/plugins

