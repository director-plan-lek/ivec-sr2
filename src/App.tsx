import { useCallback, useEffect, useRef, useState } from "react";
import * as api from "./lib/bridge";
import type { AluxApp } from "./lib/bridge";
import AppsPanel from "./components/AppsPanel";
import NewTabPage from "./components/NewTabPage";
import {
  ArrowLeft,
  ArrowRight,
  Cross,
  Grid,
  Lock,
  Plus,
  Reload,
  Shield,
} from "./components/Icons";

type Tab = { id: string; url: string | null };

let counter = 0;
const nextId = () => `t${++counter}`;

const prettyHost = (url: string) => {
  try {
    const host = new URL(url).hostname;
    return host.replace(/^www\./, "") || url;
  } catch {
    return url;
  }
};

const isSecure = (url: string | null) =>
  !!url && (url.startsWith("https://") || url.startsWith("alux://"));

export default function App() {
  const [tabs, setTabs] = useState<Tab[]>(() => [{ id: nextId(), url: null }]);
  const [activeId, setActiveId] = useState<string>(() => tabs[0].id);
  const [address, setAddress] = useState("");
  const [editing, setEditing] = useState(false);
  const [rect, setRect] = useState<api.Rect | null>(null);
  const [appsOpen, setAppsOpen] = useState(false);
  const [apps, setApps] = useState<AluxApp[]>([]);
  const [dir, setDir] = useState("");
  const [loading, setLoading] = useState(false);

  const contentRef = useRef<HTMLDivElement | null>(null);
  const addressRef = useRef<HTMLInputElement | null>(null);

  const activeTab = tabs.find((t) => t.id === activeId) ?? tabs[0];
  const activeLabel = activeTab?.url ? api.tabLabel(activeTab.id) : null;

  // ---- apps ------------------------------------------------------------
  const refreshApps = useCallback(async () => {
    try {
      setApps(await api.appsList());
    } catch (err) {
      console.error(err);
    }
  }, []);

  useEffect(() => {
    void refreshApps();
    api.appsDir().then(setDir).catch(() => setDir(""));
  }, [refreshApps]);

  // ---- viewport measurement -------------------------------------------
  useEffect(() => {
    const el = contentRef.current;
    if (!el) return;
    const measure = () => {
      const r = el.getBoundingClientRect();
      // Child webviews are positioned in *physical* pixels; CSS pixels must be
      // scaled by the display's device-pixel ratio (e.g. 1.1 at 110% scaling).
      const dpr = window.devicePixelRatio || 1;
      setRect({
        x: Math.round(r.left * dpr),
        y: Math.round(r.top * dpr),
        width: Math.max(1, Math.round(r.width * dpr)),
        height: Math.max(1, Math.round(r.height * dpr)),
      });
    };
    measure();
    const raf = requestAnimationFrame(measure);
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
    };
  }, []);

  // ---- keep child webviews in sync with the tab model ------------------
  useEffect(() => {
    if (!rect) return;
    const labels = tabs.filter((t) => t.url).map((t) => api.tabLabel(t.id));
    api.tabSync(rect, labels, activeLabel).catch(() => {});
  }, [tabs, activeLabel, rect]);

  // ---- navigation ------------------------------------------------------
  const go = useCallback(
    async (raw: string) => {
      if (!rect) return;
      const target = tabs.find((t) => t.id === activeId);
      if (!target) return;
      const label = api.tabLabel(target.id);
      const value = raw.trim() || "https://www.google.com/";
      setEditing(false);
      setLoading(true);
      // On Android the page opens in its own window and this invoke can take a
      // while (or never settle if the window fails to build) — never leave the
      // spinner running forever.
      const watchdog = window.setTimeout(() => setLoading(false), 6000);
      try {
        if (target.url) {
          await api.tabNavigate(label, value);
        } else {
          await api.tabOpen(label, value, rect);
        }
        setTabs((prev) => prev.map((t) => (t.id === target.id ? { ...t, url: value } : t)));
      } catch (err) {
        console.error(err);
      } finally {
        window.clearTimeout(watchdog);
        window.setTimeout(() => setLoading(false), 500);
      }
    },
    [rect, tabs, activeId],
  );

  const action = useCallback(
    async (kind: api.TabAction) => {
      if (!activeTab?.url) return;
      try {
        await api.tabAction(api.tabLabel(activeTab.id), kind);
      } catch (err) {
        console.error(err);
      }
    },
    [activeTab],
  );

  const newTab = useCallback(() => {
    const fresh: Tab = { id: nextId(), url: null };
    setTabs((prev) => [...prev, fresh]);
    setActiveId(fresh.id);
    setAddress("");
    setEditing(false);
  }, []);

  const clearData = useCallback(async () => {
    const ok = window.confirm(
      "ล้างคุกกี้และข้อมูลเว็บทั้งหมด?\nคุณจะออกจากบัญชี Google ทุกบัญชีใน Alux",
    );
    if (!ok) return;
    try {
      await api.clearBrowsingData();
    } catch (err) {
      console.error(err);
    }
    window.location.reload();
  }, []);

  const closeTab = useCallback(
    async (id: string) => {
      await api.tabClose(api.tabLabel(id)).catch(() => {});
      const remaining = tabs.filter((t) => t.id !== id);
      if (remaining.length === 0) {
        const fresh: Tab = { id: nextId(), url: null };
        setTabs([fresh]);
        setActiveId(fresh.id);
        return;
      }
      setTabs(remaining);
      if (activeId === id) setActiveId(remaining[remaining.length - 1].id);
    },
    [tabs, activeId],
  );

  // ---- follow in-page navigation (address bar stays truthful) ----------
  useEffect(() => {
    if (!activeTab?.url) return;
    const id = activeTab.id;
    const label = api.tabLabel(id);
    const expected = activeTab.url;
    const timer = window.setInterval(async () => {
      const current = await api.tabUrl(label).catch(() => "");
      if (current && current !== expected) {
        setTabs((prev) => prev.map((t) => (t.id === id ? { ...t, url: current } : t)));
      }
    }, 1200);
    return () => window.clearInterval(timer);
  }, [activeTab?.id, activeTab?.url]);

  // ---- shortcuts -------------------------------------------------------
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey)) return;
      const key = event.key.toLowerCase();
      if (key === "t") {
        event.preventDefault();
        newTab();
      } else if (key === "w") {
        event.preventDefault();
        void closeTab(activeId);
      } else if (key === "l") {
        event.preventDefault();
        addressRef.current?.focus();
        addressRef.current?.select();
      } else if (key === "r") {
        event.preventDefault();
        void action("reload");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [newTab, closeTab, activeId, action]);

  // ---- Alux Apps actions ----------------------------------------------
  const install = useCallback(
    async (source: string) => {
      await api.appsInstall(source);
      await refreshApps();
      setAppsOpen(true);
    },
    [refreshApps],
  );

  const remove = useCallback(
    async (id: string) => {
      await api.appsDelete(id);
      await refreshApps();
    },
    [refreshApps],
  );

  const openApp = useCallback(
    (app: AluxApp) => {
      void go(api.appUrl(app));
    },
    [go],
  );

  const installDemo = useCallback(async () => {
    try {
      const app = await api.appsInstallDemo();
      await refreshApps();
      setAppsOpen(false);
      await go(api.appUrl(app));
    } catch (err) {
      console.error(err);
      window.alert(String(err));
    }
  }, [go, refreshApps]);

  const shownAddress = editing ? address : (activeTab?.url ?? "");

  return (
    <div className="app">
      <div className="browser">
        <div className="tabstrip">
          <div className="brand-mark">A</div>
          <div className="tabs">
            {tabs.map((tab) => (
              <div
                key={tab.id}
                className={`tab${tab.id === activeId ? " active" : ""}`}
                onMouseDown={() => {
                  setActiveId(tab.id);
                  setEditing(false);
                }}
              >
                <span className="tab-dot" />
                <span className="tab-title">{tab.url ? prettyHost(tab.url) : "หน้าใหม่"}</span>
                <button
                  className="tab-close"
                  aria-label="ปิดแท็บ"
                  onMouseDown={(event) => {
                    event.stopPropagation();
                    void closeTab(tab.id);
                  }}
                >
                  <Cross size={11} />
                </button>
              </div>
            ))}
          </div>
          <button className="icon-button" onClick={newTab} title="แท็บใหม่ (Ctrl+T)">
            <Plus />
          </button>
          <div className="flex-spacer" />
          <button
            className={`pill${appsOpen ? " on" : ""}`}
            onClick={() => setAppsOpen((open) => !open)}
          >
            <Grid size={15} />
            Alux Apps
          </button>
        </div>

        <div className="toolbar">
          <button
            className="icon-button"
            onClick={() => void action("back")}
            disabled={!activeTab?.url}
            title="ย้อนกลับ"
          >
            <ArrowLeft />
          </button>
          <button
            className="icon-button"
            onClick={() => void action("forward")}
            disabled={!activeTab?.url}
            title="ไปข้างหน้า"
          >
            <ArrowRight />
          </button>
          <button
            className="icon-button"
            onClick={() => void action("reload")}
            disabled={!activeTab?.url}
            title="โหลดใหม่ (Ctrl+R)"
          >
            <Reload />
          </button>

          <form
            className="addressbar"
            onSubmit={(event) => {
              event.preventDefault();
              void go(shownAddress);
            }}
          >
            <span className="scheme">
              {isSecure(activeTab?.url ?? null) ? <Lock /> : <Shield />}
            </span>
            <input
              ref={addressRef}
              value={shownAddress}
              placeholder="ค้นหาด้วย Google หรือพิมพ์ที่อยู่เว็บ"
              spellCheck={false}
              autoComplete="off"
              onFocus={() => {
                setEditing(true);
                setAddress(activeTab?.url ?? "");
              }}
              onBlur={() => setEditing(false)}
              onChange={(event) => setAddress(event.target.value)}
            />
            {loading && <span className="spinner" aria-hidden />}
          </form>

          <button className="pill" onClick={() => setAppsOpen(true)} title="ช่องติดตั้งแอป">
            <Plus size={14} />
            ติดตั้งแอป
          </button>
        </div>

        <main className="viewport" ref={contentRef}>
          {!activeTab?.url && (
            <NewTabPage
              apps={apps}
              onSearch={(query) => void go(query)}
              onOpenApp={openApp}
              onOpenApps={() => setAppsOpen(true)}
              onInstallDemo={() => void installDemo()}
              onClearData={() => void clearData()}
            />
          )}
        </main>
      </div>

      {appsOpen && (
        <AppsPanel
          apps={apps}
          dir={dir}
          onClose={() => setAppsOpen(false)}
          onOpenApp={openApp}
          onInstall={install}
          onDelete={remove}
        />
      )}
    </div>
  );
}
