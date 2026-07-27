<script setup lang="ts">
import { ref, onMounted } from "vue";
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

const emit = defineEmits<{
  (e: "select", entry: McpEntry): void;
  (e: "connected"): void;
  (e: "disconnected"): void;
}>();

const mcpList = ref<McpEntry[]>([]);
const selectedName = ref<string>("");
const loading = ref(false);
/** 记录正在连接/断开的 MCP 名称 */
const busyNames = ref<Set<string>>(new Set());

/** 加载 MCP 列表 */
async function loadList() {
  loading.value = true;
  try {
    mcpList.value = await invoke<McpEntry[]>("list_mcp");
    mcpList.value.sort((a, b) => a.name.localeCompare(b.name));
  } catch (err) {
    console.error("加载 MCP 列表失败:", err);
  } finally {
    loading.value = false;
  }
}

/** 选中一个 MCP 条目 */
function selectMcp(entry: McpEntry) {
  selectedName.value = entry.name;
  emit("select", entry);
}

/** 连接 MCP */
async function connectMcp(name: string, event: Event) {
  event.stopPropagation();
  busyNames.value.add(name);
  try {
    await invoke("connect_mcp", { name });
    emit("connected");
    await loadList();
  } catch (err) {
    console.error("连接 MCP 失败:", err);
  } finally {
    busyNames.value.delete(name);
  }
}

/** 断开 MCP */
async function disconnectMcp(name: string, event: Event) {
  event.stopPropagation();
  busyNames.value.add(name);
  try {
    await invoke("disconnect_mcp", { name });
    emit("disconnected");
    await loadList();
  } catch (err) {
    console.error("断开 MCP 失败:", err);
  } finally {
    busyNames.value.delete(name);
  }
}

onMounted(loadList);

defineExpose({ refresh: loadList });
</script>

<template>
  <div class="mcp-list">
    <div class="list-header">
      <h3>{{ t("mcpList.title") }}</h3>
      <button class="refresh-btn" @click="loadList" :disabled="loading">
        <span class="refresh-icon">↻</span>
        {{ loading ? t("mcpList.loading") : t("mcpList.refresh") }}
      </button>
    </div>

    <div v-if="mcpList.length === 0 && !loading" class="empty-state">
      <div class="empty-box"></div>
      <div class="empty-text">{{ t("mcpList.empty.text") }}</div>
      <div class="empty-hint">{{ t("mcpList.empty.hint") }}</div>
    </div>

    <ul class="list-items">
      <li
        v-for="entry in mcpList"
        :key="entry.name"
        :class="['list-item', { selected: selectedName === entry.name }]"
        @click="selectMcp(entry)"
      >
        <div class="item-main">
          <span class="box-icon"></span>
          <span class="item-name">{{ entry.name }}</span>
          <span :class="['status-dot', entry.connected ? 'connected' : 'disconnected']" />
        </div>
        <div class="item-meta">
          <span class="tool-count">{{ t("mcpList.toolCount", { n: entry.tools.length }) }}</span>
          <span :class="['conn-text', entry.connected ? 'connected' : 'disconnected']">
            {{ entry.connected ? t("mcpList.status.connected") : t("mcpList.status.disconnected") }}
          </span>
        </div>
        <div class="item-actions">
          <button
            v-if="!entry.connected"
            class="action-btn connect-btn"
            :disabled="busyNames.has(entry.name)"
            @click="connectMcp(entry.name, $event)"
          >
            {{ busyNames.has(entry.name) ? t("mcpList.actions.connecting") : t("mcpList.actions.connect") }}
          </button>
          <button
            v-else
            class="action-btn disconnect-btn"
            :disabled="busyNames.has(entry.name)"
            @click="disconnectMcp(entry.name, $event)"
          >
            {{ busyNames.has(entry.name) ? t("mcpList.actions.disconnecting") : t("mcpList.actions.disconnect") }}
          </button>
        </div>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.mcp-list {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px 12px;
  border-bottom: 1px solid var(--mc-border-primary);
}

.list-header h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--mc-text-primary);
}

.refresh-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  font-size: 11px;
  font-weight: 500;
  background: var(--mc-bg-button);
  color: var(--mc-text-muted);
  border: 1px solid var(--mc-border-primary);
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.2s ease;
}

.refresh-btn:hover {
  background: var(--mc-border-primary);
  color: var(--mc-text-primary);
}

.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.refresh-icon {
  font-size: 13px;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  text-align: center;
}

.empty-box {
  width: 48px;
  height: 48px;
  border: 2px dashed var(--mc-text-dim);
  border-radius: 8px;
  opacity: 0.5;
  margin-bottom: 12px;
}

.empty-text {
  font-size: 14px;
  color: var(--mc-text-muted);
  font-weight: 500;
  margin-bottom: 4px;
}

.empty-hint {
  font-size: 12px;
  color: var(--mc-text-dim);
}

/* 列表 */
.list-items {
  list-style: none;
  margin: 0;
  padding: 4px 8px;
  overflow-y: auto;
  flex: 1;
}

.list-item {
  padding: 10px 12px;
  margin: 4px 0;
  cursor: pointer;
  background: var(--mc-bg-card);
  border: 1px solid transparent;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.list-item:hover {
  background: var(--mc-bg-card-hover);
  border-color: var(--mc-border-primary);
  transform: translateY(-1px);
}

.list-item.selected {
  background: var(--mc-bg-card-hover);
  border-left: 3px solid var(--mc-accent-blue);
  border-color: var(--mc-border-primary);
  border-left-color: var(--mc-accent-blue);
}

.item-main {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 服务器图标 - CSS 绘制（三条横线堆叠） */
.box-icon {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  justify-content: center;
  opacity: 0.7;
}

.box-icon::before,
.box-icon::after {
  content: "";
  display: block;
  height: 2px;
  border-radius: 1px;
  background: var(--mc-text-muted);
}

.box-icon::before {
  width: 100%;
}

.box-icon::after {
  width: 60%;
}

/* 中间横线（通过内联元素模拟） */
.list-item:hover .box-icon::before,
.list-item:hover .box-icon::after,
.list-item.selected .box-icon::before,
.list-item.selected .box-icon::after {
  background: var(--mc-accent-blue);
}

.item-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--mc-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-dot.connected {
  background: var(--mc-accent-green);
  box-shadow: var(--mc-status-dot-glow);
}

.status-dot.disconnected {
  background: var(--mc-text-dim);
}

.item-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 6px;
  padding-left: 20px;
  font-size: 11px;
}

.tool-count {
  background: var(--mc-bg-button);
  color: var(--mc-text-muted);
  padding: 1px 8px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 500;
}

.conn-text.connected {
  color: var(--mc-accent-green);
  font-weight: 500;
}

.conn-text.disconnected {
  color: var(--mc-text-muted);
}

.item-actions {
  margin-top: 8px;
  padding-left: 20px;
}

.action-btn {
  padding: 4px 14px;
  font-size: 11px;
  font-weight: 500;
  border: 1px solid;
  border-radius: 20px;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.2s ease;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.connect-btn {
  background: var(--mc-btn-connect-bg);
  color: var(--mc-accent-green);
  border-color: color-mix(in srgb, var(--mc-accent-green) 30%, transparent);
}

.connect-btn:hover:not(:disabled) {
  background: var(--mc-btn-connect-hover);
  border-color: color-mix(in srgb, var(--mc-accent-green) 50%, transparent);
}

.disconnect-btn {
  background: var(--mc-btn-delete-bg);
  color: var(--mc-accent-red);
  border-color: color-mix(in srgb, var(--mc-accent-red) 30%, transparent);
}

.disconnect-btn:hover:not(:disabled) {
  background: var(--mc-btn-delete-hover);
  border-color: color-mix(in srgb, var(--mc-accent-red) 50%, transparent);
}
</style>