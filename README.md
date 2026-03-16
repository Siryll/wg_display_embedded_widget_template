# wg_display_embedded_widget_template
Template repo for the wg_display_embedded widgets to make creation of own widgets easy

## Size-optimization
- Strips custom sections from the core wasm with `wasm-tools strip`.
- optionally runs `wasm-opt -Oz --enable-bulk-memory --enable-sign-ext` (Binaryen) before component conversion.

### Workflow knobs

In `.github/workflows/build_release.yml`:

- `USE_WASM_OPT`: `"1"` to run `wasm-opt -Oz`, `"0"` to skip it.
- `MAX_PRECOMPILED_SIZE_BYTES`: size budget for `widget.precompiled.wasm`. Due to hardware restrictions the max size of a widget binary can be 500kb right now, this might change in the future.
