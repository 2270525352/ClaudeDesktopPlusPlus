(function claudePlusInject() {
  const markerId = "claude-plus-injected-marker";
  const panelId = "claude-plus-overlay";
  if (document.getElementById(markerId) || document.getElementById(panelId)) {
    return;
  }

  const payload = window.__CLAUDE_PLUS_PROVIDER__ || {};
  const providers = Array.isArray(payload.providers) ? payload.providers : [];

  window.__CLAUDE_PLUS__ = {
    injectedAt: new Date().toISOString(),
    version: "0.1.13",
    providerId: payload.providerId || null,
    gatewayUrl: payload.gatewayUrl || null,
  };

  const css = `
    #${panelId}, #${panelId} * { box-sizing: border-box; }
    #${panelId} {
      position: fixed;
      right: 14px;
      bottom: 14px;
      z-index: 2147483647;
      width: min(360px, calc(100vw - 28px));
      color: #dfffee;
      font: 12px/1.45 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      pointer-events: auto;
    }
    #${panelId} button {
      font: inherit;
      color: inherit;
      cursor: pointer;
    }
    #${panelId} .cp-launcher {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-height: 34px;
      padding: 0 12px;
      margin-left: auto;
      border: 1px solid rgba(66, 255, 155, .62);
      background: rgba(2, 12, 14, .94);
      color: #42ff9b;
      box-shadow: 0 0 18px rgba(66, 255, 155, .2), inset 0 0 14px rgba(66, 255, 155, .08);
    }
    #${panelId} .cp-card {
      display: none;
      margin-bottom: 10px;
      border: 1px solid rgba(32, 231, 255, .42);
      background: rgba(1, 8, 11, .96);
      box-shadow: 0 0 22px rgba(32, 231, 255, .18), inset 0 0 20px rgba(32, 231, 255, .06);
    }
    #${panelId}.open .cp-card { display: block; }
    #${panelId} .cp-head {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 10px;
      padding: 10px 12px;
      border-bottom: 1px solid rgba(32, 231, 255, .22);
    }
    #${panelId} .cp-title {
      color: #42ff9b;
      font-weight: 800;
      letter-spacing: 0;
    }
    #${panelId} .cp-close {
      width: 28px;
      min-height: 28px;
      border: 1px solid rgba(32, 231, 255, .34);
      background: rgba(0, 7, 10, .78);
      color: #20e7ff;
    }
    #${panelId} .cp-body {
      display: grid;
      gap: 9px;
      padding: 12px;
    }
    #${panelId} .cp-row {
      display: grid;
      grid-template-columns: 82px minmax(0, 1fr);
      gap: 8px;
      align-items: start;
    }
    #${panelId} .cp-label { color: #8bb0a8; }
    #${panelId} .cp-value {
      color: #e7fff8;
      overflow-wrap: anywhere;
    }
    #${panelId} .cp-ok { color: #42ff9b; }
    #${panelId} .cp-warn { color: #ffc857; }
    #${panelId} .cp-list {
      display: grid;
      gap: 6px;
      max-height: 170px;
      overflow: auto;
      padding-right: 2px;
    }
    #${panelId} .cp-provider {
      display: grid;
      gap: 2px;
      padding: 8px;
      border: 1px solid rgba(32, 231, 255, .18);
      background: rgba(4, 16, 18, .72);
    }
    #${panelId} .cp-provider.active {
      border-color: rgba(66, 255, 155, .48);
      background: rgba(66, 255, 155, .08);
    }
    #${panelId} .cp-provider strong {
      color: #e7fff8;
      font-size: 12px;
    }
    #${panelId} .cp-provider span {
      color: #8bb0a8;
      overflow-wrap: anywhere;
    }
    #${panelId} .cp-note {
      color: #ffc857;
      font-size: 11px;
      line-height: 1.5;
    }
  `;

  function text(value, fallback) {
    const raw = value == null || value === "" ? fallback : value;
    return String(raw == null ? "-" : raw);
  }

  function providerRows() {
    if (!providers.length) {
      return '<div class="cp-provider"><strong>未同步 Provider</strong><span>请回到 Claude++ 同步 cc-switch 配置。</span></div>';
    }
    return providers
      .map((provider) => {
        const name = text(provider.name || provider.baseUrl, "未命名");
        const baseUrl = text(provider.baseUrl, "-");
        const key = provider.hasApiKey ? text(provider.keyMask, "已配置 Key") : "无 Key";
        return `
          <div class="cp-provider${provider.active ? " active" : ""}">
            <strong>${escapeHtml(name)}${provider.active ? " / 当前" : ""}</strong>
            <span>${escapeHtml(baseUrl)}</span>
            <span>${escapeHtml(key)}</span>
          </div>
        `;
      })
      .join("");
  }

  function escapeHtml(value) {
    return String(value)
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#39;");
  }

  const root = document.createElement("section");
  root.id = panelId;
  root.innerHTML = `
    <div class="cp-card" role="dialog" aria-label="Claude++ Provider Switcher">
      <div class="cp-head">
        <div class="cp-title">Claude++ Provider Switcher</div>
        <button class="cp-close" type="button" aria-label="关闭">X</button>
      </div>
      <div class="cp-body">
        <div class="cp-row">
          <div class="cp-label">当前</div>
          <div class="cp-value cp-ok">${escapeHtml(text(payload.providerName, "未选择"))}</div>
        </div>
        <div class="cp-row">
          <div class="cp-label">Base URL</div>
          <div class="cp-value">${escapeHtml(text(payload.baseUrl, "-"))}</div>
        </div>
        <div class="cp-row">
          <div class="cp-label">Gateway</div>
          <div class="cp-value ${payload.gatewayEnabled ? "cp-ok" : "cp-warn"}">${escapeHtml(payload.gatewayEnabled ? text(payload.gatewayUrl, "已启用") : "未启用")}</div>
        </div>
        <div class="cp-row">
          <div class="cp-label">Key</div>
          <div class="cp-value">${escapeHtml(payload.hasApiKey ? "已配置" : "未配置")}</div>
        </div>
        <div class="cp-list">${providerRows()}</div>
        <div class="cp-note">第一阶段为安全只读浮层；切换 Provider 请先回 Claude++ 控制台执行，后续会接入本地控制 API。</div>
      </div>
    </div>
    <button id="${markerId}" class="cp-launcher" type="button">Claude++</button>
  `;

  const style = document.createElement("style");
  style.textContent = css;

  function attach() {
    if (!document.body || document.getElementById(panelId)) {
      return;
    }
    document.head.appendChild(style);
    document.body.appendChild(root);
    root.querySelector(".cp-launcher").addEventListener("click", () => {
      root.classList.toggle("open");
    });
    root.querySelector(".cp-close").addEventListener("click", () => {
      root.classList.remove("open");
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach, { once: true });
  } else {
    attach();
  }
})();
