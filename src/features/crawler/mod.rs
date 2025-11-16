use headless_chrome::protocol::cdp::{Emulation, Page};
use headless_chrome::{Browser as CBrowser, Tab};
use serde_json::{json, to_string};
use std::error::Error as StdError;
use std::sync::Arc;
use std::time::Duration;

pub type AnyError = Box<dyn StdError + Send + Sync>;

pub struct Browser {
    _browser: Arc<CBrowser>,
    pub tab: Arc<Tab>,
    pub width: u32,
    pub height: u32,
}

impl Browser {
    pub fn new(url: &str, width: Option<u32>, height: Option<u32>) -> Result<Browser, AnyError> {
        let viewport_width = width.unwrap_or(1920);
        let viewport_height = height.unwrap_or(1080);

        // Create browser and wrap in Arc so it outlives this function
        let browser = CBrowser::default()?;
        let browser = Arc::new(browser);

        // Create tab from the same browser
        let tab = browser.new_tab()?;

        let set_device_metrics = Emulation::SetDeviceMetricsOverride {
            width: viewport_width,
            height: viewport_height,
            device_scale_factor: 1.0,
            mobile: false,
            scale: None,
            screen_width: None,
            screen_height: None,
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
            display_feature: None,
            device_posture: None,
        };

        tab.call_method(set_device_metrics)?;

        tab.navigate_to(url)?;
        tab.wait_for_element_with_custom_timeout("body", Duration::from_secs(10))?;

        std::thread::sleep(Duration::from_millis(2000));

        Ok(Self {
            _browser: browser,
            tab,
            height: viewport_height,
            width: viewport_width,
        })
    }

    pub async fn screenshot(&self, output_path: Option<&str>) -> Result<String, AnyError> {
        let output_file = output_path.unwrap_or("screenshot.jpeg");

        let tab = self.tab.clone();

        let vp = tab
            .wait_for_element_with_custom_timeout("body", Duration::from_secs(10))?
            .get_box_model()?
            .margin_viewport();

        let set_device_metrics = Emulation::SetDeviceMetricsOverride {
            width: self.width,
            height: self.height,
            device_scale_factor: 1.0,
            mobile: false,
            scale: None,
            screen_width: None,
            screen_height: None,
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
            display_feature: None,
            device_posture: None,
        };

        tab.call_method(set_device_metrics)?;

        let jpeg_data = tab.capture_screenshot(
            Page::CaptureScreenshotFormatOption::Jpeg,
            None,
            Some(vp),
            false,
        )?;

        std::fs::write(output_file, jpeg_data)?;

        Ok(output_file.to_string())
    }

    pub fn get_element_text(&self, selector: &str) -> Result<String, AnyError> {
        let element = self.tab.find_element(selector)?;
        let text = element.get_inner_text()?;
        Ok(text)
    }

    pub fn get_element_html(&self, selector: &str) -> Result<String, AnyError> {
        let element = self.tab.find_element(selector)?;

        let result =
            element.call_js_fn(r#"function() { return this.outerHTML; }"#, vec![], false)?;

        let value = result.value.ok_or("JS did not return a value")?;

        let s = value.as_str().ok_or("JS did not return a string")?;

        Ok(s.to_string())
    }

    pub fn get_attr(&self, selector: &str, attr: &str) -> Result<String, AnyError> {
        let element = self.tab.find_element(selector)?;
        let result = element.call_js_fn(
            r#"function(name) { return this.getAttribute(name); }"#,
            vec![json!(attr)],
            false,
        )?;
        Ok(result.value.unwrap_or_default().to_string())
    }

    pub fn get_attrs(&self, selector: &str, attr: &str) -> Result<Vec<String>, AnyError> {
        // Find all elements that match selector
        let elements = self.tab.find_elements(selector)?;
        let mut results = Vec::new();

        for element in elements {
            let value = element.call_js_fn(
                r#"function(name) { return this.getAttribute(name); }"#,
                vec![json!(attr)],
                false,
            )?;

            if let Some(s) = value.value
                && s.is_string()
            {
                results.push(s.to_string());
            }
        }

        Ok(results)
    }

    pub fn remove_elements(&self, selectors: Vec<&str>) -> Result<(), AnyError> {
        // Convert Rust Vec<&str> to JS array literal as string
        let js_array = to_string(&selectors)?; // e.g. ["div.ad",".banner"]

        // Build JS code that removes all matched elements
        let script = format!(
            r#"(function(selectors) {{
                selectors.forEach(function(sel) {{
                    document.querySelectorAll(sel).forEach(function(el) {{
                        el.remove();
                    }});
                }});
            }})({});"#,
            js_array
        );

        // Execute JS in the page context
        self.tab.evaluate(&script, false)?;

        Ok(())
    }
}
