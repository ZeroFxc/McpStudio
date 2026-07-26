<script setup lang="ts">
import { ref, computed, inject, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "../i18n";

const { t } = useI18n();

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

const props = defineProps<{
  entry: McpEntry | null;
}>();

const emit = defineEmits<{
  (e: "deleted", name: string): void;
  (e: "connected"): void;
  (e: "disconnected"): void;
}>();

/** 从 App.vue 注入的 Toast 函数 */
const showToast = inject<(text: string, type?: "success" | "error" | "info") => void>("showToast", () => {});

const deleting = ref(false);
const busy = ref(false);

/** 手风琴展开状态 */
const expandedTools = ref<Set<string>>(new Set());
const expandedSchemas = ref<Set<string>>(new Set());

/** 初始化时展开第一个工具 */
function initExpanded() {
  if (props.entry && props.entry.tools.length > 0) {
    expandedTools.value = new Set([props.entry.tools[0].name]);
  }
}

/** 切换工具展开/收起 */
function toggleTool(toolName: string) {
  const next = new Set(expandedTools.value);
  if (next.has(toolName)) {
    next.delete(toolName);
  } else {
    next.add(toolName);
  }
  expandedTools.value = next;
}

/** 切换 Schema 展开/收起 */
function toggleSchema(toolName: string) {
  const next = new Set(expandedSchemas.value);
  if (next.has(toolName)) {
    next.delete(toolName);
  } else {
    next.add(toolName);
  }
  expandedSchemas.value = next;
}

/** 复制工具名到剪贴板 */
async function copyToolName(toolName: string, event: Event) {
  event.stopPropagation();
  try {
    await navigator.clipboard.writeText(toolName);
    showToast(t("mcpDetail.toast.copied") + ": " + toolName, "success");
  } catch {
    showToast(t("mcpDetail.toast.copyFailed"), "error");
  }
}

/** JSON 语法高亮 */
function highlightJson(obj: any): string {
  const json = JSON.stringify(obj, null, 2);
  return json.replace(
    /("(\\u[a-fA-F0-9]{4}|\\[^u]|[^"\\])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g,
    (match) => {
      if (/^"/.test(match)) {
        if (/:$/.test(match)) {
          return '<span class="json-key">' + match + '</span>';
        }
        return '<span class="json-string">' + match + '</span>';
      }
      if (/^(true|false)$/.test(match)) {
        return '<span class="json-bool">' + match + '</span>';
      }
      if (/^null$/.test(match)) {
        return '<span class="json-null">' + match + '</span>';
      }
      return '<span class="json-number">' + match + '</span>';
    }
  );
}

/** 连接配置的 JSON 高亮 HTML */
const connectionJsonHtml = computed(() => {
  if (!props.entry) return "";
  return highlightJson(props.entry.connection);
});

/** 删除当前 MCP */
async function removeMcp() {
  if (!props.entry) return;
  const confirmed = confirm(t("mcpDetail.confirmDelete", { name: props.entry.name }));
  if (!confirmed) return;

  deleting.value = true;
  try {
    await invoke("remove_mcp", { name: props.entry.name });
    emit("deleted", props.entry.name);
    showToast(t("mcpDetail.toast.deleted"), "success");
  } catch (err) {
    console.error("删除 MCP 失败:", err);
    showToast(t("mcpDetail.toast.deleteFailed") + ": " + String(err), "error");
  } finally {
    deleting.value = false;
  }
}

/** 连接 MCP */
async function connectMcp() {
  if (!props.entry) return;
  busy.value = true;
  try {
    await invoke("connect_mcp", { name: props.entry.name });
    emit("connected");
    showToast(t("mcpDetail.toast.connected") + " " + props.entry.name, "success");
  } catch (err) {
    console.error("连接 MCP 失败:", err);
    showToast(t("mcpDetail.toast.connectFailed") + ": " + String(err), "error");
  } finally {
    busy.value = false;
  }
}

/** 断开 MCP */
async function disconnectMcp() {
  if (!props.entry) return;
  busy.value = true;
  try {
    await invoke("disconnect_mcp", { name: props.entry.name });
    emit("disconnected");
    showToast(t("mcpDetail.toast.disconnected") + " " + props.entry.name, "success");
  } catch (err) {
    console.error("断开 MCP 失败:", err);
    showToast(t("mcpDetail.toast.disconnectFailed") + ": " + String(err), "error");
  } finally {
    busy.value = false;
  }
}

/** 监听 entry 变化，初始化手风琴 */
watch(() => props.entry, () => {
  initExpanded();
}, { immediate: true });
</script>

<template>
  <div class="mcp-detail">
    <div v-if="!entry" class="placeholder">
      <div class="doc-icon"></div>
      <p>{{ t("mcpDetail.placeholder") }}</p>
    </div>

    <div v-else class="detail-content">
      <!-- Hero 区域 -->
      <div class="hero-section">
        <div class="hero-left">
          <h2 class="hero-title">{{ entry.name }}</h2>
          <span :class="['hero-badge', entry.connected ? 'connected' : 'disconnected']">
            <span class="badge-dot"></span>
            {{ entry.connected ? t("mcpDetail.status.connected") : t("mcpDetail.status.disconnected") }}
          </span>
        </div>
        <div class="hero-actions">
          <button
            v-if="!entry.connected"
            class="btn btn-connect"
            :disabled="busy"
            @click="connectMcp"
          >
            {{ busy ? t("mcpDetail.actions.connecting") : t("mcpDetail.actions.connect") }}
          </button>
          <button
            v-else
            class="btn btn-disconnect"
            :disabled="busy"
            @click="disconnectMcp"
          >
            {{ busy ? t("mcpDetail.actions.disconnecting") : t("mcpDetail.actions.disconnect") }}
          </button>
          <button class="btn btn-delete" @click="removeMcp" :disabled="deleting">
            {{ deleting ? t("mcpDetail.actions.deleting") : t("mcpDetail.actions.delete") }}
          </button>
        </div>
      </div>

      <!-- 信息卡片网格 -->
      <div class="info-cards">
        <div class="info-card">
          <div class="card-icon desc-icon"></div>
          <div class="card-label">{{ t("mcpDetail.info.description") }}</div>
          <div class="card-value">{{ entry.description || t("mcpDetail.info.noDescription") }}</div>
        </div>
        <div class="info-card">
          <div class="card-icon link-icon"></div>
          <div class="card-label">{{ t("mcpDetail.info.connectionType") }}</div>
          <div class="card-value conn-type">{{ entry.connection.type }}</div>
        </div>
        <div class="info-card">
          <div :class="['card-icon', 'status-dot-icon', entry.connected ? 'connected' : 'disconnected']"></div>
          <div class="card-label">{{ t("mcpDetail.info.status") }}</div>
          <div class="card-value">
            <span :class="['status-text', entry.connected ? 'connected' : 'disconnected']">
              {{ entry.connected ? t("mcpDetail.status.running") : t("mcpDetail.status.offline") }}
            </span>
          </div>
        </div>
        <div class="info-card">
          <div class="card-icon gear-icon"></div>
          <div class="card-label">{{ t("mcpDetail.info.toolCount") }}</div>
          <div class="card-value">{{ entry.tools.length }}</div>
        </div>
      </div>

      <!-- 连接配置 -->
      <div class="connection-section">
        <div class="section-label">{{ t("mcpDetail.connection.title") }}</div>
        <pre class="json-block" v-html="connectionJsonHtml"></pre>
      </div>

      <!-- 工具列表 -->
      <div class="tools-section">
        <div class="section-label">{{ t("mcpDetail.tools.title") }}</div>
        <div v-if="entry.tools.length === 0" class="empty-tools">
          <span>{{ t("mcpDetail.tools.empty") }}</span>
        </div>
        <div
          v-for="tool in entry.tools"
          :key="tool.name"
          class="tool-card"
          :class="{ expanded: expandedTools.has(tool.name) }"
        >
          <div class="tool-header" @click="toggleTool(tool.name)">
            <div class="tool-header-left">
              <span class="tool-chevron">{{ expandedTools.has(tool.name) ? "▾" : "▸" }}</span>
              <span class="tool-name">{{ tool.name }}</span>
              <button class="copy-btn" @click="copyToolName(tool.name, $event)" title="复制工具名">
                <span class="copy-icon"></span>
              </button>
            </div>
          </div>
          <div v-if="expandedTools.has(tool.name)" class="tool-body">
            <div class="tool-field">
              <label>{{ t("mcpDetail.tools.description") }}</label>
              <span>{{ tool.description || t("mcpDetail.tools.none") }}</span>
            </div>
            <div class="tool-field">
              <label
                class="schema-toggle"
                @click="toggleSchema(tool.name)"
              >
                <span class="tool-chevron-sm">{{ expandedSchemas.has(tool.name) ? "▾" : "▸" }}</span>
                {{ t("mcpDetail.tools.paramsSchema") }}
              </label>
              <pre
                v-if="expandedSchemas.has(tool.name)"
                class="schema-block"
                v-html="highlightJson(tool.parameters || {})"
              ></pre>
            </div>
            <div class="tool-field">
              <label>{{ t("mcpDetail.tools.returns") }}</label>
              <span>{{ tool.returns || t("mcpDetail.tools.none") }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mcp-detail {
  height: 100%;
  overflow-y: auto;
}

/* 占位状态 */
.placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #8b949e;
  font-size: 15px;
  gap: 12px;
}

/* 文档轮廓 CSS 形状（替代 📋） */
.doc-icon {
  width: 32px;
  height: 40px;
  border: 2px solid #8b949e;
  border-radius: 3px;
  position: relative;
  opacity: 0.4;
}
.doc-icon::after {
  content: '';
  position: absolute;
  left: 6px;
  right: 6px;
  height: 2px;
  background: #8b949e;
  box-shadow: 0 8px 0 #8b949e, 0 16px 0 #8b949e;
}

/* 详情内容 */
.detail-content {
  padding: 28px 32px;
  max-width: 900px;
}

/* Hero 区域 */
.hero-section {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 24px;
  flex-wrap: wrap;
  gap: 16px;
}

.hero-left {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-wrap: wrap;
}

.hero-title {
  margin: 0;
  font-size: 26px;
  font-weight: 700;
  color: #e6edf3;
  letter-spacing: -0.5px;
}

.hero-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
}

.hero-badge.connected {
  background: rgba(63, 185, 80, 0.12);
  color: #3fb950;
  border: 1px solid rgba(63, 185, 80, 0.3);
}

.hero-badge.disconnected {
  background: rgba(139, 148, 158, 0.12);
  color: #8b949e;
  border: 1px solid rgba(139, 148, 158, 0.3);
}

.badge-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}

.hero-actions {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 7px 18px;
  font-size: 13px;
  font-weight: 500;
  border: 1px solid;
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.2s ease;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-connect {
  background: rgba(63, 185, 80, 0.1);
  color: #3fb950;
  border-color: rgba(63, 185, 80, 0.3);
}

.btn-connect:hover:not(:disabled) {
  background: rgba(63, 185, 80, 0.2);
  border-color: rgba(63, 185, 80, 0.5);
}

.btn-disconnect {
  background: rgba(210, 153, 29, 0.1);
  color: #d2991d;
  border-color: rgba(210, 153, 29, 0.3);
}

.btn-disconnect:hover:not(:disabled) {
  background: rgba(210, 153, 29, 0.2);
  border-color: rgba(210, 153, 29, 0.5);
}

.btn-delete {
  background: rgba(248, 81, 73, 0.1);
  color: #f85149;
  border-color: rgba(248, 81, 73, 0.3);
}

.btn-delete:hover:not(:disabled) {
  background: rgba(248, 81, 73, 0.2);
  border-color: rgba(248, 81, 73, 0.5);
}

/* 信息卡片网格 */
.info-cards {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 24px;
}

.info-card {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 14px;
  transition: border-color 0.2s ease;
}

.info-card:hover {
  border-color: #484f58;
}

.card-icon {
  margin-bottom: 8px;
}

/* 描述图标：左下角缺口的矩形 */
.desc-icon {
  width: 20px;
  height: 16px;
  border: 2px solid #8b949e;
  border-radius: 2px;
  position: relative;
}
.desc-icon::after {
  content: '';
  position: absolute;
  bottom: -2px;
  left: -2px;
  width: 8px;
  height: 8px;
  background: #0d1117;
  transform: rotate(45deg);
  transform-origin: bottom left;
}

/* 连接环图标（替代 🔗） */
.link-icon {
  width: 18px;
  height: 10px;
  border: 2px solid #8b949e;
  border-radius: 10px 10px 0 0;
  border-bottom: none;
  position: relative;
}
.link-icon::after {
  content: '';
  position: absolute;
  right: -6px;
  top: 2px;
  width: 10px;
  height: 6px;
  border: 2px solid #8b949e;
  border-radius: 0 0 10px 10px;
  border-top: none;
}

/* 状态圆点图标（替代 🟢/⚪） */
.status-dot-icon {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #8b949e;
}
.status-dot-icon.connected {
  background: #3fb950;
  box-shadow: 0 0 6px rgba(63, 185, 80, 0.5);
}
.status-dot-icon.disconnected {
  background: #484f58;
}

/* 齿轮图标（替代 🔧） */
.gear-icon {
  width: 20px;
  height: 20px;
  border: 2px solid #8b949e;
  border-radius: 50%;
  position: relative;
}
.gear-icon::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  width: 6px;
  height: 6px;
  background: #8b949e;
  border-radius: 50%;
  transform: translate(-50%, -50%);
}

.card-label {
  font-size: 11px;
  color: #8b949e;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
  font-weight: 500;
}

.card-value {
  font-size: 14px;
  color: #e6edf3;
  font-weight: 500;
  word-break: break-word;
}

.conn-type {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  color: #58a6ff !important;
  font-size: 13px !important;
}

.status-text.connected {
  color: #3fb950;
}

.status-text.disconnected {
  color: #8b949e;
}

/* 连接配置 */
.connection-section {
  margin-bottom: 28px;
}

.section-label {
  font-size: 13px;
  font-weight: 600;
  color: #e6edf3;
  margin-bottom: 10px;
}

.json-block {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 14px 16px;
  font-size: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  line-height: 1.6;
  overflow-x: auto;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
}

/* JSON 语法高亮 */
:deep(.json-block .json-key) {
  color: #58a6ff;
}

:deep(.json-block .json-string) {
  color: #3fb950;
}

:deep(.json-block .json-number) {
  color: #d2991d;
}

:deep(.json-block .json-bool) {
  color: #58a6ff;
}

:deep(.json-block .json-null) {
  color: #8b949e;
}

/* 工具列表 */
.tools-section {
  border-top: 1px solid #30363d;
  padding-top: 24px;
}

.empty-tools {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px;
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  color: #8b949e;
  font-size: 13px;
}

/* 工具卡片 */
.tool-card {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  margin-bottom: 8px;
  overflow: hidden;
  transition: border-color 0.2s ease;
}

.tool-card:hover {
  border-color: #484f58;
}

.tool-card.expanded {
  border-color: #30363d;
}

.tool-header {
  padding: 12px 16px;
  cursor: pointer;
  user-select: none;
  transition: background 0.15s ease;
}

.tool-header:hover {
  background: #1c2333;
}

.tool-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tool-chevron {
  font-size: 12px;
  color: #8b949e;
  width: 14px;
  flex-shrink: 0;
}

.tool-name {
  font-size: 14px;
  font-weight: 600;
  color: #58a6ff;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}

/* 复制按钮：双矩形 CSS 形状 */
.copy-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  opacity: 0;
  transition: opacity 0.15s ease, background 0.15s ease;
  line-height: 1;
  display: flex;
  align-items: center;
}

.tool-header:hover .copy-btn {
  opacity: 0.7;
}

.copy-btn:hover {
  opacity: 1 !important;
  background: #21262d;
}

.copy-icon {
  position: relative;
  width: 14px;
  height: 14px;
  display: inline-block;
}
.copy-icon::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 10px;
  height: 10px;
  border: 1.5px solid #8b949e;
  border-radius: 2px;
}
.copy-icon::after {
  content: '';
  position: absolute;
  bottom: 0;
  right: 0;
  width: 10px;
  height: 10px;
  border: 1.5px solid #8b949e;
  border-radius: 2px;
  background: #161b22;
}

.tool-body {
  padding: 0 16px 16px 34px;
  animation: slide-down 0.2s ease;
}

@keyframes slide-down {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.tool-field {
  margin-bottom: 12px;
}

.tool-field:last-child {
  margin-bottom: 0;
}

.tool-field label {
  display: block;
  font-size: 11px;
  color: #8b949e;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 500;
  margin-bottom: 4px;
}

.tool-field span {
  font-size: 13px;
  color: #c9d1d9;
}

.schema-toggle {
  cursor: pointer;
  display: flex !important;
  align-items: center;
  gap: 4px;
  transition: color 0.15s;
}

.schema-toggle:hover {
  color: #e6edf3;
}

.tool-chevron-sm {
  font-size: 10px;
  width: 12px;
  flex-shrink: 0;
}

.schema-block {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 12px 14px;
  font-size: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  line-height: 1.6;
  overflow-x: auto;
  margin: 6px 0 0;
  white-space: pre-wrap;
  word-break: break-all;
}

:deep(.schema-block .json-key) {
  color: #58a6ff;
}

:deep(.schema-block .json-string) {
  color: #3fb950;
}

:deep(.schema-block .json-number) {
  color: #d2991d;
}

:deep(.schema-block .json-bool) {
  color: #58a6ff;
}

:deep(.schema-block .json-null) {
  color: #8b949e;
}
</style>