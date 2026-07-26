import { ref, computed, inject, provide, type InjectionKey } from "vue";
import zhCN, { type LocaleDict } from "./locales/zh-CN";
import enUS from "./locales/en-US";

/** 已注册的语言包 */
const locales: Record<string, LocaleDict> = {
  "zh-CN": zhCN,
  "en-US": enUS,
};

/** 支持的语言列表 */
export const supportedLocales = [
  { key: "zh-CN", label: "中文" },
  { key: "en-US", label: "English" },
] as const;

/** 当前语言 */
const currentLocale = ref("zh-CN");

/** 当前语言字典 */
const dict = computed(() => locales[currentLocale.value] ?? zhCN);

/** 从嵌套对象中取值 */
function getNested(obj: any, path: string): string {
  const keys = path.split(".");
  let current = obj;
  for (const key of keys) {
    if (current == null) return path;
    current = current[key];
  }
  return typeof current === "string" ? current : path;
}

/** 注入 key */
export const I18N_KEY: InjectionKey<(key: string, params?: Record<string, any>) => string> = Symbol("i18n");

/** 提供 i18n 上下文 */
export function provideI18n() {
  const t = (key: string, params?: Record<string, any>): string => {
    let text = getNested(dict.value, key);
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        text = text.replace(`{${k}}`, String(v));
      }
    }
    return text;
  };

  /** 切换语言 */
  function setLocale(locale: string) {
    if (locales[locale]) {
      currentLocale.value = locale;
    }
  }

  provide(I18N_KEY, t);
  return { t, currentLocale, setLocale, locales };
}

/** 在组件中使用 t 函数 */
export function useI18n() {
  const t = inject(I18N_KEY);
  if (!t) {
    // 回退：直接使用字典
    const fallback = (key: string, params?: Record<string, any>): string => {
      let text = getNested(zhCN, key);
      if (params) {
        for (const [k, v] of Object.entries(params)) {
          text = text.replace(`{${k}}`, String(v));
        }
      }
      return text;
    };
    return { t: fallback };
  }
  return { t };
}