use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use serde_json::{Error, Value};

use widget::widget::{http, logging};

wit_bindgen::generate!({
    path: "../wg_display_widget_wit/wit",
    world: "widget"
});

const WIDGET_NAME: &str = "ESP32 Widget Template";

#[derive(JsonSchema, Deserialize)]
struct WidgetConfig {
    city: String,
}

struct MyWidget;

impl Guest for MyWidget {
    fn get_name() -> String {
        WIDGET_NAME.into()
    }

    fn run(context: WidgetContext) -> WidgetResult {
        // Widgets can log to the console
        logging::log(logging::Level::Info, WIDGET_NAME, "Widget run started");

        // Widgets can handle the case where no config is provided
        if "{}" == context.config {
            return WidgetResult {
                data: "No config provided".into(),
            };
        }

        // Widgets can parse their config with ease using serde
        let config: WidgetConfig =
            serde_json::from_str(&context.config).expect("Failed to parse config");

        // Widgets can make network requests
        let response = http::request(
            http::Method::Get,
            format!(
                "https://aareguru.existenz.ch/v2018/today?city={}",
                config.city
            )
            .as_str(),
            None,
        );
        let Ok(response) = response else {
            return WidgetResult {
                data: "Failed to make network request".into(),
            };
        };

        if 200 != response.status {
            return WidgetResult {
                data: format!("Response status != 200: {}", response.status),
            };
        }

        let data: Result<Value, Error> = serde_json::from_slice(response.bytes.as_slice());
        let result = match data {
            Ok(data) => format!("Aare Temperature in {}: {} C", config.city, data["aare"]),
            Err(_) => "Response from AareGuru could not be parsed".into(),
        };

        WidgetResult { data: result }
    }

    fn get_config_schema() -> String  {
        let schema = schema_for!(WidgetConfig);
        serde_json::to_string_pretty(&schema).unwrap()
    }

    fn get_version() -> String {
        "0.1.0-interface-test".into()
    }

    fn get_run_update_cycle_seconds() -> u32 {
        10
    }
}

export!(MyWidget);
