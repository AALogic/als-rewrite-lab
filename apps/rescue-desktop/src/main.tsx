import React, { lazy, Suspense } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";

function currentSurface(): "main" | "quick-copy" {
  if (import.meta.env.DEV && new URLSearchParams(window.location.search).get("surface") === "quick-copy") {
    return "quick-copy";
  }
  try {
    return getCurrentWindow().label === "quick-copy" ? "quick-copy" : "main";
  } catch {
    return "main";
  }
}

const MainSurface = lazy(() => import("./App"));
const QuickSurface = lazy(() => import("./quick/QuickCopyAssistant").then((module) => ({
  default: module.QuickCopyAssistant,
})));
const RootSurface = currentSurface() === "quick-copy" ? QuickSurface : MainSurface;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Suspense fallback={null}>
      <RootSurface />
    </Suspense>
  </React.StrictMode>,
);
