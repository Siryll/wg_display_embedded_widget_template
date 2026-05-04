# Widget Template Development

Template for creating Widgets for the [WG-Display-Embedded](https://github.com/Siryll/wg_display_embedded), included with an automatic build pipeline to create the final widget binary.

Widgets are Rust crates compiled to `wasm32-unknown-unknown` WebAssembly and packaged as WASM components using the [WIT Component Model](https://component-model.bytecodealliance.org/). They implement a standard interface (defined in WIT) and are installed and managed through the Web UI without reflashing the device.

---

## Quick Start

Use the [Widget Template](https://github.com/Siryll/wg_display_embedded_widget_template) as a starting point:

```bash
git clone https://github.com/Siryll/wg_display_embedded_widget_template my-widget
cd my-widget
```

The template includes the correct `Cargo.toml`, WIT files, and a working `lib.rs` example.

For reference implementations see:
- [`wg_display_embedded_widget_time`](https://github.com/Siryll/wg_display_embedded_widget_time) — small time widget
- [`wg_display_embedded_widget_aareguru`](https://github.com/Siryll/wg_display_embedded_widget_aareguru) — widget to display the current Aare temperature
- [`wg_display_embedded_widget_public_transport`](https://github.com/Siryll/wg_display_embedded_widget_public_transport) — configurable widget to show public transport connections

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
- Maximum response size: **~1MB** (HTTP client buffer limit)
- Timeout: **30 seconds**
- No TLS certificate verification
- Redirects: automatic, up to 5 hops
- No streaming — entire response is buffered

#### `clocks`

```wit
interface clocks {
    now: func() -> datetime;
}
```

Returns the current UTC time (synced from `timeapi.io` at boot). `seconds` is a Unix timestamp; `nanoseconds` is the sub-second part. Returns `{0, 0}` if time has not been synced yet.

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

### 1. Clone submodules

```bash
git submodule update --init
```

### 2. Compile to WASM

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --manifest-path widget/Cargo.toml --target wasm32-unknown-unknown
```

### 3. Create a WASM component
Install `wasm-tools`:
```bash
cargo install wasm-tools
```

```bash
wasm-tools component new \
  widget/target/wasm32-unknown-unknown/release/widget.wasm \
  -o widget/widget.component.wasm
```

### 4. Precompile for Wasmtime on ESP32

The on-device runtime is unable to compile the binary at runtime, for that reason it needs to be compiled for the xtensa architecture beforehand.

```bash
cargo build --release --manifest-path wg_display_embedded_precompiler/Cargo.toml
./wg_display_embedded_precompiler/target/release/wg-display-embedded-precompiler widget/widget.component.wasm  widget.precompiled.wasm
```

## Size Constraints

Since embedded WG Display version 1.3.0 the max size depends on the size of the download buffer, currently this is set to 1MB.
Increase the buffer size in [http_client/mod.rs](https://github.com/Siryll/wg_display_embedded/blob/master/embedded_app/src/http_client/mod.rs) should your widget exceed this 1MB limit.

---
