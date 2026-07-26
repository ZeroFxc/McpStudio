<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "../i18n";

const { t } = useI18n();

/** 后端返回的使用统计记录 */
interface UsageRecord {
  mcp_name: string;
  tool_name: string;
  count: number;
  last_used: string;
}

const records = ref<UsageRecord[]>([]);
const loading = ref(false);
const limit = ref<number | null>(null);

/** 最大使用次数，用于进度条计算 */
const maxCount = computed(() => {
  if (records.value.length === 0) return 1;
  return Math.max(...records.value.map((r) => r.count), 1);
});

/** 进度条宽度百分比 */
function barWidth(count: number): string {
  return Math.round((count / maxCount.value) * 100) + "%";
}

/** 热度条颜色 */
function barColor(count: number): string {
  const ratio = count / maxCount.value;
  if (ratio > 0.8) return "#3fb950";
  if (ratio > 0.5) return "#58a6ff";
  if (ratio > 0.2) return "#d2991d";
  return "#484f58";
}

/** 加载使用统计 */
async function loadStats() {
  loading.value = true;
  try {
    const args: { limit: number | null } = { limit: limit.value };
    records.value = await invoke<UsageRecord[]>("get_usage_stats", args);
    records.value.sort((a, b) => b.count - a.count);
  } catch (err) {
    console.error("加载使用统计失败:", err);
  } finally {
    loading.value = false;
  }
}

/** 格式化时间字符串 */
function formatTime(iso: string): string {
  if (!iso) return "-";
  try {
    const d = new Date(iso);
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
  } catch {
    return iso;
  }
}

onMounted(loadStats);
</script>

<template>
  <div class="usage-stats">
    <div class="stats-header">
      <div class="stats-header-left">
        <h2>{{ t("usageStats.title") }}</h2>
        <span class="stats-count">{{ t("usageStats.recordCount", { n: records.length }) }}</span>
      </div>
      <div class="stats-controls">
        <label class="limit-label">
          {{ t("usageStats.limit") }}
          <input
            type="number"
            v-model.number="limit"
            :placeholder="t('usageStats.limitPlaceholder')"
            min="1"
            class="limit-input"
          />
        </label>
        <button class="query-btn" @click="loadStats" :disabled="loading">
          <span class="query-icon">↻</span>
          {{ loading ? t("usageStats.querying") : t("usageStats.query") }}
        </button>
      </div>
    </div>

    <div v-if="records.length === 0 && !loading" class="empty-state">
      <div class="bar-icon">
        <i></i>
        <i></i>
        <i></i>
      </div>
      <div class="empty-text">{{ t("usageStats.empty.text") }}</div>
      <div class="empty-hint">{{ t("usageStats.empty.hint") }}</div>
    </div>

    <div v-else class="table-container">
      <table class="stats-table">
        <thead>
          <tr>
            <th class="col-mcp">{{ t("usageStats.columns.mcpName") }}</th>
            <th class="col-tool">{{ t("usageStats.columns.toolName") }}</th>
            <th class="col-count">{{ t("usageStats.columns.count") }}</th>
            <th class="col-time">{{ t("usageStats.columns.lastUsed") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(rec, idx) in records" :key="idx">
            <td class="cell-mcp">
              <span class="mcp-box-icon"></span>
              {{ rec.mcp_name }}
            </td>
            <td class="cell-tool">{{ rec.tool_name }}</td>
            <td class="cell-count">
              <div class="count-bar-wrapper">
                <div
                  class="count-bar"
                  :style="{
                    width: barWidth(rec.count),
                    background: barColor(rec.count),
                  }"
                ></div>
                <span class="count-value">{{ rec.count }}</span>
              </div>
            </td>
            <td class="cell-time">{{ formatTime(rec.last_used) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.usage-stats {
  padding: 28px 32px;
}

/* 头部 */
.stats-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
  flex-wrap: wrap;
  gap: 16px;
}

.stats-header-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.stats-header-left h2 {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: #e6edf3;
  letter-spacing: -0.3px;
}

.stats-count {
  font-size: 12px;
  color: #8b949e;
  font-weight: 500;
}

.stats-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.limit-label {
  font-size: 12px;
  color: #8b949e;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 6px;
}

.limit-input {
  width: 64px;
  padding: 6px 10px;
  font-size: 12px;
  background: #0d1117;
  color: #e6edf3;
  border: 1px solid #30363d;
  border-radius: 6px;
  outline: none;
  font-family: inherit;
  transition: border-color 0.2s ease;
}

.limit-input:focus {
  border-color: #58a6ff;
}

.query-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 6px 16px;
  font-size: 12px;
  font-weight: 500;
  background: #21262d;
  color: #e6edf3;
  border: 1px solid #30363d;
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.2s ease;
}

.query-btn:hover:not(:disabled) {
  background: #30363d;
  border-color: #58a6ff;
}

.query-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.query-icon {
  font-size: 14px;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
}

/* 统计条 CSS 形状（替代 📊） */
.bar-icon {
  display: flex;
  align-items: flex-end;
  gap: 4px;
  height: 28px;
  margin-bottom: 16px;
  opacity: 0.4;
}
.bar-icon i {
  display: block;
  width: 6px;
  border-radius: 3px;
  background: #8b949e;
}
.bar-icon i:nth-child(1) { height: 12px; }
.bar-icon i:nth-child(2) { height: 22px; }
.bar-icon i:nth-child(3) { height: 16px; }

.empty-text {
  font-size: 16px;
  color: #8b949e;
  font-weight: 500;
  margin-bottom: 6px;
}

.empty-hint {
  font-size: 13px;
  color: #484f58;
}

/* 表格容器 */
.table-container {
  border: 1px solid #30363d;
  border-radius: 8px;
  overflow: hidden;
}

.stats-table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 0;
  font-size: 13px;
}

.stats-table thead {
  position: sticky;
  top: 0;
  z-index: 1;
}

.stats-table th {
  text-align: left;
  padding: 10px 16px;
  background: #161b22;
  color: #8b949e;
  font-weight: 600;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid #30363d;
}

.col-mcp { width: 25%; }
.col-tool { width: 30%; }
.col-count { width: 20%; }
.col-time { width: 25%; }

.stats-table td {
  padding: 12px 16px;
  border-bottom: 1px solid #21262d;
  color: #c9d1d9;
}

.stats-table tbody tr {
  transition: background 0.15s ease;
}

.stats-table tbody tr:last-child td {
  border-bottom: none;
}

.stats-table tbody tr:hover {
  background: #161b22;
}

/* MCP 名称 */
.cell-mcp {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #e6edf3;
  font-weight: 500;
}

/* 小方框 CSS 形状（替代 📦） */
.mcp-box-icon {
  width: 12px;
  height: 12px;
  border: 1.5px solid #8b949e;
  border-radius: 2px;
  flex-shrink: 0;
  opacity: 0.7;
}

/* 工具名称 */
.cell-tool {
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  color: #58a6ff;
  font-size: 12px;
}

/* 使用次数 - 进度条 */
.cell-count {
  min-width: 120px;
}

.count-bar-wrapper {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
}

.count-bar {
  height: 8px;
  border-radius: 4px;
  min-width: 4px;
  transition: width 0.4s ease;
  opacity: 0.7;
}

.count-value {
  font-size: 12px;
  font-weight: 600;
  color: #e6edf3;
  white-space: nowrap;
  min-width: 24px;
  text-align: right;
}

/* 最近使用时间 */
.cell-time {
  color: #8b949e;
  font-size: 12px;
  white-space: nowrap;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
}
</style>