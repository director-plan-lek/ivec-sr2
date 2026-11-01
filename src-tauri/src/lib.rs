//! Alux — backend for a lightweight cross-platform browser.
//!
//! Design notes
//! ------------
//! * The window hosts one "chrome" webview (label `main`) that draws the tab strip,
//!   the address bar and the Alux Apps panel.
//! * Every tab is a *child webview* (`tab-<n>`) placed exactly over the content area.
//! * Every installed Alux App is served from a private URI scheme (`alux://`) out of
//!   the app data directory, and runs in its own child webview (`app-<id>-<n>`), which
//!   is not allowed to call the privileged commands below.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::http::{header, Response, StatusCode};
use tauri::webview::WebviewBuilder;
use tauri::{AppHandle, PhysicalPosition, PhysicalSize, Manager, Webview, WebviewUrl};

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub accent: String,
    pub entry: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct AppManifest {
    id: Option<String>,
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    accent: Option<String>,
    entry: Option<String>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Injected into every mobile tab window.
///
/// On mobile a tab opens as a full-screen window with no browser chrome, so the
/// page carries its own bottom toolbar: back / forward / reload / home.
/// `history` and `location` are used directly — no IPC round trip needed except
/// for closing the tab.
#[cfg(mobile)]
fn mobile_controls_js(label: &str) -> String {
    format!(
        r#"(function(){{
  if (window.__aluxControls) return;
  window.__aluxControls = true;
  var label = {label:?};
  // Android reaches a Tauri custom protocol as http://<scheme>.localhost, while
  // iOS/macOS keep the bare scheme form (alux://localhost). Same for the app's
  // own bundled page (tauri://localhost on iOS).
  var isIOS = /iPhone|iPad|iPod/i.test(navigator.userAgent);
  var CLOSE_URL = isIOS ? 'alux://localhost/close' : 'http://alux.localhost/close';
  var HOME_URL = isIOS ? 'tauri://localhost/' : 'http://tauri.localhost/';

  function closeTab() {{
    // Remote pages have no IPC access, and Android refuses to navigate to a
    // bare custom scheme (net::ERR_UNKNOWN_URL_SCHEME). A Tauri custom protocol
    // is reachable as http://<scheme>.localhost/..., so try a fetch first (it
    // closes the window without leaving the page) and fall back to navigating
    // to the same URL, which the Rust handler also answers.
    var done = false;
    var nav = function () {{
      if (done) return;
      done = true;
      try {{ location.href = CLOSE_URL; }} catch (e) {{}}
    }};
    // Last resort: if the window did not go away, load the app's own home page
    // in this window so the user always ends up on the search screen.
    var home = function () {{
      if (done) return;
      done = true;
      try {{ location.href = HOME_URL; }} catch (e) {{}}
    }};
    try {{
      fetch(CLOSE_URL, {{ cache: 'no-store' }})
        .then(function () {{ setTimeout(home, 800); }}, nav);
    }} catch (e) {{
      nav();
    }}
    setTimeout(home, 2000);
  }}

  function mount() {{
    if (!document.body) return;
    if (document.querySelector('[data-alux="toolbar"]')) return;

    var bar = document.createElement('div');
    bar.setAttribute('data-alux', 'toolbar');
    bar.style.cssText = [
      'position:fixed',
      'left:50%',
      'transform:translateX(-50%)',
      'bottom:calc(14px + env(safe-area-inset-bottom, 0px))',
      'z-index:2147483647',
      'display:flex',
      'align-items:center',
      'gap:2px',
      'padding:6px 8px',
      'border-radius:28px',
      'background:rgba(17,24,39,.88)',
      'box-shadow:0 8px 28px rgba(0,0,0,.5)',
      'font-family:system-ui,-apple-system,sans-serif',
      'pointer-events:auto'
    ].join(';');

    function make(glyph, title, onclick) {{
      var b = document.createElement('button');
      b.type = 'button';
      b.textContent = glyph;
      b.setAttribute('aria-label', title);
      b.title = title;
      b.style.cssText = [
        'width:44px',
        'height:44px',
        'min-width:44px',
        'border:0',
        'border-radius:22px',
        'background:transparent',
        'color:#fff',
        'font-size:19px',
        'line-height:44px',
        'text-align:center',
        'padding:0',
        'margin:0',
        'cursor:pointer',
        'touch-action:manipulation',
        '-webkit-tap-highlight-color:transparent'
      ].join(';');
      b.addEventListener('click', function (ev) {{
        ev.preventDefault();
        ev.stopPropagation();
        onclick();
      }}, true);
      return b;
    }}

    bar.appendChild(make('\u2190', '\u0e22\u0e49\u0e2d\u0e19\u0e01\u0e25\u0e31\u0e1a', function () {{
      if (history.length > 1) {{ history.back(); }} else {{ closeTab(); }}
    }}));
    bar.appendChild(make('\u2192', '\u0e44\u0e1b\u0e02\u0e49\u0e32\u0e07\u0e2b\u0e19\u0e49\u0e32', function () {{
      history.forward();
    }}));
    bar.appendChild(make('\u21bb', '\u0e42\u0e2b\u0e25\u0e14\u0e43\u0e2b\u0e21\u0e48', function () {{
      location.reload();
    }}));
    bar.appendChild(make('\u2302', '\u0e2b\u0e19\u0e49\u0e32\u0e2b\u0e25\u0e31\u0e01', closeTab));

    document.body.appendChild(bar);
  }}

  if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', mount);
  }} else {{
    mount();
  }}
  // single-page apps and clientside routing replace the body; re-attach if it goes away
  setInterval(function () {{
    if (window.__aluxControls && !document.querySelector('[data-alux="toolbar"]')) mount();
  }}, 1500);
}})();"#
    )
}



/// Chrome-like user agent. Google refuses to sign in from a "browser" it does not
/// recognise (WebView), so Alux presents itself as regular Chrome/Edge.
fn chrome_ua() -> &'static str {
    if cfg!(target_os = "windows") {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0"
    } else if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15"
    } else if cfg!(target_os = "android") {
        "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Mobile Safari/537.36"
    } else {
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36"
    }
}

/// Only the browser chrome may drive the privileged commands.
fn ensure_chrome(wv: &Webview) -> Result<(), String> {
    let label = wv.label().to_string();
    if label.starts_with("tab-") || label.starts_with("app-") {
        return Err("Alux: privileged command called from a content webview".into());
    }
    Ok(())
}

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b' ' => out.push('+'),
            other => out.push_str(&format!("%{:02X}", other)),
        }
    }
    out
}

fn looks_like_host(raw: &str) -> bool {
    if raw.contains(' ') {
        return false;
    }
    let host = raw.split('/').next().unwrap_or(raw);
    let host = host.split(':').next().unwrap_or(host);
    if host.is_empty() {
        return false;
    }
    // bare IPv4 such as 192.168.1.39 (or 192.168.1.39:8765) is a host, not a query
    let octets: Vec<&str> = host.split('.').collect();
    if octets.len() == 4 && octets.iter().all(|p| p.parse::<u8>().is_ok()) {
        return true;
    }
    host.contains('.') && host.chars().any(|c| c.is_ascii_alphabetic())
}

/// Hosts that live on the local network and normally speak plain HTTP.
fn is_lan_host(raw: &str) -> bool {
    let host = raw.split('/').next().unwrap_or(raw);
    let host = host.split(':').next().unwrap_or(host);
    if host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".lan")
    {
        return true;
    }
    let octets: Vec<u8> = host.split('.').filter_map(|p| p.parse().ok()).collect();
    if octets.len() != 4 {
        return false;
    }
    matches!(octets[0], 10 | 127)
        || (octets[0] == 192 && octets[1] == 168)
        || (octets[0] == 172 && (16..=31).contains(&octets[1]))
        || (octets[0] == 169 && octets[1] == 254)
}

/// Wipe cookies and site storage in every webview.
///
/// A Google session lives entirely in cookies, and the account pages refuse to
/// sign out from some embedded browsers, so this is the reliable way out.
#[tauri::command]
async fn clear_browsing_data(app: AppHandle, wv: Webview) -> Result<(), String> {
    ensure_chrome(&wv)?;
    for (_, webview) in app.webviews() {
        let _ = webview.clear_all_browsing_data();
    }
    Ok(())
}

/// Turn whatever the user typed in the address bar into a real URL.
fn parse_url(raw: &str) -> Result<tauri::Url, String> {
    let trimmed = raw.trim();
    let candidate = if trimmed.is_empty() {
        "https://www.google.com/".to_string()
    } else if trimmed.contains("://") {
        trimmed.to_string()
    } else if looks_like_host(trimmed) && is_lan_host(trimmed) {
        // A router page, a phone sharing files, `raspberrypi.local` — these
        // almost never speak TLS, and https:// here just looks broken.
        format!("http://{trimmed}")
    } else if looks_like_host(trimmed) {
        format!("https://{trimmed}")
    } else {
        format!(
            "https://www.google.com/search?q={}",
            percent_encode(trimmed)
        )
    };
    candidate
        .parse::<tauri::Url>()
        .map_err(|e| format!("URL ไม่ถูกต้อง: {e}"))
}

fn sanitize_id(raw: &str) -> String {
    let mut out: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "app".to_string()
    } else {
        out.chars().take(64).collect()
    }
}

fn apps_root(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("apps");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn read_manifest(dir: &Path) -> AppManifest {
    let path = dir.join("alux-app.json");
    if !path.exists() {
        return AppManifest::default();
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<AppManifest>(&raw).ok())
        .unwrap_or_default()
}

fn copy_dir(src: &Path, dest: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            std::fs::copy(&from, &to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn mime_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "wasm" => "application/wasm",
        "txt" | "md" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn reply(status: StatusCode, body: Vec<u8>, mime: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, mime)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::CACHE_CONTROL, "no-store")
        .body(body)
        .unwrap_or_else(|_| Response::new(Vec::new()))
}

/// Serves `alux://localhost/app/<id>/<file>` (and the Windows form
/// `http://alux.localhost/app/<id>/<file>`) from the installed apps directory.
fn serve_alux(app: &AppHandle, request: &tauri::http::Request<Vec<u8>>) -> Response<Vec<u8>> {
    let uri = request.uri();
    let host = uri.host().unwrap_or("");
    let path = uri.path().trim_start_matches('/');
    let rel = if host.is_empty() || host == "localhost" || host.ends_with(".localhost") {
        path.to_string()
    } else {
        format!("{host}/{path}")
    };

    let mut parts = rel.splitn(3, '/');
    let kind = parts.next().unwrap_or("");
    let id = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("");
    if kind != "app" || id.is_empty() {
        return reply(StatusCode::NOT_FOUND, b"Not found".to_vec(), "text/plain; charset=utf-8");
    }

    let Ok(root) = apps_root(app) else {
        return reply(StatusCode::INTERNAL_SERVER_ERROR, b"no apps dir".to_vec(), "text/plain");
    };
    let base = root.join(sanitize_id(id));
    let mut target = base.join(if rest.is_empty() { "index.html" } else { rest });
    if target.is_dir() {
        target = target.join("index.html");
    }

    let (Ok(canon_base), Ok(canon_target)) = (base.canonicalize(), target.canonicalize()) else {
        return reply(StatusCode::NOT_FOUND, b"Not found".to_vec(), "text/plain; charset=utf-8");
    };
    if !canon_target.starts_with(&canon_base) {
        return reply(StatusCode::FORBIDDEN, b"Forbidden".to_vec(), "text/plain; charset=utf-8");
    }

    match std::fs::read(&canon_target) {
        Ok(bytes) => reply(StatusCode::OK, bytes, mime_for(&canon_target)),
        Err(_) => reply(StatusCode::NOT_FOUND, b"Not found".to_vec(), "text/plain; charset=utf-8"),
    }
}

// ---------------------------------------------------------------------------
// Tab commands
// ---------------------------------------------------------------------------

#[cfg(desktop)]
#[tauri::command]
async fn tab_open(
    app: AppHandle,
    wv: Webview,
    label: String,
    url: String,
    rect: Rect,
) -> Result<String, String> {
    ensure_chrome(&wv)?;
    let window = app
        .get_window("main")
        .ok_or_else(|| "main window missing".to_string())?;

    if let Some(existing) = app.get_webview(&label) {
        existing
            .set_position(PhysicalPosition::new(rect.x, rect.y))
            .map_err(|e| e.to_string())?;
        existing
            .set_size(PhysicalSize::new(rect.width, rect.height))
            .map_err(|e| e.to_string())?;
        let _ = existing.show();
        return Ok(label);
    }

    let parsed = parse_url(&url)?;
    let builder = WebviewBuilder::new(label.clone(), WebviewUrl::External(parsed))
        .user_agent(chrome_ua());
    window
        .add_child(
            builder,
            PhysicalPosition::new(rect.x, rect.y),
            PhysicalSize::new(rect.width, rect.height),
        )
        .map_err(|e| e.to_string())?;
    Ok(label)
}

/// Mobile: `Window::add_child` is desktop-only in Tauri, so every tab becomes
/// its own full-screen webview window instead of a child of the main window.
#[cfg(mobile)]
#[tauri::command]
async fn tab_open(
    app: AppHandle,
    wv: Webview,
    label: String,
    url: String,
    _rect: Rect,
) -> Result<String, String> {
    ensure_chrome(&wv)?;
    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(label);
    }
    let parsed = parse_url(&url)?;
    tauri::WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(parsed))
        .title("IVEC SR 2")
        .user_agent(chrome_ua())
        .initialization_script(mobile_controls_js(&label))
        .build()
        .map_err(|e| e.to_string())?;
    Ok(label)
}

#[cfg(desktop)]
#[tauri::command]
fn tab_sync(
    app: AppHandle,
    wv: Webview,
    rect: Rect,
    labels: Vec<String>,
    visible: Option<String>,
) -> Result<(), String> {
    ensure_chrome(&wv)?;
    for label in labels {
        let Some(view) = app.get_webview(&label) else {
            continue;
        };
        let _ = view.set_position(PhysicalPosition::new(rect.x, rect.y));
        let _ = view.set_size(PhysicalSize::new(rect.width, rect.height));
        match &visible {
            Some(active) if active == &label => {
                let _ = view.show();
            }
            _ => {
                let _ = view.hide();
            }
        }
    }
    Ok(())
}

/// Mobile tabs are separate windows, so there is nothing to lay out.
#[cfg(mobile)]
#[tauri::command]
fn tab_sync(
    _app: AppHandle,
    wv: Webview,
    _rect: Rect,
    _labels: Vec<String>,
    _visible: Option<String>,
) -> Result<(), String> {
    ensure_chrome(&wv)?;
    Ok(())
}

#[tauri::command]
fn tab_navigate(app: AppHandle, wv: Webview, label: String, url: String) -> Result<String, String> {
    ensure_chrome(&wv)?;
    let parsed = parse_url(&url)?;
    let view = app
        .get_webview(&label)
        .ok_or_else(|| "ไม่พบแท็บนี้".to_string())?;
    view.navigate(parsed.clone()).map_err(|e| e.to_string())?;
    Ok(parsed.to_string())
}

#[tauri::command]
fn tab_action(app: AppHandle, wv: Webview, label: String, action: String) -> Result<(), String> {
    ensure_chrome(&wv)?;
    let view = app
        .get_webview(&label)
        .ok_or_else(|| "ไม่พบแท็บนี้".to_string())?;
    let script = match action.as_str() {
        "back" => "history.back()",
        "forward" => "history.forward()",
        "reload" => "location.reload()",
        "stop" => "window.stop()",
        "devtools" => "void 0",
        _ => return Err(format!("ไม่รู้จักคำสั่ง: {action}")),
    };
    view.eval(script).map_err(|e| e.to_string())
}

#[tauri::command]
fn tab_url(app: AppHandle, wv: Webview, label: String) -> Result<String, String> {
    ensure_chrome(&wv)?;
    let Some(view) = app.get_webview(&label) else {
        return Ok(String::new());
    };
    Ok(view
        .url()
        .map(|u| u.to_string())
        .unwrap_or_else(|_| String::new()))
}

#[cfg(desktop)]
#[tauri::command]
fn tab_close(app: AppHandle, wv: Webview, label: String) -> Result<(), String> {
    ensure_chrome(&wv)?;
    if let Some(view) = app.get_webview(&label) {
        view.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Mobile: closing a tab means closing the window that hosts it.
#[cfg(mobile)]
#[tauri::command]
fn tab_close(app: AppHandle, wv: Webview, label: String) -> Result<(), String> {
    ensure_chrome(&wv)?;
    if let Some(win) = app.get_webview_window(&label) {
        win.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Alux Apps — the "install slot" inside the browser
// ---------------------------------------------------------------------------

/// The sample app embedded in the binary, so any install can prove the
/// "install an app" slot works without needing a file picker.
const DEMO_APP_HTML: &str = include_str!("../assets/demo-app/index.html");

#[tauri::command]
fn apps_install_demo(app: AppHandle, wv: Webview) -> Result<AppInfo, String> {
    ensure_chrome(&wv)?;
    let root = apps_root(&app)?;
    let id = "ivec-demo";
    let dir = root.join(id);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("index.html"), DEMO_APP_HTML).map_err(|e| e.to_string())?;

    let manifest = serde_json::json!({
        "id": id,
        "name": "IVEC SR 2 Demo",
        "version": "1.0.0",
        "description": "แอปตัวอย่างสำหรับทดสอบช่องติดตั้งแอป",
        "accent": "#2f7dff",
        "entry": "index.html"
    });
    std::fs::write(dir.join("alux-app.json"), manifest.to_string()).map_err(|e| e.to_string())?;

    Ok(AppInfo {
        id: id.into(),
        name: "IVEC SR 2 Demo".into(),
        version: "1.0.0".into(),
        description: "แอปตัวอย่างสำหรับทดสอบช่องติดตั้งแอป".into(),
        accent: "#2f7dff".into(),
        entry: "index.html".into(),
    })
}

#[tauri::command]
fn apps_list(app: AppHandle) -> Result<Vec<AppInfo>, String> {
    let root = apps_root(&app)?;
    let mut items: Vec<AppInfo> = Vec::new();
    for entry in std::fs::read_dir(&root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        let manifest = read_manifest(&entry.path());
        items.push(AppInfo {
            name: manifest.name.clone().unwrap_or_else(|| id.clone()),
            version: manifest.version.clone().unwrap_or_else(|| "1.0.0".into()),
            description: manifest.description.clone().unwrap_or_default(),
            accent: manifest.accent.clone().unwrap_or_else(|| "#5b8cff".into()),
            entry: manifest.entry.clone().unwrap_or_else(|| "index.html".into()),
            id,
        });
    }
    items.sort_by_key(|a| a.name.to_lowercase());
    Ok(items)
}

/// Install an Alux App from a folder that contains `index.html`
/// (and optionally an `alux-app.json` manifest).
#[tauri::command]
fn apps_install(app: AppHandle, wv: Webview, source: String) -> Result<AppInfo, String> {
    ensure_chrome(&wv)?;
    let root = apps_root(&app)?;
    let src = PathBuf::from(&source);
    if !src.is_dir() {
        return Err("กรุณาเลือกโฟลเดอร์แอป".into());
    }
    let manifest = read_manifest(&src);
    let entry = manifest.entry.clone().unwrap_or_else(|| "index.html".into());
    if !src.join(&entry).exists() {
        return Err(format!("ไม่พบไฟล์ {entry} ในโฟลเดอร์แอป"));
    }

    let raw_id = manifest
        .id
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            src.file_name()
                .map(|n| n.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "app".into());
    let id = sanitize_id(&raw_id);
    if id.is_empty() || id == ".." {
        return Err("ชื่อแอปไม่ถูกต้อง".into());
    }

    let dest = root.join(&id);
    if dest.exists() {
        std::fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
    }
    copy_dir(&src, &dest)?;

    // Make sure an installed app always has a manifest to read next time.
    if !dest.join("alux-app.json").exists() {
        let fallback = serde_json::json!({
            "id": id,
            "name": manifest.name.clone().unwrap_or_else(|| id.clone()),
            "version": manifest.version.clone().unwrap_or_else(|| "1.0.0".into()),
            "description": manifest.description.clone().unwrap_or_default(),
            "entry": entry,
        });
        let _ = std::fs::write(
            dest.join("alux-app.json"),
            serde_json::to_string_pretty(&fallback).unwrap_or_default(),
        );
    }

    Ok(AppInfo {
        name: manifest.name.unwrap_or_else(|| id.clone()),
        version: manifest.version.unwrap_or_else(|| "1.0.0".into()),
        description: manifest.description.unwrap_or_default(),
        accent: manifest.accent.unwrap_or_else(|| "#5b8cff".into()),
        entry,
        id,
    })
}

#[tauri::command]
fn apps_delete(app: AppHandle, wv: Webview, id: String) -> Result<(), String> {
    ensure_chrome(&wv)?;
    let root = apps_root(&app)?;
    let safe = sanitize_id(&id);
    let dir = root.join(&safe);
    if dir.exists() {
        let canon_root = root.canonicalize().map_err(|e| e.to_string())?;
        let canon_dir = dir.canonicalize().map_err(|e| e.to_string())?;
        if !canon_dir.starts_with(&canon_root) || canon_dir == canon_root {
            return Err("path ไม่ถูกต้อง".into());
        }
        std::fs::remove_dir_all(&canon_dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Open an installed app in its own sandboxed child webview.
#[cfg(desktop)]
#[tauri::command]
async fn app_open(
    app: AppHandle,
    wv: Webview,
    id: String,
    label: String,
    rect: Rect,
) -> Result<String, String> {
    ensure_chrome(&wv)?;
    let root = apps_root(&app)?;
    let id = sanitize_id(&id);
    if !root.join(&id).is_dir() {
        return Err("ไม่พบแอปนี้ (ติดตั้งหรือยัง?)".into());
    }
    let manifest = read_manifest(&root.join(&id));
    let entry = manifest.entry.unwrap_or_else(|| "index.html".into());
    let url = format!("alux://localhost/app/{id}/{entry}");

    let window = app
        .get_window("main")
        .ok_or_else(|| "main window missing".to_string())?;

    if let Some(existing) = app.get_webview(&label) {
        let _ = existing.set_position(PhysicalPosition::new(rect.x, rect.y));
        let _ = existing.set_size(PhysicalSize::new(rect.width, rect.height));
        let _ = existing.show();
        return Ok(url);
    }

    let parsed = url
        .parse::<tauri::Url>()
        .map_err(|e| format!("URL แอปไม่ถูกต้อง: {e}"))?;
    let builder = WebviewBuilder::new(label, WebviewUrl::External(parsed)).user_agent(chrome_ua());
    window
        .add_child(
            builder,
            PhysicalPosition::new(rect.x, rect.y),
            PhysicalSize::new(rect.width, rect.height),
        )
        .map_err(|e| e.to_string())?;
    Ok(url)
}

/// Mobile counterpart of `app_open`: a dedicated window per installed app.
#[cfg(mobile)]
#[tauri::command]
async fn app_open(
    app: AppHandle,
    wv: Webview,
    id: String,
    label: String,
    _rect: Rect,
) -> Result<String, String> {
    ensure_chrome(&wv)?;
    let root = apps_root(&app)?;
    let id = sanitize_id(&id);
    if !root.join(&id).is_dir() {
        return Err("ไม่พบแอปนี้ (ติดตั้งหรือยัง?)".into());
    }
    let manifest = read_manifest(&root.join(&id));
    let entry = manifest.entry.unwrap_or_else(|| "index.html".into());
    let url = format!("alux://localhost/app/{id}/{entry}");

    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(url);
    }
    let parsed = url
        .parse::<tauri::Url>()
        .map_err(|e| format!("URL แอปไม่ถูกต้อง: {e}"))?;
    tauri::WebviewWindowBuilder::new(&app, &label, WebviewUrl::External(parsed))
        .title(manifest.name.unwrap_or_else(|| "IVEC SR 2".into()))
        .user_agent(chrome_ua())
        .initialization_script(mobile_controls_js(&label))
        .build()
        .map_err(|e| e.to_string())?;
    Ok(url)
}

/// Where installed apps live — shown in the UI so อาเล็ก knows where to drop bundles.
#[tauri::command]
fn apps_dir(app: AppHandle) -> Result<String, String> {
    Ok(apps_root(&app)?.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("alux", |ctx, request| {
            let app = ctx.app_handle();
            // `alux://close` is the escape hatch the mobile page toolbar uses:
            // remote pages are not allowed to call IPC commands, but they can
            // always navigate to a custom scheme, and this handler is Rust.
            let uri = request.uri();
            let is_close = uri.host() == Some("close")
                || uri.path().trim_start_matches('/').starts_with("close");
            if is_close {
                let label = ctx.webview_label().to_string();
                let mut closed = false;
                if let Some(win) = app.get_webview_window(&label) {
                    closed = win.close().is_ok();
                }
                if !closed {
                    // Fallback: close whatever tab/app window currently has focus.
                    for (win_label, win) in app.webview_windows() {
                        let is_tab = win_label.starts_with("tab-") || win_label.starts_with("app-");
                        if is_tab && win.is_focused().unwrap_or(false) {
                            let _ = win.close();
                            closed = true;
                            break;
                        }
                    }
                }
                return reply(StatusCode::OK, b"closed".to_vec(), "text/plain; charset=utf-8");
            }
            serve_alux(app, &request)
        })
        .invoke_handler(tauri::generate_handler![
            tab_open,
            tab_sync,
            tab_navigate,
            tab_action,
            tab_url,
            tab_close,
            clear_browsing_data,
            apps_list,
            apps_install_demo,
            apps_install,
            apps_delete,
            app_open,
            apps_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running Alux");
}
