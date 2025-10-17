import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { applyTheme, getLogoPath, ThemeMode, ColorMode } from '../styles/themes';

interface ThemeState {
  themeMode: ThemeMode;
  colorMode: ColorMode;
  logoPath: string;

  // Actions
  setTheme: (themeMode: ThemeMode, colorMode: ColorMode) => void;
  setThemeMode: (mode: ThemeMode) => void;
  setColorMode: (mode: ColorMode) => void;
  toggleColorMode: () => void;
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set, get) => ({
      // Default to monochromatic dark
      themeMode: 'monochromatic',
      colorMode: 'dark',
      logoPath: '/logo-aura-mono-dark.svg',

      setTheme: (themeMode: ThemeMode, colorMode: ColorMode) => {
        applyTheme(themeMode, colorMode);
        const logoPath = getLogoPath(themeMode, colorMode);
        set({ themeMode, colorMode, logoPath });
      },

      setThemeMode: (themeMode: ThemeMode) => {
        const { colorMode } = get();
        applyTheme(themeMode, colorMode);
        const logoPath = getLogoPath(themeMode, colorMode);
        set({ themeMode, logoPath });
      },

      setColorMode: (colorMode: ColorMode) => {
        const { themeMode } = get();
        applyTheme(themeMode, colorMode);
        const logoPath = getLogoPath(themeMode, colorMode);
        set({ colorMode, logoPath });
      },

      toggleColorMode: () => {
        const { themeMode, colorMode } = get();
        const newColorMode = colorMode === 'dark' ? 'light' : 'dark';
        applyTheme(themeMode, newColorMode);
        const logoPath = getLogoPath(themeMode, newColorMode);
        set({ colorMode: newColorMode, logoPath });
      },
    }),
    {
      name: 'aura-theme-storage', // localStorage key
    }
  )
);

// Initialize theme on app load
export function initializeTheme() {
  const { themeMode, colorMode } = useThemeStore.getState();
  applyTheme(themeMode, colorMode);
}
