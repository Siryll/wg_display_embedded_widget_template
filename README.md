# Widget Template Development

Template for creating Widgets for [WG-Display-Embedded-multi](https://github.com/Siryll/wg_display_embedded_multi), included with an automatic build pipeline to create the final widget binary.

> [!NOTE]
> The template for the legacy [WG-Display-Embedded](https://github.com/Siryll/wg_display_embedded) project was moved to the [legacy](https://github.com/Siryll/wg_display_embedded_widget_template/tree/legacy) branch.

Widgets are Rust crates compiled to `wasm32-unknown-unknown` WebAssembly and packaged as WASM components using the [WIT Component Model](https://component-model.bytecodealliance.org/). They implement a standard interface (defined in WIT) and are installed and managed through the Web UI without needing to reflash the device.

---

## Quick Start

Create a repo based on this template, or clone it:

```bash
git clone https://github.com/Siryll/wg_display_embedded_widget_template my-widget
cd my-widget
```

The template includes the correct `Cargo.toml`, a working `lib.rs` example and the release workflow.

<!-- For reference implementations see:
- [`wg_display_embedded_widget_time`](https://github.com/Siryll/wg_display_embedded_widget_time) — small time widget
- [`wg_display_embedded_widget_aareguru`](https://github.com/Siryll/wg_display_embedded_widget_aareguru) — widget to display the current Aare temperature
- [`wg_display_embedded_widget_public_transport`](https://github.com/Siryll/wg_display_embedded_widget_public_transport) — configurable widget to show public transport connections -->

---

## Install on device

1. Push your widget to GitHub
2. Create a new Tag (needs to start with 'v', e.g. v0.0.1), the Pipeline will automatically build and put the `widget.precompiled.wasm` binary into the release.
3. In the Web UI of the embedded WG-Display, click **Install from URL** and enter the direct download URL

---

## WIT Interface

### Exported functions (widget implements these)

```wit
package widget:widget;

export get-name: func() -> string;
export get-version: func() -> string;
export get-config-schema: func() -> string;
export get-run-update-cycle-seconds: func() -> u32;
export run: func(context: widget-context) -> widget-result;
```

| Export | Return | Description |
|---|---|---|
| `get-name` | `string` | Display name shown in the Web UI and on screen |
| `get-version` | `string` | Semver string (e.g. `"0.1.0"`) |
| `get-config-schema` | `string` | JSON Schema for the configuration form. Return `"{}"` if no config is needed |
| `get-run-update-cycle-seconds` | `u32` | How often the widget should be invoked (seconds) |
| `run` | `widget-result` | Main entry point — fetch data, compute output, return string |

### Types (`types.wit`)

```wit
record datetime {
    seconds: u64,
    nanoseconds: u32,
}

record widget-context {
    last-invocation: datetime,   // UTC time of previous run (0 on first run)
    config: string,              // JSON matching your config schema
}

record widget-result {
    data: string,                // Text displayed on screen (or error message)
}
```

### Imported host functions (the device provides these)

#### `http`

```wit
interface http {
    type status = u16;

    variant method { get, head, post, put, delete }

    record response {
        status: status,
        content-length: option<u64>,
        bytes: list<u8>,
    }

    request: func(method: method, url: string, body: option<list<u8>>) -> result<response>;
}
```

**Limitations:**
- Timeout: **30 seconds**
- No streaming — entire response is buffered

#### `clocks`

```wit
interface clocks {
    now: func() -> datetime;
}
```

Returns the current UTC time. `seconds` is a Unix timestamp; `nanoseconds` is the sub-second part. Returns `{0, 0}` if time has not been synced yet.

#### `logging`

```wit
interface logging {
    enum level { debug, info, warn, error }
    log: func(level: level, context: string, message: string);
}
```

Output appears in the `defmt` serial monitor.

#### `random`

```wit
interface random {
    get-random: func() -> u64;
}
```

Returns a 64-bit random value from the ESP32 hardware RNG. Not cryptographically secure.

---

## Configuration Schema

Return a valid [JSON Schema](https://json-schema.org/) from `get-config-schema`. The Web UI uses the `jsonform` library to render a form from the schema automatically.

In `run()`, parse the config from `context.config`:

```rust
#[derive(serde::Deserialize)]
struct Config {
    location: String,
}

fn run(context: WidgetContext) -> WidgetResult {
    let config: Config = serde_json::from_str(&context.config)
        .unwrap_or(Config { location: "Bern".into() });
    // ...
}
```

---

## Manually Compiling and Installing

### 1. Compile to WASM

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --manifest-path widget/Cargo.toml --target wasm32-unknown-unknown
```

### 2. Create a WASM component
Install `wasm-tools`:
```bash
cargo install wasm-tools
```

```bash
wasm-tools component new \
  widget/target/wasm32-unknown-unknown/release/widget.wasm \
  -o widget/widget.component.wasm
```

### 3. Precompile for Wasmtime on ESP32

The on-device runtime is unable to compile the binary at runtime, for that reason it needs to be precompiled beforehand with the [precompiler](https://github.com/Siryll/wg_display_embedded_precompiler), see its README.

## Updating

Should new versions with changes to the widget build process be released, then your widget needs to be update to be complatible again with the new version:
- Update the `tag` of the `wg-display-widget` git dependency in `widget/Cargo.toml`.
- Update the version of `Siryll/wg_display_embedded_precompiler@vX` in `.github/workflows/build_release.yml`.

Change both versions, run `cargo update -p wg-display-widget` in `widget/` and create a new tag.

---
