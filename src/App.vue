<script setup lang="ts">
import { ref, onMounted, provide } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { provideI18n } from "./i18n";
import { initTheme, useTheme } from "./themes";
import McpList from "./components/McpList.vue";
import McpDetail from "./components/McpDetail.vue";
import AddMcpForm from "./components/AddMcpForm.vue";
import UsageStats from "./components/UsageStats.vue";
import SettingsPage from "./components/SettingsPage.vue";
import Marketplace from "./components/Marketplace.vue";

/** 初始化 i18n */
const { t, currentLocale, setLocale } = provideI18n();

/** 向子组件提供语言切换能力 */
provide("setLocale", setLocale);
provide("currentLocale", currentLocale);

/** 初始化主题系统 */
const { currentThemeName, setTheme, getThemes } = useTheme();
provide("setTheme", setTheme);
provide("currentThemeName", currentThemeName);
provide("getThemes", getThemes);

/** 连接配置 */
interface ConnectionConfig {
  type: "stdio" | "streamable_http";
  command?: string;
  args?: string[];
  url?: string;
}

/** 后端返回的 MCP 条目类型 */
interface McpTool {
  name: string;
  description: string;
  parameters: any;
  returns: string;
}

interface McpEntry {
  name: string;
  description: string;
  connection: ConnectionConfig;
  tools: McpTool[];
  connected: boolean;
}

/** Toast 消息类型 */
interface ToastMessage {
  text: string;
  type: "success" | "error" | "info";
}

/** 全局 Toast 通知 */
const toast = ref<ToastMessage | null>(null);
let toastTimer: ReturnType<typeof setTimeout> | null = null;

/** 显示 Toast 通知，3 秒后自动消失 */
function showToast(text: string, type: "success" | "error" | "info" = "info") {
  if (toastTimer) clearTimeout(toastTimer);
  toast.value = { text, type };
  toastTimer = setTimeout(() => {
    toast.value = null;
  }, 3000);
}

/** 通过 provide 让子组件可以调用 showToast */
provide("showToast", showToast);

/** 当前活跃的标签页 */
type TabKey = "add" | "detail" | "stats" | "settings" | "marketplace";

const activeTab = ref<TabKey>("add");
const selectedEntry = ref<McpEntry | null>(null);
const mcpListRef = ref<InstanceType<typeof McpList> | null>(null);

/** 窗口控制 */
const appWindow = getCurrentWindow();
const isMaximized = ref(false);
/** resize 防抖定时器 */
let resizeTimer: ReturnType<typeof setTimeout> | null = null;

/** 窗口操作函数 */
async function minimizeWindow() {
  try {
    await appWindow.minimize();
  } catch (e) {
    console.error("minimizeWindow:", e);
  }
}

async function toggleMaximize() {
  try {
    await appWindow.toggleMaximize();
    // 直接切换状态，不依赖 isMaximized() 返回值（decorations: false 下可能不准）
    isMaximized.value = !isMaximized.value;
  } catch (e) {
    console.error("toggleMaximize:", e);
  }
}

async function closeWindow() {
  try {
    await appWindow.close();
  } catch (e) {
    console.error("closeWindow:", e);
  }
}

/** 更新窗口最大化状态（用于 onResized 兜底，处理 Win+↑ 等快捷键） */
async function updateMaximizedState() {
  try {
    isMaximized.value = await appWindow.isMaximized();
  } catch (e) {
    console.error("updateMaximizedState:", e);
  }
}

/** 服务器状态（侧边栏展示用） */
const serverPort = ref(9277);
const localIps = ref<string[]>([]);

onMounted(async () => {
  // 初始化主题
  initTheme();
  await updateMaximizedState();
  // 防抖的 resize 监听，处理 Win+↑ 等非按钮触发的最大化
  appWindow.onResized(() => {
    if (resizeTimer) clearTimeout(resizeTimer);
    resizeTimer = setTimeout(() => {
      updateMaximizedState();
    }, 250);
  });
  // 获取服务器配置和本地 IP
  try {
    const config = await invoke<{ http_port: number; bind_address: string }>("get_server_config");
    serverPort.value = config.http_port;
  } catch { /* 忽略 */ }
  try {
    localIps.value = await invoke<string[]>("get_local_ips");
  } catch { /* 忽略 */ }
});

/** 标签页配置 */
const tabs: { key: TabKey; label: string }[] = [
  { key: "add", label: t("app.tabs.add") },
  { key: "detail", label: t("app.tabs.detail") },
  { key: "stats", label: t("app.tabs.stats") },
  { key: "marketplace", label: t("app.tabs.marketplace") },
  { key: "settings", label: t("app.tabs.settings") },
];

/** 从侧边栏选中 MCP 时，切换到详情标签页 */
function onMcpSelect(entry: McpEntry) {
  selectedEntry.value = entry;
  activeTab.value = "detail";
}

/** 添加成功后刷新列表并切换到详情页 */
function onMcpAdded(entry: McpEntry) {
  selectedEntry.value = entry;
  activeTab.value = "detail";
  mcpListRef.value?.refresh();
}

/** 删除成功后刷新列表并清空详情 */
function onMcpDeleted(_name: string) {
  selectedEntry.value = null;
  activeTab.value = "add";
  mcpListRef.value?.refresh();
}

/** 重新从后端获取当前选中的条目 */
async function refreshSelectedEntry() {
  if (!selectedEntry.value) return;
  try {
    const list = await invoke<McpEntry[]>("list_mcp");
    const updated = list.find((e) => e.name === selectedEntry.value!.name);
    if (updated) {
      selectedEntry.value = updated;
    }
  } catch {
    // 静默失败，保持旧数据
  }
}

/** 连接成功后刷新列表和详情 */
async function onMcpConnected() {
  mcpListRef.value?.refresh();
  await refreshSelectedEntry();
}

/** 断开成功后刷新列表和详情 */
async function onMcpDisconnected() {
  mcpListRef.value?.refresh();
  await refreshSelectedEntry();
}
</script>

<template>
  <div class="app-layout">
    <!-- Toast 通知 -->
    <Transition name="toast">
      <div v-if="toast" :class="['toast', `toast-${toast.type}`]">
        <span class="toast-icon">
          {{ toast.type === "success" ? "✓" : toast.type === "error" ? "✕" : "ℹ" }}
        </span>
        {{ toast.text }}
      </div>
    </Transition>

    <!-- 自定义标题栏 -->
    <div class="titlebar" data-tauri-drag-region>
      <div class="titlebar-title">{{ t("app.title") }}</div>
      <div class="titlebar-controls">
        <button class="titlebar-btn" @click="minimizeWindow">
          <span class="ctrl-icon ctrl-minimize"></span>
        </button>
        <button class="titlebar-btn" @click="toggleMaximize">
          <span :class="['ctrl-icon', isMaximized ? 'ctrl-restore' : 'ctrl-maximize']"></span>
        </button>
        <button class="titlebar-btn titlebar-close" @click="closeWindow">
          <span class="ctrl-icon ctrl-close"></span>
        </button>
      </div>
    </div>

    <div class="app-body">
      <!-- 左侧侧边栏 -->
      <aside class="sidebar">
        <div class="sidebar-title">
          <h2>{{ t("app.title") }}</h2>
        </div>
        <McpList
          ref="mcpListRef"
          @select="onMcpSelect"
          @connected="onMcpConnected"
          @disconnected="onMcpDisconnected"
        />

        <!-- 服务器状态 -->
        <div class="server-status-bar">
          <span class="status-dot"></span>
          <span class="status-port">:{{ serverPort }}</span>
          <span v-if="localIps.length" class="status-lan-ip">{{ localIps[0] }}:{{ serverPort }}</span>
        </div>
      </aside>

      <!-- 右侧主区域 -->
      <main class="main-area">
        <!-- 标签页导航 -->
        <nav class="tab-bar">
          <button
            v-for="tab in tabs"
            :key="tab.key"
            :class="['tab-btn', { active: activeTab === tab.key }]"
            @click="activeTab = tab.key"
          >
            {{ tab.label }}
          </button>
        </nav>

        <!-- 标签页内容 -->
        <div class="tab-content">
          <AddMcpForm v-if="activeTab === 'add'" @added="onMcpAdded" />
          <McpDetail
            v-else-if="activeTab === 'detail'"
            :entry="selectedEntry"
            @deleted="onMcpDeleted"
            @connected="onMcpConnected"
            @disconnected="onMcpDisconnected"
          />
          <UsageStats v-else-if="activeTab === 'stats'" />
          <Marketplace v-else-if="activeTab === 'marketplace'" />
          <SettingsPage v-else-if="activeTab === 'settings'" />
        </div>
      </main>
    </div>
  </div>
</template>

<style>
/* 全局重置 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html,
body,
#app {
  height: 100%;
  overflow: hidden;
}

body {
  font-family: Inter, -apple-system, sans-serif;
  font-size: 14px;
  color: var(--mc-text-primary);
  background: var(--mc-bg-primary);
}

/* 全局滚动条 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--mc-scrollbar-thumb);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--mc-scrollbar-thumb-hover);
}

::-webkit-scrollbar-corner {
  background: transparent;
}
</style>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  border: 1px solid var(--mc-window-border);
  border-radius: 8px;
  overflow: hidden;
}

/* ===== 自定义标题栏 ===== */
.titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  flex-shrink: 0;
  background: var(--mc-titlebar-bg);
  border-bottom: 1px solid var(--mc-titlebar-border);
  padding: 0 12px;
  user-select: none;
  cursor: default;
}

.titlebar-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--mc-titlebar-text);
  padding-left: 8px;
}

.titlebar-controls {
  display: flex;
  height: 100%;
}

.titlebar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 100%;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--mc-titlebar-text);
  transition: background 0.15s ease, color 0.15s ease;
}

.titlebar-btn .ctrl-icon {
  pointer-events: none;
}

.titlebar-btn:hover {
  background: var(--mc-titlebar-btn-hover);
  color: var(--mc-text-primary);
}

.titlebar-close:hover {
  background: var(--mc-close-btn-hover);
  color: #ffffff;
}

/* 窗口控制按钮图标 - CSS 绘制 */
.ctrl-icon {
  display: block;
  position: relative;
}

.ctrl-minimize {
  width: 10px;
  height: 0;
  border-bottom: 1.5px solid currentColor;
}

.ctrl-maximize {
  width: 10px;
  height: 10px;
  border: 1.5px solid currentColor;
  border-radius: 1px;
}

.ctrl-restore {
  width: 8px;
  height: 8px;
  border: 1.5px solid currentColor;
  border-radius: 1px;
  position: relative;
}

.ctrl-restore::after {
  content: "";
  position: absolute;
  top: -3px;
  right: -3px;
  width: 8px;
  height: 8px;
  border: 1.5px solid currentColor;
  border-radius: 1px;
  background: var(--mc-titlebar-bg);
}

.ctrl-close {
  width: 12px;
  height: 12px;
  position: relative;
}

.ctrl-close::before,
.ctrl-close::after {
  content: "";
  position: absolute;
  top: 50%;
  left: 50%;
  width: 1.5px;
  height: 14px;
  background: currentColor;
  border-radius: 1px;
}

.ctrl-close::before {
  transform: translate(-50%, -50%) rotate(45deg);
}

.ctrl-close::after {
  transform: translate(-50%, -50%) rotate(-45deg);
}

/* ===== 主体区域 ===== */
.app-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* ===== Toast 通知 ===== */
.toast {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 9999;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  color: var(--mc-text-primary);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  pointer-events: none;
}

.toast-success {
  background: var(--mc-toast-success-bg);
  border: 1px solid var(--mc-accent-green);
}

.toast-error {
  background: var(--mc-toast-error-bg);
  border: 1px solid var(--mc-accent-red);
}

.toast-info {
  background: var(--mc-toast-info-bg);
  border: 1px solid var(--mc-accent-blue);
}

.toast-icon {
  font-weight: 700;
  font-size: 14px;
}

.toast-success .toast-icon {
  color: var(--mc-accent-green);
}

.toast-error .toast-icon {
  color: var(--mc-accent-red);
}

.toast-info .toast-icon {
  color: var(--mc-accent-blue);
}

/* Toast 动画 */
.toast-enter-active {
  animation: toast-in 0.3s ease;
}

.toast-leave-active {
  animation: toast-out 0.25s ease;
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(-12px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

@keyframes toast-out {
  from {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
  to {
    opacity: 0;
    transform: translateX(-50%) translateY(-12px);
  }
}

/* ===== 左侧侧边栏 ===== */
.sidebar {
  width: 260px;
  flex-shrink: 0;
  background: var(--mc-sidebar-bg);
  border-right: 1px solid var(--mc-sidebar-border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-title {
  padding: 20px 16px 16px;
  border-bottom: 1px solid var(--mc-sidebar-border);
}

.sidebar-title h2 {
  font-size: 18px;
  font-weight: 700;
  background: var(--mc-sidebar-title-gradient);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin: 0;
  letter-spacing: -0.3px;
}

/* ===== 右侧主区域 ===== */
.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--mc-main-bg);
}

/* ===== 标签页导航 ===== */
.tab-bar {
  display: flex;
  background: var(--mc-tab-bar-bg);
  border-bottom: 1px solid var(--mc-tab-border);
  padding: 0 20px;
  gap: 4px;
  flex-shrink: 0;
}

.tab-btn {
  padding: 12px 18px;
  font-size: 13px;
  font-weight: 500;
  color: var(--mc-text-muted);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: color 0.2s ease, border-color 0.2s ease;
  font-family: inherit;
  position: relative;
}

.tab-btn:hover {
  color: var(--mc-text-primary);
}

.tab-btn.active {
  color: var(--mc-text-primary);
  border-bottom-color: var(--mc-tab-active-border);
}

/* ===== 标签页内容 ===== */
.tab-content {
  flex: 1;
  overflow-y: auto;
}

/* ===== 侧边栏服务器状态条 ===== */
.server-status-bar {
  margin-top: auto;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-top: 1px solid var(--mc-status-bar-border);
  background: var(--mc-status-bar-bg);
  font-size: 11px;
  font-family: "JetBrains Mono", "Fira Code", "Consolas", monospace;
  color: var(--mc-status-bar-text);
  flex-shrink: 0;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--mc-accent-green);
  flex-shrink: 0;
  animation: pulse-dot 2s ease-in-out infinite;
}

@keyframes pulse-dot {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(63, 185, 80, 0.4);
  }
  50% {
    box-shadow: 0 0 0 4px rgba(63, 185, 80, 0);
  }
}

.status-port {
  color: var(--mc-accent-blue);
}

.status-lan-ip {
  color: var(--mc-text-dim);
  margin-left: auto;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>