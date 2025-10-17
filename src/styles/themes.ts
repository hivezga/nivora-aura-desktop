/**
 * Aura Theme System
 *
 * Defines color palettes for "Monochromatic" and "Vibrant" themes
 * Each theme supports both Dark and Light modes
 */

export type ThemeMode = 'monochromatic' | 'vibrant';
export type ColorMode = 'dark' | 'light';

export interface ThemeColors {
  backgroundPrimary: string;
  textPrimary: string;
  textSecondary: string;
  accentPrimary: string;
  activeIndicator: string;
  inputBackground: string;
}

export const themes: Record<ThemeMode, Record<ColorMode, ThemeColors>> = {
  monochromatic: {
    dark: {
      backgroundPrimary: '#343A40',
      textPrimary: '#F5F5F5',
      textSecondary: '#AAAAAA',
      accentPrimary: '#6C757D',
      activeIndicator: '#808080',
      inputBackground: '#495057',
    },
    light: {
      backgroundPrimary: '#F8F9FA',
      textPrimary: '#212529',
      textSecondary: '#6C757D',
      accentPrimary: '#6C757D',
      activeIndicator: '#AAAAAA',
      inputBackground: '#FFFFFF',
    },
  },
  vibrant: {
    dark: {
      backgroundPrimary: '#343A40',
      textPrimary: '#F5F5F5',
      textSecondary: '#AAAAAA',
      accentPrimary: '#D81B60',
      activeIndicator: '#E91E63',
      inputBackground: '#495057',
    },
    light: {
      backgroundPrimary: '#F8F9FA',
      textPrimary: '#212529',
      textSecondary: '#6C757D',
      accentPrimary: '#D81B60',
      activeIndicator: '#E91E63',
      inputBackground: '#FFFFFF',
    },
  },
};

/**
 * Apply theme to document root using CSS variables
 */
export function applyTheme(themeMode: ThemeMode, colorMode: ColorMode) {
  const colors = themes[themeMode][colorMode];
  const root = document.documentElement;

  root.style.setProperty('--background-primary', colors.backgroundPrimary);
  root.style.setProperty('--text-primary', colors.textPrimary);
  root.style.setProperty('--text-secondary', colors.textSecondary);
  root.style.setProperty('--accent-primary', colors.accentPrimary);
  root.style.setProperty('--active-indicator', colors.activeIndicator);
  root.style.setProperty('--input-background', colors.inputBackground);

  // Set data attributes for theme and color mode
  root.setAttribute('data-theme', themeMode);
  root.setAttribute('data-color-mode', colorMode);
}

/**
 * Get logo asset path based on theme
 */
export function getLogoPath(themeMode: ThemeMode, colorMode: ColorMode): string {
  if (themeMode === 'vibrant') {
    return '/logo-aura-vibrant.svg';
  }

  // Monochromatic theme
  return colorMode === 'dark' ? '/logo-aura-mono-dark.svg' : '/logo-aura-mono-light.svg';
}
