use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use serde_json::json;

use wg_display_widget::{clocks, export, http, logging, random, Guest, WidgetContext, WidgetResult};

const WIDGET_NAME: &str = "ESP32 Widget Template";

#[derive(JsonSchema, Deserialize)]
struct WidgetConfig {
    /// Title shown in the widget
    #[serde(default = "default_title")]
    title: String,
    /// URL to make request to
    #[serde(default = "default_url")]
    url: String,
}

fn default_title() -> String {
    "Widget template".into()
}

fn default_url() -> String {
    "https://example.com".into()
}

struct MyWidget;

impl Guest for MyWidget {
    fn get_name() -> String {
        WIDGET_NAME.into()
    }

    fn run(context: WidgetContext) -> WidgetResult {
        // logging: write to the device's serial console
        logging::log(logging::Level::Info, WIDGET_NAME, "Widget run started");

        // config: the user's settings as JSON, parsed with serde
        let Ok(config) = serde_json::from_str::<WidgetConfig>(&context.config) else {
            return WidgetResult {
                data: "Invalid config".into(),
            };
        };

        // clocks: current UTC time
        let now = clocks::now().seconds;
        let time = format!(
            "Time (UTC): {:02}:{:02}:{:02}",
            now / 3600 % 24,
            now / 60 % 60,
            now % 60
        );
        let last_run = match context.last_invocation.seconds {
            0 => "Last run: first run".to_string(),
            last => format!("Last run: {} s ago", now.saturating_sub(last)),
        };

        // random: a random u64
        let random = random::get_random();

        // http: make a request. The response body is limited by the firmware, this can be adjusted.
        let request = match http::request(http::Method::Get, &config.url, None) {
            Ok(response) => format!("HTTP {}: {} bytes", response.status, response.bytes.len()),
            Err(_) => "HTTP request failed".to_string(),
        };

        WidgetResult {
            data: render(&config.title, &[&time, &last_run, &request], random),
        }
    }

    fn get_config_schema() -> String {
        let schema = schema_for!(WidgetConfig);
        serde_json::to_string_pretty(&schema).unwrap()
    }

    fn get_version() -> String {
        "0.1.0".into()
    }

    fn get_run_update_cycle_seconds() -> u32 {
        10
    }
}

/// Returns the screen layout as JSON: `{"elements": [...]}`.
/// See the README in the firmware repository for more details under shared/ui.
/// Design the layout in `shared/ui/main.slint` of the firmware repo and convert it with `shared/scripts/slint_to_widget_elements.py`.
fn render(title: &str, lines: &[&str; 3], random: u64) -> String {
    json!({ "elements": [
        // Rect
        { "type": "rect", "x": 0.02, "y": 0.03, "w": 0.96, "h": 0.94,
          "background": 0, "border_color": 0xFF4FC3F7u32, "border_width": 2,
          "border_radius": 14,
          "drop_shadow_blur": 8, "drop_shadow_color": 0x80000000u32 },

        // Text
        { "type": "text", "x": 0.08, "y": 0.07, "w": 0.84, "h": 0.15, "text": title,
          "font_size": 32, "font_weight": 700, "color": 0xFFFFFFFFu32,
          "horizontal_alignment": "center", "vertical_alignment": "center",
          "overflow": "elide" },

        // Text
        { "type": "text", "x": 0.08, "y": 0.26, "w": 0.58, "h": 0.10, "text": lines[0],
          "font_size": 20, "color": 0xFF9FB8CCu32 },
        { "type": "text", "x": 0.08, "y": 0.38, "w": 0.58, "h": 0.10, "text": lines[1],
          "font_size": 20, "color": 0xFF9FB8CCu32 },
        { "type": "text", "x": 0.08, "y": 0.50, "w": 0.58, "h": 0.10, "text": lines[2],
          "font_size": 20, "color": 0xFF9FB8CCu32 },

        // Text
        { "type": "text", "x": 0.08, "y": 0.80, "w": 0.84, "h": 0.14,
          "text": "Text can be italic, spaced out and wrap onto several lines.",
          "font_size": 18, "font_italic": true, "letter_spacing": 1, "color": 0xFFFFCA28u32,
          "wrap": "word-wrap" },

        // Progress
        { "type": "progress", "x": 0.08, "y": 0.65, "w": 0.58, "h": 0.05,
          "value": random % 100, "max": 100, "fill": 0xFF66BB6Au32,
          "background": 0xFF1F3347u32 },

        // Image: downloaded by the display, in preview only local files can be used
        { "type": "image", "x": 0.70, "y": 0.22, "w": 0.24, "h": 0.44,
          "image_url": "https://avatars.githubusercontent.com/u/50954074?v=4",
          "image_fit": "contain" },
    ]})
    .to_string()
}

export!(MyWidget);
