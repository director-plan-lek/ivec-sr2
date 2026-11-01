import type { AluxApp } from "../lib/bridge";
import { Download, Grid, Sparkle } from "./Icons";

type Props = {
  apps: AluxApp[];
  onSearch: (query: string) => void;
  onOpenApp: (app: AluxApp) => void;
  onOpenApps: () => void;
  onClearData: () => void;
  onInstallDemo: () => void;
};

const SHORTCUTS = [
  { name: "Google Docs", url: "https://docs.google.com/document/u/0/", glyph: "📄" },
  { name: "Google Drive", url: "https://drive.google.com/", glyph: "🗂️" },
  { name: "Gmail", url: "https://mail.google.com/", glyph: "✉️" },
  { name: "YouTube", url: "https://www.youtube.com/", glyph: "▶️" },
  { name: "Sheets", url: "https://docs.google.com/spreadsheets/u/0/", glyph: "📊" },
  { name: "Wikipedia", url: "https://th.wikipedia.org/", glyph: "🌐" },
];

export default function NewTabPage({
  apps,
  onSearch,
  onOpenApp,
  onOpenApps,
  onClearData,
  onInstallDemo,
}: Props) {
  // The phone (Android or iOS) already ships Docs/Drive/Gmail/YouTube, so those
  // shortcut tiles are hidden there. The search bar stays on every platform.
  const isMobileNative = /Android|iPhone|iPad|iPod/i.test(navigator.userAgent);

  return (
    <div className="newtab">
      <div className="newtab-inner">
        <h1 className="wordmark">
          IVEC SR 2<span className="wordmark-dot" />
        </h1>

        <form
          className="searchbox"
          onSubmit={(event) => {
            event.preventDefault();
            const value = new FormData(event.currentTarget).get("q");
            if (typeof value === "string" && value.trim()) onSearch(value.trim());
          }}
        >
          <span className="searchbox-glyph">⌕</span>
          <input name="q" placeholder="ค้นหาด้วย Google หรือพิมพ์ที่อยู่เว็บ" autoComplete="off" />
          <button type="submit">ค้นหา</button>
        </form>

        {!isMobileNative && (
          <div className="shortcuts">
            {SHORTCUTS.map((item) => (
              <button key={item.url} className="shortcut" onClick={() => onSearch(item.url)}>
                <span className="shortcut-glyph">{item.glyph}</span>
                <span className="shortcut-name">{item.name}</span>
              </button>
            ))}
          </div>
        )}

        <section className="apps-strip">
          <header>
            <div className="apps-strip-title">
              <Grid size={16} />
              <span>IVEC Apps ของฉัน</span>
            </div>
            <div className="apps-actions">
              <button className="ghost" onClick={onInstallDemo}>
                <Sparkle size={15} /> แอปตัวอย่าง
              </button>
              <button className="ghost" onClick={onOpenApps}>
                <Download size={15} /> ติดตั้งแอป
              </button>
            </div>
          </header>

          {apps.length === 0 ? (
            <p className="apps-empty">
              ยังไม่มีแอปที่ติดตั้ง — กด “แอปตัวอย่าง” เพื่อลองติดตั้ง 1 รายการ
              หรือกด “ติดตั้งแอป” แล้วเลือกโฟลเดอร์แอปของคุณ (ต้องมี <code>index.html</code>)
            </p>
          ) : (
            <div className="apps-grid">
              {apps.map((app) => (
                <button key={app.id} className="app-tile" onClick={() => onOpenApp(app)}>
                  <span className="app-badge" style={{ background: app.accent }}>
                    {app.name.slice(0, 1).toUpperCase()}
                  </span>
                  <span className="app-meta">
                    <span className="app-name">{app.name}</span>
                    <span className="app-version">v{app.version}</span>
                  </span>
                </button>
              ))}
            </div>
          )}
        </section>

        <footer className="newtab-footer">
          <button className="ghost" onClick={onClearData}>
            ล้างคุกกี้และออกจากบัญชี Google
          </button>
        </footer>
      </div>
    </div>
  );
}
