<script setup lang="ts">
import { ref, inject, nextTick, watch } from "vue";
import { useI18n } from "../i18n";

const { t } = useI18n();

/** 从 App.vue 注入的 Toast 函数 */
const showToast = inject<(text: string, type?: "success" | "error" | "info") => void>("showToast", () => {});

/** GitHub 仓库结果 */
interface RepoResult {
  id: number;
  name: string;
  full_name: string;
  description: string;
  html_url: string;
  stargazers_count: number;
  language: string;
  topics: string[];
  owner: {
    login: string;
    avatar_url: string;
  };
}

/** GitHub 文件/目录项 */
interface GitHubFile {
  name: string;
  path: string;
  type: "file" | "dir";
  html_url: string;
}

const searchQuery = ref("");
const searching = ref(false);
const results = ref<RepoResult[]>([]);
const errorMsg = ref("");

/** 预览相关状态 */
const view = ref<"search" | "preview">("search");
const selectedRepo = ref<RepoResult | null>(null);
const files = ref<GitHubFile[]>([]);
const loadingFiles = ref(false);
const readmeContent = ref("");
const loadingReadme = ref(false);
const activeTab = ref<"files" | "readme">("files");
const readmeBodyRef = ref<HTMLElement | null>(null);

/** 搜索 GitHub 仓库 */
async function searchGitHub() {
  errorMsg.value = "";
  results.value = [];

  const query = searchQuery.value.trim();

  searching.value = true;
  try {
    // 始终在 MCP 相关话题中搜索，关键词可选
    let q = "topic:mcp-server";
    if (query) {
      q = `${encodeURIComponent(query)}+topic:mcp-server`;
    }
    const url = `https://api.github.com/search/repositories?q=${q}&sort=stars&order=desc&per_page=20`;

    const resp = await fetch(url, {
      headers: { Accept: "application/vnd.github.v3+json" },
    });

    if (!resp.ok) {
      if (resp.status === 403) {
        throw new Error("GitHub API rate limit exceeded. Try again later.");
      }
      throw new Error(`GitHub API error: ${resp.status}`);
    }

    const data = await resp.json();
    results.value = (data.items || []).map((item: any) => ({
      id: item.id,
      name: item.name,
      full_name: item.full_name,
      description: item.description || "",
      html_url: item.html_url,
      stargazers_count: item.stargazers_count,
      language: item.language || "",
      topics: item.topics || [],
      owner: {
        login: item.owner.login,
        avatar_url: item.owner.avatar_url,
      },
    }));
  } catch (err) {
    errorMsg.value = String(err);
    showToast(String(err), "error");
  } finally {
    searching.value = false;
  }
}

/** 格式化星数 */
function formatStars(n: number): string {
  if (n >= 1000) {
    return (n / 1000).toFixed(1) + "k";
  }
  return String(n);
}

/** 进入仓库预览页 */
function openRepo(repo: RepoResult) {
  selectedRepo.value = repo;
  view.value = "preview";
  files.value = [];
  readmeContent.value = "";
  activeTab.value = "files";
  // 重置委托标记，确保新 readme 能重新绑定
  if (readmeBodyRef.value) {
    delete readmeBodyRef.value.dataset.copyDelegation;
  }
  loadFiles(repo.full_name);
  loadReadme(repo.full_name);
}

/** 返回搜索结果 */
function backToSearch() {
  view.value = "search";
}

/** 在外部浏览器打开链接 */
function openExternal(url: string) {
  window.open(url, "_blank");
}

/** 加载仓库文件列表 */
async function loadFiles(fullName: string) {
  loadingFiles.value = true;
  try {
    const resp = await fetch(`https://api.github.com/repos/${fullName}/contents/`, {
      headers: { Accept: "application/vnd.github.v3+json" },
    });
    if (!resp.ok) {
      files.value = [];
      return;
    }
    const data = await resp.json();
    if (Array.isArray(data)) {
      files.value = data.map((item: any) => ({
        name: item.name,
        path: item.path,
        type: item.type as "file" | "dir",
        html_url: item.html_url,
      }));
    }
  } catch {
    files.value = [];
  } finally {
    loadingFiles.value = false;
  }
}

/** 加载 README 内容 */
async function loadReadme(fullName: string) {
  loadingReadme.value = true;
  try {
    const resp = await fetch(`https://api.github.com/repos/${fullName}/readme`, {
      headers: { Accept: "application/vnd.github.v3+json" },
    });
    if (!resp.ok) {
      readmeContent.value = "";
      return;
    }
    const data = await resp.json();
    const raw = atob(data.content);
    readmeContent.value = renderMarkdown(raw);
    // 如果当前已在 readme 标签页，绑定事件委托
    if (activeTab.value === "readme") {
      await nextTick();
      bindCopyDelegation();
    }
  } catch {
    readmeContent.value = "";
  } finally {
    loadingReadme.value = false;
  }
}

/** 复制文本到剪贴板（兼容 WebView 环境） */
async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    try {
      const ta = document.createElement("textarea");
      ta.value = text;
      ta.style.position = "fixed";
      ta.style.left = "-9999px";
      ta.style.top = "-9999px";
      document.body.appendChild(ta);
      ta.focus();
      ta.select();
      const ok = document.execCommand("copy");
      document.body.removeChild(ta);
      return ok;
    } catch {
      return false;
    }
  }
}

/** 使用事件委托处理复制按钮点击 */
function handleCopyClick(e: Event) {
  const target = e.target as HTMLElement;
  const btn = target.closest(".copy-code-btn") as HTMLElement | null;
  if (!btn) return;
  e.stopPropagation();
  const code = btn.getAttribute("data-code") || "";
  const decoded = code
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'");
  copyToClipboard(decoded).then((ok) => {
    if (ok) {
      btn.classList.add("copied");
      setTimeout(() => btn.classList.remove("copied"), 1500);
    } else {
      showToast("Copy failed", "error");
    }
  });
}

/** 绑定事件委托到 readme 容器 */
function bindCopyDelegation() {
  if (!readmeBodyRef.value) return;
  // 避免重复绑定
  if (readmeBodyRef.value.dataset.copyDelegation === "1") return;
  readmeBodyRef.value.dataset.copyDelegation = "1";
  readmeBodyRef.value.addEventListener("click", handleCopyClick);
}

/** 监听 activeTab 切换到 readme 时绑定事件 */
watch(activeTab, (tab) => {
  if (tab === "readme") {
    nextTick(() => bindCopyDelegation());
  }
});

/** Markdown 转 HTML */
function renderMarkdown(md: string): string {
  // 统一换行符
  let html = md.replace(/\r\n/g, "\n");

  // HTML 转义（先对非代码块内容做基础转义）
  const escapeHtml = (s: string) =>
    s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

  // 1. 代码块 (```) —— 先提取，避免内部内容被后续正则干扰
  const codeBlocks: { placeholder: string; html: string }[] = [];
  html = html.replace(/```(\w*)\n([\s\S]*?)```/g, (_m, lang, code) => {
    const placeholder = `\x00CODEBLOCK${codeBlocks.length}\x00`;
    const escaped = escapeHtml(code);
    codeBlocks.push({
      placeholder,
      html: `<div class="code-block-wrapper"><pre><code class="language-${lang || ""}">${escaped}</code></pre></div>`,
    });
    return placeholder;
  });

  // 2. 行内代码 (`code`) —— 在 HTML 标签处理前
  html = html.replace(/`([^`\n]+)`/g, (_m, code) => {
    return `<code>${escapeHtml(code)}</code>`;
  });

  // 3. 图片 ![alt](url)
  html = html.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" loading="lazy" />');

  // 4. 链接 [text](url)
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>');

  // 5. 标题
  html = html.replace(/^#### (.+)$/gm, "<h4>$1</h4>");
  html = html.replace(/^### (.+)$/gm, "<h3>$1</h3>");
  html = html.replace(/^## (.+)$/gm, "<h2>$1</h2>");
  html = html.replace(/^# (.+)$/gm, "<h1>$1</h1>");

  // 6. 水平线
  html = html.replace(/^[-*_]{3,}\s*$/gm, "<hr />");

  // 7. 粗体 + 斜体
  html = html.replace(/\*\*\*(.+?)\*\*\*/g, "<strong><em>$1</em></strong>");
  html = html.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/\*(.+?)\*/g, "<em>$1</em>");

  // 8. 引用块
  html = html.replace(/^&gt;\s?(.+)$/gm, "<blockquote>$1</blockquote>");
  // 合并连续引用块
  html = html.replace(/<\/blockquote>\n<blockquote>/g, "<br />");

  // 9. 有序列表
  html = html.replace(/^(\d+)\.\s+(.+)$/gm, '<li value="$1">$2</li>');
  // 将连续的 <li> 包裹在 <ol> 中
  html = html.replace(/((?:<li[^>]*>.*?<\/li>\n?)+)/g, (match) => {
    if (match.includes('value="')) {
      return `<ol>${match}</ol>`;
    }
    return match;
  });

  // 10. 无序列表
  html = html.replace(/^(\s*)[-*]\s+(.+)$/gm, (_m, indent, text) => {
    const depth = indent.length / 2;
    const margin = depth * 20;
    return `<li style="margin-left:${margin}px">${text}</li>`;
  });
  // 将尚未包裹的连续 <li> 包裹在 <ul> 中
  html = html.replace(/((?:<li[^>]*>.*?<\/li>\n?)+)/g, (match) => {
    if (match.includes('value="')) return match; // 有序列表已处理
    return `<ul>${match}</ul>`;
  });

  // 11. 段落：未被 HTML 标签包裹的文本行
  const lines = html.split("\n");
  const result: string[] = [];
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) {
      result.push("");
      continue;
    }
    // 已经是 HTML 标签开头的行跳过
    if (/^<[a-zA-Z/]/.test(trimmed)) {
      result.push(trimmed);
    } else {
      result.push(`<p>${trimmed}</p>`);
    }
  }
  html = result.join("\n");

  // 清理多余空段落
  html = html.replace(/<p>\s*<\/p>/g, "");

  // 12. 还原代码块占位符，并注入复制按钮
  for (const block of codeBlocks) {
    // 提取代码文本用于复制
    const codeMatch = block.html.match(/<code[^>]*>([\s\S]*?)<\/code>/);
    const rawCode = codeMatch ? codeMatch[1] : "";
    const finalHtml = `<div class="code-block-wrapper"><button class="copy-code-btn" data-code="${escapeHtml(rawCode)}"><span class="copy-icon"></span></button>${block.html.slice(block.html.indexOf("<pre>"))}</div>`;
    html = html.replace(block.placeholder, finalHtml);
  }

  return html;
}
</script>

<template>
  <div class="marketplace-page">
    <div class="page-header">
      <h2 class="page-title">{{ t("marketplace.title") }}</h2>
      <p class="page-subtitle">{{ t("marketplace.subtitle") }}</p>
    </div>

    <!-- 搜索区域 -->
    <div class="search-area">
      <div class="search-row">
        <input
          v-model="searchQuery"
          type="text"
          class="search-input"
          :placeholder="t('marketplace.search.placeholder')"
          @keyup.enter="searchGitHub"
        />
        <button class="search-btn" :disabled="searching" @click="searchGitHub">
          {{ searching ? t("marketplace.search.searching") : t("marketplace.search.button") }}
        </button>
      </div>
    </div>

    <!-- 搜索视图 -->
    <template v-if="view === 'search'">
      <div v-if="errorMsg" class="error-banner">{{ errorMsg }}</div>

      <div class="results-area">
        <div class="results-header" v-if="results.length > 0">
          <h3>{{ t("marketplace.results.title") }} ({{ results.length }})</h3>
        </div>

        <div v-if="searching" class="loading-state">
          <div class="loading-spinner"></div>
          <span>{{ t("marketplace.results.loading") }}</span>
        </div>

        <div v-else-if="results.length === 0 && !searching" class="empty-state">
          <div class="empty-icon">
            <span class="empty-box"></span>
          </div>
          <p class="empty-text">{{ t("marketplace.results.empty") }}</p>
        </div>

        <div v-else class="results-grid">
          <div
            v-for="repo in results"
            :key="repo.id"
            class="repo-card"
            @click="openRepo(repo)"
          >
            <div class="repo-header">
              <img
                v-if="repo.owner.avatar_url"
                :src="repo.owner.avatar_url"
                :alt="repo.owner.login"
                class="repo-avatar"
              />
              <div class="repo-meta">
                <span class="repo-name">{{ repo.full_name }}</span>
                <span class="repo-stars">
                  <span class="star-icon"></span>
                  {{ formatStars(repo.stargazers_count) }}
                </span>
              </div>
            </div>
            <p class="repo-desc" v-if="repo.description">{{ repo.description }}</p>
            <div class="repo-footer">
              <span v-if="repo.language" class="repo-lang">
                <span class="lang-dot"></span>
                {{ repo.language }}
              </span>
              <div class="repo-topics" v-if="repo.topics.length > 0">
                <span v-for="topic in repo.topics.slice(0, 3)" :key="topic" class="repo-topic">
                  {{ topic }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- 预览视图 -->
    <template v-if="view === 'preview' && selectedRepo">
      <div class="preview-header">
        <div class="preview-header-top">
          <img
            v-if="selectedRepo.owner.avatar_url"
            :src="selectedRepo.owner.avatar_url"
            :alt="selectedRepo.owner.login"
            class="preview-avatar"
          />
          <div class="preview-info">
            <h2 class="preview-repo-name">{{ selectedRepo.full_name }}</h2>
            <p class="preview-desc" v-if="selectedRepo.description">{{ selectedRepo.description }}</p>
            <div class="preview-meta">
              <span class="preview-stars">
                <span class="star-icon"></span>
                {{ formatStars(selectedRepo.stargazers_count) }}
              </span>
              <span v-if="selectedRepo.language" class="preview-lang">
                <span class="lang-dot"></span>
                {{ selectedRepo.language }}
              </span>
              <span v-for="topic in selectedRepo.topics.slice(0, 5)" :key="topic" class="preview-topic">
                {{ topic }}
              </span>
            </div>
          </div>
        </div>
        <div class="preview-header-actions">
          <button class="preview-btn-back" @click="backToSearch">
            <span class="back-arrow"></span>
            {{ t("marketplace.preview.back") }}
          </button>
          <button class="preview-btn-gh" @click="openExternal(selectedRepo.html_url)">
            {{ t("marketplace.preview.openGithub") }}
          </button>
        </div>
      </div>

      <div class="preview-tabs">
        <button
          :class="['preview-tab', { active: activeTab === 'files' }]"
          @click="activeTab = 'files'"
        >
          {{ t("marketplace.preview.files") }}
        </button>
        <button
          :class="['preview-tab', { active: activeTab === 'readme' }]"
          @click="activeTab = 'readme'"
        >
          {{ t("marketplace.preview.readme") }}
        </button>
      </div>

      <div v-if="activeTab === 'files'" class="preview-content">
        <div v-if="loadingFiles" class="preview-loading">
          {{ t("marketplace.preview.loadingRepo") }}
        </div>
        <div v-else class="file-tree">
          <div
            v-for="item in files"
            :key="item.path"
            class="file-tree-item"
            @click="openExternal(item.html_url)"
          >
            <span :class="['file-icon', item.type]">
              <span class="file-icon-inner"></span>
            </span>
            <span class="file-name">{{ item.name }}</span>
          </div>
          <div v-if="files.length === 0" class="preview-empty">
            {{ t("marketplace.results.empty") }}
          </div>
        </div>
      </div>

      <div v-if="activeTab === 'readme'" class="preview-content">
        <div v-if="loadingReadme" class="preview-loading">
          {{ t("marketplace.preview.loadingReadme") }}
        </div>
        <div v-else-if="readmeContent" ref="readmeBodyRef" class="readme-body" v-html="readmeContent"></div>
        <div v-else class="preview-empty">
          {{ t("marketplace.preview.noReadme") }}
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.marketplace-page {
  padding: var(--mc-space-page-padding);
  max-width: 900px;
}

.page-header {
  margin-bottom: var(--mc-space-section-gap);
}

.page-title {
  font-size: var(--mc-font-page-title);
  font-weight: 700;
  color: var(--mc-text-primary);
  margin: 0 0 4px;
}

.page-subtitle {
  font-size: 13px;
  color: var(--mc-text-muted);
  margin: 0;
}

/* 搜索区域 */
.search-area {
  background: var(--mc-bg-card);
  border: 1px solid var(--mc-border-primary);
  border-radius: var(--mc-radius-md);
  padding: var(--mc-space-card-padding);
  margin-bottom: 20px;
}

.search-row {
  display: flex;
  gap: 8px;
}

.search-input {
  flex: 1;
  padding: 9px 14px;
  font-size: 13px;
  background: var(--mc-chip-bg);
  color: var(--mc-text-primary);
  border: 1px solid var(--mc-border-primary);
  border-radius: var(--mc-radius-sm);
  outline: none;
  font-family: inherit;
  transition: border-color 0.2s ease;
}

.search-input:focus {
  border-color: var(--mc-accent-blue);
}

.search-input::placeholder {
  color: var(--mc-text-dim);
}

.search-btn {
  padding: 9px 18px;
  font-size: 13px;
  font-weight: 600;
  color: var(--mc-text-white);
  background: var(--mc-btn-gradient);
  border: none;
  border-radius: var(--mc-radius-md);
  cursor: pointer;
  font-family: inherit;
  transition: opacity 0.2s ease, box-shadow 0.2s ease;
  white-space: nowrap;
}

.search-btn:hover {
  box-shadow: var(--mc-shadow-btn);
}

.search-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 错误提示 */
.error-banner {
  padding: 10px 16px;
  font-size: 13px;
  color: var(--mc-accent-red);
  background: var(--mc-error-banner-bg);
  border: 1px solid var(--mc-accent-red);
  border-radius: var(--mc-radius-sm);
  margin-bottom: 16px;
}

/* 加载状态 */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 48px 0;
  color: var(--mc-text-muted);
  font-size: 13px;
}

.loading-spinner {
  width: 28px;
  height: 28px;
  border: 3px solid var(--mc-border-primary);
  border-top-color: var(--mc-accent-blue);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 48px 0;
  gap: 12px;
}

.empty-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-box {
  display: block;
  width: 32px;
  height: 32px;
  border: 2px dashed var(--mc-border-primary);
  border-radius: 6px;
}

.empty-text {
  font-size: 13px;
  color: var(--mc-text-muted);
  margin: 0;
}

/* 结果网格 */
.results-header {
  margin-bottom: 14px;
}

.results-header h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--mc-text-primary);
  margin: 0;
}

.results-grid {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.repo-card {
  background: var(--mc-repo-card-bg);
  border: 1px solid var(--mc-border-primary);
  border-radius: var(--mc-radius-md);
  padding: 16px 18px;
  cursor: pointer;
  transition: border-color 0.2s ease, transform 0.15s ease;
}

.repo-card:hover {
  border-color: var(--mc-accent-blue);
  transform: translateY(-1px);
}

.repo-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.repo-avatar {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  flex-shrink: 0;
}

.repo-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.repo-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--mc-repo-name-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.repo-stars {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--mc-text-primary);
  flex-shrink: 0;
}

.star-icon {
  display: inline-block;
  width: 14px;
  height: 14px;
  background: var(--mc-accent-yellow);
  clip-path: polygon(50% 0%, 61% 35%, 98% 35%, 68% 57%, 79% 91%, 50% 70%, 21% 91%, 32% 57%, 2% 35%, 39% 35%);
}

.repo-desc {
  font-size: 12px;
  color: var(--mc-text-muted);
  margin: 0 0 10px;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.repo-footer {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.repo-lang {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--mc-text-muted);
}

.lang-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--mc-accent-blue);
  flex-shrink: 0;
}

.repo-topics {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.repo-topic {
  font-size: 10px;
  padding: 2px 8px;
  color: var(--mc-repo-topic-text);
  background: var(--mc-repo-topic-bg);
  border-radius: 10px;
  font-family: var(--mc-font-mono);
}

/* ========== 预览页 ========== */

.preview-header {
  background: var(--mc-bg-card);
  border: 1px solid var(--mc-border-primary);
  border-radius: var(--mc-radius-md);
  padding: var(--mc-space-card-padding);
  margin-bottom: 16px;
}

.preview-header-top {
  display: flex;
  gap: 14px;
  margin-bottom: 14px;
}

.preview-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  flex-shrink: 0;
  border: 2px solid var(--mc-border-primary);
}

.preview-info {
  flex: 1;
  min-width: 0;
}

.preview-repo-name {
  font-size: 18px;
  font-weight: 700;
  color: var(--mc-repo-name-text);
  margin: 0 0 6px;
  font-family: var(--mc-font-mono);
}

.preview-desc {
  font-size: 13px;
  color: var(--mc-text-muted);
  margin: 0 0 10px;
  line-height: 1.5;
}

.preview-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.preview-stars {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--mc-text-primary);
}

.preview-lang {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--mc-text-muted);
}

.preview-topic {
  font-size: 10px;
  padding: 2px 8px;
  color: var(--mc-repo-topic-text);
  background: var(--mc-repo-topic-bg);
  border-radius: 10px;
  font-family: var(--mc-font-mono);
}

.preview-header-actions {
  display: flex;
  gap: 8px;
}

.preview-btn-back {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 16px;
  font-size: 12px;
  font-weight: 500;
  color: var(--mc-text-primary);
  background: var(--mc-bg-button);
  border: 1px solid var(--mc-border-primary);
  border-radius: var(--mc-radius-sm);
  cursor: pointer;
  font-family: inherit;
  transition: background 0.2s ease, border-color 0.2s ease;
}

.preview-btn-back:hover {
  background: var(--mc-bg-button-hover);
  border-color: var(--mc-accent-blue);
}

/* 返回箭头 - CSS 绘制 */
.back-arrow {
  display: inline-block;
  width: 0;
  height: 0;
  border-top: 5px solid transparent;
  border-bottom: 5px solid transparent;
  border-right: 7px solid currentColor;
}

.preview-btn-gh {
  padding: 7px 16px;
  font-size: 12px;
  font-weight: 500;
  color: var(--mc-text-white);
  background: var(--mc-btn-gradient);
  border: none;
  border-radius: var(--mc-radius-sm);
  cursor: pointer;
  font-family: inherit;
  transition: box-shadow 0.2s ease, opacity 0.2s ease;
}

.preview-btn-gh:hover {
  box-shadow: var(--mc-shadow-btn);
}

/* 标签切换 */
.preview-tabs {
  display: flex;
  gap: 0;
  margin-bottom: 0;
  border-bottom: 1px solid var(--mc-border-primary);
}

.preview-tab {
  padding: 10px 20px;
  font-size: 13px;
  font-weight: 500;
  color: var(--mc-text-muted);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  font-family: inherit;
  transition: color 0.2s ease, border-color 0.2s ease;
}

.preview-tab:hover {
  color: var(--mc-text-primary);
}

.preview-tab.active {
  color: var(--mc-accent-blue);
  border-bottom-color: var(--mc-accent-blue);
}

/* 预览内容区 */
.preview-content {
  background: var(--mc-bg-card);
  border: 1px solid var(--mc-border-primary);
  border-top: none;
  border-radius: 0 0 var(--mc-radius-md) var(--mc-radius-md);
  padding: var(--mc-space-card-padding);
  min-height: 200px;
  max-height: 500px;
  overflow-y: auto;
}

.preview-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 0;
  font-size: 13px;
  color: var(--mc-text-muted);
}

.preview-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 0;
  font-size: 13px;
  color: var(--mc-text-muted);
}

/* 文件树 */
.file-tree {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.file-tree-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: var(--mc-radius-sm);
  cursor: pointer;
  transition: background 0.15s ease;
}

.file-tree-item:hover {
  background: var(--mc-bg-card-hover);
}

.file-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  flex-shrink: 0;
}

.file-icon.dir .file-icon-inner {
  display: block;
  width: 14px;
  height: 10px;
  background: var(--mc-accent-blue);
  border-radius: 2px;
  position: relative;
}

.file-icon.dir .file-icon-inner::before {
  content: "";
  position: absolute;
  top: -2px;
  left: 0;
  width: 7px;
  height: 3px;
  background: var(--mc-accent-blue);
  border-radius: 2px 2px 0 0;
}

.file-icon.file .file-icon-inner {
  display: block;
  width: 12px;
  height: 14px;
  background: var(--mc-text-muted);
  border-radius: 2px;
  position: relative;
}

.file-icon.file .file-icon-inner::after {
  content: "";
  position: absolute;
  top: 0;
  right: 0;
  width: 0;
  height: 0;
  border-style: solid;
  border-width: 0 4px 4px 0;
  border-color: transparent var(--mc-bg-card) transparent transparent;
}

.file-name {
  font-size: 13px;
  color: var(--mc-text-primary);
  font-family: var(--mc-font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* README 渲染内容 */
.readme-body {
  font-size: 14px;
  line-height: 1.7;
  color: var(--mc-text-primary);
}

.readme-body :deep(h1) {
  font-size: 22px;
  font-weight: 700;
  color: var(--mc-text-primary);
  margin: 0 0 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--mc-border-primary);
}

.readme-body :deep(h2) {
  font-size: 18px;
  font-weight: 600;
  color: var(--mc-text-primary);
  margin: 20px 0 10px;
}

.readme-body :deep(h3) {
  font-size: 15px;
  font-weight: 600;
  color: var(--mc-text-primary);
  margin: 16px 0 8px;
}

.readme-body :deep(h4) {
  font-size: 14px;
  font-weight: 600;
  color: var(--mc-text-primary);
  margin: 14px 0 6px;
}

.readme-body :deep(p) {
  margin: 0 0 10px;
}

.readme-body :deep(ul),
.readme-body :deep(ol) {
  margin: 0 0 10px;
  padding-left: 20px;
}

.readme-body :deep(ul) {
  list-style: disc;
}

.readme-body :deep(ol) {
  list-style: decimal;
}

.readme-body :deep(li) {
  margin-bottom: 4px;
}

.readme-body :deep(hr) {
  border: none;
  border-top: 1px solid var(--mc-border-primary);
  margin: 16px 0;
}

.readme-body :deep(blockquote) {
  margin: 0 0 10px;
  padding: 8px 14px;
  border-left: 3px solid var(--mc-accent-blue);
  background: var(--mc-chip-bg);
  border-radius: 0 var(--mc-radius-sm) var(--mc-radius-sm) 0;
  color: var(--mc-text-muted);
}

.readme-body :deep(img) {
  max-width: 100%;
  border-radius: var(--mc-radius-sm);
}

.readme-body :deep(code) {
  font-family: var(--mc-font-mono);
  font-size: 12px;
  padding: 2px 6px;
  background: var(--mc-chip-bg);
  border: 1px solid var(--mc-border-primary);
  border-radius: 3px;
  color: var(--mc-accent-blue);
}

.readme-body :deep(pre) {
  background: var(--mc-chip-bg);
  border: 1px solid var(--mc-border-primary);
  border-radius: var(--mc-radius-sm);
  padding: 12px 16px;
  overflow-x: auto;
  margin: 0;
}

.readme-body :deep(pre code) {
  background: none;
  border: none;
  padding: 0;
  color: var(--mc-text-primary);
  font-size: 12px;
  line-height: 1.6;
}

.readme-body :deep(a) {
  color: var(--mc-accent-blue);
  text-decoration: none;
}

.readme-body :deep(a:hover) {
  text-decoration: underline;
}

.readme-body :deep(strong) {
  font-weight: 600;
  color: var(--mc-text-primary);
}

.readme-body :deep(em) {
  font-style: italic;
}

/* 代码块容器 */
.readme-body :deep(.code-block-wrapper) {
  position: relative;
  margin: 0 0 12px;
}

/* 复制按钮 */
.readme-body :deep(.copy-code-btn) {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  background: var(--mc-bg-card);
  border: 1px solid var(--mc-border-primary);
  border-radius: var(--mc-radius-sm);
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s ease, background 0.2s ease;
  z-index: 1;
}

.readme-body :deep(.code-block-wrapper:hover .copy-code-btn) {
  opacity: 1;
}

.readme-body :deep(.copy-code-btn:hover) {
  background: var(--mc-bg-button-hover);
  border-color: var(--mc-accent-blue);
}

.readme-body :deep(.copy-code-btn.copied) {
  background: var(--mc-accent-green);
  border-color: var(--mc-accent-green);
}

/* 复制图标 - CSS 绘制双文档图标 */
.readme-body :deep(.copy-icon) {
  display: block;
  width: 14px;
  height: 14px;
  position: relative;
}

/* 底层文档 */
.readme-body :deep(.copy-icon)::before {
  content: "";
  position: absolute;
  top: 2px;
  left: 0;
  width: 10px;
  height: 12px;
  border: 1.5px solid var(--mc-text-muted);
  border-radius: 1px;
  background: var(--mc-bg-card);
}

/* 顶层文档（含折角效果） */
.readme-body :deep(.copy-icon)::after {
  content: "";
  position: absolute;
  top: 0;
  right: 0;
  width: 10px;
  height: 12px;
  border: 1.5px solid var(--mc-text-muted);
  border-radius: 1px;
  background:
    linear-gradient(135deg, transparent 4px, var(--mc-text-muted) 4px, var(--mc-text-muted) 5px, transparent 5px) top right / 8px 8px no-repeat,
    var(--mc-bg-card);
}

.readme-body :deep(.copy-code-btn.copied .copy-icon)::before,
.readme-body :deep(.copy-code-btn.copied .copy-icon)::after {
  border-color: #fff;
}

.readme-body :deep(.copy-code-btn.copied .copy-icon)::after {
  background:
    linear-gradient(135deg, transparent 4px, #fff 4px, #fff 5px, transparent 5px) top right / 8px 8px no-repeat,
    var(--mc-accent-green);
}
</style>