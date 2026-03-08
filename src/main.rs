use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use webkit6::WebView;
use webkit6::prelude::*;

const APP_ID: &str = "wuespace.tilestion-viewer";
const DEFAULT_URI: &str = "http://localhost:3000/";

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
        webview.load_uri(&config.uri);

        window.set_child(Some(&webview));
        window.present();
    });

    app.run_with_args::<&str>(&[]);
}
