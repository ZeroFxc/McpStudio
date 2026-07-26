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
  border-bottom: 1px solid #30363d;
}

.list-header h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: #e6edf3;
}

.refresh-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  font-size: 11px;
  font-weight: 500;
  background: #21262d;
  color: #8b949e;
  border: 1px solid #30363d;
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.2s ease;
}

.refresh-btn:hover {
  background: #30363d;
  color: #e6edf3;
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
  border: 2px dashed #484f58;
  border-radius: 8px;
  opacity: 0.5;
  margin-bottom: 12px;
}

.empty-text {
  font-size: 14px;
  color: #8b949e;
  font-weight: 500;
  margin-bottom: 4px;
}

.empty-hint {
  font-size: 12px;
  color: #484f58;
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
  background: #161b22;
  border: 1px solid transparent;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.list-item:hover {
  background: #1c2333;
  border-color: #30363d;
  transform: translateY(-1px);
}

.list-item.selected {
  background: #1c2333;
  border-left: 3px solid #58a6ff;
  border-color: #30363d;
  border-left-color: #58a6ff;
}

.item-main {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 小方框 CSS 形状（替代 📦） */
.box-icon {
  width: 12px;
  height: 12px;
  border: 1.5px solid #8b949e;
  border-radius: 2px;
  flex-shrink: 0;
  opacity: 0.7;
}

.item-name {
  font-size: 13px;
  font-weight: 500;
  color: #e6edf3;
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
  background: #3fb950;
  box-shadow: 0 0 6px rgba(63, 185, 80, 0.5);
}

.status-dot.disconnected {
  background: #484f58;
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
  background: #21262d;
  color: #8b949e;
  padding: 1px 8px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 500;
}

.conn-text.connected {
  color: #3fb950;
  font-weight: 500;
}

.conn-text.disconnected {
  color: #8b949e;
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
  background: rgba(63, 185, 80, 0.1);
  color: #3fb950;
  border-color: rgba(63, 185, 80, 0.3);
}

.connect-btn:hover:not(:disabled) {
  background: rgba(63, 185, 80, 0.2);
  border-color: rgba(63, 185, 80, 0.5);
}

.disconnect-btn {
  background: rgba(248, 81, 73, 0.1);
  color: #f85149;
  border-color: rgba(248, 81, 73, 0.3);
}

.disconnect-btn:hover:not(:disabled) {
  background: rgba(248, 81, 73, 0.2);
  border-color: rgba(248, 81, 73, 0.5);
}
</style>