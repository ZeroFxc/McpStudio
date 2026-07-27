<script setup lang="ts">
import { ref, onMounted, provide } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { provideI18n } from "./i18n";
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

/** 窗口操作函数 */
async function minimizeWindow() {
  await appWindow.minimize();
}

async function toggleMaximize() {
  await appWindow.toggleMaximize();
}

async function closeWindow() {
  await appWindow.close();
}

/** 更新窗口最大化状态 */
async function updateMaximizedState() {
  isMaximized.value = await appWindow.isMaximized();
}

/** 服务器状态（侧边栏展示用） */
const serverPort = ref(9277);
const localIps = ref<string[]>([]);

onMounted(async () => {
  await updateMaximizedState();
  appWindow.onResized(() => {
    updateMaximizedState();
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
    <div class="titlebar">
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
  color: #e6edf3;
  background: #0d1117;
}

/* 全局滚动条 - GitHub Dark 风格 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: #30363d;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #484f58;
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
  border: 1px solid #30363d;
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
  background: #161b22;
  border-bottom: 1px solid #30363d;
  padding: 0 12px;
  user-select: none;
  -webkit-app-region: drag;
  cursor: default;
}

.titlebar-title {
  font-size: 13px;
  font-weight: 600;
  color: #8b949e;
  padding-left: 8px;
}

.titlebar-controls {
  display: flex;
  height: 100%;
  -webkit-app-region: no-drag;
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
  color: #8b949e;
  transition: background 0.15s ease, color 0.15s ease;
}

.titlebar-btn:hover {
  background: #30363d;
  color: #e6edf3;
}

.titlebar-close:hover {
  background: #e81123;
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
  background: #161b22;
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
  color: #e6edf3;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  pointer-events: none;
}

.toast-success {
  background: #1a3a2a;
  border: 1px solid #3fb950;
}

.toast-error {
  background: #3a1a1a;
  border: 1px solid #f85149;
}

.toast-info {
  background: #1a2a3a;
  border: 1px solid #58a6ff;
}

.toast-icon {
  font-weight: 700;
  font-size: 14px;
}

.toast-success .toast-icon {
  color: #3fb950;
}

.toast-error .toast-icon {
  color: #f85149;
}

.toast-info .toast-icon {
  color: #58a6ff;
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
  background: #161b22;
  border-right: 1px solid #30363d;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-title {
  padding: 20px 16px 16px;
  border-bottom: 1px solid #30363d;
}

.sidebar-title h2 {
  font-size: 18px;
  font-weight: 700;
  background: linear-gradient(135deg, #58a6ff, #a371f7);
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
  background: #0d1117;
}

/* ===== 标签页导航 ===== */
.tab-bar {
  display: flex;
  background: #0d1117;
  border-bottom: 1px solid #30363d;
  padding: 0 20px;
  gap: 4px;
  flex-shrink: 0;
}

.tab-btn {
  padding: 12px 18px;
  font-size: 13px;
  font-weight: 500;
  color: #8b949e;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: color 0.2s ease, border-color 0.2s ease;
  font-family: inherit;
  position: relative;
}

.tab-btn:hover {
  color: #e6edf3;
}

.tab-btn.active {
  color: #e6edf3;
  border-bottom-color: #58a6ff;
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
  border-top: 1px solid #30363d;
  background: #0d1117;
  font-size: 11px;
  font-family: "JetBrains Mono", "Fira Code", "Consolas", monospace;
  color: #8b949e;
  flex-shrink: 0;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #3fb950;
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
  color: #58a6ff;
}

.status-lan-ip {
  color: #6e7681;
  margin-left: auto;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>