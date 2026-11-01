import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { AluxApp } from "../lib/bridge";
import { Download, Trash } from "./Icons";

type Props = {
  apps: AluxApp[];
  dir: string;
  onClose: () => void;
  onOpenApp: (app: AluxApp) => void;
  onInstall: (source: string) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
};

export default function AppsPanel({ apps, dir, onClose, onOpenApp, onInstall, onDelete }: Props) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  async function pick() {
    setError(null);
    try {
      const picked = await open({ directory: true, multiple: false, title: "เลือกโฟลเดอร์แอป Alux" });
      if (typeof picked !== "string") return;
      setBusy(true);
      await onInstall(picked);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <aside className="apps-panel">
      <header className="apps-panel-head">
        <div>
          <h2>Alux Apps</h2>
          <p>ช่องติดตั้งแอปในเบราว์เซอร์</p>
        </div>
        <button className="icon-button" onClick={onClose} aria-label="ปิด">
          ✕
        </button>
      </header>

      <button className="primary install" onClick={pick} disabled={busy}>
        <Download size={16} />
        {busy ? "กำลังติดตั้ง…" : "ติดตั้งจากโฟลเดอร์"}
      </button>

      {error && <p className="panel-error">{error}</p>}

      <ul className="apps-list">
        {apps.map((app) => (
          <li key={app.id}>
            <button className="apps-list-main" onClick={() => onOpenApp(app)}>
              <span className="app-badge" style={{ background: app.accent }}>
                {app.name.slice(0, 1).toUpperCase()}
              </span>
              <span className="app-meta">
                <span className="app-name">{app.name}</span>
                <span className="app-version">
                  v{app.version}
                  {app.description ? ` · ${app.description}` : ""}
                </span>
              </span>
            </button>
            <button
              className="icon-button danger"
              title="ถอนการติดตั้ง"
              onClick={() => onDelete(app.id)}
            >
              <Trash size={15} />
            </button>
          </li>
        ))}
        {apps.length === 0 && <li className="apps-list-empty">ยังไม่มีแอปที่ติดตั้ง</li>}
      </ul>

      <footer className="apps-panel-foot">
        <p className="field-label">โฟลเดอร์เก็บแอป</p>
        <code className="path">{dir || "…"}</code>
        <button
          className="ghost small"
          onClick={async () => {
            try {
              await navigator.clipboard.writeText(dir);
              setCopied(true);
              setTimeout(() => setCopied(false), 1500);
            } catch {
              /* clipboard may be blocked — ignore */
            }
          }}
        >
          {copied ? "คัดลอกแล้ว" : "คัดลอกที่อยู่"}
        </button>
        <p className="hint">
          แอปหนึ่งตัว = หนึ่งโฟลเดอร์ที่มี <code>index.html</code> และ (ไม่บังคับ)
          <code>alux-app.json</code> — วางโฟลเดอร์ไว้ที่นี่แล้วกดรีเฟรชก็ได้
        </p>
      </footer>
    </aside>
  );
}
