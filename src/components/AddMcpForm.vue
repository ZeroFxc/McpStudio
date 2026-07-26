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

/** 从 App.vue 注入的 Toast 函数 */
const showToast = inject<(text: string, type?: "success" | "error" | "info") => void>("showToast", () => {});

const jsonInput = ref("");
const submitting = ref(false);
const errorMsg = ref("");

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

  // 兼容两种格式：标准 mcpServers 和内部格式
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
    jsonInput.value = "";
  }
  if (errors.length > 0) {
    errorMsg.value = `${added} ${t("addMcp.toast.partial", { added, failed: errors.length })}: ${errors.join("; ")}`;
  }

  submitting.value = false;
}
</script>

<template>
  <div class="add-form">
    <div class="form-header">
      <h2>{{ t("addMcp.title") }}</h2>
      <p class="form-subtitle">{{ t("addMcp.subtitle") }}</p>
    </div>

    <div v-if="errorMsg" class="error-card">
      <span class="error-bar"></span>
      <span>{{ errorMsg }}</span>
    </div>

    <div class="form-group">
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

    <div class="form-actions">
      <button class="submit-btn" @click="submitJson" :disabled="submitting">
        <span class="submit-btn-content">
          <span v-if="!submitting">{{ t("addMcp.submit") }}</span>
          <span v-else>{{ t("addMcp.submitting") }}</span>
        </span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.add-form {
  max-width: 720px;
  padding: 28px 32px;
}

.form-header {
  margin-bottom: 24px;
}

.form-header h2 {
  margin: 0 0 6px;
  font-size: 22px;
  font-weight: 700;
  color: #e6edf3;
  letter-spacing: -0.3px;
}

.form-subtitle {
  margin: 0;
  font-size: 13px;
  color: #8b949e;
}

/* 错误卡片 */
.error-card {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  background: rgba(248, 81, 73, 0.08);
  border: 1px solid rgba(248, 81, 73, 0.3);
  border-left: 3px solid #f85149;
  color: #f85149;
  padding: 12px 14px;
  border-radius: 6px;
  margin-bottom: 20px;
  font-size: 13px;
  line-height: 1.5;
}

.error-bar {
  width: 3px;
  height: 18px;
  background: #f85149;
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
  color: #8b949e;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.editor-wrapper {
  border: 1px solid #30363d;
  border-radius: 8px;
  overflow: hidden;
  transition: border-color 0.2s ease;
  background: #0d1117;
}

.editor-wrapper:focus-within {
  border-color: #58a6ff;
}

.form-group textarea {
  width: 100%;
  padding: 14px 16px;
  font-size: 13px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  background: #0d1117;
  color: #e6edf3;
  border: none;
  outline: none;
  box-sizing: border-box;
  resize: vertical;
  line-height: 1.6;
  tab-size: 2;
}

.form-group textarea::placeholder {
  color: #484f58;
}

/* 提交按钮 */
.form-actions {
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid #30363d;
}

.submit-btn {
  width: 100%;
  padding: 12px 24px;
  font-size: 14px;
  font-weight: 600;
  color: #fff;
  background: linear-gradient(135deg, #1f6feb, #58a6ff);
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.2s ease;
  position: relative;
}

.submit-btn:hover:not(:disabled) {
  box-shadow: 0 0 20px rgba(88, 166, 255, 0.3);
  transform: translateY(-1px);
}

.submit-btn:active:not(:disabled) {
  transform: translateY(0);
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