import { ref } from "vue";
import type { Theme } from "./types";
import { darkTheme } from "./dark";
import { lightTheme } from "./light";

export type { Theme } from "./types";

/** 已注册的主题，扩展新主题只需在此添加 */
const themeRegistry: Record<string, Theme> = {
  dark: darkTheme,
  light: lightTheme,
};

/** 默认主题 */
const DEFAULT_THEME = "dark";

/** 从 localStorage 读取持久化的主题 */
function loadThemeName(): string {
  try {
    const stored = localStorage.getItem("mcpstudio-theme");
    if (stored && themeRegistry[stored]) return stored;
  } catch {
    // localStorage 不可用时忽略
  }
  return DEFAULT_THEME;
}

/** 持久化主题名 */
function saveThemeName(name: string) {
  try {
    localStorage.setItem("mcpstudio-theme", name);
  } catch {
    // 忽略
  }
}

/** 当前主题名称 */
const currentThemeName = ref(loadThemeName());

/** 将 camelCase 转为 kebab-case */
function toCssVar(key: string): string {
  return "--mc-" + key.replace(/([A-Z])/g, "-$1").toLowerCase();
}

/** 将主题颜色应用到 document.documentElement */
function applyTheme(theme: Theme) {
  const root = document.documentElement;
  for (const [key, value] of Object.entries(theme.colors)) {
    root.style.setProperty(toCssVar(key), value);
  }
}

/** 初始化主题 */
export function initTheme() {
  applyTheme(themeRegistry[currentThemeName.value]);
}

/** 切换主题 */
export function setTheme(name: string) {
  const theme = themeRegistry[name];
  if (!theme) return;
  currentThemeName.value = name;
  saveThemeName(name);
  applyTheme(theme);
}

/** 获取已注册的主题列表 */
export function getThemes(): Theme[] {
  return Object.values(themeRegistry);
}

/** 注册新主题（第三方扩展入口） */
export function registerTheme(theme: Theme) {
  themeRegistry[theme.name] = theme;
}

/** 主题 composable */
export function useTheme() {
  return {
    currentThemeName,
    setTheme,
    getThemes,
  };
}