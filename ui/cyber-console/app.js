(function () {
  const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const canvas = document.getElementById("matrixRain");
  const ctx = canvas.getContext("2d");
  const glyphs = "01ABCDEF0123456789";
  let columns = [];
  let width = 0;
  let height = 0;
  let rainTimer = null;
  let visualFxEnabled = window.localStorage.getItem("claude-plus-visual-fx") === "on";

  const translations = {
    zh: {
      tagline: "桌面增强控制塔",
      navOverview: "总览",
      navSystem: "系统就绪",
      navCapabilities: "内置能力",
      navTools: "能力插件",
      navLaunch: "启动诊断",
      navProviders: "API 配置",
      navRecommendations: "推荐",
      navHistory: "历史对话",
      navSandbox: "安全策略",
      navAbout: "关于",
      navLogs: "运行日志",
      systemNode: "SYSTEM NODE 07",
      overviewTitle: "Claude 体检报告",
      systemPageTitle: "系统就绪",
      capabilitiesPageTitle: "内置能力",
      toolsPageTitle: "能力插件",
      launchPageTitle: "Claude 启动",
      providersPageTitle: "API 配置",
      recommendationsPageTitle: "推荐",
      historyPageTitle: "历史对话修复",
      sandboxPageTitle: "安全策略",
      aboutPageTitle: "关于",
      logsPageTitle: "运行日志",
      refresh: "刷新状态",
      loading: "扫描中",
      launchClaude: "诊断启动 Claude Desktop",
      launchClaudeInjected: "启动 Claude Desktop",
      launchClaudeClean: "诊断启动",
      connectionMode: "连接模式",
      directModeButton: "直连",
      gatewayModeButton: "Gateway",
      systemReadyTitle: "系统就绪检查",
      windowsStatus: "Windows 版本",
      adminStatus: "管理员权限",
      firmwareVirtualization: "固件虚拟化",
      hypervisorLaunchType: "Hypervisor 启动项",
      hypervisorRuntime: "Hypervisor 运行",
      modernInstallStatus: "Modern Installer",
      appxPackage: "Appx 包",
      vmpStatus: "Virtual Machine Platform",
      hypervisorStatus: "Hypervisor Platform",
      rebootStatus: "重启需求",
      relaunchAsAdmin: "以管理员重新打开",
      installClaudeModern: "一键安装 Claude Desktop",
      enableVmp: "一键启用 VMP",
      systemReadyHint: "Cowork 需要 modern installer、VMP 和一次系统重启。",
      doctorTitle: "Claude 体检报告",
      doctorSubtitle: "检查安装、系统、API、插件、汉化和历史状态。",
      doctorPrimaryAction: "一键修复并启动",
      doctorNextQueue: "建议处理顺序",
      doctorNextTitle: "下一步",
      doctorScanning: "扫描中",
      doctorReady: "就绪",
      doctorNeedsAction: "需处理",
      doctorError: "异常",
      doctorReadyVerdict: "Claude Desktop 已准备好",
      doctorIssuesVerdict: "发现 {count} 个待处理项",
      doctorScanningVerdict: "正在补齐体检数据",
      doctorReadyDetail: "可以按当前配置启动 Claude Desktop。",
      doctorIssueDetail: "优先处理：{item}",
      doctorNoState: "等待桌面桥返回状态。",
      doctorInstallTitle: "Claude Desktop",
      doctorSystemTitle: "系统能力",
      doctorApiTitle: "API 渠道",
      doctorPluginTitle: "插件能力",
      doctorLocalizationTitle: "汉化状态",
      doctorHistoryTitle: "历史对话",
      doctorInstallOk: "已检测到 Claude Desktop",
      doctorInstallMissing: "未检测到 Claude Desktop",
      doctorSystemScanning: "正在读取 Windows / macOS 系统能力",
      doctorSystemReady: "系统能力满足当前启动需求",
      doctorSystemNeedsModern: "需要 modern installer 才能完整使用 Cowork",
      doctorSystemNeedsVmp: "Virtual Machine Platform 未启用",
      doctorSystemNeedsRestart: "系统功能已暂存，需要重启",
      doctorSystemMeta: "VMP: {vmp} / Hypervisor: {hypervisor}",
      doctorApiReady: "当前 API 可用于启动",
      doctorApiMissing: "还没有当前 API 配置",
      doctorApiNeedsConfig: "当前 API 缺少可用 Base URL 或 Key",
      doctorApiOpenAIDirectWarning: "OpenAI / Codex 直连依赖上游模型映射",
      doctorApiMeta: "{protocol} / {mode}",
      doctorPluginScanning: "正在读取内置能力和插件目录",
      doctorPluginReady: "插件运行环境和官方目录可用",
      doctorPluginNeedsSetup: "插件或内置能力还未准备好",
      doctorPluginMeta: "官方目录 {count} 个插件",
      doctorLocalizationReady: "中文资源补丁已启用",
      doctorLocalizationMissing: "未启用中文资源补丁",
      doctorHistoryScanning: "正在扫描本地历史来源",
      doctorHistoryReady: "没有发现需要修复的本地历史",
      doctorHistoryRecoverable: "发现可恢复历史来源",
      doctorHistoryMeta: "目标：{target}",
      doctorOpenSystem: "系统就绪",
      doctorOpenProviders: "API 配置",
      doctorOpenTools: "能力插件",
      doctorOpenHistory: "历史修复",
      doctorQueueEmpty: "暂无待处理项。",
      doctorQueueLaunchReady: "启动 Claude Desktop",
      doctorQueueGo: "处理",
      capabilitiesTitle: "Claude Desktop 内置能力中心",
      enableDeveloperCapabilities: "一键启用内置能力",
      capabilityBrowserMcp: "浏览器控制能力",
      capabilityWorkspaceMcp: "工作区文件能力",
      capabilityNpx: "插件运行时",
      capabilityWorkspace: "默认工作区",
      capabilityConfigTargets: "能力安装位置",
      officialPluginsTitle: "官方插件市场",
      officialPluginsSync: "同步官方目录",
      officialPluginsInstall: "安装",
      officialPluginsInstalled: "已安装",
      officialPluginsCli: "Claude CLI",
      officialPluginsMarketplace: "官方目录",
      officialPluginsCount: "插件数量",
      officialPluginsLastUpdated: "最近同步",
      officialPluginsEmpty: "没有读取到官方插件目录",
      officialPluginsSyncing: "正在同步官方插件目录...",
      officialPluginsInstalling: "正在安装 {plugin}...",
      officialPluginsSynced: "官方插件目录已同步",
      officialPluginsInstallOk: "官方插件已安装：{plugin}",
      officialPluginsInstallFailed: "官方插件安装失败：{plugin}",
      officialPluginsHint: "使用 Claude 官方插件市场和 CLI 安装本地插件。组织后台的插件列表仍由 Claude 账号/组织策略决定。",
      officialPluginsSearch: "搜索插件",
      officialPluginsSearchPlaceholder: "搜索名称、分类、作者或说明",
      officialPluginsLoadMore: "加载更多",
      officialPluginsVisibleCount: "显示 {visible} / {total}",
      officialPluginsNoResults: "没有匹配的插件",
      officialPluginsInstalls: "{count} 次安装",
      openChromeConnector: "打开 Chrome 连接器",
      openChromeGuide: "Chrome 控制指南",
      capabilitiesHint: "写入 Claude Desktop 开发者能力配置，重启后生效。",
      capabilitiesApplied: "内置能力配置已写入",
      capabilityReady: "已启用",
      capabilityMissing: "未启用",
      capabilityWritable: "可写",
      capabilityNeedsRestart: "重启 Claude Desktop 后生效",
      adminOk: "已获得",
      adminMissing: "未获得，系统功能启用可能失败",
      boolYes: "是",
      boolNo: "否",
      unknown: "未知",
      modernInstallOk: "已安装 MSIX / modern installer",
      modernInstallMissing: "未检测到 modern installer",
      featureEnabled: "已启用",
      featureDisabled: "未启用",
      featurePending: "已暂存，等待重启",
      rebootRequired: "需要重启",
      rebootNotRequired: "暂不需要",
      systemActionOk: "系统操作完成：{message}",
      systemActionFailed: "系统操作失败：{message} / exit {code}",
      systemActionRunning: "正在执行系统操作...",
      adminRelaunchRequested: "已请求管理员权限窗口，请在 UAC 中确认",
      vmpLocalHintReady: "系统虚拟化组件已就绪",
      vmpLocalHintRestart: "Windows 已暂存组件，请重启后再检查",
      vmpLocalHintFirmware: "固件虚拟化未开启，请先在 BIOS/UEFI 中开启 VT-x/SVM",
      vmpLocalHintAdmin: "需要管理员权限运行本工具",
      vmpLocalHintMissing: "VMP 仍未启用，请查看下方 DISM 诊断",
      installStatus: "Claude 安装",
      ccSwitchStatus: "cc-switch 同步",
      activeProvider: "当前 API",
      sandboxStatus: "沙盒策略",
      gatewayStatus: "Gateway 网关",
      directCompatibilityTitle: "直连兼容提醒",
      directCompatibilityHint: "OpenAI / Codex 直连需要上游已做好 Claude 模型映射；否则用 Gateway。",
      quickActions: "快捷操作",
      nextSteps: "下一步",
      syncCcSwitch: "同步 cc-switch 配置",
      syncRunning: "正在同步 cc-switch...",
      fxPerformanceMode: "性能模式",
      fxVisualMode: "视觉模式",
      launchTitle: "启动诊断",
      launcherRoute: "启动方式",
      injectionChannel: "配置方式",
      liveInjection: "实时脚本",
      installPath: "安装路径",
      workingDir: "工作目录",
      launchPolicy: "启动策略",
      verificationTitle: "启动验证",
      verificationWaiting: "等待启动",
      verificationVerdict: "验证结论",
      verificationNoRun: "还没有启动验证",
      verificationGatewayHit: "已验证：Claude Desktop 请求命中了本地 Gateway",
      verificationDirect3p: "已验证：Claude Desktop 正在使用直连 3P Provider 配置",
      verificationProviderRejected: "Gateway 已命中，但上游拒绝凭据；请更换 Key 或启用该 Key 所属分组",
      verificationUpstreamError: "Gateway 已命中，但上游返回错误；请查看上游状态和错误摘要",
      verificationConfigNoHit: "3P 配置已应用，但 Claude 暂未请求 Gateway",
      verificationStill1p: "未生效：Claude 仍在官方登录/1P 网络路径",
      verificationNotVerified: "未验证：没有看到 3P 日志或网关请求",
      gatewayRequests: "网关请求",
      gatewayForwarded: "转发成功",
      gatewayLastRequest: "最近请求",
      gatewayUpstreamStatus: "上游状态",
      gatewayUpstreamError: "上游错误",
      claude3pApplied: "3P 配置状态",
      claude3pDeployment: "Deployment Mode",
      claude3pAppliedId: "Applied ID",
      claude3pDesktopConfig: "桌面配置",
      claude3pMetaPath: "配置索引",
      claude3pConfigPath: "Gateway 配置",
      claude3pReady: "已应用",
      claude3pMissing: "未应用",
      filePresent: "存在",
      fileMissing: "缺失",
      ccSwitchImport: "同步已有配置",
      ccSwitchRoot: "配置目录",
      ccSwitchDb: "数据库",
      ccSwitchProviders: "可同步配置",
      providerList: "第三方 API",
      providerTestTitle: "凭据测试",
      testActiveProvider: "测试当前 API",
      providerTestTarget: "测试目标",
      providerTestStatus: "HTTP 状态",
      providerTestCode: "错误码",
      providerTestMessage: "错误说明",
      providerTestBody: "响应摘要",
      manualProvider: "手动添加",
      editProvider: "编辑配置",
      editingProvider: "编辑 API 配置",
      addProvider: "添加 API",
      providerModalTitle: "API 配置",
      cancelEdit: "取消编辑",
      providerFormHint: "保存后会设为当前 API；编辑已有配置时，Key 留空会保留原 Key。",
      providerName: "名称",
      appType: "类型",
      baseUrl: "Base URL",
      apiKey: "API Key",
      enabled: "启用",
      saveProvider: "保存并设为当前",
      modelMappingTitle: "模型映射",
      modelMappingHint: "Claude Desktop 需要 Claude 风格路由；这里把这些路由映射到真实上游模型。",
      addModelMapping: "添加映射",
      discoverModels: "获取模型",
      discoveringModels: "正在根据 URL 和 Key 获取模型...",
      modelDiscoveryStatusIdle: "根据 Base URL 和 API Key 自动获取模型。",
      modelDiscoveryOk: "已获取 {count} 个模型：{models}",
      modelDiscoveryFailed: "获取模型失败：{error}",
      mappingClaudeRoute: "Claude 路由",
      mappingTargetModel: "真实模型",
      mappingLabel: "显示名",
      mappingEnabled: "启用",
      removeMapping: "删除",
      sandboxTitle: "安全策略",
      saveSandbox: "保存安全策略",
      gatewayTitle: "3P 连接模式",
      enableGateway: "使用本地 Gateway 转发",
      gatewayPort: "监听端口",
      gatewayUrl: "网关地址",
      gatewayTarget: "转发目标",
      saveGateway: "保存网关设置",
      startGateway: "启动网关",
      stopGateway: "停止网关",
      gatewayBoundary: "Gateway 只负责把本地 API 请求转发到当前 provider；直连模式可在顶部切换。",
      injectProvider: "使用 API Base URL",
      injectApiKey: "使用 API Key",
      relaxSandbox: "允许 --no-sandbox 启动（高级）",
      sandboxAck: "我理解解除沙盒会降低隔离强度，只在可信环境使用",
      officialCapabilityTitle: "内置能力入口",
      officialCapabilityHint: "Cowork、Code、浏览器控制和插件市场能力已移到“内置能力”页面统一启用。",
      sandboxWarning: "这里只保留高级启动安全开关。",
      logsTitle: "运行日志",
      clearLogs: "清空日志",
      bridgeChecking: "桌面桥检测中",
      bridgePreview: "浏览器预览模式：桌面命令仅在 Tauri 应用中执行",
      bridgeConnected: "桌面桥已连接",
      installFound: "已安装",
      installMissing: "未找到",
      ccSwitchFound: "找到 {count} 个配置",
      ccSwitchMissing: "未找到 cc-switch",
      noProvider: "未配置",
      sandboxDefault: "默认隔离",
      sandboxRelaxed: "已请求解除",
      gatewayRunning: "运行中 {url}",
      gatewayStopped: "未运行",
      gatewayDisabled: "已关闭",
      direct3pMode: "直连 3P Provider",
      providerInjected: "Base URL 已启用",
      keyInjected: "Key 已启用",
      launchDefaultPolicy: "默认使用当前 API 配置启动",
      launchInjectPolicy: "使用当前 API 配置：{provider}",
      launchNoActiveProvider: "还没有当前 provider；请先同步 cc-switch 或手动添加 API 配置",
      launchNeedsInjectableProvider: "当前 provider 没有可用的 Base URL/Key，或启动策略未开启",
      launchAutoSwitched: "当前 provider 不可用，已切换到 {provider}",
      providerEmpty: "还没有 API 配置。可以同步 cc-switch 或手动添加。",
      providerActive: "当前",
      providerUse: "设为当前",
      providerInjectLaunch: "启动",
      providerEdit: "编辑",
      providerDelete: "删除",
      providerTest: "测试凭据",
      providerSource: "来源",
      providerTypeClaude: "Claude 兼容 API",
      providerTypeClaudeDesktop: "Claude Desktop 配置",
      providerProtocol: "协议",
      providerProtocolAnthropic: "Anthropic 兼容",
      providerProtocolOpenAI: "OpenAI / Codex 兼容",
      providerProtocolLabel: "协议",
      providerCompatibility: "兼容性",
      providerModels: "模型",
      providerTestLocalResult: "测试结果",
      providerKey: "Key",
      providerInjectable: "可用于启动",
      providerSwitchOnly: "仅切换，无 API 信息",
      providerEnabled: "启用",
      providerDisabled: "停用",
      providerSaved: "API 配置已保存",
      providerTestOk: "凭据可用：{provider} 返回 HTTP {status}",
      providerTestFailed: "凭据被拒绝或不可用：{provider} / HTTP {status} / {code} / {message}",
      providerEditLoaded: "已载入 {provider}，修改后保存即可替换配置",
      providerDeleteConfirm: "删除 API 配置「{provider}」？",
      providerDeleted: "已删除 API 配置：{provider}",
      syncDone: "cc-switch 同步完成：新增 {imported}，更新 {updated}，删除 {removed}",
      launchDone: "Claude Desktop 已诊断启动，进程 {pid}",
      injectLaunchDone: "Claude Desktop 已启动，进程 {pid}",
      injectLaunchVerified: "Claude Desktop 已启动，进程 {pid}，验证：{verdict}",
      cdpInjected: "实时能力脚本已启用，端口 {port}",
      cdpFailed: "实时能力脚本失败：{error}",
      cdpMsixUnavailable: "当前 MSIX 版 Claude Desktop 已改用 Claude-3p 配置方式",
      localizationTitle: "一键汉化",
      localizationStatus: "汉化状态",
      localizationRuntime: "生效通道",
      localizationNotInstalled: "未安装",
      localizationEnabled: "已启用",
      localizationDisabled: "已停用",
      localizationResourcePatch: "资源补丁 / zh-CN",
      enableChineseLocalization: "一键启用汉化",
      disableChineseLocalization: "停用汉化",
      localizationHint: "安装 zh-CN 语言资源并写入 Claude locale；Cowork 兼容模式，不修改 app.asar。",
      localizationEnabledLog: "汉化资源补丁已安装；重启 Claude Desktop 后以 zh-CN 加载",
      localizationDisabledLog: "汉化资源补丁已移除；语言已恢复 en-US",
      launcherRouteExternalProcess: "进程启动",
      launcherRouteAppActivation: "系统应用激活",
      launcherRouteLocalizedSidecar: "本地汉化启动",
      injectionDiagnosticClean: "诊断启动",
      injectionLiveGateway: "实时脚本 + Gateway",
      injectionLiveDirect: "实时脚本 + 直连",
      injectionPreloadGateway: "预加载脚本 + Gateway",
      injectionPreloadDirect: "预加载脚本 + 直连",
      injectionConfigGateway: "配置启动 + Gateway",
      injectionConfigDirect: "配置启动 + 直连",
      liveInjectionReady: "支持实时脚本",
      preloadInjectionReady: "预加载脚本已启用",
      liveInjectionConfigOnly: "使用配置启动",
      liveInjectionAttempted: "已尝试实时脚本",
      sandboxSaved: "沙盒策略已保存",
      gatewaySaved: "网关设置已保存",
      directModeSaved: "已切换为直连 3P Provider",
      gatewayModeSaved: "已切换为本地 Gateway 转发",
      directModeOpenAIWarning: "已强制直连。注意：OpenAI/Codex 直连需要上游支持 Anthropic /v1/messages，否则 Claude Desktop 可能不可用。",
      gatewayStarted: "Gateway 已启动：{url}",
      gatewayStoppedLog: "Gateway 已停止",
      refreshDone: "状态已刷新",
      commandUnavailable: "桌面桥不可用，当前只运行 UI 预览",
      commandFailed: "命令失败：{error}",
      requiredName: "请填写名称",
      requiredUrl: "请填写 Base URL",
      sandboxNeedAck: "解除沙盒前必须勾选确认",
      toolsTitle: "能力插件",
      toolsBoundary: "管理 Claude++ 能力脚本。",
      scriptEditor: "插件脚本编辑器",
      scriptName: "脚本名称",
      scriptCode: "脚本代码",
      saveScript: "保存脚本",
      scriptEmpty: "还没有能力脚本。",
      scriptSaved: "能力插件脚本已保存",
      scriptEnabled: "启用",
      scriptDisabled: "停用",
      accountBoundary: "Cowork、Code、浏览器控制和插件市场能力请在“内置能力”页面统一启用。",
      historyTitle: "一键修复历史对话",
      historyRefresh: "重新扫描",
      historyTarget: "修复目标",
      historyBackupRoot: "备份目录",
      historyRecommendedSource: "推荐来源",
      historySources: "诊断来源",
      historyRepairLogTitle: "修复结果",
      historyRepairHint: "自动选择最近的可恢复 Claude 历史来源，先备份当前 Claude-3p，再恢复聊天 IndexedDB、附件和本地 Code/Cowork 会话。",
      historyCurrent: "当前目标",
      historyEmpty: "没有扫描到可恢复的本地历史来源",
      historyRepair: "修复到当前 Claude-3p",
      historyRepairOneClick: "一键修复",
      historyFiles: "文件",
      historyBytes: "容量",
      historyLatest: "最近更新",
      historyItems: "包含",
      historyNoItems: "未发现可恢复项",
      historyDefaultSkipped: "Local/Session Storage 仅扫描，默认不恢复",
      historyShowDetails: "查看诊断详情",
      historyAutoReady: "将从 {source} 修复历史对话",
      historyAutoMissing: "没有可用于一键修复的历史来源",
      historyRecoverableSummary: "{items} / {files} 个文件 / {size}",
      historyRepairRunning: "正在修复历史，本步骤会关闭 Claude Desktop...",
      historyRepairDone: "历史修复完成：{files} 个文件，备份在 {backup}",
      historyRepairFailed: "历史修复失败：{error}",
      historyBoundary: "只修复本机缓存和工作会话；云端账号历史仍由 Claude 官方账号决定。",
      historyScanDone: "历史来源已重新扫描",
      historyScanIdle: "尚未扫描，点击重新扫描获取历史来源。",
      recommendationsTitle: "独家赞助商",
      recommendationsHint: "推荐页用于展示赞助商内容；AI 快捷配置已统一移动到 API 配置页面。",
      sponsorBadge: "OFFICIAL RELAY",
      sponsorEyebrow: "独家赞助商",
      sponsorBody: "JOJO Code 是 Codex++ 官方中转站，提供价格划算、稳定易接入的 Codex API 中转服务，支持 GPT-5.5、GPT-5.4、Claude Opus 4.8、Claude Opus 4.7、gpt-image-2 等模型与图像能力，适合日常开发、快速配置、团队协作和长期使用。",
      sponsorVisit: "访问 JOJO Code",
      externalOpened: "已打开：{url}",
      quickProviderTitle: "AI 快捷配置",
      recommendedModesTitle: "推荐使用模式",
      recommendationUse: "填入配置",
      recommendationSave: "保存为配置",
      recommendationDocs: "文档",
      recommendationBase: "Base URL",
      recommendationProtocol: "协议",
      recommendationDirectReady: "适合直连",
      recommendationNeedsAdapter: "需要适配网关",
      recommendationFilled: "已填入 {provider}，补上 API Key 后保存即可",
      recommendationSaved: "{provider} 已保存为当前配置",
      modeOfficialTitle: "官方账号模式",
      modeOfficialBody: "需要 Cowork、Code、浏览器控制和插件市场能力时，使用“内置能力”页面启用，并保持 Claude Desktop 系统就绪。",
      modeDirectTitle: "直连 3P 模式",
      modeDirectBody: "选择 Anthropic-compatible Provider，关闭 Gateway，减少本地转发层，适合追求稳定和低延迟。",
      modeGatewayTitle: "Gateway/适配模式",
      modeGatewayBody: "当 Provider 只支持 OpenAI-compatible 协议时，本地 Gateway 会自动转换 Claude Desktop 请求。",
      aboutTitle: "关于 Claude++",
      aboutVersion: "当前版本",
      aboutInstall: "安装位置",
      aboutConfig: "配置文件",
      aboutBoundary: "Claude++ 负责本地配置、增强启动、内置能力、历史修复和系统诊断。",
      aboutFeaturesTitle: "主要功能",
      featureCcSwitchTitle: "cc-switch 同步",
      featureCcSwitchBody: "读取并同步 cc-switch 配置，支持在 Claude++ 中切换和启动。",
      featureProviderTitle: "第三方 API 配置",
      featureProviderBody: "支持直连 3P Provider 和本地 Gateway 适配，并提供凭据测试。",
      featureLaunchTitle: "Claude Desktop 启动",
      featureLaunchBody: "Provider 配置启动、实时能力脚本、MSIX 检测和系统就绪检查。",
      featureHistoryTitle: "历史修复",
      featureHistoryBody: "扫描本机 Claude 历史缓存，带备份地恢复 IndexedDB、附件和本地工作会话。",
      featureSystemTitle: "系统就绪",
      featureSystemBody: "检测 modern installer、VMP、Hypervisor、管理员权限和重启状态。",
      featureBoundaryTitle: "内置能力",
      featureBoundaryBody: "启用 Claude Desktop 官方开发者模式、浏览器控制、工作区文件访问和插件运行入口。",
      featureToolsTitle: "能力插件",
      featureToolsBody: "内置一键汉化，并接入 Claude 官方插件市场。",
    },
    en: {
      tagline: "Desktop enhancement control tower",
      navOverview: "Overview",
      navSystem: "System Ready",
      navCapabilities: "Built-in Capabilities",
      navTools: "Plugins",
      navLaunch: "Launch Diagnostics",
      navProviders: "API Config",
      navRecommendations: "Recommended",
      navHistory: "History",
      navSandbox: "Security",
      navAbout: "About",
      navLogs: "Run Logs",
      systemNode: "SYSTEM NODE 07",
      overviewTitle: "Claude Health Report",
      systemPageTitle: "System Ready",
      capabilitiesPageTitle: "Built-in Capabilities",
      toolsPageTitle: "Plugins",
      launchPageTitle: "Claude Launch",
      providersPageTitle: "API Config",
      recommendationsPageTitle: "Sponsor",
      historyPageTitle: "History Repair",
      sandboxPageTitle: "Security",
      aboutPageTitle: "About",
      logsPageTitle: "Run Logs",
      refresh: "Refresh State",
      loading: "Scanning",
      launchClaude: "Diagnostic Launch Claude Desktop",
      launchClaudeInjected: "Start Claude Desktop",
      launchClaudeClean: "Diagnostic Launch",
      connectionMode: "Connection",
      directModeButton: "Direct",
      gatewayModeButton: "Gateway",
      systemReadyTitle: "System Readiness",
      windowsStatus: "Windows Version",
      adminStatus: "Administrator",
      firmwareVirtualization: "Firmware Virtualization",
      hypervisorLaunchType: "Hypervisor Launch",
      hypervisorRuntime: "Hypervisor Runtime",
      modernInstallStatus: "Modern Installer",
      appxPackage: "Appx Package",
      vmpStatus: "Virtual Machine Platform",
      hypervisorStatus: "Hypervisor Platform",
      rebootStatus: "Reboot",
      relaunchAsAdmin: "Relaunch as Admin",
      installClaudeModern: "Install Claude Desktop",
      enableVmp: "Enable VMP",
      systemReadyHint: "Cowork needs the modern installer, VMP, and one Windows restart.",
      doctorTitle: "Claude Health Report",
      doctorSubtitle: "Checks installation, system, API, plugins, localization, and history.",
      doctorPrimaryAction: "Fix and Launch",
      doctorNextQueue: "Suggested Order",
      doctorNextTitle: "Next Step",
      doctorScanning: "Scanning",
      doctorReady: "Ready",
      doctorNeedsAction: "Needs action",
      doctorError: "Error",
      doctorReadyVerdict: "Claude Desktop is ready",
      doctorIssuesVerdict: "{count} item(s) need attention",
      doctorScanningVerdict: "Completing health checks",
      doctorReadyDetail: "Claude Desktop can be launched with the current setup.",
      doctorIssueDetail: "Handle first: {item}",
      doctorNoState: "Waiting for the desktop bridge state.",
      doctorInstallTitle: "Claude Desktop",
      doctorSystemTitle: "System",
      doctorApiTitle: "API Provider",
      doctorPluginTitle: "Plugin Runtime",
      doctorLocalizationTitle: "Chinese UI",
      doctorHistoryTitle: "History",
      doctorInstallOk: "Claude Desktop detected",
      doctorInstallMissing: "Claude Desktop was not detected",
      doctorSystemScanning: "Reading Windows / macOS system capabilities",
      doctorSystemReady: "System capabilities are ready for launch",
      doctorSystemNeedsModern: "Modern installer is required for full Cowork support",
      doctorSystemNeedsVmp: "Virtual Machine Platform is not enabled",
      doctorSystemNeedsRestart: "System features are staged; restart required",
      doctorSystemMeta: "VMP: {vmp} / Hypervisor: {hypervisor}",
      doctorApiReady: "The active API can be used for launch",
      doctorApiMissing: "No active API provider is configured",
      doctorApiNeedsConfig: "The active API lacks a usable Base URL or key",
      doctorApiOpenAIDirectWarning: "OpenAI / Codex direct mode depends on upstream model mapping",
      doctorApiMeta: "{protocol} / {mode}",
      doctorPluginScanning: "Reading built-in capabilities and plugin marketplace",
      doctorPluginReady: "Plugin runtime and official marketplace are available",
      doctorPluginNeedsSetup: "Plugins or built-in capabilities are not ready yet",
      doctorPluginMeta: "Official marketplace: {count} plugins",
      doctorLocalizationReady: "Chinese resource patch is enabled",
      doctorLocalizationMissing: "Chinese resource patch is not enabled",
      doctorHistoryScanning: "Scanning local history sources",
      doctorHistoryReady: "No local history repair is needed",
      doctorHistoryRecoverable: "Recoverable history source found",
      doctorHistoryMeta: "Target: {target}",
      doctorOpenSystem: "System",
      doctorOpenProviders: "API Config",
      doctorOpenTools: "Plugins",
      doctorOpenHistory: "History",
      doctorQueueEmpty: "No action needed.",
      doctorQueueLaunchReady: "Launch Claude Desktop",
      doctorQueueGo: "Open",
      capabilitiesTitle: "Claude Desktop Built-in Capability Center",
      enableDeveloperCapabilities: "Enable Built-in Capabilities",
      capabilityBrowserMcp: "Browser Control Capability",
      capabilityWorkspaceMcp: "Workspace Files Capability",
      capabilityNpx: "Plugin Runtime",
      capabilityWorkspace: "Default Workspace",
      capabilityConfigTargets: "Capability Install Locations",
      officialPluginsTitle: "Official Plugin Marketplace",
      officialPluginsSync: "Sync Official Directory",
      officialPluginsInstall: "Install",
      officialPluginsInstalled: "Installed",
      officialPluginsCli: "Claude CLI",
      officialPluginsMarketplace: "Official Directory",
      officialPluginsCount: "Plugin Count",
      officialPluginsLastUpdated: "Last Sync",
      officialPluginsEmpty: "No official plugin directory was found",
      officialPluginsSyncing: "Syncing official plugin directory...",
      officialPluginsInstalling: "Installing {plugin}...",
      officialPluginsSynced: "Official plugin directory synced",
      officialPluginsInstallOk: "Official plugin installed: {plugin}",
      officialPluginsInstallFailed: "Official plugin install failed: {plugin}",
      officialPluginsHint: "Installs local plugins through Claude's official marketplace and CLI. Organization-provided plugin lists are still controlled by the Claude account/org policy.",
      officialPluginsSearch: "Search Plugins",
      officialPluginsSearchPlaceholder: "Search name, category, author, or description",
      officialPluginsLoadMore: "Load More",
      officialPluginsVisibleCount: "Showing {visible} / {total}",
      officialPluginsNoResults: "No matching plugins",
      officialPluginsInstalls: "{count} installs",
      openChromeConnector: "Open Chrome Connector",
      openChromeGuide: "Chrome Control Guide",
      capabilitiesHint: "Writes Claude Desktop developer-mode capability config. Restart Claude Desktop to apply.",
      capabilitiesApplied: "Built-in capability config written",
      capabilityReady: "Enabled",
      capabilityMissing: "Missing",
      capabilityWritable: "Writable",
      capabilityNeedsRestart: "Restart Claude Desktop to apply",
      adminOk: "Available",
      adminMissing: "Missing; feature enablement may fail",
      boolYes: "Yes",
      boolNo: "No",
      unknown: "Unknown",
      modernInstallOk: "MSIX / modern installer installed",
      modernInstallMissing: "Modern installer not detected",
      featureEnabled: "Enabled",
      featureDisabled: "Disabled",
      featurePending: "Staged; restart pending",
      rebootRequired: "Restart required",
      rebootNotRequired: "No restart pending",
      systemActionOk: "System action finished: {message}",
      systemActionFailed: "System action failed: {message} / exit {code}",
      systemActionRunning: "Running system action...",
      adminRelaunchRequested: "Admin relaunch requested. Confirm the UAC prompt.",
      vmpLocalHintReady: "Windows virtualization components are ready",
      vmpLocalHintRestart: "Windows staged components; restart and check again",
      vmpLocalHintFirmware: "Firmware virtualization is disabled; enable VT-x/SVM in BIOS/UEFI",
      vmpLocalHintAdmin: "Run this tool as administrator",
      vmpLocalHintMissing: "VMP is still disabled; inspect the DISM diagnostics below",
      installStatus: "Claude Install",
      ccSwitchStatus: "cc-switch Sync",
      activeProvider: "Active API",
      sandboxStatus: "Sandbox Policy",
      gatewayStatus: "Gateway",
      directCompatibilityTitle: "Direct Compatibility",
      directCompatibilityHint: "OpenAI / Codex direct mode needs upstream Claude model mapping; otherwise use Gateway.",
      quickActions: "Quick Actions",
      nextSteps: "Next Steps",
      syncCcSwitch: "Sync cc-switch Config",
      syncRunning: "Syncing cc-switch...",
      fxPerformanceMode: "Performance Mode",
      fxVisualMode: "Visual Mode",
      launchTitle: "Launch Diagnostics",
      launcherRoute: "Start Method",
      injectionChannel: "Config Mode",
      liveInjection: "Live Script",
      installPath: "Install Path",
      workingDir: "Working Dir",
      launchPolicy: "Launch Policy",
      verificationTitle: "Launch Verification",
      verificationWaiting: "Waiting",
      verificationVerdict: "Verdict",
      verificationNoRun: "No launch verification yet",
      verificationGatewayHit: "Verified: Claude Desktop hit the local Gateway",
      verificationDirect3p: "Verified: Claude Desktop is using direct 3P provider config",
      verificationProviderRejected: "Gateway was hit, but the provider rejected the credentials. Replace the key or enable its group.",
      verificationUpstreamError: "Gateway was hit, but upstream returned an error. Check upstream status and summary.",
      verificationConfigNoHit: "3P config applied, but Claude has not hit Gateway yet",
      verificationStill1p: "Not active: Claude is still on official login/1P network path",
      verificationNotVerified: "Not verified: no 3P log or gateway request seen",
      gatewayRequests: "Gateway Requests",
      gatewayForwarded: "Forwarded",
      gatewayLastRequest: "Last Request",
      gatewayUpstreamStatus: "Upstream Status",
      gatewayUpstreamError: "Upstream Error",
      claude3pApplied: "3P Config State",
      claude3pDeployment: "Deployment Mode",
      claude3pAppliedId: "Applied ID",
      claude3pDesktopConfig: "Desktop Config",
      claude3pMetaPath: "Config Index",
      claude3pConfigPath: "Gateway Config",
      claude3pReady: "Applied",
      claude3pMissing: "Missing",
      filePresent: "Present",
      fileMissing: "Missing",
      ccSwitchImport: "Sync Existing Config",
      ccSwitchRoot: "Config Root",
      ccSwitchDb: "Database",
      ccSwitchProviders: "Syncable Configs",
      providerList: "Third-party APIs",
      providerTestTitle: "Credential Test",
      testActiveProvider: "Test Active API",
      providerTestTarget: "Target",
      providerTestStatus: "HTTP Status",
      providerTestCode: "Error Code",
      providerTestMessage: "Message",
      providerTestBody: "Body Summary",
      manualProvider: "Manual Add",
      editProvider: "Edit Config",
      editingProvider: "Edit API Config",
      addProvider: "Add API",
      providerModalTitle: "API Config",
      cancelEdit: "Cancel Edit",
      providerFormHint: "Saving sets this as active. When editing, leave Key empty to keep the existing key.",
      providerName: "Name",
      appType: "Type",
      baseUrl: "Base URL",
      apiKey: "API Key",
      enabled: "Enabled",
      saveProvider: "Save and Set Active",
      modelMappingTitle: "Model Mapping",
      modelMappingHint: "Claude Desktop needs Claude-style routes; map those routes to real upstream models here.",
      addModelMapping: "Add Mapping",
      discoverModels: "Fetch Models",
      discoveringModels: "Fetching models from URL and key...",
      modelDiscoveryStatusIdle: "Fetch models from the Base URL and API key.",
      modelDiscoveryOk: "Fetched {count} model(s): {models}",
      modelDiscoveryFailed: "Model discovery failed: {error}",
      mappingClaudeRoute: "Claude Route",
      mappingTargetModel: "Target Model",
      mappingLabel: "Display Name",
      mappingEnabled: "Enabled",
      removeMapping: "Remove",
      sandboxTitle: "Security Policy",
      saveSandbox: "Save Security Policy",
      gatewayTitle: "3P Connection Mode",
      enableGateway: "Use local Gateway forwarding",
      gatewayPort: "Listen Port",
      gatewayUrl: "Gateway URL",
      gatewayTarget: "Forward Target",
      saveGateway: "Save Gateway",
      startGateway: "Start Gateway",
      stopGateway: "Stop Gateway",
      gatewayBoundary: "Gateway only forwards local API requests to the active provider; direct mode can be switched from the top bar.",
      injectProvider: "Use API Base URL",
      injectApiKey: "Use API Key",
      relaxSandbox: "Allow --no-sandbox launch (advanced)",
      sandboxAck: "I understand relaxing sandbox reduces isolation and will only use it in trusted environments",
      officialCapabilityTitle: "Built-in Capability Entry",
      officialCapabilityHint: "Cowork, Code, browser control, and plugin marketplace capabilities are managed from the Built-in Capabilities page.",
      sandboxWarning: "This page only keeps advanced launch-security switches.",
      logsTitle: "Run Logs",
      clearLogs: "Clear Logs",
      bridgeChecking: "Checking desktop bridge",
      bridgePreview: "Browser preview mode: desktop commands only run inside the Tauri app",
      bridgeConnected: "Desktop bridge connected",
      installFound: "Installed",
      installMissing: "Not found",
      ccSwitchFound: "{count} configs found",
      ccSwitchMissing: "cc-switch not found",
      noProvider: "Not configured",
      sandboxDefault: "Default isolation",
      sandboxRelaxed: "Relax requested",
      gatewayRunning: "Running {url}",
      gatewayStopped: "Stopped",
      gatewayDisabled: "Disabled",
      direct3pMode: "Direct 3P Provider",
      providerInjected: "Base URL enabled",
      keyInjected: "Key enabled",
      launchDefaultPolicy: "Default launch uses the active API config; diagnostic launch is only for network/login troubleshooting",
      launchInjectPolicy: "Using active API config: {provider}",
      launchNoActiveProvider: "No active provider yet. Sync cc-switch or add an API config first.",
      launchNeedsInjectableProvider: "The active provider has no usable Base URL/key, or launch policy is disabled",
      launchAutoSwitched: "The active provider was not usable; switched to {provider}",
      providerEmpty: "No API config yet. Sync cc-switch or add one manually.",
      providerActive: "Active",
      providerUse: "Set Active",
      providerInjectLaunch: "Start",
      providerEdit: "Edit",
      providerDelete: "Delete",
      providerTest: "Test Credentials",
      providerSource: "Source",
      providerTypeClaude: "Claude-compatible API",
      providerTypeClaudeDesktop: "Claude Desktop config",
      providerProtocol: "Protocol",
      providerProtocolAnthropic: "Anthropic compatible",
      providerProtocolOpenAI: "OpenAI / Codex compatible",
      providerProtocolLabel: "Protocol",
      providerCompatibility: "Compatibility",
      providerModels: "Models",
      providerTestLocalResult: "Test Result",
      providerKey: "Key",
      providerInjectable: "Usable",
      providerSwitchOnly: "Switch only, no API info",
      providerEnabled: "Enabled",
      providerDisabled: "Disabled",
      providerSaved: "API config saved",
      providerTestOk: "Credentials accepted: {provider} returned HTTP {status}",
      providerTestFailed: "Credentials rejected or unusable: {provider} / HTTP {status} / {code} / {message}",
      providerEditLoaded: "Loaded {provider}. Save to replace this config.",
      providerDeleteConfirm: "Delete API config \"{provider}\"?",
      providerDeleted: "Deleted API config: {provider}",
      syncDone: "cc-switch sync done: imported {imported}, updated {updated}, removed {removed}",
      launchDone: "Claude Desktop launched in diagnostic mode, process {pid}",
      injectLaunchDone: "Claude Desktop started, process {pid}",
      injectLaunchVerified: "Claude Desktop started, process {pid}, verification: {verdict}",
      cdpInjected: "Live capability script enabled on port {port}",
      cdpFailed: "Live capability script failed: {error}",
      cdpMsixUnavailable: "The current MSIX Claude Desktop uses Claude-3p config mode instead.",
      localizationTitle: "One-click Chinese UI",
      localizationStatus: "Localization Status",
      localizationRuntime: "Runtime Channel",
      localizationNotInstalled: "Not installed",
      localizationEnabled: "Enabled",
      localizationDisabled: "Disabled",
      localizationResourcePatch: "Resource patch / zh-CN",
      enableChineseLocalization: "Enable Chinese UI",
      disableChineseLocalization: "Disable Chinese UI",
      localizationHint: "Installs zh-CN language resources and writes Claude locale. Cowork-compatible mode; app.asar is not modified.",
      localizationEnabledLog: "Chinese resource patch installed; restart Claude Desktop to load zh-CN",
      localizationDisabledLog: "Chinese resource patch removed; locale restored to en-US",
      launcherRouteExternalProcess: "Process start",
      launcherRouteAppActivation: "System app activation",
      launcherRouteLocalizedSidecar: "Local localization start",
      injectionDiagnosticClean: "Diagnostic start",
      injectionLiveGateway: "Live script + Gateway",
      injectionLiveDirect: "Live script + Direct",
      injectionPreloadGateway: "Preload script + Gateway",
      injectionPreloadDirect: "Preload script + Direct",
      injectionConfigGateway: "Configured start + Gateway",
      injectionConfigDirect: "Configured start + Direct",
      liveInjectionReady: "Live script supported",
      preloadInjectionReady: "Preload script enabled",
      liveInjectionConfigOnly: "Using configured start",
      liveInjectionAttempted: "Live script attempted",
      sandboxSaved: "Sandbox policy saved",
      gatewaySaved: "Gateway settings saved",
      directModeSaved: "Switched to direct 3P provider mode",
      gatewayModeSaved: "Switched to local Gateway forwarding",
      directModeOpenAIWarning: "Forced direct mode. Note: OpenAI/Codex direct mode requires upstream Anthropic /v1/messages support, otherwise Claude Desktop may not work.",
      gatewayStarted: "Gateway started: {url}",
      gatewayStoppedLog: "Gateway stopped",
      refreshDone: "State refreshed",
      commandUnavailable: "Desktop bridge unavailable; UI preview only",
      commandFailed: "Command failed: {error}",
      requiredName: "Provider name is required",
      requiredUrl: "Base URL is required",
      sandboxNeedAck: "Acknowledgement is required before relaxing sandbox",
      toolsTitle: "Capability Plugins",
      toolsBoundary: "Manage Claude++ capability scripts.",
      scriptEditor: "Plugin Script Editor",
      scriptName: "Script Name",
      scriptCode: "Script Code",
      saveScript: "Save Script",
      scriptEmpty: "No capability scripts yet.",
      scriptSaved: "Capability plugin script saved",
      scriptEnabled: "Enabled",
      scriptDisabled: "Disabled",
      accountBoundary: "Manage Cowork, Code, browser control, and plugin marketplace capabilities from the Built-in Capabilities page.",
      historyTitle: "One-Click History Repair",
      historyRefresh: "Rescan",
      historyTarget: "Repair Target",
      historyBackupRoot: "Backup Directory",
      historyRecommendedSource: "Recommended Source",
      historySources: "Diagnostic Sources",
      historyRepairLogTitle: "Repair Result",
      historyRepairHint: "Automatically picks the latest recoverable Claude history source, backs up current Claude-3p, then restores chat IndexedDB, attachments, and local Code/Cowork sessions.",
      historyCurrent: "Current target",
      historyEmpty: "No recoverable local history source was found",
      historyRepair: "Repair to current Claude-3p",
      historyRepairOneClick: "One-click repair",
      historyFiles: "Files",
      historyBytes: "Size",
      historyLatest: "Latest",
      historyItems: "Items",
      historyNoItems: "No repairable items found",
      historyDefaultSkipped: "Local/Session Storage is scanned only and not restored by default",
      historyShowDetails: "Show diagnostic details",
      historyAutoReady: "History will be repaired from {source}",
      historyAutoMissing: "No history source is available for one-click repair",
      historyRecoverableSummary: "{items} / {files} files / {size}",
      historyRepairRunning: "Repairing history. Claude Desktop will be closed...",
      historyRepairDone: "History repair finished: {files} files, backup at {backup}",
      historyRepairFailed: "History repair failed: {error}",
      historyBoundary: "This only repairs local cache and workspace sessions; cloud history is still determined by the official Claude account.",
      historyScanDone: "History sources rescanned",
      historyScanIdle: "Not scanned yet. Click Rescan to load history sources.",
      recommendationsTitle: "Exclusive Sponsor",
      recommendationsHint: "The Recommended page is reserved for sponsor content. AI quick configs now live in API Config.",
      sponsorBadge: "OFFICIAL RELAY",
      sponsorEyebrow: "Exclusive Sponsor",
      sponsorBody: "JOJO Code is the official Codex++ relay, providing affordable, stable, and easy-to-integrate Codex API relay service. It supports GPT-5.5, GPT-5.4, Claude Opus 4.8, Claude Opus 4.7, gpt-image-2, and image capabilities for daily development, fast setup, team collaboration, and long-term use.",
      sponsorVisit: "Visit JOJO Code",
      externalOpened: "Opened: {url}",
      quickProviderTitle: "AI Quick Config",
      recommendedModesTitle: "Recommended Work Modes",
      recommendationUse: "Fill Config",
      recommendationSave: "Save Config",
      recommendationDocs: "Docs",
      recommendationBase: "Base URL",
      recommendationProtocol: "Protocol",
      recommendationDirectReady: "Direct ready",
      recommendationNeedsAdapter: "Adapter needed",
      recommendationFilled: "{provider} filled. Add your API key, then save.",
      recommendationSaved: "{provider} saved as the active config",
      modeOfficialTitle: "Official Account Mode",
      modeOfficialBody: "Use Built-in Capabilities when you need Cowork, Code, browser control, and plugin marketplace features, then keep Claude Desktop system readiness healthy.",
      modeDirectTitle: "Direct 3P Mode",
      modeDirectBody: "Pick an Anthropic-compatible provider and disable Gateway to avoid local forwarding. Best for stability and latency.",
      modeGatewayTitle: "Gateway/Adapter Mode",
      modeGatewayBody: "OpenAI-compatible-only providers use the local Gateway to translate Claude Desktop requests.",
      aboutTitle: "About Claude++",
      aboutVersion: "Version",
      aboutInstall: "Install Path",
      aboutConfig: "Config File",
      aboutBoundary: "Claude++ handles local config, enhanced start, built-in capabilities, history repair, and system diagnostics.",
      aboutFeaturesTitle: "Main Features",
      featureCcSwitchTitle: "cc-switch Sync",
      featureCcSwitchBody: "Read and sync cc-switch configs, then switch and launch with them from Claude++.",
      featureProviderTitle: "Third-party API Config",
      featureProviderBody: "Use direct 3P Provider mode or local Gateway adaptation, with credential testing.",
      featureLaunchTitle: "Claude Desktop Launch",
      featureLaunchBody: "Provider configured start, live capability scripts, MSIX detection, and system readiness checks.",
      featureHistoryTitle: "History Repair",
      featureHistoryBody: "Scan local Claude history caches and restore IndexedDB, attachments, and local work sessions with backups.",
      featureSystemTitle: "System Readiness",
      featureSystemBody: "Check modern installer, VMP, Hypervisor, administrator rights, and restart state.",
      featureBoundaryTitle: "Built-in Capabilities",
      featureBoundaryBody: "Enables Claude Desktop developer mode, browser control, workspace file access, and plugin runtime entry points.",
      featureToolsTitle: "Capability Plugins",
      featureToolsBody: "Includes one-click Chinese localization and Claude's official plugin marketplace.",
    },
  };

  const pageTitles = {
    overview: "overviewTitle",
    system: "systemPageTitle",
    capabilities: "capabilitiesPageTitle",
    tools: "toolsPageTitle",
    launch: "launchPageTitle",
    providers: "providersPageTitle",
    recommendations: "recommendationsPageTitle",
    history: "historyPageTitle",
    sandbox: "sandboxPageTitle",
    about: "aboutPageTitle",
    logs: "logsPageTitle",
  };

  const recommendedProviders = [
    {
      name: "DeepSeek Anthropic",
      baseUrl: "https://api.deepseek.com/anthropic",
      docsUrl: "https://api-docs.deepseek.com/guides/anthropic_api",
      protocol: "Anthropic-compatible",
      directReady: true,
      note: {
        zh: "适合把 DeepSeek 模型作为 Claude Desktop 的 3P 推理入口，优先用于直连模式。",
        en: "Suitable for using DeepSeek models as a Claude Desktop 3P inference endpoint. Prefer direct mode.",
      },
    },
    {
      name: "MiniMax Anthropic",
      baseUrl: "https://api.minimax.io/anthropic",
      docsUrl: "https://platform.minimax.io/docs/use-cases/claude-code",
      protocol: "Anthropic-compatible",
      directReady: true,
      note: {
        zh: "适合 MiniMax/海螺模型的 Claude Code / Claude Desktop 兼容场景。",
        en: "For MiniMax models in Claude Code / Claude Desktop compatible workflows.",
      },
    },
    {
      name: "Z.AI / GLM Anthropic",
      baseUrl: "https://api.z.ai/api/anthropic",
      docsUrl: "https://docs.z.ai/guides/llm/glm-4.5",
      protocol: "Anthropic-compatible",
      directReady: true,
      note: {
        zh: "适合 GLM 系列模型的 Anthropic 兼容接入，Key 与模型权限以 Z.AI 控制台为准。",
        en: "For GLM models through Z.AI's Anthropic-compatible endpoint. Keys and model permissions depend on Z.AI.",
      },
    },
    {
      name: "OpenRouter Anthropic",
      baseUrl: "https://openrouter.ai/api",
      docsUrl: "https://openrouter.ai/docs/api-reference/overview",
      protocol: "Anthropic-compatible router",
      directReady: true,
      note: {
        zh: "适合通过 OpenRouter 选择 Claude、DeepSeek、Kimi 等模型；Claude Desktop 会继续请求 /v1/messages。",
        en: "Use OpenRouter to route to Claude, DeepSeek, Kimi, and other models. Claude Desktop continues to call /v1/messages.",
      },
    },
    {
      name: "Kimi / Moonshot",
      baseUrl: "https://api.moonshot.cn/v1",
      docsUrl: "https://platform.moonshot.cn/docs/api-reference",
      protocol: "OpenAI-compatible",
      directReady: false,
      note: {
        zh: "适合 Kimi 模型；保存后自动使用本地转换网关。",
        en: "For Kimi models. Saving uses the local protocol adapter Gateway automatically.",
      },
    },
    {
      name: "OpenAI / Codex",
      baseUrl: "https://api.openai.com/v1",
      docsUrl: "https://platform.openai.com/docs/api-reference",
      protocol: "OpenAI-compatible",
      directReady: false,
      note: {
        zh: "适合 OpenAI/Codex 类模型；保存后自动使用本地转换网关。",
        en: "For OpenAI/Codex-style models. Saving uses the local protocol adapter Gateway automatically.",
      },
    },
    {
      name: "SiliconFlow",
      baseUrl: "https://api.siliconflow.cn/v1",
      docsUrl: "https://docs.siliconflow.cn/api-reference/chat-completions/chat-completions",
      protocol: "OpenAI-compatible",
      directReady: false,
      note: {
        zh: "适合通过硅基流动接入 DeepSeek、Qwen、GLM 等模型；保存后自动使用本地转换网关。",
        en: "For SiliconFlow-hosted DeepSeek, Qwen, GLM, and other models. Saving uses the local protocol adapter Gateway automatically.",
      },
    },
  ];

  const officialPluginTranslations = {
    zh: {
      playwright: {
        category: "浏览器自动化",
        description: "Microsoft 提供的浏览器自动化与端到端测试插件。让 Claude 操作网页、截图、填写表单、点击元素，并执行自动化浏览器测试流程。",
      },
      github: {
        category: "代码协作",
        description: "GitHub 官方插件。让 Claude 管理仓库、创建 Issue、处理 Pull Request、审查代码、搜索仓库，并调用 GitHub API。",
      },
      gitlab: {
        category: "代码协作",
        description: "GitLab DevOps 平台集成。支持管理仓库、合并请求、CI/CD 流水线、Issue 与 Wiki。",
      },
      linear: {
        category: "项目管理",
        description: "Linear 任务管理集成。支持创建 Issue、管理项目、更新状态，并在开发流程中衔接 Linear 工作区。",
      },
      asana: {
        category: "项目管理",
        description: "Asana 项目管理集成。支持创建和管理任务、搜索项目、更新负责人、跟踪进度。",
      },
      context7: {
        category: "开发资料",
        description: "Upstash Context7 文档检索插件。把指定版本的官方文档和代码示例直接拉入 Claude 上下文。",
      },
      firebase: {
        category: "后端服务",
        description: "Google Firebase 集成。支持管理 Firestore、认证、云函数、托管和存储，用于构建与维护 Firebase 后端。",
      },
      serena: {
        category: "代码理解",
        description: "语义代码分析插件。通过语言服务提供代码理解、重构建议和代码库导航能力。",
      },
      terraform: {
        category: "基础设施",
        description: "Terraform 集成插件。为基础设施即代码开发提供 Terraform 生态的自动化与交互能力。",
      },
      "code-review": {
        category: "代码质量",
        description: "自动代码审查插件。使用多个专门代理审查 Pull Request，并用置信度评分降低误报。",
      },
      "security-guidance": {
        category: "安全",
        description: "安全审查插件。可在编辑和停止时检查注入、XSS、SSRF、硬编码密钥等常见漏洞风险。",
      },
      "pr-review-toolkit": {
        category: "代码质量",
        description: "Pull Request 审查工具包。覆盖评论、测试、错误处理、类型设计、代码质量和代码简化等方向。",
      },
      "plugin-dev": {
        category: "插件开发",
        description: "Claude Code 插件开发工具包。提供 hooks、命令、代理、技能、MCP 集成和插件结构的开发指导。",
      },
      "skill-creator": {
        category: "技能开发",
        description: "用于创建、改进和评估 Claude 技能。适合从零创建技能、优化现有技能或做效果评测。",
      },
      "mcp-server-dev": {
        category: "工具开发",
        description: "用于设计和构建 Claude 可用的 MCP 服务，覆盖远程 HTTP、本地服务、MCPB、工具设计和认证。",
      },
      "claude-code-setup": {
        category: "开发配置",
        description: "分析代码库并推荐适合项目的 Claude Code 自动化配置，例如 hooks、技能、MCP 服务和子代理。",
      },
      "claude-md-management": {
        category: "项目记忆",
        description: "维护和改进 CLAUDE.md 文件，帮助审计质量、沉淀会话经验，并保持项目记忆最新。",
      },
      "frontend-design": {
        category: "前端设计",
        description: "用于创建高质量、可上线的前端界面。强调独特视觉、精致交互，并避免模板化 AI 风格。",
      },
      "feature-dev": {
        category: "功能开发",
        description: "完整功能开发流程插件，包含代码库探索、架构设计和质量审查等专门代理。",
      },
      "commit-commands": {
        category: "Git 工作流",
        description: "Git 提交流程命令插件，支持 commit、push 和创建 PR 等常用操作。",
      },
      "code-simplifier": {
        category: "代码质量",
        description: "代码简化与整理代理。在保持功能不变的前提下，提高代码清晰度、一致性和可维护性。",
      },
      "typescript-lsp": {
        category: "语言服务",
        description: "TypeScript/JavaScript 语言服务插件，提供更好的代码智能、导航和分析能力。",
      },
      "pyright-lsp": {
        category: "语言服务",
        description: "Python 语言服务插件，基于 Pyright 提供类型检查和代码智能。",
      },
      "rust-analyzer-lsp": {
        category: "语言服务",
        description: "Rust 语言服务插件，提供 Rust 代码智能、导航和分析能力。",
      },
      "gopls-lsp": {
        category: "语言服务",
        description: "Go 语言服务插件，提供代码智能和重构支持。",
      },
      "clangd-lsp": {
        category: "语言服务",
        description: "C/C++ 语言服务插件，基于 clangd 提供代码智能。",
      },
      "csharp-lsp": {
        category: "语言服务",
        description: "C# 语言服务插件，提供 C# 代码智能。",
      },
      "jdtls-lsp": {
        category: "语言服务",
        description: "Java 语言服务插件，基于 Eclipse JDT.LS 提供代码智能。",
      },
      "kotlin-lsp": {
        category: "语言服务",
        description: "Kotlin 语言服务插件，提供 Kotlin 代码智能。",
      },
      "lua-lsp": {
        category: "语言服务",
        description: "Lua 语言服务插件，提供 Lua 代码智能。",
      },
      "php-lsp": {
        category: "语言服务",
        description: "PHP 语言服务插件，基于 Intelephense 提供代码智能。",
      },
      "ruby-lsp": {
        category: "语言服务",
        description: "Ruby 语言服务插件，提供 Ruby 代码智能和分析能力。",
      },
      "swift-lsp": {
        category: "语言服务",
        description: "Swift 语言服务插件，基于 SourceKit-LSP 提供代码智能。",
      },
      discord: {
        category: "消息连接",
        description: "Discord 消息桥接插件，带访问控制。可通过 /discord:access 管理配对、白名单和策略。",
      },
      telegram: {
        category: "消息连接",
        description: "Telegram 消息桥接插件，带访问控制。可通过 /telegram:access 管理配对、白名单和策略。",
      },
      fakechat: {
        category: "本地测试",
        description: "本地网页聊天测试插件，用于测试通知和通道流程。不需要 token，也不连接第三方服务。",
      },
    },
  };

  const recommendedModes = [
    { titleKey: "modeOfficialTitle", bodyKey: "modeOfficialBody" },
    { titleKey: "modeDirectTitle", bodyKey: "modeDirectBody" },
    { titleKey: "modeGatewayTitle", bodyKey: "modeGatewayBody" },
  ];

  const aboutFeatures = [
    { titleKey: "featureCcSwitchTitle", bodyKey: "featureCcSwitchBody" },
    { titleKey: "featureProviderTitle", bodyKey: "featureProviderBody" },
    { titleKey: "featureLaunchTitle", bodyKey: "featureLaunchBody" },
    { titleKey: "featureHistoryTitle", bodyKey: "featureHistoryBody" },
    { titleKey: "featureSystemTitle", bodyKey: "featureSystemBody" },
    { titleKey: "featureBoundaryTitle", bodyKey: "featureBoundaryBody" },
    { titleKey: "featureToolsTitle", bodyKey: "featureToolsBody" },
  ];

  let locale = "zh";
  let state = null;
  let lastLaunchResult = null;
  let lastProviderTest = null;
  let systemLoaded = false;
  let systemLoading = false;
  let historyLoaded = false;
  let historyLoading = false;
  let capabilitiesStatus = null;
  let capabilitiesLoading = false;
  let officialPluginsStatus = null;
  let officialPluginsLoading = false;
  let overviewDiagnosticsStarted = false;
  let overviewDiagnosticsLoading = false;
  let officialPluginSearchQuery = "";
  let officialPluginVisibleLimit = 12;
  const providerTestResults = new Map();
  const terminal = document.getElementById("terminalWindow");
  const noticeBar = document.getElementById("noticeBar");
  const toastHost = document.getElementById("toastHost");

  function t(key, values = {}) {
    const template = translations[locale][key] || translations.zh[key] || key;
    return String(template).replace(/\{(\w+)\}/g, (_, name) => values[name] ?? "");
  }

  function nextFrame() {
    return new Promise((resolve) => window.requestAnimationFrame(resolve));
  }

  function formField(form, name) {
    const field = form.elements.namedItem(name);
    if (!field) {
      throw new Error(`Missing form field: ${name}`);
    }
    return field;
  }

  function setText(id, value) {
    const element = document.getElementById(id);
    if (element) {
      element.textContent = value;
    }
  }

  function resizeCanvas() {
    const ratio = window.devicePixelRatio || 1;
    width = window.innerWidth;
    height = window.innerHeight;
    canvas.width = Math.floor(width * ratio);
    canvas.height = Math.floor(height * ratio);
    canvas.style.width = width + "px";
    canvas.style.height = height + "px";
    ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
    columns = Array.from({ length: Math.ceil(width / 18) }, () => Math.random() * height);
  }

  function drawRain() {
    if (!visualFxEnabled || prefersReducedMotion) return;
    ctx.fillStyle = "rgba(2, 4, 6, 0.08)";
    ctx.fillRect(0, 0, width, height);
    ctx.font = "14px Cascadia Mono, Consolas, monospace";
    columns.forEach((y, index) => {
      const x = index * 18;
      const char = glyphs[Math.floor(Math.random() * glyphs.length)];
      ctx.fillStyle = Math.random() > 0.92 ? "rgba(32, 231, 255, 0.82)" : "rgba(66, 255, 155, 0.62)";
      ctx.fillText(char, x, y);
      columns[index] = y > height + Math.random() * 600 ? 0 : y + 18;
    });
  }

  function startRain() {
    stopRain();
    resizeCanvas();
    if (!visualFxEnabled || prefersReducedMotion) {
      drawRain();
      return;
    }
    rainTimer = window.setInterval(drawRain, 140);
  }

  function stopRain() {
    if (rainTimer) {
      window.clearInterval(rainTimer);
      rainTimer = null;
    }
    ctx.clearRect(0, 0, canvas.width, canvas.height);
  }

  function applyVisualFx() {
    document.body.classList.toggle("fx-on", visualFxEnabled && !prefersReducedMotion);
    document.body.classList.toggle("fx-off", !visualFxEnabled || prefersReducedMotion);
    const toggle = document.getElementById("fxToggle");
    if (toggle) {
      toggle.textContent = visualFxEnabled && !prefersReducedMotion ? t("fxVisualMode") : t("fxPerformanceMode");
      toggle.setAttribute("aria-pressed", visualFxEnabled && !prefersReducedMotion ? "true" : "false");
    }
    startRain();
  }

  function toggleVisualFx() {
    visualFxEnabled = !visualFxEnabled;
    window.localStorage.setItem("claude-plus-visual-fx", visualFxEnabled ? "on" : "off");
    applyVisualFx();
  }

  function invoke(command, args) {
    const tauri = window.__TAURI__;
    const bridge = tauri?.core?.invoke || tauri?.invoke;
    if (!bridge) {
      log("WARN", t("commandUnavailable"));
      return Promise.resolve(undefined);
    }
    return (tauri.core?.invoke ? tauri.core.invoke(command, args) : tauri.invoke(command, args));
  }

  function log(level, message) {
    showNotice(level, message);
    showToast(level, message);
    const line = document.createElement("p");
    const badge = document.createElement("span");
    badge.className = level === "OK" ? "ok" : level === "WARN" ? "warn" : level === "ERR" ? "err" : "prompt";
    badge.textContent = level;
    line.appendChild(badge);
    line.appendChild(document.createTextNode(" " + message));
    terminal.appendChild(line);
    while (terminal.children.length > 80) {
      terminal.firstElementChild?.remove();
    }
    terminal.scrollTop = terminal.scrollHeight;
  }

  function showNotice(level, message) {
    noticeBar.hidden = false;
    noticeBar.className = "notice-bar " + (level === "OK" ? "ok" : level === "WARN" ? "warn" : level === "ERR" ? "err" : "");
    noticeBar.textContent = message;
  }

  function showToast(level, message) {
    if (level === "TRACE" || !toastHost) return;

    const toast = document.createElement("section");
    toast.className = "toast " + (level === "OK" ? "ok" : level === "WARN" ? "warn" : level === "ERR" ? "err" : "");
    toast.setAttribute("role", level === "ERR" ? "alert" : "status");

    const content = document.createElement("div");
    const title = document.createElement("strong");
    title.textContent = level;
    const copy = document.createElement("p");
    copy.textContent = message;
    content.append(title, copy);

    const close = document.createElement("button");
    close.className = "toast-close";
    close.type = "button";
    close.setAttribute("aria-label", locale === "zh" ? "关闭提示" : "Close notification");
    close.textContent = "X";
    close.addEventListener("click", () => toast.remove());

    toast.append(content, close);
    toastHost.appendChild(toast);
    while (toastHost.children.length > 3) {
      toastHost.firstElementChild?.remove();
    }
    window.setTimeout(() => toast.remove(), level === "ERR" ? 8000 : 5000);
  }

  function applyLocale(nextLocale) {
    locale = nextLocale;
    document.documentElement.lang = locale === "zh" ? "zh-CN" : "en";
    document.querySelectorAll("[data-i18n]").forEach((element) => {
      element.textContent = t(element.getAttribute("data-i18n"));
    });
    document.querySelectorAll("[data-i18n-placeholder]").forEach((element) => {
      element.setAttribute("placeholder", t(element.getAttribute("data-i18n-placeholder")));
    });
    document.querySelectorAll(".lang-button[data-locale]").forEach((button) => {
      const active = button.getAttribute("data-locale") === locale;
      button.classList.toggle("active", active);
      button.setAttribute("aria-pressed", String(active));
    });
    applyVisualFx();
    const activePage = document.querySelector(".nav-item.active")?.getAttribute("data-page") || "overview";
    document.getElementById("pageTitle").textContent = t(pageTitles[activePage]);
    renderState();
  }

  function setPage(page) {
    document.querySelectorAll(".nav-item").forEach((button) => {
      button.classList.toggle("active", button.getAttribute("data-page") === page);
    });
    document.querySelectorAll(".page").forEach((view) => {
      view.classList.toggle("active", view.getAttribute("data-view") === page);
    });
    document.getElementById("pageTitle").textContent = t(pageTitles[page]);
    renderVisiblePage();
    hydrateLazyPage(page);
  }

  function currentPage() {
    return document.querySelector(".nav-item.active")?.getAttribute("data-page") || "overview";
  }

  function renderVisiblePage() {
    if (!state) return;
    const page = currentPage();
    if (page === "overview") {
      renderDoctorReport();
    }
    if (page === "providers") {
      renderProviders(state.config.providers || []);
      renderRecommendations();
    }
    if (page === "recommendations" || page === "about") {
      renderRecommendations();
    }
    if (page === "history") {
      renderHistory(state.history);
    }
    if (page === "system") {
      renderSystemReadiness();
    }
    if (page === "capabilities") {
      renderDeveloperCapabilities();
    }
    if (page === "tools") {
      renderLocalizationStatus();
      renderOfficialPlugins();
    }
  }

  async function refreshState({ silent = false } = {}) {
    const previousState = state;
    const keepSystem = systemLoaded && previousState?.system;
    const keepHistory = historyLoaded && previousState?.history;
    const result = await invoke("read_app_state");
    if (!result) {
      document.getElementById("bridgeState").textContent = t("bridgePreview");
      if (!silent) log("WARN", t("bridgePreview"));
      return;
    }
    state = result;
    if (keepSystem) {
      state.system = previousState.system;
    } else {
      systemLoaded = false;
    }
    if (keepHistory) {
      state.history = previousState.history;
    } else {
      historyLoaded = false;
    }
    await refreshGatewayStatus();
    document.getElementById("bridgeState").textContent = t("bridgeConnected");
    renderState();
    hydrateLazyPage(currentPage());
    if (!silent) log("OK", t("refreshDone"));
  }

  async function hydrateLazyPage(page, { force = false } = {}) {
    if (page === "overview") {
      hydrateOverviewDiagnostics({ force });
      return;
    }
    if (page === "system" && (force || !systemLoaded)) {
      await refreshSystemReadiness();
    }
    if (page === "history" && (force || !historyLoaded)) {
      await refreshHistoryStatus();
    }
    if (page === "capabilities" && (force || !capabilitiesStatus)) {
      await refreshDeveloperCapabilities();
    }
    if (page === "tools" && (force || !officialPluginsStatus)) {
      await refreshOfficialPlugins();
    }
  }

  async function hydrateOverviewDiagnostics({ force = false } = {}) {
    if (!state || overviewDiagnosticsLoading) return;
    if (overviewDiagnosticsStarted && !force) return;
    overviewDiagnosticsStarted = true;
    overviewDiagnosticsLoading = true;
    renderDoctorReport();
    const tasks = [];
    if (force || !systemLoaded) tasks.push(refreshSystemReadiness());
    if (force || !capabilitiesStatus) tasks.push(refreshDeveloperCapabilities());
    if (force || !officialPluginsStatus) tasks.push(refreshOfficialPlugins());
    if (force || !historyLoaded) tasks.push(refreshHistoryStatus({ silent: true }));
    try {
      await Promise.allSettled(tasks);
    } finally {
      overviewDiagnosticsLoading = false;
      renderDoctorReport();
    }
  }

  async function refreshGatewayStatus() {
    if (!state) return;
    try {
      state.gateway = await invoke("gateway_status");
    } catch (error) {
      state.gateway = {
        enabled: state.config?.gateway?.enabled ?? true,
        running: false,
        url: `http://127.0.0.1:${state.config?.gateway?.port || 49331}`,
        port: state.config?.gateway?.port || 49331,
        last_error: String(error),
      };
    }
  }

  async function refreshSystemReadiness() {
    if (!state) return;
    if (systemLoading) return;
    systemLoading = true;
    renderSystemReadiness();
    try {
      await nextFrame();
      const system = await invoke("system_readiness_status");
      if (!system) return;
      if (state) {
        state.system = system;
      }
      systemLoaded = true;
      renderSystemReadiness();
    } catch (error) {
      log("ERR", t("commandFailed", { error: String(error) }));
    } finally {
      systemLoading = false;
      renderSystemReadiness();
    }
  }

  async function refreshHistoryStatus({ silent = false } = {}) {
    if (!state) return;
    if (historyLoading) return;
    historyLoading = true;
    renderHistory(state?.history || null);
    try {
      await nextFrame();
      const scan = await invoke("history_scan_status");
      if (!scan) return;
      if (state) {
        state.history = scan;
      }
      historyLoaded = true;
      renderHistory(scan);
      if (!silent) log("OK", t("historyScanDone"));
    } catch (error) {
      log("ERR", t("commandFailed", { error: String(error) }));
    } finally {
      historyLoading = false;
      renderHistory(state?.history || null);
    }
  }

  async function refreshDeveloperCapabilities() {
    if (capabilitiesLoading) return;
    capabilitiesLoading = true;
    renderDeveloperCapabilities();
    try {
      await nextFrame();
      const status = await invoke("developer_capabilities_status");
      if (!status) return;
      capabilitiesStatus = status;
      renderDeveloperCapabilities();
    } catch (error) {
      log("ERR", t("commandFailed", { error: String(error) }));
    } finally {
      capabilitiesLoading = false;
      renderDeveloperCapabilities();
    }
  }

  async function enableDeveloperCapabilities() {
    if (capabilitiesLoading) return;
    capabilitiesLoading = true;
    renderDeveloperCapabilities();
    try {
      await nextFrame();
      const status = await invoke("enable_developer_capabilities");
      if (!status) return;
      capabilitiesStatus = status;
      renderDeveloperCapabilities();
      renderCapabilityLog(status);
      log("OK", `${t("capabilitiesApplied")} / ${t("capabilityNeedsRestart")}`);
    } catch (error) {
      log("ERR", t("commandFailed", { error: String(error) }));
    } finally {
      capabilitiesLoading = false;
      renderDeveloperCapabilities();
    }
  }

  async function refreshOfficialPlugins() {
    if (officialPluginsLoading) return;
    officialPluginsLoading = true;
    renderOfficialPlugins();
    try {
      await nextFrame();
      const status = await invoke("official_plugins_status");
      if (!status) return;
      officialPluginsStatus = status;
      renderOfficialPlugins();
    } catch (error) {
      log("ERR", t("commandFailed", { error: String(error) }));
    } finally {
      officialPluginsLoading = false;
      renderOfficialPlugins();
    }
  }

  async function syncOfficialPluginMarketplace() {
    if (officialPluginsLoading) return;
    officialPluginsLoading = true;
    renderOfficialPlugins();
    showNotice("WARN", t("officialPluginsSyncing"));
    try {
      await nextFrame();
      const result = await invoke("sync_official_plugin_marketplace");
      officialPluginsStatus = result.status;
      renderOfficialPlugins();
      renderOfficialPluginLog(result);
      log(result.ok ? "OK" : "ERR", result.ok ? t("officialPluginsSynced") : result.message);
    } catch (error) {
      log("ERR", t("commandFailed", { error: String(error) }));
    } finally {
      officialPluginsLoading = false;
      renderOfficialPlugins();
    }
  }

  async function installOfficialPlugin(plugin) {
    if (officialPluginsLoading) return;
    officialPluginsLoading = true;
    renderOfficialPlugins();
    showNotice("WARN", t("officialPluginsInstalling", { plugin }));
    try {
      await nextFrame();
      const result = await invoke("install_official_plugin", { plugin });
      officialPluginsStatus = result.status;
      renderOfficialPlugins();
      renderOfficialPluginLog(result);
      log(
        result.ok ? "OK" : "ERR",
        result.ok ? t("officialPluginsInstallOk", { plugin }) : t("officialPluginsInstallFailed", { plugin })
      );
    } catch (error) {
      log("ERR", t("commandFailed", { error: String(error) }));
    } finally {
      officialPluginsLoading = false;
      renderOfficialPlugins();
    }
  }

  function renderState() {
    if (!state) {
      renderDoctorReport();
      renderConnectionMode({ enabled: false });
      renderVerification();
      renderHistory(null);
      renderRecommendations();
      return;
    }

    const install = state.install;
    const config = state.config;
    const ccSwitch = state.cc_switch;
    const activeProvider = config.providers.find((provider) => provider.active);

    renderConnectionMode(config.gateway || { enabled: true, port: 49331 });
    renderGatewayStatus();
    renderDoctorReport();
    renderVerification();
    renderProviderTest();
    renderSystemReadiness();
    renderVisiblePage();

    document.getElementById("installPath").textContent = install?.executable || "-";
    document.getElementById("workingDir").textContent = install?.working_dir || "-";
    document.getElementById("asarPath").textContent = install?.app_asar || "-";
    const currentLauncherRoute = lastLaunchResult?.launcher_route || install?.launcher_route;
    const currentInjectionChannel = lastLaunchResult?.injection_channel || defaultInjectionChannel(install);
    document.getElementById("launcherRoute").textContent = launcherRouteLabel(currentLauncherRoute);
    document.getElementById("injectionChannel").textContent = injectionChannelLabel(currentInjectionChannel);
    document.getElementById("liveInjection").textContent = liveInjectionLabel(lastLaunchResult, install);
    document.getElementById("launchPolicy").textContent = activeProvider
      ? t("launchInjectPolicy", { provider: activeProvider.name })
      : t("launchDefaultPolicy");
    document.querySelector('[data-i18n="sandboxWarning"]').textContent = t("sandboxWarning");

    document.getElementById("ccSwitchRoot").textContent = ccSwitch?.root || "-";
    document.getElementById("ccSwitchDb").textContent = ccSwitch?.database_path || "-";
    document.getElementById("ccSwitchProviders").textContent = String(ccSwitch?.provider_count || 0);

    hydrateSandboxForm(config.sandbox);
    hydrateGatewayForm(config.gateway || { enabled: true, port: 49331 });
  }

  function featureEnabled(value) {
    return String(value || "").toLowerCase() === "enabled";
  }

  function doctorBadgeLabel(level) {
    if (level === "ok") return t("doctorReady");
    if (level === "err") return t("doctorError");
    if (level === "pending") return t("doctorScanning");
    return t("doctorNeedsAction");
  }

  function doctorBadgeClass(level) {
    if (level === "ok") return "seal ok";
    if (level === "err") return "seal err";
    return "seal warn";
  }

  function doctorSetCard(prefix, item) {
    const badge = document.getElementById(`${prefix}Badge`);
    const summary = document.getElementById(`${prefix}Summary`);
    const meta = document.getElementById(`${prefix}Meta`);
    const card = badge?.closest(".doctor-card");
    if (badge) {
      badge.className = doctorBadgeClass(item.level);
      badge.textContent = doctorBadgeLabel(item.level);
    }
    if (summary) summary.textContent = item.summary || "-";
    if (meta) meta.textContent = item.meta || "-";
    if (card) {
      card.classList.toggle("ok", item.level === "ok");
      card.classList.toggle("warn", item.level === "warn" || item.level === "pending");
      card.classList.toggle("err", item.level === "err");
    }
  }

  function doctorItems() {
    const config = state?.config || {};
    const gateway = config.gateway || { enabled: true, port: 49331 };
    const activeProvider = config.providers?.find((provider) => provider.active);
    const system = state?.system || {};
    const script = localizationScript();
    const recommendedHistory = recommendedHistoryProfile(state?.history || null);
    const pluginCount = officialPluginsStatus?.marketplace_plugin_count ?? officialPluginsStatus?.plugins?.length ?? 0;

    const installItem = state?.install
      ? {
          key: "install",
          prefix: "doctorInstall",
          level: "ok",
          summary: t("doctorInstallOk"),
          meta: state.install.executable || state.install.working_dir || "-",
          page: "system",
          blocking: false,
        }
      : {
          key: "install",
          prefix: "doctorInstall",
          level: "err",
          summary: t("doctorInstallMissing"),
          meta: t("installClaudeModern"),
          page: "system",
          blocking: true,
        };

    let systemItem = {
      key: "system",
      prefix: "doctorSystem",
      level: "pending",
      summary: t("doctorSystemScanning"),
      meta: "-",
      page: "system",
      blocking: false,
    };
    if (systemLoaded) {
      const vmpReady = !system.is_windows || featureEnabled(system.virtual_machine_platform);
      const hypervisorReady = !system.is_windows || system.hypervisor_present !== false;
      const systemIssue = !system.claude_modern_installer
        ? t("doctorSystemNeedsModern")
        : system.reboot_required
          ? t("doctorSystemNeedsRestart")
          : !vmpReady
            ? t("doctorSystemNeedsVmp")
            : null;
      systemItem = {
        ...systemItem,
        level: systemIssue ? "warn" : "ok",
        summary: systemIssue || t("doctorSystemReady"),
        meta: t("doctorSystemMeta", {
          vmp: formatFeatureState(system.virtual_machine_platform),
          hypervisor: hypervisorReady ? t("boolYes") : t("boolNo"),
        }),
      };
    }

    let apiItem = {
      key: "api",
      prefix: "doctorApi",
      level: "warn",
      summary: t("doctorApiMissing"),
      meta: t("syncCcSwitch"),
      page: "providers",
      blocking: true,
    };
    if (activeProvider) {
      const gatewayMode = gateway.enabled ? "Gateway" : t("directModeButton");
      const openAiDirect = activeProvider.protocol === "openai" && !gateway.enabled;
      const injectable = canInjectProvider(activeProvider, config.sandbox || {});
      const summary = !injectable
        ? t("doctorApiNeedsConfig")
        : openAiDirect
          ? t("doctorApiOpenAIDirectWarning")
          : t("doctorApiReady");
      apiItem = {
        ...apiItem,
        level: injectable && !openAiDirect ? "ok" : "warn",
        summary: `${activeProvider.name} / ${summary}`,
        meta: t("doctorApiMeta", {
          protocol: providerProtocolLabel(activeProvider.protocol),
          mode: gatewayMode,
        }),
        blocking: !injectable,
      };
    }

    let pluginItem = {
      key: "plugins",
      prefix: "doctorPlugin",
      level: "pending",
      summary: t("doctorPluginScanning"),
      meta: "-",
      page: "tools",
      blocking: false,
    };
    if (capabilitiesStatus || officialPluginsStatus) {
      const capabilityReady = Boolean(
        capabilitiesStatus?.browser_mcp_configured &&
        capabilitiesStatus?.workspace_mcp_configured &&
        capabilitiesStatus?.npx_available
      );
      const marketplaceReady = Boolean(officialPluginsStatus?.marketplace_configured || pluginCount > 0);
      pluginItem = {
        ...pluginItem,
        level: capabilityReady && marketplaceReady ? "ok" : "warn",
        summary: capabilityReady && marketplaceReady ? t("doctorPluginReady") : t("doctorPluginNeedsSetup"),
        meta: t("doctorPluginMeta", { count: pluginCount || "-" }),
        page: capabilityReady ? "tools" : "capabilities",
      };
    }

    const localizationItem = {
      key: "localization",
      prefix: "doctorLocalization",
      level: script?.enabled ? "ok" : "warn",
      summary: script?.enabled ? t("doctorLocalizationReady") : t("doctorLocalizationMissing"),
      meta: script?.enabled ? t("localizationResourcePatch") : t("enableChineseLocalization"),
      page: "tools",
      blocking: false,
    };

    let historyItem = {
      key: "history",
      prefix: "doctorHistory",
      level: "pending",
      summary: t("doctorHistoryScanning"),
      meta: "-",
      page: "history",
      blocking: false,
    };
    if (historyLoaded) {
      historyItem = {
        ...historyItem,
        level: recommendedHistory ? "warn" : "ok",
        summary: recommendedHistory ? t("doctorHistoryRecoverable") : t("doctorHistoryReady"),
        meta: t("doctorHistoryMeta", { target: state?.history?.target_path || "-" }),
      };
    }

    return [installItem, systemItem, apiItem, pluginItem, localizationItem, historyItem];
  }

  function doctorActionableItems() {
    return doctorItems().filter((item) => item.level === "err" || item.level === "warn");
  }

  function doctorPrimaryItem() {
    const items = doctorActionableItems();
    return items.find((item) => item.blocking) || null;
  }

  function renderDoctorReport() {
    const items = doctorItems();
    items.forEach((item) => doctorSetCard(item.prefix, item));

    const actionable = items.filter((item) => item.level === "err" || item.level === "warn");
    const pending = items.filter((item) => item.level === "pending");
    const first = doctorPrimaryItem() || actionable[0];
    const verdictBadge = document.getElementById("doctorVerdictBadge");
    const verdict = document.getElementById("doctorVerdict");
    const detail = document.getElementById("doctorVerdictDetail");
    if (verdictBadge && verdict && detail) {
      const level = first ? first.level : pending.length ? "pending" : "ok";
      verdictBadge.className = doctorBadgeClass(level);
      verdictBadge.textContent = doctorBadgeLabel(level);
      verdict.textContent = first
        ? t("doctorIssuesVerdict", { count: actionable.length })
        : pending.length
          ? t("doctorScanningVerdict")
          : t("doctorReadyVerdict");
      detail.textContent = first
        ? t("doctorIssueDetail", { item: first.summary })
        : state
          ? t("doctorReadyDetail")
          : t("doctorNoState");
    }
    renderDoctorQueue(actionable, pending);
  }

  function renderDoctorQueue(actionable, pending) {
    const queue = document.getElementById("doctorQueue");
    if (!queue) return;
    queue.replaceChildren();

    const rows = actionable.length ? actionable : [];
    if (!rows.length) {
      const empty = document.createElement("p");
      empty.className = "bridge-state";
      empty.textContent = pending.length ? t("doctorScanningVerdict") : t("doctorQueueEmpty");
      queue.appendChild(empty);
      if (!pending.length) {
        const button = document.createElement("button");
        button.className = "text-button primary";
        button.type = "button";
        button.textContent = t("doctorQueueLaunchReady");
        button.addEventListener("click", doctorFixAndLaunch);
        queue.appendChild(button);
      }
      return;
    }

    rows.forEach((item) => {
      const row = document.createElement("article");
      row.className = "doctor-queue-item " + (item.level === "err" ? "err" : "warn");
      const text = document.createElement("div");
      const title = document.createElement("strong");
      title.textContent = item.summary;
      const meta = document.createElement("span");
      meta.textContent = item.meta || "-";
      text.append(title, meta);

      const button = document.createElement("button");
      button.className = "text-button";
      button.type = "button";
      button.textContent = t("doctorQueueGo");
      button.addEventListener("click", () => setPage(item.page));
      row.append(text, button);
      queue.appendChild(row);
    });
  }

  async function doctorFixAndLaunch() {
    if (!state) {
      await refreshState({ silent: true });
    }
    if (!overviewDiagnosticsStarted) {
      await hydrateOverviewDiagnostics();
    }
    const first = doctorPrimaryItem();
    if (first) {
      setPage(first.page);
      log("WARN", t("doctorIssueDetail", { item: first.summary }));
      return;
    }
    await handleAction("launchClaudeInjected");
  }

  function renderSystemReadiness() {
    if (!systemLoaded) {
      const value = systemLoading ? t("loading") : "-";
      [
        "windowsStatus",
        "adminStatus",
        "firmwareVirtualization",
        "hypervisorLaunchType",
        "hypervisorRuntime",
        "modernInstallStatus",
        "appxPackage",
        "vmpStatus",
        "hypervisorStatus",
        "rebootStatus",
      ].forEach((id) => {
        document.getElementById(id).textContent = value;
      });
      renderDoctorReport();
      return;
    }
    const system = state?.system || {};
    const windows = [system.os_name, system.os_build ? `build ${system.os_build}` : ""].filter(Boolean).join(" / ");
    document.getElementById("windowsStatus").textContent = windows || "-";
    document.getElementById("adminStatus").textContent = system.is_admin ? t("adminOk") : t("adminMissing");
    document.getElementById("firmwareVirtualization").textContent = formatBool(system.virtualization_firmware_enabled);
    document.getElementById("hypervisorLaunchType").textContent = system.hypervisor_launch_type || "-";
    document.getElementById("hypervisorRuntime").textContent = formatBool(system.hypervisor_present);
    document.getElementById("modernInstallStatus").textContent = system.claude_modern_installer
      ? t("modernInstallOk")
      : t("modernInstallMissing");
    document.getElementById("appxPackage").textContent = system.claude_appx_package || "-";
    document.getElementById("vmpStatus").textContent = formatFeatureState(system.virtual_machine_platform);
    document.getElementById("hypervisorStatus").textContent = [
      formatFeatureState(system.hypervisor_platform),
      system.hyper_v ? `Hyper-V: ${formatFeatureState(system.hyper_v)}` : "",
    ].filter(Boolean).join(" / ") || "-";
    document.getElementById("rebootStatus").textContent = system.reboot_required ? t("rebootRequired") : t("rebootNotRequired");
    renderDoctorReport();
  }

  function renderDeveloperCapabilities() {
    const status = capabilitiesStatus;
    const loading = capabilitiesLoading ? t("loading") : "-";
    setText("capabilityBrowserMcp", status ? formatReady(status.browser_mcp_configured) : loading);
    setText("capabilityWorkspaceMcp", status ? formatReady(status.workspace_mcp_configured) : loading);
    setText("capabilityNpx", status ? formatReady(status.npx_available) : loading);
    setText("capabilityWorkspace", status?.workspace_path || loading);

    const list = document.getElementById("capabilityConfigList");
    if (!list) return;
    list.replaceChildren();
    const paths = status?.config_paths || [];
    if (!paths.length) {
      const empty = document.createElement("p");
      empty.className = "bridge-state";
      empty.textContent = capabilitiesLoading ? t("loading") : "-";
      list.appendChild(empty);
      return;
    }
    paths.forEach((target) => {
      const item = document.createElement("article");
      item.className = "provider-item" + (target.browser_mcp || target.workspace_mcp ? " active" : "");
      const details = document.createElement("div");
      const title = document.createElement("strong");
      title.textContent = target.label;
      const path = document.createElement("span");
      path.textContent = target.path;
      const summary = document.createElement("code");
      summary.textContent = [
        `${t("capabilityBrowserMcp")}: ${formatReady(target.browser_mcp)}`,
        `${t("capabilityWorkspaceMcp")}: ${formatReady(target.workspace_mcp)}`,
        `${target.writable ? t("capabilityWritable") : t("fileMissing")}`,
      ].join(" / ");
      details.append(title, path, summary);
      item.appendChild(details);
      list.appendChild(item);
    });
    renderDoctorReport();
  }

  function renderCapabilityLog(status) {
    const logBox = document.getElementById("capabilityActionLog");
    if (!logBox) return;
    logBox.replaceChildren();
    [
      status.message,
      `${t("capabilityBrowserMcp")}: ${formatReady(status.browser_mcp_configured)}`,
      `${t("capabilityWorkspaceMcp")}: ${formatReady(status.workspace_mcp_configured)}`,
      `${t("capabilityWorkspace")}: ${status.workspace_path}`,
      t("capabilityNeedsRestart"),
    ]
      .filter(Boolean)
      .forEach((line) => {
        const item = document.createElement("p");
        item.textContent = line;
        logBox.appendChild(item);
      });
  }

  function renderOfficialPlugins() {
    const status = officialPluginsStatus;
    const loading = officialPluginsLoading ? t("loading") : "-";
    setText("officialPluginsCli", status ? (status.claude_cli_available ? status.claude_cli_path || t("capabilityReady") : t("capabilityMissing")) : loading);
    setText("officialPluginsMarketplace", status ? (status.marketplace_configured ? status.marketplace_path || status.marketplace_name : t("capabilityMissing")) : loading);
    setText("officialPluginsCount", status ? String(status.marketplace_plugin_count || 0) : loading);
    setText("officialPluginsLastUpdated", status?.marketplace_last_updated || loading);

    const search = document.getElementById("officialPluginSearch");
    if (search && search.value !== officialPluginSearchQuery) {
      search.value = officialPluginSearchQuery;
    }
    const list = document.getElementById("officialPluginList");
    if (!list) return;
    list.replaceChildren();
    const plugins = (status?.plugins?.length ? status.plugins : status?.featured_plugins) || [];
    const filteredPlugins = filterOfficialPlugins(plugins);
    const visiblePlugins = filteredPlugins.slice(0, officialPluginVisibleLimit);
    setText(
      "officialPluginVisibleCount",
      status
        ? t("officialPluginsVisibleCount", {
            visible: Math.min(visiblePlugins.length, filteredPlugins.length),
            total: filteredPlugins.length,
          })
        : loading
    );
    const loadMore = document.getElementById("officialPluginLoadMore");
    if (loadMore) {
      loadMore.hidden = !status || visiblePlugins.length >= filteredPlugins.length;
      loadMore.disabled = officialPluginsLoading;
    }
    if (!plugins.length) {
      const empty = document.createElement("p");
      empty.className = "bridge-state";
      empty.textContent = officialPluginsLoading ? t("loading") : t("officialPluginsEmpty");
      list.appendChild(empty);
      renderDoctorReport();
      return;
    }
    if (!filteredPlugins.length) {
      const empty = document.createElement("p");
      empty.className = "bridge-state";
      empty.textContent = t("officialPluginsNoResults");
      list.appendChild(empty);
      renderDoctorReport();
      return;
    }
    visiblePlugins.forEach((plugin) => {
      const localizedPlugin = localizeOfficialPlugin(plugin);
      const item = document.createElement("article");
      item.className = "provider-item" + (plugin.installed ? " active" : "");
      const details = document.createElement("div");
      const title = document.createElement("strong");
      title.textContent = localizedPlugin.name;
      const summary = document.createElement("span");
      summary.textContent = [
        localizedPlugin.category,
        plugin.author,
        plugin.install_count ? t("officialPluginsInstalls", { count: plugin.install_count }) : "",
      ].filter(Boolean).join(" / ");
      const description = document.createElement("code");
      description.textContent = localizedPlugin.description || plugin.source || "-";
      details.append(title, summary, description);

      const actions = document.createElement("div");
      actions.className = "provider-actions";
      if (plugin.homepage) {
        const docsAction = document.createElement("button");
        docsAction.type = "button";
        docsAction.className = "text-button";
        docsAction.dataset.action = "openExternalUrl";
        docsAction.dataset.url = plugin.homepage;
        docsAction.textContent = t("recommendationDocs");
        docsAction.addEventListener("click", () => openExternalUrl(plugin.homepage));
        actions.appendChild(docsAction);
      }
      const installAction = document.createElement("button");
      installAction.type = "button";
      installAction.className = "text-button primary";
      installAction.dataset.action = "installOfficialPlugin";
      installAction.dataset.plugin = plugin.name;
      installAction.disabled = officialPluginsLoading || plugin.installed || !status?.claude_cli_available;
      installAction.textContent = plugin.installed ? t("officialPluginsInstalled") : t("officialPluginsInstall");
      installAction.addEventListener("click", () => installOfficialPlugin(plugin.name));
      actions.appendChild(installAction);
      item.append(details, actions);
      list.appendChild(item);
    });
    renderDoctorReport();
  }

  function filterOfficialPlugins(plugins) {
    const query = officialPluginSearchQuery.trim().toLowerCase();
    if (!query) return plugins;
    return plugins.filter((plugin) =>
      [
        plugin.name,
        plugin.plugin_id,
        plugin.description,
        plugin.category,
        plugin.author,
        plugin.homepage,
        plugin.source,
        localizeOfficialPlugin(plugin).category,
        localizeOfficialPlugin(plugin).description,
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase()
        .includes(query)
    );
  }

  function localizeOfficialPlugin(plugin) {
    if (locale !== "zh") {
      return {
        name: plugin.name,
        category: plugin.category,
        description: plugin.description,
      };
    }
    const translated = officialPluginTranslations.zh[plugin.name] || {};
    return {
      name: translated.name || plugin.name,
      category: translated.category || translateOfficialPluginCategory(plugin.category),
      description: translated.description || plugin.description,
    };
  }

  function translateOfficialPluginCategory(category) {
    const categories = {
      automation: "自动化",
      database: "数据库",
      deployment: "部署",
      design: "设计",
      development: "开发",
      location: "位置服务",
      migration: "迁移",
      monitoring: "监控",
      productivity: "生产力",
      security: "安全",
    };
    return categories[String(category || "").toLowerCase()] || category;
  }

  function renderOfficialPluginLog(result) {
    const logBox = document.getElementById("officialPluginActionLog");
    if (!logBox) return;
    logBox.replaceChildren();
    [result.message, result.stdout, result.stderr]
      .filter(Boolean)
      .join("\n")
      .split(/\r?\n/)
      .filter(Boolean)
      .slice(-16)
      .forEach((line) => {
        const item = document.createElement("p");
        item.textContent = line;
        logBox.appendChild(item);
      });
  }

  function formatReady(value) {
    return value ? t("capabilityReady") : t("capabilityMissing");
  }

  function formatBool(value) {
    if (value === true) return t("boolYes");
    if (value === false) return t("boolNo");
    return t("unknown");
  }

  function formatFeatureState(value) {
    if (!value) return "-";
    const normalized = String(value).toLowerCase();
    if (normalized === "enabled") return t("featureEnabled");
    if (normalized === "disabled") return t("featureDisabled");
    if (normalized === "enablepending" || normalized === "disablepending") return t("featurePending");
    return value;
  }

  function localSystemHint(system) {
    if (!system?.is_admin) return t("vmpLocalHintAdmin");
    if (system.virtualization_firmware_enabled === false) return t("vmpLocalHintFirmware");
    if (String(system.virtual_machine_platform || "").toLowerCase() === "enabled") return t("vmpLocalHintReady");
    if (system.reboot_required) return t("vmpLocalHintRestart");
    return t("vmpLocalHintMissing");
  }

  function renderSystemAction(result) {
    state.system = result.system || state.system;
    systemLoaded = true;
    renderSystemReadiness();
    const logBox = document.getElementById("systemActionLog");
    logBox.replaceChildren();
    [localSystemHint(result.system), result.message, result.stdout, result.stderr]
      .filter(Boolean)
      .join("\n")
      .split(/\r?\n/)
      .filter(Boolean)
      .slice(-18)
      .forEach((line) => {
        const item = document.createElement("p");
        item.textContent = line;
        logBox.appendChild(item);
      });
  }

  function historyDefaultItems(profile) {
    return (profile?.items || []).filter((entry) => entry.exists && entry.default_restore);
  }

  function recommendedHistoryProfile(scan) {
    const candidates = (scan?.profiles || []).filter((profile) => !profile.is_target && historyDefaultItems(profile).length);
    candidates.sort((a, b) => {
      const filesA = historyDefaultItems(a).reduce((sum, entry) => sum + (Number(entry.file_count) || 0), 0);
      const filesB = historyDefaultItems(b).reduce((sum, entry) => sum + (Number(entry.file_count) || 0), 0);
      return filesB - filesA || (Number(b.latest_write_ms) || 0) - (Number(a.latest_write_ms) || 0);
    });
    return candidates[0] || null;
  }

  function historyRecoverableSummary(profile) {
    const items = historyDefaultItems(profile);
    if (!items.length) return t("historyNoItems");
    const names = items.map((entry) => entry.label).join(" / ");
    const files = items.reduce((sum, entry) => sum + (Number(entry.file_count) || 0), 0);
    return t("historyRecoverableSummary", {
      items: names,
      files,
      size: formatBytes(profile?.total_bytes || 0),
    });
  }

  function renderHistory(scan) {
    const target = document.getElementById("historyTargetPath");
    const backup = document.getElementById("historyBackupRoot");
    const recommendedPath = document.getElementById("historyRecommendedSource");
    const summary = document.getElementById("historyOneClickSummary");
    const recoverable = document.getElementById("historyRecoverableSummary");
    const autoButton = document.getElementById("historyRepairAutoButton");
    const list = document.getElementById("historyProfiles");
    if (!target || !backup || !list) return;

    target.textContent = scan?.target_path || "-";
    backup.textContent = scan?.backup_root || "-";
    const recommended = recommendedHistoryProfile(scan);
    if (recommendedPath) {
      recommendedPath.textContent = recommended ? `${recommended.name} (${recommended.path})` : "-";
    }
    if (summary) {
      summary.textContent = !historyLoaded
        ? historyLoading
          ? t("loading")
          : t("historyScanIdle")
        : recommended
          ? t("historyAutoReady", { source: recommended.name })
          : t("historyAutoMissing");
    }
    if (recoverable) {
      recoverable.textContent = recommended ? historyRecoverableSummary(recommended) : "-";
    }
    if (autoButton) {
      autoButton.disabled = historyLoading || !recommended;
    }
    list.replaceChildren();

    if (!historyLoaded) {
      const empty = document.createElement("p");
      empty.className = "bridge-state";
      empty.textContent = historyLoading ? t("loading") : t("historyScanIdle");
      list.appendChild(empty);
      return;
    }

    const profiles = scan?.profiles || [];
    if (!profiles.length) {
      const empty = document.createElement("p");
      empty.className = "bridge-state";
      empty.textContent = t("historyEmpty");
      list.appendChild(empty);
      return;
    }

    profiles.forEach((profile) => {
      const item = document.createElement("article");
      item.className = "provider-item" + (profile.is_target ? " active" : "");

      const details = document.createElement("div");
      const title = document.createElement("strong");
      title.textContent = profile.name + (profile.is_target ? ` / ${t("historyCurrent")}` : "");

      const path = document.createElement("span");
      path.textContent = profile.path;

      const stats = document.createElement("code");
      stats.textContent = `${t("historyFiles")}: ${profile.file_count || 0} / ${t("historyBytes")}: ${formatBytes(profile.total_bytes || 0)} / ${t("historyLatest")}: ${formatDateMs(profile.latest_write_ms)}`;

      const existingItems = (profile.items || []).filter((entry) => entry.exists);
      const defaultItems = historyDefaultItems(profile);
      const itemText = document.createElement("span");
      itemText.textContent = existingItems.length
        ? `${t("historyItems")}: ${existingItems.map((entry) => `${entry.label} (${entry.file_count})`).join(" / ")}${defaultItems.length < existingItems.length ? " / " + t("historyDefaultSkipped") : ""}`
        : t("historyNoItems");

      details.append(title, path, stats, itemText);
      item.append(details);
      list.appendChild(item);
    });
  }

  function renderRecommendations() {
    const list = document.getElementById("providerPresetList");
    if (list) {
      list.replaceChildren();
      recommendedProviders.forEach((preset) => {
        const card = document.createElement("article");
        card.className = "recommendation-card" + (preset.directReady ? " ready" : "");

        const head = document.createElement("div");
        head.className = "recommendation-head";
        const title = document.createElement("strong");
        title.textContent = preset.name;
        const badge = document.createElement("span");
        badge.className = preset.directReady ? "seal ok" : "seal warn";
        badge.textContent = preset.directReady ? t("recommendationDirectReady") : t("recommendationNeedsAdapter");
        head.append(title, badge);

        const meta = document.createElement("p");
        meta.className = "recommendation-meta";
        meta.textContent = `${t("recommendationProtocol")}: ${preset.protocol}`;

        const base = document.createElement("code");
        base.textContent = `${t("recommendationBase")}: ${preset.baseUrl}`;

        const note = document.createElement("p");
        note.className = "recommendation-note";
        note.textContent = preset.note[locale] || preset.note.en;

        const actions = document.createElement("div");
        actions.className = "button-row";
        const fill = document.createElement("button");
        fill.className = "text-button primary";
        fill.type = "button";
        fill.textContent = t("recommendationUse");
        fill.addEventListener("click", () => fillRecommendationProvider(preset));

        const save = document.createElement("button");
        save.className = "text-button";
        save.type = "button";
        save.textContent = t("recommendationSave");
        save.addEventListener("click", () => saveRecommendationProvider(preset));

        const docs = document.createElement("button");
        docs.className = "text-button";
        docs.type = "button";
        docs.textContent = t("recommendationDocs");
        docs.addEventListener("click", () => window.open(preset.docsUrl, "_blank", "noopener"));

        actions.append(fill, save, docs);
        card.append(head, meta, base, note, actions);
        list.appendChild(card);
      });
    }

    const modes = document.getElementById("recommendedModes");
    if (modes) {
      modes.replaceChildren();
      recommendedModes.forEach((mode) => {
        const card = document.createElement("article");
        card.className = "recommendation-card";
        const title = document.createElement("strong");
        title.textContent = t(mode.titleKey);
        const body = document.createElement("p");
        body.className = "recommendation-note";
        body.textContent = t(mode.bodyKey);
        card.append(title, body);
        modes.appendChild(card);
      });
    }

    const aboutVersion = document.getElementById("aboutVersion");
    const aboutInstallPath = document.getElementById("aboutInstallPath");
    const aboutConfigPath = document.getElementById("aboutConfigPath");
    if (aboutVersion) aboutVersion.textContent = "0.1.32";
    if (aboutInstallPath) aboutInstallPath.textContent = state?.install?.executable || "-";
    if (aboutConfigPath) aboutConfigPath.textContent = state?.config?.config_path || "-";

    const featureList = document.getElementById("aboutFeatureList");
    if (featureList) {
      featureList.replaceChildren();
      aboutFeatures.forEach((feature) => {
        const card = document.createElement("article");
        card.className = "recommendation-card";
        const title = document.createElement("strong");
        title.textContent = t(feature.titleKey);
        const body = document.createElement("p");
        body.className = "recommendation-note";
        body.textContent = t(feature.bodyKey);
        card.append(title, body);
        featureList.appendChild(card);
      });
    }
  }

  function openProviderModal({ provider = null, preset = null } = {}) {
    const modal = document.getElementById("providerModal");
    const form = document.getElementById("providerForm");
    const source = provider || preset || {};
    const protocol = provider?.protocol || (preset ? presetProtocolValue(preset) : "anthropic");

    form.reset();
    formField(form, "id").value = provider?.id || "";
    formField(form, "name").value = source.name || "";
    formField(form, "app_type").value = provider?.app_type || "claude";
    formField(form, "protocol").value = protocol;
    formField(form, "base_url").value = source.base_url || source.baseUrl || "";
    formField(form, "api_key").value = "";
    formField(form, "api_key").placeholder = provider?.has_key ? `${provider.key_mask} / ${t("providerFormHint")}` : "sk-...";
    formField(form, "enabled").checked = provider ? Boolean(provider.enabled) : true;

    const mappings = provider?.model_mappings?.length
      ? provider.model_mappings
      : [];
    renderMappingRows(mappings);
    syncMappingVisibility();
    setModelDiscoveryStatus(t("modelDiscoveryStatusIdle"));

    document.getElementById("providerModalTitle").textContent = provider ? t("editingProvider") : t("providerModalTitle");
    document.getElementById("providerFormTitle").textContent = provider ? t("editingProvider") : t("manualProvider");
    modal.hidden = false;
    setPage("providers");
    window.setTimeout(() => formField(form, "name").focus({ preventScroll: true }), 0);
  }

  function closeProviderModal() {
    const modal = document.getElementById("providerModal");
    if (modal) modal.hidden = true;
  }

  function setModelDiscoveryStatus(message, level = "") {
    const status = document.getElementById("providerModelDiscoveryStatus");
    if (!status) return;
    status.textContent = message;
    status.classList.toggle("ok", level === "ok");
    status.classList.toggle("warn", level === "warn");
    status.classList.toggle("err", level === "err");
  }

  function renderMappingRows(mappings = []) {
    const rows = document.getElementById("mappingRows");
    rows.replaceChildren();
    mappings.forEach((mapping) => addMappingRow(mapping));
  }

  function addMappingRow(mapping = {}) {
    const rows = document.getElementById("mappingRows");
    const row = document.createElement("div");
    row.className = "mapping-row";

    [
      ["claude_route", "mappingClaudeRoute", "claude-opus-4-5"],
      ["target_model", "mappingTargetModel", "gpt-5.5"],
      ["label", "mappingLabel", "gpt-5.5 via claude-opus-4-5"],
    ].forEach(([field, labelKey, placeholder]) => {
      const label = document.createElement("label");
      const span = document.createElement("span");
      span.textContent = t(labelKey);
      const input = document.createElement("input");
      input.type = "text";
      input.dataset.field = field;
      input.placeholder = placeholder;
      input.value = mapping[field] || "";
      label.append(span, input);
      row.appendChild(label);
    });

    const enabled = document.createElement("label");
    enabled.className = "check-line";
    const enabledInput = document.createElement("input");
    enabledInput.type = "checkbox";
    enabledInput.dataset.field = "enabled";
    enabledInput.checked = mapping.enabled !== false;
    const enabledText = document.createElement("span");
    enabledText.textContent = t("mappingEnabled");
    enabled.append(enabledInput, enabledText);

    const remove = document.createElement("button");
    remove.className = "text-button danger";
    remove.type = "button";
    remove.textContent = t("removeMapping");
    remove.addEventListener("click", () => row.remove());

    row.append(enabled, remove);
    rows.appendChild(row);
  }

  function collectModelMappings() {
    const form = document.getElementById("providerForm");
    if (formField(form, "protocol").value !== "openai") {
      return [];
    }
    return Array.from(document.querySelectorAll("#mappingRows .mapping-row"))
      .map((row) => {
        const get = (field) => row.querySelector(`[data-field="${field}"]`);
        const claudeRoute = get("claude_route")?.value.trim() || "";
        const targetModel = get("target_model")?.value.trim() || "";
        return {
          claude_route: claudeRoute,
          target_model: targetModel,
          label: get("label")?.value.trim() || `${targetModel} via ${claudeRoute}`,
          enabled: get("enabled")?.checked ?? true,
        };
      })
      .filter((mapping) => mapping.claude_route && mapping.target_model);
  }

  function syncMappingVisibility() {
    const form = document.getElementById("providerForm");
    const editor = document.getElementById("mappingEditor");
    const openai = formField(form, "protocol").value === "openai";
    editor.hidden = !openai;
  }

  function modelMappingSummary(provider) {
    if (provider.protocol !== "openai") return "";
    const mappings = (provider.model_mappings || []).filter((mapping) => mapping.enabled);
    if (!mappings.length) {
      return `${t("modelMappingTitle")}: AUTO`;
    }
    const summary = mappings
      .slice(0, 3)
      .map((mapping) => `${mapping.claude_route} -> ${mapping.target_model}`)
      .join(" / ");
    return `${t("modelMappingTitle")}: ${summary}${mappings.length > 3 ? ` +${mappings.length - 3}` : ""}`;
  }

  function fillRecommendationProvider(preset) {
    openProviderModal({ preset });
    log("OK", t("recommendationFilled", { provider: preset.name }));
  }

  async function saveRecommendationProvider(preset) {
    await runCommand(
      "save_api_provider",
      {
        provider: {
          name: preset.name,
          app_type: "claude",
          protocol: presetProtocolValue(preset),
          base_url: preset.baseUrl,
          api_key: "",
          model_mappings: [],
          enabled: true,
        },
      },
      (config) => {
        if (state) {
          state.config = config;
        }
        renderState();
        log("OK", t("recommendationSaved", { provider: preset.name }));
      },
    );
  }

  function presetProtocolValue(preset) {
    return String(preset.protocol || "").toLowerCase().includes("openai") ? "openai" : "anthropic";
  }

  function renderHistoryRepair(result) {
    const logBox = document.getElementById("historyRepairLog");
    if (!logBox) return;
    logBox.replaceChildren();
    [
      result.message,
      t("historyBoundary"),
      `${t("historyFiles")}: ${result.copied_files}`,
      `${t("historyBytes")}: ${formatBytes(result.copied_bytes || 0)}`,
      `${t("historyBackupRoot")}: ${result.backup_path}`,
      `${t("historyItems")}: ${(result.restored_items || []).join(" / ")}`,
    ]
      .filter(Boolean)
      .forEach((line) => {
        const item = document.createElement("p");
        item.textContent = line;
        logBox.appendChild(item);
      });
  }

  function renderHistoryError(error) {
    const logBox = document.getElementById("historyRepairLog");
    if (!logBox) return;
    logBox.replaceChildren();
    const item = document.createElement("p");
    item.textContent = t("historyRepairFailed", { error: String(error) });
    logBox.appendChild(item);
  }

  function formatBytes(bytes) {
    const value = Number(bytes) || 0;
    if (value < 1024) return `${value} B`;
    const units = ["KB", "MB", "GB", "TB"];
    let current = value / 1024;
    for (const unit of units) {
      if (current < 1024) return `${current.toFixed(current >= 100 ? 0 : 1)} ${unit}`;
      current /= 1024;
    }
    return `${current.toFixed(1)} PB`;
  }

  function formatDateMs(ms) {
    if (!ms) return "-";
    const timestamp = Number(ms);
    if (!Number.isFinite(timestamp)) return "-";
    return new Date(timestamp).toLocaleString(locale === "zh" ? "zh-CN" : "en-US");
  }

  function renderGatewayStatus() {
    const gateway = state.gateway;
    const configGateway = state.config.gateway || { enabled: true, port: 49331 };
    const gatewayEffective = Boolean(gateway?.enabled ?? configGateway.enabled);
    const fallbackUrl = `http://127.0.0.1:${configGateway.port || 49331}`;
    const summary = document.getElementById("gatewaySummary");
    const url = gateway?.url || fallbackUrl;
    if (!gatewayEffective) {
      summary.textContent = t("direct3pMode");
    } else if (gateway?.running) {
      summary.textContent = t("gatewayRunning", { url });
    } else {
      summary.textContent = t("gatewayStopped");
    }
    setText("gatewayUrl", gatewayEffective ? url : t("direct3pMode"));
    setText("gatewayTarget", gateway?.target_base_url || "-");
    const requests = gatewayEffective ? String(gateway?.request_count ?? 0) : "-";
    const forwarded = gatewayEffective ? String(gateway?.forwarded_count ?? 0) : "-";
    const lastRequest = gatewayEffective ? gateway?.last_request_path || "-" : "-";
    const upstreamStatus = gatewayEffective && gateway?.last_upstream_status ? String(gateway.last_upstream_status) : "-";
    const upstreamError = gatewayEffective ? gateway?.last_upstream_error || "-" : "-";
    setText("gatewayRequests", requests);
    setText("gatewayForwarded", forwarded);
    setText("gatewayLastRequest", lastRequest);
    setText("gatewayUpstreamStatus", upstreamStatus);
    setText("gatewayUpstreamError", upstreamError);
    setText("gatewayRequestsMirror", requests);
    setText("gatewayLastRequestMirror", lastRequest);
    setText("gatewayUpstreamStatusMirror", upstreamStatus);
    setText("gatewayUpstreamErrorMirror", upstreamError);
  }

  function renderConnectionMode(gateway) {
    const gatewayEnabled = Boolean(gateway?.enabled);
    const directButton = document.getElementById("directModeButton");
    const gatewayButton = document.getElementById("gatewayModeButton");
    if (!directButton || !gatewayButton) return;
    directButton.classList.toggle("active", !gatewayEnabled);
    gatewayButton.classList.toggle("active", gatewayEnabled);
    directButton.disabled = false;
    directButton.title = "";
    directButton.setAttribute("aria-pressed", String(!gatewayEnabled));
    gatewayButton.setAttribute("aria-pressed", String(gatewayEnabled));
  }

  function renderVerification() {
    const status = lastLaunchResult?.claude_3p || state?.claude_3p || null;
    const verification = lastLaunchResult?.verification || null;
    const verdict = verification?.verdict || "no_run";
    const label = verdictLabel(verdict);
    const badge = document.getElementById("verificationBadge");

    badge.textContent = label;
    const isProviderError = verdict === "provider_credentials_rejected" || verdict === "gateway_hit_upstream_error";
    badge.classList.toggle("ok", verdict === "verified_gateway_hit");
    badge.classList.toggle("warn", verdict === "3p_config_applied_but_no_gateway_request_yet");
    badge.classList.toggle("err", verdict === "still_official_login_or_1p_network" || isProviderError);

    document.getElementById("verificationVerdict").textContent = label;
    document.getElementById("claude3pApplied").textContent = status
      ? `${status.applied ? t("claude3pReady") : t("claude3pMissing")} / desktop:${status.file_exists ? t("filePresent") : t("fileMissing")} / meta:${status.meta_exists ? t("filePresent") : t("fileMissing")} / config:${status.active_config_exists ? t("filePresent") : t("fileMissing")}`
      : "-";
    document.getElementById("claude3pDeploymentMode").textContent = status?.deployment_mode || "-";
    document.getElementById("claude3pAppliedId").textContent = status?.applied_id || status?.config_id || "-";
    document.getElementById("claude3pDesktopConfig").textContent = status?.desktop_config_path || "-";
    document.getElementById("claude3pMetaPath").textContent = status?.meta_path || "-";
    document.getElementById("claude3pConfigPath").textContent = status?.config_path || "-";

    if (verification) {
      document.getElementById("gatewayRequests").textContent = String(verification.request_delta);
      document.getElementById("gatewayForwarded").textContent = String(verification.forwarded_delta);
      document.getElementById("gatewayLastRequest").textContent = verification.last_request_path || "-";
      document.getElementById("gatewayUpstreamStatus").textContent = verification.last_upstream_status ? String(verification.last_upstream_status) : "-";
      document.getElementById("gatewayUpstreamError").textContent = verification.last_upstream_error || "-";
    }

    const logs = document.getElementById("verificationLogs");
    logs.replaceChildren();
    const evidence = verification?.claude_log_evidence || [];
    if (!evidence.length) {
      const empty = document.createElement("p");
      empty.textContent = t("verificationNoRun");
      logs.appendChild(empty);
      return;
    }
    evidence.forEach((line) => {
      const item = document.createElement("p");
      item.textContent = line;
      logs.appendChild(item);
    });
  }

  function verdictLabel(verdict) {
    if (verdict === "verified_gateway_hit") return t("verificationGatewayHit");
    if (verdict === "verified_direct_3p_config") return t("verificationDirect3p");
    if (verdict === "provider_credentials_rejected") return t("verificationProviderRejected");
    if (verdict === "gateway_hit_upstream_error") return t("verificationUpstreamError");
    if (verdict === "3p_config_applied_but_no_gateway_request_yet") return t("verificationConfigNoHit");
    if (verdict === "still_official_login_or_1p_network") return t("verificationStill1p");
    if (verdict === "not_verified") return t("verificationNotVerified");
    return t("verificationNoRun");
  }

  function injectLaunchSummary(result) {
    const parts = [t("injectLaunchVerified", { pid: result.process_id, verdict: verdictLabel(result.verification?.verdict) })];
    parts.push(launcherRouteLabel(result.launcher_route));
    parts.push(injectionChannelLabel(result.injection_channel));
    if (result.gateway_url) {
      parts.push(`Gateway ${result.gateway_url}`);
    } else if (result.claude_3p) {
      parts.push(t("direct3pMode"));
    }
    if (result.cdp_injected) {
      parts.push(t("cdpInjected", { port: result.cdp_port || "-" }));
    } else if (result.cdp_error) {
      parts.push(t("cdpFailed", { error: cdpErrorLabel(result.cdp_error) }));
    }
    return parts.join(" / ");
  }

  function defaultInjectionChannel(install) {
    if (!install) return null;
    const gatewayEnabled = Boolean(state?.config?.gateway?.enabled);
    if (install.live_injection_supported) {
      return gatewayEnabled ? "live_script_plus_gateway_config" : "live_script_plus_direct_config";
    }
    return gatewayEnabled ? "config_injection_plus_gateway" : "config_injection_direct";
  }

  function launcherRouteLabel(route) {
    if (route === "external_launcher_localized_sidecar") return t("launcherRouteLocalizedSidecar");
    if (route === "external_launcher_app_activation") return t("launcherRouteAppActivation");
    if (route === "external_launcher_process") return t("launcherRouteExternalProcess");
    return "-";
  }

  function injectionChannelLabel(channel) {
    if (channel === "diagnostic_clean_launch") return t("injectionDiagnosticClean");
    if (channel === "live_script_plus_gateway_config") return t("injectionLiveGateway");
    if (channel === "live_script_plus_direct_config") return t("injectionLiveDirect");
    if (channel === "preload_script_plus_gateway_config") return t("injectionPreloadGateway");
    if (channel === "preload_script_plus_direct_config") return t("injectionPreloadDirect");
    if (channel === "config_injection_plus_gateway") return t("injectionConfigGateway");
    if (channel === "config_injection_direct") return t("injectionConfigDirect");
    return "-";
  }

  function liveInjectionLabel(result, install) {
    if (String(result?.injection_channel || "").startsWith("preload_script")) return t("preloadInjectionReady");
    if (result?.live_injection_attempted) return t("liveInjectionAttempted");
    if (result?.live_injection_supported || install?.live_injection_supported) return t("liveInjectionReady");
    if (install || result) return t("liveInjectionConfigOnly");
    return "-";
  }

  function cdpErrorLabel(error) {
    if (String(error).includes("MSIX Claude Desktop")) {
      return t("cdpMsixUnavailable");
    }
    return error;
  }

  function renderProviderTest() {
    const result = lastProviderTest;
    document.getElementById("providerTestTarget").textContent = result ? `${result.provider_name} / ${result.url}` : "-";
    document.getElementById("providerTestStatus").textContent = result?.status ? String(result.status) : "-";
    document.getElementById("providerTestCode").textContent = result?.code || "-";
    document.getElementById("providerTestMessage").textContent = result?.compatibility_message || result?.message || "-";
    document.getElementById("providerTestBody").textContent = result
      ? `${t("providerProtocolLabel")}: ${providerProtocolLabel(result.protocol)} / ${t("providerModels")}: ${result.model_count ?? "-"} / ${result.body_excerpt || "-"}`
      : "-";
  }

  function providerTypeLabel(appType) {
    if (appType === "claude-desktop") return t("providerTypeClaudeDesktop");
    return t("providerTypeClaude");
  }

  function providerProtocolLabel(protocol) {
    return protocol === "openai" ? t("providerProtocolOpenAI") : t("providerProtocolAnthropic");
  }

  function renderProviders(providers) {
    const list = document.getElementById("providerList");
    const sandbox = state?.config?.sandbox || {};
    list.replaceChildren();
    if (!providers.length) {
      const empty = document.createElement("p");
      empty.className = "bridge-state";
      empty.textContent = t("providerEmpty");
      list.appendChild(empty);
      return;
    }

    providers.forEach((provider) => {
      const canInject = canInjectProvider(provider, sandbox);
      const item = document.createElement("article");
      item.className = "provider-item" + (provider.active ? " active" : "");
      const details = document.createElement("div");
      details.innerHTML = `
        <strong></strong>
        <span></span>
        <code></code>
        <span></span>
        <span></span>
        <span></span>
      `;
      details.querySelector("strong").textContent = provider.name + (provider.active ? ` / ${t("providerActive")}` : "");
      const spans = details.querySelectorAll("span");
      spans[0].textContent = `${t("providerSource")}: ${provider.source} / ${t("providerProtocolLabel")}: ${providerProtocolLabel(provider.protocol)}`;
      details.querySelector("code").textContent = provider.base_url || "-";
      spans[1].textContent = `${canInject ? t("providerInjectable") : t("providerSwitchOnly")} / ${t("providerKey")}: ${provider.has_key ? provider.key_mask : "-"} / ${provider.enabled ? t("providerEnabled") : t("providerDisabled")}`;
      const testResult = providerTestResults.get(provider.id);
      spans[2].textContent = testResult
        ? `${t("providerTestLocalResult")}: HTTP ${testResult.status || "-"} / ${t("providerModels")}: ${testResult.model_count ?? "-"} / ${testResult.compatibility_message || "-"}`
        : `${t("providerCompatibility")}: -`;
      const mappingLine = modelMappingSummary(provider);
      spans[3].textContent = mappingLine;
      spans[3].hidden = !mappingLine;

      const actions = document.createElement("div");
      actions.className = "action-stack";

      const selectAction = document.createElement("button");
      selectAction.className = "text-button";
      selectAction.type = "button";
      selectAction.textContent = provider.active ? t("providerActive") : t("providerUse");
      selectAction.disabled = provider.active;
      selectAction.addEventListener("click", async () => {
        await runCommand("set_active_provider", { id: provider.id }, (config) => {
          state.config = config;
          renderState();
          log("OK", `${t("activeProvider")}: ${provider.name}`);
        });
      });

      const launchAction = document.createElement("button");
      launchAction.className = "text-button primary";
      launchAction.type = "button";
      launchAction.textContent = t("providerInjectLaunch");
      launchAction.disabled = !canInject;
      launchAction.addEventListener("click", async () => {
        await runCommand("launch_claude_desktop_with_provider", { id: provider.id }, (result) => {
          lastLaunchResult = result;
          state.config.active_provider_id = provider.id;
          state.claude_3p = result.claude_3p || state.claude_3p;
          renderState();
          log(result.live_injection_attempted && result.cdp_error ? "WARN" : "OK", injectLaunchSummary(result));
        });
        await refreshGatewayStatus();
        renderState();
      });

      const editAction = document.createElement("button");
      editAction.className = "text-button";
      editAction.type = "button";
      editAction.textContent = t("providerEdit");
      editAction.addEventListener("click", () => {
        loadProviderIntoForm(provider);
      });

      const testAction = document.createElement("button");
      testAction.className = "text-button";
      testAction.type = "button";
      testAction.textContent = t("providerTest");
      testAction.disabled = !provider.base_url;
      testAction.addEventListener("click", async () => {
        await testProvider(provider.id);
      });

      const deleteAction = document.createElement("button");
      deleteAction.className = "text-button danger";
      deleteAction.type = "button";
      deleteAction.textContent = t("providerDelete");
      deleteAction.addEventListener("click", async () => {
        if (!window.confirm(t("providerDeleteConfirm", { provider: provider.name }))) {
          return;
        }
        await runCommand("delete_api_provider", { id: provider.id }, (config) => {
          providerTestResults.delete(provider.id);
          state.config = config;
          renderState();
          log("OK", t("providerDeleted", { provider: provider.name }));
        });
      });

      actions.append(selectAction, launchAction, testAction, editAction, deleteAction);
      item.append(details, actions);
      list.appendChild(item);
    });
  }

  function localizationScript() {
    return state?.config?.scripts?.find((script) => script.id === "builtin-chinese-localization") || null;
  }

  function renderLocalizationStatus() {
    const script = localizationScript();
    const badge = document.getElementById("localizationBadge");
    const status = document.getElementById("localizationStatus");
    const runtime = document.getElementById("localizationRuntime");
    if (!badge || !status || !runtime) return;

    const label = script ? (script.enabled ? t("localizationEnabled") : t("localizationDisabled")) : t("localizationNotInstalled");
    badge.textContent = label;
    badge.classList.toggle("ok", Boolean(script?.enabled));
    badge.classList.toggle("warn", Boolean(script && !script.enabled));
    status.textContent = label;
    runtime.textContent = script?.enabled ? t("localizationResourcePatch") : liveInjectionLabel(lastLaunchResult, state?.install);
    renderDoctorReport();
  }

  async function setChineseLocalization(enabled) {
    const buttons = Array.from(document.querySelectorAll('[data-action="enableChineseLocalization"], [data-action="disableChineseLocalization"]'));
    buttons.forEach((button) => {
      button.disabled = true;
    });
    try {
      await runCommand(enabled ? "enable_chinese_localization" : "disable_chinese_localization", undefined, (result) => {
        state.config = result.config || state.config;
        const status = result.status || {};
        const details = [
          result.message,
          status.resources_dir ? `resources: ${status.resources_dir}` : "",
          `desktop:${status.desktop_json ? "ok" : "-"} frontend:${status.frontend_json ? "ok" : "-"} statsig:${status.statsig_json ? "ok" : "-"} whitelist:${status.whitelist_patched ? "ok" : "-"}`,
          status.current_locale ? `locale: ${status.current_locale}` : "",
        ].filter(Boolean).join(" / ");
        renderState();
        log(result.ok ? "OK" : "ERR", details || (enabled ? t("localizationEnabledLog") : t("localizationDisabledLog")));
        const diagnostics = [result.stdout, result.stderr].filter(Boolean).join("\n").split(/\r?\n/).filter(Boolean).slice(-8);
        diagnostics.forEach((line) => log("TRACE", line));
      });
    } finally {
      buttons.forEach((button) => {
        button.disabled = false;
      });
    }
  }

  async function activateProvider(id, name) {
    if (state?.config?.active_provider_id === id) {
      return;
    }
    await runCommand("set_active_provider", { id }, (config) => {
      state.config = config;
      renderState();
      log("OK", `${t("activeProvider")}: ${name}`);
    });
  }

  function activeInjectableProvider() {
    const provider = state?.config?.providers?.find((item) => item.active);
    if (!provider) return null;
    return canInjectProvider(provider, state?.config?.sandbox) ? provider : null;
  }

  function firstInjectableProvider() {
    return state?.config?.providers?.find((provider) => canInjectProvider(provider, state?.config?.sandbox)) || null;
  }

  function canInjectProvider(provider, sandbox = {}) {
    if (!provider?.enabled) return false;
    return Boolean((sandbox.inject_provider && provider.base_url) || (sandbox.inject_api_key && provider.has_key));
  }

  function explainMissingInjectionTarget() {
    if (!state?.config?.providers?.some((provider) => provider.active)) {
      return t("launchNoActiveProvider");
    }
    return t("launchNeedsInjectableProvider");
  }

  function hydrateSandboxForm(sandbox) {
    const form = document.getElementById("sandboxForm");
    formField(form, "inject_provider").checked = Boolean(sandbox.inject_provider);
    formField(form, "inject_api_key").checked = Boolean(sandbox.inject_api_key);
    formField(form, "relax_sandbox").checked = Boolean(sandbox.relax_sandbox);
    formField(form, "acknowledged").checked = Boolean(sandbox.acknowledged);
  }

  function hydrateGatewayForm(gateway) {
    const form = document.getElementById("gatewayForm");
    if (!form) return;
    formField(form, "enabled").checked = Boolean(gateway.enabled);
    formField(form, "port").value = String(gateway.port || 49331);
  }

  function loadProviderIntoForm(provider) {
    openProviderModal({ provider });
    log("OK", t("providerEditLoaded", { provider: provider.name }));
  }

  function resetProviderForm() {
    const form = document.getElementById("providerForm");
    form.reset();
    formField(form, "id").value = "";
    formField(form, "protocol").value = "anthropic";
    formField(form, "enabled").checked = true;
    formField(form, "api_key").placeholder = "sk-...";
    renderMappingRows([]);
    closeProviderModal();
    document.getElementById("providerFormTitle").textContent = t("manualProvider");
  }

  function providerTestSummary(result) {
    const status = result.status || "-";
    const code = result.code || "-";
    const message = result.compatibility_message || result.message || result.body_excerpt || "-";
    return result.ok
      ? t("providerTestOk", { provider: result.provider_name, status })
      : t("providerTestFailed", { provider: result.provider_name, status, code, message });
  }

  async function testProvider(id) {
    await runCommand("test_provider", { id }, (result) => {
      lastProviderTest = result;
      providerTestResults.set(id, result);
      renderProviderTest();
      renderProviders(state?.config?.providers || []);
      log(result.ok ? "OK" : "ERR", providerTestSummary(result));
    });
  }

  async function discoverProviderModels() {
    const form = document.getElementById("providerForm");
    const payload = {
      id: formField(form, "id").value.trim() || undefined,
      name: formField(form, "name").value.trim() || "Provider",
      app_type: formField(form, "app_type").value,
      protocol: formField(form, "protocol").value,
      base_url: formField(form, "base_url").value.trim(),
      api_key: formField(form, "api_key").value.trim(),
      model_mappings: [],
      enabled: formField(form, "enabled").checked,
    };
    if (!payload.base_url) {
      log("WARN", t("requiredUrl"));
      return;
    }
    setModelDiscoveryStatus(t("discoveringModels"), "warn");
    try {
      const result = await invoke("discover_provider_models", { provider: payload });
      if (!result) return;
      if (result.protocol === "openai") {
        renderMappingRows(result.model_mappings || []);
        syncMappingVisibility();
      } else {
        renderMappingRows([]);
        syncMappingVisibility();
      }
      const models = (result.models || []).slice(0, 8).join(" / ");
      setModelDiscoveryStatus(t("modelDiscoveryOk", { count: result.model_count ?? 0, models }), "ok");
      log("OK", t("modelDiscoveryOk", { count: result.model_count ?? 0, models }));
    } catch (error) {
      const message = t("modelDiscoveryFailed", { error: String(error) });
      setModelDiscoveryStatus(message, "err");
      log("ERR", message);
    }
  }

  async function runCommand(command, args, onSuccess) {
    try {
      const result = await invoke(command, args);
      if (result === undefined) return;
      onSuccess?.(result);
    } catch (error) {
      log("ERR", t("commandFailed", { error: String(error) }));
    }
  }

  async function openExternalUrl(url) {
    const bridge = window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke;
    if (!bridge) {
      window.open(url, "_blank", "noopener,noreferrer");
      log("OK", t("externalOpened", { url }));
      return;
    }
    await runCommand("open_external_url", { url }, () => {
      log("OK", t("externalOpened", { url }));
    });
  }

  async function syncCcSwitchConfig() {
    const buttons = Array.from(document.querySelectorAll('[data-action="syncCcSwitch"]'));
    buttons.forEach((button) => {
      button.disabled = true;
      button.dataset.idleText = button.textContent;
      button.textContent = t("syncRunning");
    });
    showNotice("WARN", t("syncRunning"));
    await new Promise((resolve) => window.requestAnimationFrame(resolve));

    try {
      const result = await invoke("sync_cc_switch_config");
      if (!result) return;
      state.config = result.config;
      state.cc_switch.provider_count = result.config.providers.filter((provider) => provider.source === "cc-switch").length;
      renderState();
      log("OK", t("syncDone", result));
    } catch (error) {
      log("ERR", t("commandFailed", { error: String(error) }));
    } finally {
      buttons.forEach((button) => {
        button.disabled = false;
        button.textContent = button.dataset.idleText || t("syncCcSwitch");
        delete button.dataset.idleText;
      });
    }
  }

  async function runSystemCommand(command) {
    const buttons = Array.from(document.querySelectorAll(".system-actions .text-button"));
    buttons.forEach((button) => {
      button.disabled = true;
    });
    const logBox = document.getElementById("systemActionLog");
    logBox.replaceChildren();
    const pending = document.createElement("p");
    pending.textContent = t("systemActionRunning");
    logBox.appendChild(pending);
    try {
      await nextFrame();
      await runCommand(command, undefined, (result) => {
        renderSystemAction(result);
        log(result.ok ? "OK" : "ERR", result.ok ? t("systemActionOk", result) : t("systemActionFailed", { ...result, code: result.exit_code ?? "-" }));
      });
    } finally {
      buttons.forEach((button) => {
        button.disabled = false;
      });
    }
  }

  async function repairHistoryAuto() {
    if (!historyLoaded && !historyLoading) {
      await refreshHistoryStatus();
    }
    const source = recommendedHistoryProfile(state?.history || null);
    if (!source) {
      const message = t("historyAutoMissing");
      renderHistoryError(message);
      log("WARN", message);
      renderHistory(state?.history || null);
      return;
    }
    await repairHistory(source.path);
  }

  async function repairHistory(sourcePath) {
    const buttons = Array.from(document.querySelectorAll(".history-repair-action"));
    buttons.forEach((button) => {
      button.disabled = true;
    });
    const logBox = document.getElementById("historyRepairLog");
    if (logBox) {
      logBox.replaceChildren();
      const pending = document.createElement("p");
      pending.textContent = t("historyRepairRunning");
      logBox.appendChild(pending);
    }

    try {
      await nextFrame();
      const result = await invoke("repair_history", {
        input: {
          source_path: sourcePath,
          item_keys: [],
        },
      });
      if (!result) return;
      state.history = result.scan || state.history;
      historyLoaded = true;
      renderHistory(state.history);
      renderHistoryRepair(result);
      log(result.ok ? "OK" : "ERR", t("historyRepairDone", { files: result.copied_files, backup: result.backup_path }));
    } catch (error) {
      renderHistoryError(error);
      log("ERR", t("historyRepairFailed", { error: String(error) }));
    } finally {
      document.querySelectorAll(".history-repair-action").forEach((button) => {
        button.disabled = false;
      });
      renderHistory(state?.history || null);
    }
  }

  function initControls() {
    document.querySelectorAll(".nav-item").forEach((button) => {
      button.addEventListener("click", () => setPage(button.getAttribute("data-page")));
    });
    document.querySelectorAll(".lang-button[data-locale]").forEach((button) => {
      button.addEventListener("click", () => applyLocale(button.getAttribute("data-locale")));
    });
    document.getElementById("fxToggle")?.addEventListener("click", toggleVisualFx);
    document.querySelectorAll("[data-action]").forEach((button) => {
      button.addEventListener("click", () => handleAction(button.getAttribute("data-action"), button));
    });
    document.getElementById("officialPluginSearch")?.addEventListener("input", (event) => {
      officialPluginSearchQuery = event.currentTarget.value;
      officialPluginVisibleLimit = 12;
      renderOfficialPlugins();
    });

    const providerForm = document.getElementById("providerForm");
    formField(providerForm, "protocol").addEventListener("change", syncMappingVisibility);
    document.getElementById("addMappingRowButton")?.addEventListener("click", () => addMappingRow({ enabled: true }));
    document.getElementById("providerModal")?.addEventListener("click", (event) => {
      if (event.target?.id === "providerModal") {
        resetProviderForm();
      }
    });
    window.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && !document.getElementById("providerModal")?.hidden) {
        resetProviderForm();
      }
    });

    providerForm.addEventListener("submit", async (event) => {
      event.preventDefault();
      const form = event.currentTarget;
      const payload = {
        id: formField(form, "id").value.trim() || undefined,
        name: formField(form, "name").value.trim(),
        app_type: formField(form, "app_type").value,
        protocol: formField(form, "protocol").value,
        base_url: formField(form, "base_url").value.trim(),
        api_key: formField(form, "api_key").value.trim(),
        model_mappings: collectModelMappings(),
        enabled: formField(form, "enabled").checked,
      };
      if (!payload.name) {
        log("WARN", t("requiredName"));
        return;
      }
      if (!payload.base_url) {
        log("WARN", t("requiredUrl"));
        return;
      }
      await runCommand("save_api_provider", { provider: payload }, (config) => {
        state.config = config;
        resetProviderForm();
        renderState();
        log("OK", t("providerSaved"));
      });
    });

  }

  async function handleAction(action, sourceElement) {
    if (action === "refreshState") {
      await refreshState();
      await hydrateLazyPage(currentPage(), { force: true });
      return;
    }
    if (action === "openExternalUrl") {
      const url = sourceElement?.getAttribute("data-url");
      if (url) {
        await openExternalUrl(url);
      }
      return;
    }
    if (action === "clearLogs") {
      terminal.replaceChildren();
      return;
    }
    if (action === "refreshHistory") {
      await refreshHistoryStatus();
      return;
    }
    if (action === "repairHistoryAuto") {
      await repairHistoryAuto();
      return;
    }
    if (action === "doctorFixAndLaunch") {
      await doctorFixAndLaunch();
      return;
    }
    if (action === "gotoSystem") {
      setPage("system");
      return;
    }
    if (action === "gotoProviders") {
      setPage("providers");
      return;
    }
    if (action === "gotoTools") {
      setPage("tools");
      return;
    }
    if (action === "gotoHistory") {
      setPage("history");
      return;
    }
    if (action === "gotoCapabilities") {
      setPage("capabilities");
      return;
    }
    if (action === "refreshCapabilities") {
      await refreshDeveloperCapabilities();
      return;
    }
    if (action === "refreshOfficialPlugins") {
      await refreshOfficialPlugins();
      return;
    }
    if (action === "enableDeveloperCapabilities") {
      await enableDeveloperCapabilities();
      return;
    }
    if (action === "syncOfficialPlugins") {
      await syncOfficialPluginMarketplace();
      return;
    }
    if (action === "installOfficialPlugin") {
      const plugin = sourceElement?.dataset?.plugin;
      if (plugin) {
        await installOfficialPlugin(plugin);
      }
      return;
    }
    if (action === "loadMoreOfficialPlugins") {
      officialPluginVisibleLimit += 12;
      renderOfficialPlugins();
      return;
    }
    if (action === "enableChineseLocalization") {
      await setChineseLocalization(true);
      return;
    }
    if (action === "disableChineseLocalization") {
      await setChineseLocalization(false);
      return;
    }
    if (action === "syncCcSwitch") {
      await syncCcSwitchConfig();
      return;
    }
    if (action === "relaunchAsAdmin") {
      await runCommand("relaunch_as_admin", undefined, () => {
        log("OK", t("adminRelaunchRequested"));
      });
      return;
    }
    if (action === "installClaudeModern") {
      await runSystemCommand("install_claude_modern");
      return;
    }
    if (action === "enableVmp") {
      await runSystemCommand("enable_virtual_machine_platform");
      return;
    }
    if (action === "openProviderModal") {
      openProviderModal();
      return;
    }
    if (action === "discoverProviderModels") {
      await discoverProviderModels();
      return;
    }
    if (action === "cancelProviderEdit") {
      resetProviderForm();
      return;
    }
    if (action === "testActiveProvider") {
      const activeProvider = state?.config?.providers?.find((provider) => provider.active);
      if (!activeProvider) {
        log("WARN", t("launchNoActiveProvider"));
        return;
      }
      await testProvider(activeProvider.id);
      return;
    }
    if (action === "setDirectMode" || action === "setGatewayMode") {
      if (!state?.config) {
        log("WARN", t("commandUnavailable"));
        return;
      }
      const activeProvider = state.config.providers?.find((provider) => provider.active);
      const enabled = action === "setGatewayMode";
      const payload = {
        enabled,
        port: Number(state.config.gateway?.port || 49331),
      };
      await runCommand("save_gateway_options", { options: payload }, async (config) => {
        state.config = config;
        await refreshGatewayStatus();
        renderState();
        log("OK", enabled ? t("gatewayModeSaved") : t("directModeSaved"));
        if (!enabled && activeProvider?.protocol === "openai") {
          log("WARN", t("directModeOpenAIWarning"));
        }
      });
      return;
    }
    if (action === "launchClaudeInjected") {
      let provider = activeInjectableProvider();
      if (!provider) {
        provider = firstInjectableProvider();
      }
      if (!provider) {
        log("WARN", explainMissingInjectionTarget());
        setPage("providers");
        return;
      }
      if (!provider.active) {
        await activateProvider(provider.id, provider.name);
        log("WARN", t("launchAutoSwitched", { provider: provider.name }));
      }
      await runCommand("launch_claude_desktop_with_provider", { id: provider.id }, (result) => {
        lastLaunchResult = result;
        state.config.active_provider_id = provider.id;
        state.claude_3p = result.claude_3p || state.claude_3p;
        if (state.config.providers) {
          state.config.providers.forEach((item) => {
            item.active = item.id === provider.id;
          });
        }
        if (result.gateway_url) {
          state.gateway = {
            ...(state.gateway || {}),
            enabled: true,
            running: true,
            url: result.gateway_url,
          };
        }
        renderState();
        log(result.live_injection_attempted && result.cdp_error ? "WARN" : "OK", injectLaunchSummary(result));
      });
      await refreshGatewayStatus();
      renderState();
      return;
    }
    if (action === "launchClaudeClean") {
      await runCommand("launch_claude_desktop", undefined, (result) => {
        lastLaunchResult = result;
        state.claude_3p = result.claude_3p || state.claude_3p;
        renderState();
        log("OK", t("launchDone", { pid: result.process_id }));
      });
      return;
    }
    if (action === "saveSandbox") {
      const form = document.getElementById("sandboxForm");
      const payload = {
        inject_provider: formField(form, "inject_provider").checked,
        inject_api_key: formField(form, "inject_api_key").checked,
        relax_sandbox: formField(form, "relax_sandbox").checked,
        acknowledged: formField(form, "acknowledged").checked,
      };
      if (payload.relax_sandbox && !payload.acknowledged) {
        log("WARN", t("sandboxNeedAck"));
        return;
      }
      await runCommand("save_sandbox_options", { options: payload }, (config) => {
        state.config = config;
        renderState();
        log("OK", t("sandboxSaved"));
      });
      return;
    }
    if (action === "saveGateway") {
      const form = document.getElementById("gatewayForm");
      const payload = {
        enabled: formField(form, "enabled").checked,
        port: Number(formField(form, "port").value || 49331),
      };
      await runCommand("save_gateway_options", { options: payload }, async (config) => {
        state.config = config;
        await refreshGatewayStatus();
        renderState();
        log("OK", t("gatewaySaved"));
      });
      return;
    }
    if (action === "startGateway") {
      await runCommand("start_gateway", undefined, (gateway) => {
        state.gateway = gateway;
        renderState();
        log("OK", t("gatewayStarted", { url: gateway.url }));
      });
      return;
    }
    if (action === "stopGateway") {
      await runCommand("stop_gateway", undefined, (gateway) => {
        state.gateway = gateway;
        renderState();
        log("OK", t("gatewayStoppedLog"));
      });
    }
  }

  window.addEventListener("resize", () => {
    if (visualFxEnabled && !prefersReducedMotion) {
      startRain();
    }
  });
  initControls();
  applyLocale("zh");
  document.getElementById("bridgeState").textContent = t("bridgeChecking");
  log("TRACE", "Claude++ Control Tower online");
  refreshState({ silent: true });
})();
