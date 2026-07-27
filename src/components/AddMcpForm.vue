<script setup lang="ts">
import { ref, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "../i18n";

const { t } = useI18n();

interface McpEntry {
  name: string;
  description: string;
  connection: any;
  tools: any[];
  connected: boolean;
}

const emit = defineEmits<{
  (e: "added", entry: McpEntry): void;
}>();

/** 从 App.vue 注入 */
const showToast = inject<(text: string, type?: "success" | "error" | "info") => void>("showToast", () => {});
const isAndroid = inject<{ value: boolean }>("isAndroid", { value: false });

/** 输入模式：JSON 或 URL */
type InputMode = "json" | "url";
const inputMode = ref<InputMode>(isAndroid.value ? "url" : "json");

/** JSON 模式 */
const jsonInput = ref("");
const submitting = ref(false);

/** URL 模式 */
const urlInput = ref("");
const urlNameInput = ref("");
const urlDescInput = ref("");

/** 错误信息 */
const errorMsg = ref("");

/** 从 URL 中提取域名作为默认名称 */
function extractDomain(url: string): string {
  try {
    const u = new URL(url);
    return u.hostname;
  } catch {
    return "";
  }
}

/** 监听 URL 输入变化，自动填充名称 */
function onUrlInput() {
  if (!urlNameInput.value) {
    urlNameInput.value = extractDomain(urlInput.value);
  }
}

/** 标准 mcpServers 配置项 */
interface ServerConfig {
  command?: string;
  args?: string[];
  url?: string;
}

/** 将标准 mcpServers 格式转为内部格式 */
function normalizeServers(raw: any): Array<{ name: string; description: string; connection: any }> {
  const servers: Record<string, ServerConfig> = raw.mcpServers;
  const entries: Array<{ name: string; description: string; connection: any }> = [];

  for (const [name, cfg] of Object.entries(servers)) {
    if (cfg && typeof cfg === "object") {
      if (cfg.command) {
        entries.push({
          name,
          description: "",
          connection: {
            type: "stdio",
            command: cfg.command,
            args: cfg.args ?? [],
          },
        });
      } else if (cfg.url) {
        entries.push({
          name,
          description: "",
          connection: {
            type: "streamable_http",
            url: String(cfg.url).replace(/`/g, ""),
          },
        });
      }
    }
  }
  return entries;
}

/** 提交 JSON 配置 */
async function submitJson() {
  errorMsg.value = "";

  if (!jsonInput.value.trim()) {
    errorMsg.value = t("addMcp.errors.empty");
    return;
  }

  let raw: any;
  try {
    raw = JSON.parse(jsonInput.value);
  } catch {
    errorMsg.value = t("addMcp.errors.parseError");
    return;
  }

  let entries: Array<{ name: string; description: string; connection: any }>;
  if (raw.mcpServers) {
    entries = normalizeServers(raw);
  } else if (raw.name && raw.connection) {
    entries = [raw];
  } else {
    errorMsg.value = t("addMcp.errors.invalidFormat");
    return;
  }

  if (entries.length === 0) {
    errorMsg.value = t("addMcp.errors.noValid");
    return;
  }

  await addEntries(entries);
}

/** 提交 URL 模式 */
async function submitUrl() {
  errorMsg.value = "";

  const url = urlInput.value.trim();
  if (!url) {
    errorMsg.value = t("addMcp.errors.urlEmpty");
    return;
  }

  // 验证 URL 格式
  if (!/^https?:\/\/.+/.test(url)) {
    errorMsg.value = t("addMcp.errors.urlInvalid");
    return;
  }

  const name = urlNameInput.value.trim() || extractDomain(url) || "MCP Service";
  const description = urlDescInput.value.trim();

  await addEntries([
    {
      name,
      description,
      connection: {
        type: "streamable_http",
        url,
      },
    },
  ]);
}

/** 通用添加逻辑 */
async function addEntries(entries: Array<{ name: string; description: string; connection: any }>) {
  submitting.value = true;
  let added = 0;
  let errors: string[] = [];

  for (const entry of entries) {
    try {
      await invoke<McpEntry>("add_mcp", {
        name: entry.name.trim(),
        description: entry.description.trim(),
        connection: entry.connection,
      });
      // 自动连接以获取工具列表
      try {
        await invoke("connect_mcp", { name: entry.name });
      } catch {
        // 连接失败不影响添加
      }
      added++;
    } catch (err) {
      errors.push(`${entry.name}: ${err}`);
    }
  }

  if (added > 0) {
    showToast(t("addMcp.toast.added", { n: added }), "success");
    emit("added", { name: entries[0].name, description: "", connection: entries[0].connection, tools: [], connected: false });
    // 清空表单
    jsonInput.value = "";
    urlInput.value = "";
    urlNameInput.value = "";
    urlDescInput.value = "";
  }
  if (errors.length > 0) {
    errorMsg.value = `${added} ${t("addMcp.toast.partial", { added, failed: errors.length })}: ${errors.join("; ")}`;
  }

  submitting.value = false;
}

/** 统一提交入口 */
function submitForm() {
  if (inputMode.value === "url") {
    submitUrl();
  } else {
    submitJson();
  }
}
</script>

<template>
  <div class="add-form">
    <div class="form-header">
      <h2>{{ t("addMcp.title") }}</h2>
      <p class="form-subtitle">{{ t("addMcp.subtitle") }}</p>
    </div>

    <!-- 模式切换 -->
    <div class="mode-switch">
      <button
        :class="['mode-btn', { active: inputMode === 'json' }]"
        @click="inputMode = 'json'"
      >{{ t("addMcp.inputMode.json") }}</button>
      <button
        :class="['mode-btn', { active: inputMode === 'url' }]"
        @click="inputMode = 'url'"
      >{{ t("addMcp.inputMode.url") }}</button>
    </div>

    <div v-if="errorMsg" class="error-card">
      <span class="error-bar"></span>
      <span>{{ errorMsg }}</span>
    </div>

    <!-- JSON 模式 -->
    <div v-if="inputMode === 'json'" class="form-group">
      <label for="mcp-json">{{ t("addMcp.label") }}</label>
      <div class="editor-wrapper">
        <textarea
          id="mcp-json"
          v-model="jsonInput"
          :placeholder="t('addMcp.placeholder')"
          rows="14"
          spellcheck="false"
        />
      </div>
    </div>

    <!-- URL 模式 -->
    <div v-if="inputMode === 'url'" class="url-form">
      <div class="form-group">
        <label for="mcp-url">{{ t("addMcp.urlLabel") }}</label>
        <div class="editor-wrapper">
          <input
            id="mcp-url"
            v-model="urlInput"
            type="url"
            :placeholder="t('addMcp.urlPlaceholder')"
            class="url-input"
            @input="onUrlInput"
          />
        </div>
      </div>

      <div class="form-group">
        <label for="mcp-url-name">{{ t("addMcp.urlNamePlaceholder") }}</label>
        <div class="editor-wrapper">
          <input
            id="mcp-url-name"
            v-model="urlNameInput"
            type="text"
            :placeholder="t('addMcp.urlNamePlaceholder')"
            class="url-input"
          />
        </div>
      </div>

      <div class="form-group">
        <label for="mcp-url-desc">{{ t("addMcp.urlDescriptionPlaceholder") }}</label>
        <div class="editor-wrapper">
          <input
            id="mcp-url-desc"
            v-model="urlDescInput"
            type="text"
            :placeholder="t('addMcp.urlDescriptionPlaceholder')"
            class="url-input"
          />
        </div>
      </div>
    </div>

    <div class="form-actions">
      <button class="submit-btn" @click="submitForm" :disabled="submitting">
        <span class="submit-btn-content">
          <span v-if="!submitting">{{ t("addMcp.submit") }}</span>
          <span v-else><span class="spinner"></span>{{ t("addMcp.submitting") }}</span>
        </span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.add-form {
  max-width: 720px;
  padding: var(--mc-space-page-padding);
}

.form-header {
  margin-bottom: var(--mc-space-section-gap);
}

.form-header h2 {
  margin: 0 0 6px;
  font-size: var(--mc-font-page-title);
  font-weight: 700;
  color: var(--mc-text-primary);
  letter-spacing: -0.3px;
}

.form-subtitle {
  margin: 0;
  font-size: 13px;
  color: var(--mc-text-muted);
}

/* 模式切换 */
.mode-switch {
  display: flex;
  gap: 0;
  margin-bottom: 20px;
  background: var(--mc-bg-button);
  border-radius: var(--mc-radius-md);
  padding: 3px;
}

.mode-btn {
  flex: 1;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  color: var(--mc-text-muted);
  background: none;
  border: none;
  border-radius: calc(var(--mc-radius-md) - 2px);
  cursor: pointer;
  font-family: inherit;
  transition: all 0.2s ease;
}

.mode-btn.active {
  background: var(--mc-bg-card);
  color: var(--mc-text-primary);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

/* 错误卡片 */
.error-card {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  background: var(--mc-error-bg);
  border: 1px solid var(--mc-error-border);
  border-left: 3px solid var(--mc-accent-red);
  color: var(--mc-accent-red);
  padding: 12px 14px;
  border-radius: var(--mc-radius-sm);
  margin-bottom: 20px;
  font-size: 13px;
  line-height: 1.5;
}

.error-bar {
  width: 3px;
  height: 18px;
  background: var(--mc-accent-red);
  border-radius: 2px;
  flex-shrink: 0;
  margin-top: 1px;
}

/* 表单组 */
.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: var(--mc-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.editor-wrapper {
  border: 1px solid var(--mc-border-primary);
  border-radius: var(--mc-radius-md);
  overflow: hidden;
  transition: border-color 0.2s ease;
  background: var(--mc-bg-input);
}

.editor-wrapper:focus-within {
  border-color: var(--mc-accent-blue);
}

.form-group textarea {
  width: 100%;
  padding: 14px 16px;
  font-size: 13px;
  font-family: var(--mc-font-mono);
  background: var(--mc-bg-input);
  color: var(--mc-text-primary);
  border: none;
  outline: none;
  box-sizing: border-box;
  resize: vertical;
  line-height: 1.6;
  tab-size: 2;
}

.form-group textarea::placeholder {
  color: var(--mc-text-dim);
}

/* URL 输入框 */
.url-form {
  margin-bottom: 0;
}

.url-input {
  width: 100%;
  padding: 12px 16px;
  font-size: 14px;
  font-family: var(--mc-font-mono);
  background: var(--mc-bg-input);
  color: var(--mc-text-primary);
  border: none;
  outline: none;
  box-sizing: border-box;
}

.url-input::placeholder {
  color: var(--mc-text-dim);
}

/* 提交按钮 */
.form-actions {
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid var(--mc-border-primary);
}

.submit-btn {
  width: 100%;
  padding: 12px 24px;
  font-size: 14px;
  font-weight: 600;
  color: var(--mc-text-white);
  background: var(--mc-btn-gradient);
  border: none;
  border-radius: var(--mc-radius-md);
  cursor: pointer;
  font-family: inherit;
  transition: all 0.2s ease;
  position: relative;
}

.submit-btn:hover:not(:disabled) {
  box-shadow: var(--mc-shadow-btn);
  transform: translateY(-1px);
}

.submit-btn:active:not(:disabled) {
  transform: scale(0.98);
}

.submit-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.submit-btn-content {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
</style>