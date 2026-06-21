import { writable } from 'svelte/store';

export type Theme = 'dark' | 'light';

const THEME_KEY = 'wind-theme';

function createThemeStore() {
  const saved = (typeof window !== 'undefined' ? localStorage.getItem(THEME_KEY) : null) as Theme | null;
  const { subscribe, set } = writable<Theme>(saved || 'dark');

  return {
    subscribe,
    set: (theme: Theme) => {
      localStorage.setItem(THEME_KEY, theme);
      document.documentElement.setAttribute('data-theme', theme);
      set(theme);
    },
    toggle: () => {
      const current = saved || 'dark';
      const next: Theme = current === 'dark' ? 'light' : 'dark';
      localStorage.setItem(THEME_KEY, next);
      document.documentElement.setAttribute('data-theme', next);
      set(next);
    }
  };
}

export const theme = createThemeStore();
