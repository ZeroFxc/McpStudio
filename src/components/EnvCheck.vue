<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "../i18n";

const { t } = useI18n();

/** 检测工具定义 */
interface ToolInfo {
  key: string;
  command: string;
}

const tools: ToolInfo[] = [
  { key: "python", command: "python" },
  { key: "uv", command: "uv" },
  { key: "node", command: "node" },
  { key: "npm", command: "npm" },
  { key: "git", command: "git" },
  { key: "pip", command: "pip" },
];

/** 单个工具的检测结果 */
interface ToolResult {
  key: string;
  installed: boolean;
  version: string;
  checking: boolean;
}

/** 所有工具的检测状态 */
const results = ref<ToolResult[]>(
  tools.map((t) => ({ key: t.key, installed: false, version: "", checking: true }))
);

/** 已安装工具数量 */
const installedCount = computed(() => results.value.filter((r) => r.installed).length);

/** 是否全部检测完成 */
const allChecked = computed(() => results.value.every((r) => !r.checking));

/** 展开详情的工具 key 集合 */
const expandedKeys = ref<Set<string>>(new Set());

/** 版本号超过此长度视为长文本 */
const MAX_VERSION_LEN = 35;

/** 是否长版本号 */
function isLongVersion(version: string): boolean {
  return version.length > MAX_VERSION_LEN;
}

/** 截断版本号 */
function truncateVersion(version: string): string {
  return version.slice(0, MAX_VERSION_LEN) + "...";
}

/** 切换展开/收起 */
function toggleExpand(key: string) {
  const next = new Set(expandedKeys.value);
  if (next.has(key)) {
    next.delete(key);
  } else {
    next.add(key);
  }
  expandedKeys.value = next;
}

/** 后端返回的检测结果 */
interface EnvCheckResult {
  installed: boolean;
  version: string;
}

/** 检测单个工具 */
async function checkTool(tool: ToolInfo, index: number): Promise<void> {
  try {
    const res = await invoke<EnvCheckResult>("check_env", { command: tool.command });
    results.value[index] = {
      key: tool.key,
      installed: res.installed,
      version: res.version,
      checking: false,
    };
  } catch {
    results.value[index] = {
      key: tool.key,
      installed: false,
      version: "",
      checking: false,
    };
  }
}

/** 获取工具名称的 i18n key */
function toolNameKey(key: string): string {
  return `envCheck.tools.${key}`;
}

/** 组件挂载时自动检测所有工具 */
onMounted(() => {
  tools.forEach((tool, index) => {
    checkTool(tool, index);
  });
});
</script>

<template>
  <div class="env-check-page">
    <div class="page-header">
      <h2 class="page-title">{{ t("envCheck.title") }}</h2>
      <p class="page-subtitle">{{ t("envCheck.subtitle") }}</p>
    </div>

    <!-- 检测摘要 -->
    <div class="summary-bar">
      <span class="summary-icon" :class="{ done: allChecked }">
        <span v-if="allChecked" class="checkmark">&#10003;</span>
        <span v-else class="spinner"></span>
      </span>
      <span class="summary-text">
        {{ t("envCheck.summary", { installed: installedCount, total: tools.length }) }}
      </span>
    </div>

    <!-- 工具卡片网格 -->
    <div class="tools-grid">
      <div
        v-for="result in results"
        :key="result.key"
        class="tool-card"
        :class="{ installed: result.installed, checking: result.checking }"
      >
        <div class="tool-status">
          <span class="status-dot" :class="{ installed: result.installed, checking: result.checking }"></span>
        </div>
        <div class="tool-info">
          <span class="tool-name">{{ t(toolNameKey(result.key)) }}</span>
          <template v-if="result.checking">
            <span class="tool-version">{{ t("envCheck.checking") }}</span>
          </template>
          <template v-else-if="result.installed">
            <div class="version-row">
              <span
                class="tool-version mono"
                :title="isLongVersion(result.version) && !expandedKeys.has(result.key) ? result.version : undefined"
              >
                <template v-if="isLongVersion(result.version) && !expandedKeys.has(result.key)">
                  {{ truncateVersion(result.version) }}
                </template>
                <template v-else>
                  {{ result.version }}
                </template>
              </span>
              <button
                v-if="isLongVersion(result.version)"
                class="expand-btn"
                @click.stop="toggleExpand(result.key)"
              >
                {{ expandedKeys.has(result.key) ? t("envCheck.collapse") : t("envCheck.expand") }}
              </button>
            </div>
            <!-- 展开详情面板 -->
            <div v-if="isLongVersion(result.version) && expandedKeys.has(result.key)" class="version-detail">
              <span class="version-detail-text">{{ result.version }}</span>
            </div>
          </template>
          <template v-else>
            <span class="tool-version">{{ t("envCheck.notInstalled") }}</span>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.env-check-page {
  padding: var(--mc-space-page-padding);
  max-width: 640px;
}

.page-header {
  margin-bottom: 24px;
}

.page-title {
  font-size: var(--mc-font-page-title);
  font-weight: 700;
  color: var(--mc-text-primary);
  margin: 0;
}

.page-subtitle {
  font-size: 13px;
  color: var(--mc-text-muted);
  margin: 6px 0 0;
  line-height: 1.5;
}

/* 检测摘要 */
.summary-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  background: var(--mc-bg-card);
  border: 1px solid var(--mc-border-primary);
  border-radius: var(--mc-radius-md);
  padding: 12px 16px;
  margin-bottom: 20px;
}

.summary-icon {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: var(--mc-bg-input);
  border: 2px solid var(--mc-border-primary);
  transition: all 0.3s ease;
}

.summary-icon.done {
  background: var(--mc-accent-green, #28a745);
  border-color: var(--mc-accent-green, #28a745);
}

.checkmark {
  color: #fff;
  font-size: 14px;
  font-weight: 700;
  line-height: 1;
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--mc-border-primary);
  border-top-color: var(--mc-accent-blue);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.summary-text {
  font-size: 13px;
  font-weight: 500;
  color: var(--mc-text-primary);
}

/* 工具卡片网格 */
.tools-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.tool-card {
  display: flex;
  align-items: center;
  gap: 12px;
  background: var(--mc-bg-card);
  border: 1px solid var(--mc-border-primary);
  border-radius: var(--mc-radius-md);
  padding: 14px 16px;
  transition: transform 0.2s ease, box-shadow 0.2s ease, border-color 0.2s ease;
  cursor: default;
}

.tool-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--mc-shadow-btn, 0 2px 12px rgba(0, 0, 0, 0.15));
  border-color: var(--mc-accent-blue);
}

.tool-card.installed {
  border-color: var(--mc-accent-green, rgba(40, 167, 69, 0.3));
}

.tool-card.checking {
  opacity: 0.7;
}

/* 状态指示器 */
.tool-status {
  flex-shrink: 0;
}

.status-dot {
  display: block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--mc-text-dim, #666);
  transition: background 0.3s ease;
}

.status-dot.installed {
  background: var(--mc-accent-green, #28a745);
  box-shadow: 0 0 6px rgba(40, 167, 69, 0.4);
}

.status-dot.checking {
  background: var(--mc-accent-blue, #007acc);
  animation: pulse 1.2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.4;
  }
}

/* 工具信息 */
.tool-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  overflow: hidden;
}

.tool-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--mc-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tool-version {
  font-size: 12px;
  color: var(--mc-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  min-width: 0;
}

.tool-version.mono {
  font-family: var(--mc-font-mono);
  color: var(--mc-accent-blue, #007acc);
}

/* 版本号行 */
.version-row {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
}

/* 展开/收起按钮 */
.expand-btn {
  flex-shrink: 0;
  padding: 1px 6px;
  font-size: 11px;
  font-weight: 500;
  color: var(--mc-accent-blue);
  background: transparent;
  border: 1px solid var(--mc-accent-blue);
  border-radius: 3px;
  cursor: pointer;
  font-family: inherit;
  transition: background 0.15s ease, color 0.15s ease;
  line-height: 1.4;
}

.expand-btn:hover {
  background: var(--mc-accent-blue);
  color: #fff;
}

/* 展开详情面板 */
.version-detail {
  margin-top: 6px;
  padding: 8px 10px;
  background: var(--mc-chip-bg);
  border: 1px solid var(--mc-border-primary);
  border-radius: 4px;
  word-break: break-all;
}

.version-detail-text {
  font-size: 12px;
  font-family: var(--mc-font-mono);
  color: var(--mc-text-primary);
  line-height: 1.5;
}
</style>