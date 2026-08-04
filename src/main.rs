use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Settings};
use gtk4::{EventControllerKey, gdk, glib};
use webkit6::{LoadEvent, WebView};
use webkit6::prelude::*;

const APP_ID: &str = "wuespace.tilestion-viewer";
const DEFAULT_URI: &str = "http://localhost:3000/";
const OFFLINE_HTML: &str = include_str!("offline.html");
const LOADING_HTML: &str = include_str!("loading.html");

const RETRY_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Clone)]
struct RuntimeConfig {
    uri: String,
    allow_insecure_tls: bool,
}

fn usage(binary: &str) -> String {
    format!(
        "Usage: {binary} [--uri <URI>] [--allow-insecure-tls] [URI]\n\
         \n\
         Options:\n\
           --uri <URI>              URI to load in the viewer.\n\
           --allow-insecure-tls     Allow invalid/self-signed TLS certificates for visited hosts.\n\
           -h, --help               Show this help message."
    )
}

fn parse_runtime_config() -> Result<RuntimeConfig, String> {
    let mut args = std::env::args();
    let binary = args
        .next()
        .unwrap_or_else(|| "tilestion-viewer".to_string());

    let mut uri = None;
    let mut allow_insecure_tls = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--uri" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("--uri requires a value.\n\n{}", usage(&binary)))?;
                uri = Some(value);
            }
            "--allow-insecure-tls" => {
                allow_insecure_tls = true;
            }
            "-h" | "--help" => {
                return Err(usage(&binary));
            }
            _ if arg.starts_with('-') => {
                return Err(format!("Unknown option: {arg}\n\n{}", usage(&binary)));
            }
            _ => {
                if uri.is_some() {
                    return Err(format!(
                        "Multiple URIs provided; use only one URI argument.\n\n{}",
                        usage(&binary)
                    ));
                }
                uri = Some(arg);
            }
        }
    }

    Ok(RuntimeConfig {
        uri: uri.unwrap_or_else(|| DEFAULT_URI.to_string()),
        allow_insecure_tls,
    })
}

fn host_from_uri(uri: &str) -> Option<&str> {
    let scheme_split = uri.find("://")?;
    let after_scheme = &uri[(scheme_split + 3)..];
    let authority = after_scheme.split('/').next()?;
    let host_port = authority.rsplit('@').next()?;
    if host_port.starts_with('[') {
        host_port.split(']').next().map(|host| &host[1..])
    } else {
        host_port.split(':').next()
    }
}

fn main() {
    let config = match parse_runtime_config() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        if let Some(settings) = Settings::default() {
            settings.set_gtk_application_prefer_dark_theme(true);
        }

        let window = ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .fullscreened(true)
            .build();

        let webview = WebView::new();
        if config.allow_insecure_tls {
            webview.connect_load_failed_with_tls_errors(|webview, uri, certificate, _errors| {
                let Some(host) = host_from_uri(uri) else {
                    return false;
                };
                let Some(network_session) = webview.network_session() else {
                    return false;
                };
                network_session.allow_tls_certificate_for_host(certificate, host);
                true
            });
        }
        let retry_uri = config.uri.clone();
        webview.connect_load_failed(move |webview, _, failing_uri, _| {
            webview.load_alternate_html(OFFLINE_HTML, failing_uri, None);

            // The backend may simply not be up yet (e.g. during boot). Keep
            // retrying the real URI on an interval; the offline page stays
            // visible meanwhile. A successful load stops the cycle because
            // load-failed no longer fires.
            let webview = webview.clone();
            let retry_uri = retry_uri.clone();
            glib::timeout_add_local_once(RETRY_INTERVAL, move || {
                webview.load_uri(&retry_uri);
            });
            true
        });

        // Render an in-memory loading page first, then navigate to the real
        // URI once that page has been committed. WebKit keeps the loading page
        // on screen until the real response arrives (or load-failed swaps in
        // the offline page), so the initial connect never flashes a blank
        // screen.
        let target_uri = config.uri.clone();
        let handler_id: Rc<RefCell<Option<glib::SignalHandlerId>>> = Rc::new(RefCell::new(None));
        let handler_id_inner = handler_id.clone();
        let id = webview.connect_load_changed(move |webview, event| {
            if event == LoadEvent::Finished {
                if let Some(id) = handler_id_inner.borrow_mut().take() {
                    webview.disconnect(id);
                }
                webview.load_uri(&target_uri);
            }
        });
        *handler_id.borrow_mut() = Some(id);
        webview.load_html(LOADING_HTML, None);

        let key_controller = EventControllerKey::new();
        let webview_handle = webview.clone();
        let app_handle = app.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::F5 {
                webview_handle.reload();
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::F12 {
                app_handle.quit();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        window.add_controller(key_controller);

        window.set_child(Some(&webview));
        window.present();
    });

    app.run_with_args::<&str>(&[]);
}
