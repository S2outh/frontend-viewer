use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use webkit6::prelude::*;
use webkit6::WebView;

const APP_ID: &str = "wuespace.tilestion-viewer";
const DEFAULT_URI: &str = "http://localhost:3000/";

fn main() {
    let uri = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_URI.to_string());

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .fullscreened(true)
            .build();

        let webview = WebView::new();
        webview.load_uri(&uri);

        window.set_child(Some(&webview));
        window.present();
    });

    app.run();
}
