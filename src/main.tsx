import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

// No StrictMode: its double-invoked effects would create each tab webview twice.
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<App />);
