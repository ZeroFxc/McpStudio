<script setup lang="ts">
import { ref, onMounted, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n, supportedLocales } from "../i18n";
import type { Theme } from "../themes";

const { t } = useI18n();

/** 从 App.vue 注入的 Toast 函数 */
const showToast = inject<(text: string, type?: "success" | "error" | "info") => void>("showToast", () => {});

/** 从 App.vue 注入的 setLocale 函数 */
const setLocale = inject<(locale: string) => void>("setLocale", () => {});
const currentLocale = inject<{ value: string }>("currentLocale", { value: "zh-CN" });

/** 从 App.vue 注入的主题相关函数 */
const setTheme = inject<(name: string) => void>("setTheme", () => {});
const currentThemeName = inject<{ value: string }>("currentThemeName", { value: "dark" });
const getThemes = inject<() => Theme[]>("getThemes", () => []);

/** 切换语言 */
function switchLanguage(locale: string) {
  setLocale(locale);
  const langLabel = supportedLocales.find((l) => l.key === locale)?.label ?? locale;
  showToast(t("settings.toast.langChanged", { lang: langLabel }), "success");
}

/** 切换主题 */
function switchTheme(name: string) {
  setTheme(name);
  const themes = getThemes();
  const themeLabel = themes.find((th) => th.name === name)?.label ?? name;
  showToast(t("settings.toast.themeChanged", { theme: themeLabel }), "success");
}

/** 打开存储目录 */
async function openStorageDir() {
  try {
    await invoke("open_data_dir");
    showToast(t("settings.toast.dirOpened"), "success");
  } catch (err) {
    showToast(String(err), "error");
  }
}

/** 服务器配置 */
const serverPort = ref(9277);
const bindAddress = ref("0.0.0.0");
const localIps = ref<string[]>([]);

/** 绑定地址选项 */
const bindAddressOptions = [
  { value: "0.0.0.0", label: t("settings.server.localAndLan") },
  { value: "127.0.0.1", label: t("settings.server.localOnly") },
];

/** 获取当前服务器配置和本地 IP */
onMounted(async () => {
  try {
    const config = await invoke<{ http_port: number; bind_address: string; auto_connect: boolean }>("get_server_config");
    serverPort.value = config.http_port;
    bindAddress.value = config.bind_address;
  } catch {
    // 获取失败时保持默认值
  }
  try {
    localIps.value = await invoke<string[]>("get_local_ips");
  } catch {
    // 获取 IP 失败时忽略
  }
});

/** 保存服务器配置 */
async function saveServerConfig() {
  try {
    await invoke("set_http_port", { port: serverPort.value });
    await invoke("set_bind_address", { address: bindAddress.value });
    showToast(t("settings.toast.serverSaved"), "success");
  } catch (err) {
    showToast(String(err), "error");
  }
}
</script>

<template>
  <div class="settings-page">
    <div class="page-header">
      <h2 class="page-title">{{ t("settings.title") }}</h2>
    </div>

    <div class="settings-sections">
      <!-- 语言设置 -->
      <section class="setting-section">
        <div class="section-header">
          <h3 class="section-title">{{ t("settings.language.title") }}</h3>
          <p class="section-desc">{{ t("settings.language.description") }}</p>
        </div>
        <div class="lang-options">
          <div class="lang-toggle">
            <button
              v-for="locale in supportedLocales"
              :key="locale.key"
              :class="['lang-option', { active: currentLocale.value === locale.key }]"
              @click="switchLanguage(locale.key)"
            >
              <span class="lang-code">{{ locale.key === 'zh-CN' ? '中' : 'EN' }}</span>
              <span class="lang-name">{{ locale.label }}</span>
            </button>
          </div>
        </div>
      </section>

      <!-- 主题设置 -->
      <section class="setting-section">
        <div class="section-header">
          <h3 class="section-title">{{ t("settings.theme.title") }}</h3>
          <p class="section-desc">{{ t("settings.theme.description") }}</p>
        </div>
        <div class="theme-options">
          <div class="theme-toggle">
            <button
              v-for="theme in getThemes()"
              :key="theme.name"
              :class="['theme-option', { active: currentThemeName.value === theme.name }]"
              @click="switchTheme(theme.name)"
            >
              <span :class="['theme-swatch', theme.name]"></span>
              <span class="theme-name">{{ theme.label }}</span>
            </button>
          </div>
        </div>
      </section>

      <!-- 服务器设置 -->
      <section class="setting-section">
        <div class="section-header">
          <h3 class="section-title">{{ t("settings.server.title") }}</h3>
          <p class="section-desc">{{ t("settings.server.description") }}</p>
        </div>
        <div class="server-config">
          <div class="server-row">
            <div class="form-group">
              <label for="server-port">{{ t("settings.server.port") }}</label>
              <input
                id="server-port"
                type="number"
                v-model.number="serverPort"
                min="1"
                max="65535"
              />
            </div>
            <div class="form-group">
              <label for="bind-address">{{ t("settings.server.bind") }}</label>
              <select id="bind-address" v-model="bindAddress">
                <option
                  v-for="opt in bindAddressOptions"
                  :key="opt.value"
                  :value="opt.value"
                >{{ opt.label }}</option>
              </select>
            </div>
          </div>
          <div class="server-urls">
            <div class="url-item">
              <span class="url-label">localhost:</span>
              <code>http://localhost:{{ serverPort }}/mcp</code>
            </div>
            <template v-if="bindAddress === '0.0.0.0'">
              <div v-for="ip in localIps" :key="ip" class="url-item">
                <span class="url-label">LAN:</span>
                <code>http://{{ ip }}:{{ serverPort }}/mcp</code>
              </div>
            </template>
          </div>
          <div class="server-hint">{{ t("settings.server.restartHint") }}</div>
          <button class="save-btn" @click="saveServerConfig">{{ t("settings.server.save") }}</button>
        </div>
      </section>

      <!-- 存储设置 -->
      <section class="setting-section">
        <div class="section-header">
          <h3 class="section-title">{{ t("settings.storage.title") }}</h3>
          <p class="section-desc">{{ t("settings.storage.description") }}</p>
        </div>
        <div class="storage-info">
          <div class="storage-path">
            <span class="storage-label">{{ t("settings.storage.path") }}:</span>
            <code class="storage-value">%APPDATA%\McpStudio</code>
          </div>
          <button class="open-dir-btn" @click="openStorageDir">
            <span class="folder-icon">
              <span class="folder-body"></span>
            </span>
            {{ t("settings.storage.openDir") }}
          </button>
        </div>
      </section>

      <!-- 关于 -->
      <section class="setting-section">
        <div class="section-header">
          <h3 class="section-title">{{ t("settings.about.title") }}</h3>
          <p class="section-desc">{{ t("settings.about.description") }}</p>
        </div>
        <div class="about-info">
          <div class="about-row">
            <span class="about-label">{{ t("settings.about.version") }}</span>
            <span class="about-value">0.1.0</span>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  padding: 24px 28px;
  max-width: 640px;
}

.page-header {
  margin-bottom: 28px;
}

.page-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--mc-text-primary);
  margin: 0;
}

.settings-sections {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.setting-section {
  background: var(--mc-bg-card);
  border: 1px solid var(--mc-border-primary);
  border-radius: 10px;
  padding: 20px 22px;
}

.section-header {
  margin-bottom: 16px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--mc-text-primary);
  margin: 0 0 4px;
}

.section-desc {
  font-size: 12px;
  color: var(--mc-text-muted);
  margin: 0;
  line-height: 1.5;
}

/* 语言选项 */
.lang-options {
  display: flex;
}

.lang-toggle {
  display: flex;
  background: var(--mc-lang-toggle-bg);
  border: 1px solid var(--mc-lang-toggle-border);
  border-radius: 8px;
  overflow: hidden;
}

.lang-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 500;
  color: var(--mc-text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.2s ease;
}

.lang-option:hover {
  color: var(--mc-text-primary);
  background: var(--mc-bg-card-hover);
}

.lang-option.active {
  color: var(--mc-lang-active-text);
  background: var(--mc-lang-active-bg);
}

.lang-code {
  font-size: 12px;
  font-weight: 700;
  width: 22px;
  height: 22px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.15);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.lang-option.active .lang-code {
  background: rgba(255, 255, 255, 0.25);
}

.lang-name {
  white-space: nowrap;
}

/* 主题选项 */
.theme-options {
  display: flex;
}

.theme-toggle {
  display: flex;
  background: var(--mc-lang-toggle-bg);
  border: 1px solid var(--mc-lang-toggle-border);
  border-radius: 8px;
  overflow: hidden;
}

.theme-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 18px;
  font-size: 13px;
  font-weight: 500;
  color: var(--mc-text-muted);
  background: transparent;
  border: none;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.2s ease;
}

.theme-option:hover {
  color: var(--mc-text-primary);
  background: var(--mc-bg-card-hover);
}

.theme-option.active {
  color: var(--mc-lang-active-text);
  background: var(--mc-lang-active-bg);
}

.theme-swatch {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  border: 2px solid var(--mc-border-primary);
  flex-shrink: 0;
}

.theme-swatch.dark {
  background: #0d1117;
}

.theme-swatch.light {
  background: #ffffff;
}

.theme-option.active .theme-swatch {
  border-color: rgba(255, 255, 255, 0.5);
}

.theme-name {
  white-space: nowrap;
}

/* 存储 */
.storage-info {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.storage-path {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.storage-label {
  color: var(--mc-text-muted);
  flex-shrink: 0;
}

.storage-value {
  font-family: "JetBrains Mono", "Fira Code", "Consolas", monospace;
  font-size: 12px;
  color: var(--mc-accent-blue);
  background: var(--mc-bg-input);
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid var(--mc-border-primary);
}

.open-dir-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  color: var(--mc-text-primary);
  background: var(--mc-bg-button);
  border: 1px solid var(--mc-border-primary);
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  transition: background 0.2s ease, border-color 0.2s ease;
  align-self: flex-start;
}

.open-dir-btn:hover {
  background: var(--mc-bg-button-hover);
  border-color: var(--mc-accent-blue);
}

.folder-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  position: relative;
  width: 16px;
  height: 12px;
}

.folder-body {
  display: block;
  width: 16px;
  height: 12px;
  background: var(--mc-accent-blue);
  border-radius: 2px;
  position: relative;
}

.folder-body::before {
  content: "";
  position: absolute;
  top: -2px;
  left: 0;
  width: 8px;
  height: 3px;
  background: var(--mc-accent-blue);
  border-radius: 2px 2px 0 0;
}

/* 关于 */
.about-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.about-row {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 13px;
}

.about-label {
  color: var(--mc-text-muted);
  min-width: 48px;
}

.about-value {
  color: var(--mc-text-primary);
  font-family: "JetBrains Mono", "Fira Code", "Consolas", monospace;
  font-size: 12px;
  background: var(--mc-bg-input);
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid var(--mc-border-primary);
}

/* 服务器设置 */
.server-config {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.server-row {
  display: flex;
  gap: 16px;
}

.server-row .form-group {
  flex: 1;
}

.server-row .form-group label {
  display: block;
  font-size: 12px;
  color: var(--mc-text-muted);
  margin-bottom: 6px;
  font-weight: 500;
}

.server-row .form-group input,
.server-row .form-group select {
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  background: var(--mc-bg-input);
  color: var(--mc-text-primary);
  border: 1px solid var(--mc-input-border);
  border-radius: 6px;
  outline: none;
  font-family: inherit;
  box-sizing: border-box;
  transition: border-color 0.2s ease;
}

.server-row .form-group input[type="number"]::-webkit-inner-spin-button,
.server-row .form-group input[type="number"]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}

.server-row .form-group input[type="number"] {
  -moz-appearance: textfield;
}

.server-row .form-group input:focus,
.server-row .form-group select:focus {
  border-color: var(--mc-input-focus-border);
}

.server-row .form-group select {
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%238b949e' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  padding-right: 28px;
}

.server-urls {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.url-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.url-label {
  color: var(--mc-text-muted);
  flex-shrink: 0;
  min-width: 72px;
}

.url-item code {
  font-family: "JetBrains Mono", "Fira Code", "Consolas", monospace;
  font-size: 12px;
  color: var(--mc-accent-blue);
  background: var(--mc-bg-input);
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid var(--mc-border-primary);
  word-break: break-all;
}

.server-hint {
  font-size: 12px;
  color: var(--mc-text-dim);
  font-style: italic;
}

.save-btn {
  align-self: flex-start;
  display: inline-flex;
  align-items: center;
  padding: 8px 20px;
  font-size: 13px;
  font-weight: 500;
  color: var(--mc-text-white);
  background: var(--mc-btn-gradient);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  transition: box-shadow 0.2s ease, opacity 0.2s ease;
}

.save-btn:hover {
  box-shadow: var(--mc-shadow-btn);
}

.save-btn:active {
  opacity: 0.85;
}
</style>