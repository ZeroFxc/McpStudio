<script setup lang="ts">
import { inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n, supportedLocales } from "../i18n";

const { t } = useI18n();

/** 从 App.vue 注入的 Toast 函数 */
const showToast = inject<(text: string, type?: "success" | "error" | "info") => void>("showToast", () => {});

/** 从 App.vue 注入的 setLocale 函数 */
const setLocale = inject<(locale: string) => void>("setLocale", () => {});
const currentLocale = inject<{ value: string }>("currentLocale", { value: "zh-CN" });

/** 切换语言 */
function switchLanguage(locale: string) {
  setLocale(locale);
  const langLabel = supportedLocales.find((l) => l.key === locale)?.label ?? locale;
  showToast(t("settings.toast.langChanged", { lang: langLabel }), "success");
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
          <button
            v-for="locale in supportedLocales"
            :key="locale.key"
            :class="['lang-btn', { active: currentLocale.value === locale.key }]"
            @click="switchLanguage(locale.key)"
          >
            <span class="lang-radio">
              <span class="radio-dot" v-if="currentLocale.value === locale.key"></span>
            </span>
            <span class="lang-label">{{ locale.label }}</span>
          </button>
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
  color: #e6edf3;
  margin: 0;
}

.settings-sections {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.setting-section {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 10px;
  padding: 20px 22px;
}

.section-header {
  margin-bottom: 16px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #e6edf3;
  margin: 0 0 4px;
}

.section-desc {
  font-size: 12px;
  color: #8b949e;
  margin: 0;
  line-height: 1.5;
}

/* 语言选项 */
.lang-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.lang-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 8px;
  cursor: pointer;
  font-family: inherit;
  font-size: 13px;
  color: #c9d1d9;
  transition: border-color 0.2s ease, background 0.2s ease;
}

.lang-btn:hover {
  border-color: #58a6ff;
  background: #1a2233;
}

.lang-btn.active {
  border-color: #58a6ff;
  background: #1a2233;
}

.lang-radio {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid #30363d;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: border-color 0.2s ease;
}

.lang-btn.active .lang-radio {
  border-color: #58a6ff;
}

.radio-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #58a6ff;
}

.lang-label {
  font-weight: 500;
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
  color: #8b949e;
  flex-shrink: 0;
}

.storage-value {
  font-family: "JetBrains Mono", "Fira Code", "Consolas", monospace;
  font-size: 12px;
  color: #58a6ff;
  background: #0d1117;
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #30363d;
}

.open-dir-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  color: #e6edf3;
  background: #21262d;
  border: 1px solid #30363d;
  border-radius: 6px;
  cursor: pointer;
  font-family: inherit;
  transition: background 0.2s ease, border-color 0.2s ease;
  align-self: flex-start;
}

.open-dir-btn:hover {
  background: #30363d;
  border-color: #58a6ff;
}

/* 文件夹图标 - CSS 绘制 */
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
  background: #58a6ff;
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
  background: #58a6ff;
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
  color: #8b949e;
  min-width: 48px;
}

.about-value {
  color: #e6edf3;
  font-family: "JetBrains Mono", "Fira Code", "Consolas", monospace;
  font-size: 12px;
  background: #0d1117;
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid #30363d;
}
</style>