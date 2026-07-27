<script setup lang="ts">
import { ref, inject } from "vue";
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

const searchQuery = ref("");
const topicFilter = ref("mcp-server");
const searching = ref(false);
const results = ref<RepoResult[]>([]);
const errorMsg = ref("");

/** 搜索 GitHub 话题仓库 */
async function searchGitHub() {
  errorMsg.value = "";
  results.value = [];

  const query = searchQuery.value.trim();
  const topic = topicFilter.value;

  if (!query && !topic) {
    errorMsg.value = "Please enter a search term or select a topic";
    return;
  }

  searching.value = true;
  try {
    let url: string;
    if (query) {
      url = `https://api.github.com/search/repositories?q=${encodeURIComponent(query)}+topic:${topic}&sort=stars&order=desc&per_page=20`;
    } else {
      url = `https://api.github.com/search/repositories?q=topic:${topic}&sort=stars&order=desc&per_page=20`;
    }

    const resp = await fetch(url, {
      headers: {
        Accept: "application/vnd.github.v3+json",
      },
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

/** 打开仓库链接 */
function openRepo(url: string) {
  window.open(url, "_blank");
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
        <div class="topic-select">
          <label class="topic-label">{{ t("marketplace.search.label") }}</label>
          <div class="topic-chips">
            <button
              :class="['topic-chip', { active: topicFilter === 'mcp-server' }]"
              @click="topicFilter = 'mcp-server'"
            >
              {{ t("marketplace.search.topicMCP") }}
            </button>
            <button
              :class="['topic-chip', { active: topicFilter === 'modelcontextprotocol' }]"
              @click="topicFilter = 'modelcontextprotocol'"
            >
              {{ t("marketplace.search.topicMCPAlt") }}
            </button>
          </div>
        </div>
      </div>
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

    <!-- 错误信息 -->
    <div v-if="errorMsg" class="error-banner">{{ errorMsg }}</div>

    <!-- 搜索结果 -->
    <div class="results-area">
      <div class="results-header" v-if="results.length > 0">
        <h3>{{ t("marketplace.results.title") }} ({{ results.length }})</h3>
      </div>

      <!-- 加载中 -->
      <div v-if="searching" class="loading-state">
        <div class="loading-spinner"></div>
        <span>{{ t("marketplace.results.loading") }}</span>
      </div>

      <!-- 空结果 -->
      <div v-else-if="results.length === 0 && !searching" class="empty-state">
        <div class="empty-icon">
          <span class="empty-box"></span>
        </div>
        <p class="empty-text">{{ t("marketplace.results.empty") }}</p>
      </div>

      <!-- 结果列表 -->
      <div v-else class="results-grid">
        <div
          v-for="repo in results"
          :key="repo.id"
          class="repo-card"
          @click="openRepo(repo.html_url)"
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
  </div>
</template>

<style scoped>
.marketplace-page {
  padding: 24px 28px;
  max-width: 900px;
}

.page-header {
  margin-bottom: 24px;
}

.page-title {
  font-size: 20px;
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
  border-radius: 10px;
  padding: 20px 22px;
  margin-bottom: 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.search-row {
  display: flex;
  gap: 8px;
}

.topic-select {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.topic-label {
  font-size: 11px;
  color: var(--mc-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 500;
}

.topic-chips {
  display: flex;
  gap: 8px;
}

.topic-chip {
  padding: 5px 14px;
  font-size: 12px;
  font-weight: 500;
  color: var(--mc-text-muted);
  background: var(--mc-chip-bg);
  border: 1px solid var(--mc-border-primary);
  border-radius: 20px;
  cursor: pointer;
  font-family: "JetBrains Mono", "Fira Code", "Consolas", monospace;
  transition: color 0.2s ease, border-color 0.2s ease, background 0.2s ease;
}

.topic-chip:hover {
  color: var(--mc-text-primary);
  border-color: var(--mc-accent-blue);
}

.topic-chip.active {
  color: var(--mc-chip-active-text);
  border-color: var(--mc-accent-blue);
  background: var(--mc-chip-active-bg);
}

.search-input {
  flex: 1;
  padding: 9px 14px;
  font-size: 13px;
  background: var(--mc-chip-bg);
  color: var(--mc-text-primary);
  border: 1px solid var(--mc-border-primary);
  border-radius: 6px;
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
  border-radius: 6px;
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
  border-radius: 6px;
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
  to {
    transform: rotate(360deg);
  }
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
  border-radius: 8px;
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

/* 星形图标 - CSS 绘制 */
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
  font-family: "JetBrains Mono", "Fira Code", "Consolas", monospace;
}
</style>