use headless_chrome::protocol::cdp::{Emulation, Page};
use headless_chrome::{Browser as CBrowser, Tab};
use serde_json::{json, to_string};
use std::error::Error as StdError;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::spawn_blocking;

pub type AnyError = Box<dyn StdError + Send + Sync>;

pub struct Browser {
    _browser: Arc<CBrowser>,
    pub tab: Arc<Tab>,
    pub width: u32,
    pub height: u32,
}

impl Browser {
    pub async fn new(
        url: &str,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Browser, AnyError> {
        let target_url = url.to_string();
        let viewport_width = width.unwrap_or(1920);
        let viewport_height = height.unwrap_or(1080);

        run_blocking_chrome_task(move || {
            let browser = Arc::new(CBrowser::default()?);
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

            let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                              AppleWebKit/537.36 (KHTML, like Gecko) \
                              Chrome/123.0.0.0 Safari/537.36";

            // Prefer Persian, then English
            let accept_language = "fa-IR,fa;q=0.9,en-US;q=0.8,en;q=0.7";

            // This becomes navigator.platform
            let platform = "Win32";

            // If your headless_chrome version has this signature:
            // fn set_user_agent(&self, ua: &str, accept_language: Option<String>, platform: Option<String>)
            tab.set_user_agent(
                user_agent,
                Some(&*accept_language.to_string()),
                Some(&*platform.to_string()),
            )?;

            tab.navigate_to(&target_url)?;

            // Wait until the page is fully loaded (document.readyState === "complete")
            wait_for_page_load(&tab, Duration::from_secs(30))?;

            Ok(Self {
                _browser: browser,
                tab,
                height: viewport_height,
                width: viewport_width,
            })
        })
        .await
    }

    pub async fn screenshot(&self, output_path: Option<&str>) -> Result<String, AnyError> {
        let output_file = output_path.unwrap_or("screenshot.jpeg").to_string();
        let width = self.width;
        let height = self.height;
        let tab = self.tab.clone();

        run_blocking_chrome_task(move || {
            let vp = tab
                .wait_for_element_with_custom_timeout("body", Duration::from_secs(10))?
                .get_box_model()?
                .margin_viewport();

            let set_device_metrics = Emulation::SetDeviceMetricsOverride {
                width,
                height,
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

            std::fs::write(&output_file, jpeg_data)?;

            Ok(output_file)
        })
        .await
    }

    pub async fn get_element_text(&self, selector: &str) -> Result<String, AnyError> {
        let tab = self.tab.clone();
        let selector = selector.to_string();

        run_blocking_chrome_task(move || {
            let element = tab.find_element(&selector)?;
            let text = element.get_inner_text()?;
            Ok(text)
        })
        .await
    }

    pub async fn get_element_html(&self, selector: &str) -> Result<String, AnyError> {
        let tab = self.tab.clone();
        let selector = selector.to_string();

        run_blocking_chrome_task(move || {
            let element = tab.find_element(&selector)?;
            let result =
                element.call_js_fn(r#"function() { return this.outerHTML; }"#, vec![], false)?;
            let value = result.value.ok_or("JS did not return a value")?;
            let s = value.as_str().ok_or("JS did not return a string")?;
            Ok(s.to_string())
        })
        .await
    }

    pub async fn get_attr(&self, selector: &str, attr: &str) -> Result<String, AnyError> {
        let tab = self.tab.clone();
        let selector = selector.to_string();
        let attr = attr.to_string();

        run_blocking_chrome_task(move || {
            let element = tab.find_element(&selector)?;
            let result = element.call_js_fn(
                r#"function(name) { return this.getAttribute(name); }"#,
                vec![json!(attr)],
                false,
            )?;
            Ok(result.value.unwrap_or_default().to_string())
        })
        .await
    }

    pub async fn get_attrs(&self, selector: &str, attr: &str) -> Result<Vec<String>, AnyError> {
        let tab = self.tab.clone();
        let selector = selector.to_string();
        let attr = attr.to_string();

        run_blocking_chrome_task(move || {
            let elements = tab.find_elements(&selector)?;
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
        })
        .await
    }

    pub async fn remove_elements(&self, selectors: Vec<String>) -> Result<(), AnyError> {
        let tab = self.tab.clone();

        run_blocking_chrome_task(move || {
            let js_array = to_string(&selectors)?;
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

            tab.evaluate(&script, false)?;
            Ok(())
        })
        .await
    }

    pub async fn wait_for_selector(
        &self,
        selector: &str,
        timeout: Duration,
    ) -> Result<(), AnyError> {
        let tab = self.tab.clone();
        let selector = selector.to_string();

        run_blocking_chrome_task(move || {
            // Wait until an element matching selector appears or timeout
            tab.wait_for_element_with_custom_timeout(&selector, timeout)?;
            Ok(())
        })
        .await
    }
}

async fn run_blocking_chrome_task<F, R>(task: F) -> Result<R, AnyError>
where
    F: FnOnce() -> Result<R, AnyError> + Send + 'static,
    R: Send + 'static,
{
    spawn_blocking(task)
        .await
        .map_err(|err| -> AnyError { Box::new(err) })?
}

// Synchronous helper used inside spawn_blocking to wait for full page load
fn wait_for_page_load(tab: &Tab, timeout: Duration) -> Result<(), AnyError> {
    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            return Err("Page load timeout".into());
        }

        let result = tab.evaluate("document.readyState", false)?;
        let state = result.value.as_ref().and_then(|v| v.as_str()).unwrap_or("");

        if state == "complete" {
            break;
        }

        std::thread::sleep(Duration::from_millis(200));
    }

    Ok(())
}
