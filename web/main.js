const empty = document.querySelector("#empty");
const viewer = document.querySelector("#viewer");
const toc = document.querySelector("#toc");
const app = document.querySelector("#app");
const resizer = document.querySelector("#toc-resizer");
const invoke = window.__TAURI__.core.invoke;

document.querySelector(".titlebar").addEventListener("pointerdown", (event) => {
  if (event.button !== 0 || event.target.closest(".window-controls")) return;
  void invoke("start_dragging_window");
});

for (const button of document.querySelectorAll("[data-window-action]")) {
  button.addEventListener("click", () => {
    const action = button.dataset.windowAction;
    const command =
      action === "minimize"
        ? "minimize_window"
        : action === "maximize"
          ? "toggle_maximize_window"
          : "close_window";
    void invoke(command);
  });
}

document.addEventListener("click", (event) => {
  if (!(event.target instanceof Element)) return;
  const link = event.target.closest("a[href]");
  if (!link) return;

  const href = link.getAttribute("href") || "";
  if (!/^https?:\/\//i.test(href) && !/^mailto:/i.test(href)) return;

  event.preventDefault();
  void invoke("open_external_link", { url: href });
});

resizer.addEventListener("pointerdown", (event) => {
  event.preventDefault();
  resizer.setPointerCapture(event.pointerId);
  app.classList.add("resizing");

  const onMove = (moveEvent) => {
    const width = Math.max(0, Math.min(420, moveEvent.clientX));
    if (width < 96) {
      app.classList.add("toc-collapsed");
      app.style.setProperty("--toc-width", "0px");
    } else {
      app.classList.remove("toc-collapsed");
      app.style.setProperty("--toc-width", `${width}px`);
    }
  };

  const onUp = () => {
    app.classList.remove("resizing");
    resizer.removeEventListener("pointermove", onMove);
    resizer.removeEventListener("pointerup", onUp);
    resizer.removeEventListener("pointercancel", onUp);
  };

  resizer.addEventListener("pointermove", onMove);
  resizer.addEventListener("pointerup", onUp);
  resizer.addEventListener("pointercancel", onUp);
});

function renderToc(items) {
  toc.replaceChildren();

  if (!items.length) {
    const emptyState = document.createElement("div");
    emptyState.className = "toc-empty";
    emptyState.textContent = "No headings";
    toc.append(emptyState);
    return;
  }

  for (const item of items) {
    const link = document.createElement("a");
    link.href = `#${item.id}`;
    link.className = `toc-link toc-level-${item.level}`;
    link.textContent = item.text;
    link.addEventListener("click", (event) => {
      event.preventDefault();
      document.getElementById(item.id)?.scrollIntoView({ block: "start" });
      history.replaceState(null, "", `#${item.id}`);
    });
    toc.append(link);
  }
}

function showEmpty() {
  empty.hidden = false;
  viewer.hidden = true;
  viewer.replaceChildren();
}

function showDocument(markdownDocument) {
  empty.hidden = true;
  viewer.hidden = false;
  viewer.innerHTML = markdownDocument.html;
  enhanceCodeBlocks();
  viewer.dataset.path = markdownDocument.path;
  viewer.dataset.renderMs = String(markdownDocument.render_ms);
  document.title = `${markdownDocument.title || "Markdown Reader"} - Markdown Reader`;
}

function enhanceCodeBlocks() {
  for (const pre of viewer.querySelectorAll("pre")) {
    if (pre.parentElement?.classList.contains("code-shell")) continue;

    const shell = document.createElement("div");
    shell.className = "code-shell";

    const button = document.createElement("button");
    button.className = "copy-code";
    button.type = "button";
    button.textContent = "复制";
    button.addEventListener("click", async () => {
      const text = pre.textContent || "";
      try {
        await navigator.clipboard.writeText(text);
        button.textContent = "已复制";
        window.setTimeout(() => {
          button.textContent = "复制";
        }, 900);
      } catch {
        button.textContent = "失败";
        window.setTimeout(() => {
          button.textContent = "复制";
        }, 900);
      }
    });

    pre.replaceWith(shell);
    shell.append(button, pre);
  }
}

function showError(message) {
  empty.hidden = false;
  viewer.hidden = true;
  viewer.replaceChildren();
  empty.innerHTML = "";

  const title = document.createElement("h1");
  title.textContent = "Unable to open Markdown";
  const body = document.createElement("p");
  body.textContent = message;
  empty.append(title, body);
}

async function bootstrap() {
  try {
    const markdownDocument = await invoke("load_startup_document");
    if (!markdownDocument) {
      showEmpty();
      renderToc([]);
      return;
    }

    showDocument(markdownDocument);
    renderToc(markdownDocument.toc);
  } catch (error) {
    showError(error instanceof Error ? error.message : String(error));
    renderToc([]);
  }
}

void bootstrap();
