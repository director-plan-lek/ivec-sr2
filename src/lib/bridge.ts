import { invoke } from "@tauri-apps/api/core";

export type Rect = { x: number; y: number; width: number; height: number };

export type AluxApp = {
  id: string;
  name: string;
  version: string;
  description: string;
  accent: string;
  entry: string;
};

export type TabAction = "back" | "forward" | "reload" | "stop";

/** Child webview label for a tab id. */
export const tabLabel = (id: string) => `tab-${id}`;

export const tabOpen = (label: string, url: string, rect: Rect) =>
  invoke<string>("tab_open", { label, url, rect });

export const tabSync = (rect: Rect, labels: string[], visible: string | null) =>
  invoke<void>("tab_sync", { rect, labels, visible });

export const tabNavigate = (label: string, url: string) =>
  invoke<string>("tab_navigate", { label, url });

export const tabAction = (label: string, action: TabAction) =>
  invoke<void>("tab_action", { label, action });

export const tabUrl = (label: string) => invoke<string>("tab_url", { label });

export const tabClose = (label: string) => invoke<void>("tab_close", { label });

export const appsList = () => invoke<AluxApp[]>("apps_list");
export const appsInstallDemo = () => invoke<AluxApp>("apps_install_demo");
export const appsInstall = (source: string) => invoke<AluxApp>("apps_install", { source });
export const appsDelete = (id: string) => invoke<void>("apps_delete", { id });
export const appsDir = () => invoke<string>("apps_dir");

/** Wipe cookies + site storage in every webview (signs out of Google etc.). */
export const clearBrowsingData = () => invoke<void>("clear_browsing_data");

/** URL under which an installed app is served by the `alux://` protocol. */
export const appUrl = (app: AluxApp) =>
  `alux://localhost/app/${encodeURIComponent(app.id)}/${app.entry || "index.html"}`;
