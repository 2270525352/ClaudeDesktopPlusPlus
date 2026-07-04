(function claudePlusChineseLocalization() {
  if (window.__CLAUDE_PLUS_CHINESE_LOCALIZATION__) return;
  window.__CLAUDE_PLUS_CHINESE_LOCALIZATION__ = true;

  const exact = new Map([
    ["Actual Size", "实际大小"],
    ["About", "关于"],
    ["About...", "关于..."],
    ["Add", "添加"],
    ["Allow", "允许"],
    ["Always Allow", "始终允许"],
    ["Apply", "应用"],
    ["Back", "返回"],
    ["Cancel", "取消"],
    ["Chat", "对话"],
    ["Chats", "对话"],
    ["Check again", "重新检查"],
    ["Checking for Updates...", "正在检查更新..."],
    ["Close", "关闭"],
    ["Code", "Code"],
    ["Configure", "配置"],
    ["Configure Third-Party Inference...", "配置第三方推理..."],
    ["Connected", "已连接"],
    ["Connect", "连接"],
    ["Continue", "继续"],
    ["Copy", "复制"],
    ["Copy Link", "复制链接"],
    ["Cowork", "Cowork"],
    ["Cut", "剪切"],
    ["Debug", "调试"],
    ["Delete", "删除"],
    ["Delete and Restart", "删除并重启"],
    ["Developer", "开发者"],
    ["Disable", "停用"],
    ["Done", "完成"],
    ["Edit", "编辑"],
    ["Enable", "启用"],
    ["Error", "错误"],
    ["Exit", "退出"],
    ["Extensions", "插件"],
    ["File", "文件"],
    ["Find", "查找"],
    ["Forget", "忘记"],
    ["Help", "帮助"],
    ["Install", "安装"],
    ["Learn Spelling", "学习拼写"],
    ["Leave", "离开"],
    ["Loading", "加载中"],
    ["Log in", "登录"],
    ["Login", "登录"],
    ["Logout", "退出登录"],
    ["Move to Applications folder?", "移动到 Applications 文件夹？"],
    ["New chat", "新建对话"],
    ["New Chat", "新建对话"],
    ["Next", "下一步"],
    ["No", "否"],
    ["Not now", "稍后"],
    ["Open", "打开"],
    ["Open Link in Browser", "在浏览器中打开链接"],
    ["Paste", "粘贴"],
    ["Plan usage", "套餐用量"],
    ["Preview", "预览"],
    ["Projects", "项目"],
    ["Quit", "退出"],
    ["Redo", "重做"],
    ["Reload", "重新加载"],
    ["Remove", "移除"],
    ["Rename", "重命名"],
    ["Reset", "重置"],
    ["Restart", "重启"],
    ["Retry", "重试"],
    ["Save", "保存"],
    ["Search", "搜索"],
    ["Select All", "全选"],
    ["Settings", "设置"],
    ["Sign in", "登录"],
    ["Sign out", "退出登录"],
    ["Submit", "提交"],
    ["Submit Feedback - Claude", "提交反馈 - Claude"],
    ["Team", "团队"],
    ["Troubleshooting", "故障排查"],
    ["Undo", "撤销"],
    ["Update", "更新"],
    ["View", "视图"],
    ["Wait for Claude", "等待 Claude"],
    ["Window", "窗口"],
    ["Yes", "是"],
    ["Your network redirected this request to www.anthropic.com. Contact your IT administrator.", "你的网络把此请求重定向到了 www.anthropic.com。请联系 IT 管理员。"],
    ["Another copy of Claude is already running", "另一个 Claude 实例已经在运行"],
    ["You are not logged in. Please log in to access the extensions directory.", "你尚未登录。请登录后访问插件目录。"],
    ["A new version is available. It will be downloaded and installed automatically.", "发现新版本，将自动下载并安装。"],
    ["Reinstall required", "需要重新安装"],
    ["Virtual Machine Platform not available", "Virtual Machine Platform 不可用"],
    ["Open Setup", "打开设置"],
    ["Details", "详情"],
  ]);

  const patterns = [
    [/^Open (.+)$/i, "打开 $1"],
    [/^Delete (.+)$/i, "删除 $1"],
    [/^Install (.+)$/i, "安装 $1"],
    [/^Failed to (.+)$/i, "无法$1"],
    [/^(.+) failed$/i, "$1失败"],
    [/^(.+) copied to clipboard$/i, "$1已复制到剪贴板"],
  ];

  const textSelector = [
    "button",
    "[role='button']",
    "[role='menuitem']",
    "[role='tab']",
    "[role='option']",
    "label",
    "nav span",
    "nav div",
    "aside span",
    "aside div",
    "header span",
    "h1",
    "h2",
    "h3",
    "h4",
    "[data-testid] span",
    "[data-testid] div",
  ].join(",");

  const skipSelector = [
    "textarea",
    "input",
    "pre",
    "code",
    "[contenteditable='true']",
    ".ProseMirror",
    "[data-claude-plus-no-localize]",
  ].join(",");

  function translate(value) {
    if (!value) return value;
    const left = value.match(/^\s*/)?.[0] || "";
    const right = value.match(/\s*$/)?.[0] || "";
    const core = value.trim();
    if (!core || core.length > 160) return value;
    if (exact.has(core)) return `${left}${exact.get(core)}${right}`;
    for (const [regex, replacement] of patterns) {
      if (regex.test(core)) return `${left}${core.replace(regex, replacement)}${right}`;
    }
    return value;
  }

  function shouldSkip(element) {
    return !element || element.closest?.(skipSelector);
  }

  function localizeAttributes(element) {
    for (const attr of ["aria-label", "title", "placeholder", "alt"]) {
      if (!element.hasAttribute?.(attr)) continue;
      const current = element.getAttribute(attr);
      const next = translate(current);
      if (next !== current) element.setAttribute(attr, next);
    }
  }

  function localizeElement(element) {
    if (!(element instanceof Element) || shouldSkip(element)) return;
    localizeAttributes(element);
    if (!element.matches(textSelector)) return;
    const textNodes = Array.from(element.childNodes).filter((node) => node.nodeType === Node.TEXT_NODE);
    if (textNodes.length !== 1 || element.children.length > 1) return;
    const node = textNodes[0];
    const next = translate(node.nodeValue || "");
    if (next !== node.nodeValue) node.nodeValue = next;
  }

  function scan(root = document.body) {
    if (!root) return;
    if (root instanceof Element) localizeElement(root);
    root.querySelectorAll?.(`${textSelector}, [aria-label], [title], [placeholder], [alt]`).forEach(localizeElement);
  }

  let scheduled = false;
  function schedule() {
    if (scheduled) return;
    scheduled = true;
    window.setTimeout(() => {
      scheduled = false;
      scan();
    }, 120);
  }

  document.documentElement.lang = "zh-CN";
  scan();
  new MutationObserver(schedule).observe(document.documentElement, {
    childList: true,
    subtree: true,
    characterData: true,
    attributes: true,
    attributeFilter: ["aria-label", "title", "placeholder", "alt"],
  });
})();
