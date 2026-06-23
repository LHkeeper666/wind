import { writable } from 'svelte/store';

export type Theme = 'dark' | 'light';

const THEME_KEY = 'wind-theme';

function createThemeStore() {
  const saved = (typeof window !== 'undefined' ? localStorage.getItem(THEME_KEY) : null) as Theme | null;
  const { subscribe, set, update } = writable<Theme>(saved || 'dark');

  return {
    subscribe,
    set: (theme: Theme) => {
      localStorage.setItem(THEME_KEY, theme);
      document.documentElement.setAttribute('data-theme', theme);
      set(theme);
    },
    toggle: () => {
      update(current => {
        const next: Theme = current === 'dark' ? 'light' : 'dark';
        localStorage.setItem(THEME_KEY, next);
        document.documentElement.setAttribute('data-theme', next);
        return next;
      });
    }
  };
}

export const theme = createThemeStore();
