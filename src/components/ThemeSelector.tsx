import React, { useState } from "react";
import { ThemeMode, ColorMode } from "../styles/themes";
import { useThemeStore } from "../stores/themeStore";

interface ThemeSelectorProps {
  onSelect?: (themeMode: ThemeMode, colorMode: ColorMode) => void;
  showTitle?: boolean;
}

const ThemeSelector: React.FC<ThemeSelectorProps> = ({ onSelect, showTitle = true }) => {
  const { themeMode, colorMode, setTheme } = useThemeStore();
  const [selectedTheme, setSelectedTheme] = useState<ThemeMode>(themeMode);
  const [selectedColorMode, setSelectedColorMode] = useState<ColorMode>(colorMode);

  const handleSelect = (theme: ThemeMode, mode: ColorMode) => {
    setSelectedTheme(theme);
    setSelectedColorMode(mode);
    setTheme(theme, mode);
    onSelect?.(theme, mode);
  };

  const isSelected = (theme: ThemeMode, mode: ColorMode) => {
    return selectedTheme === theme && selectedColorMode === mode;
  };

  return (
    <div className="space-y-6">
      {showTitle && (
        <div className="text-center">
          <h2 className="text-2xl font-bold text-gray-100 mb-2">Choose Your Visual Experience</h2>
          <p className="text-gray-400 text-sm">
            Select a theme that matches your style. You can change this anytime in settings.
          </p>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Monochromatic Theme */}
        <div className="space-y-3">
          <h3 className="text-lg font-semibold text-gray-200 text-center">Monochromatic</h3>
          <p className="text-xs text-gray-400 text-center mb-3">
            Clean, focused, and distraction-free
          </p>

          {/* Dark Mode */}
          <button
            onClick={() => handleSelect('monochromatic', 'dark')}
            className={`
              w-full p-4 rounded-lg border-2 transition-all duration-200
              ${isSelected('monochromatic', 'dark')
                ? 'border-blue-500 shadow-lg shadow-blue-500/20'
                : 'border-gray-700 hover:border-gray-600'
              }
            `}
          >
            <div className="bg-[#343A40] p-6 rounded-lg">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-8 h-8 rounded-full border-2 border-gray-400"></div>
                <div className="flex-1">
                  <div className="h-3 bg-gray-400 rounded w-20 mb-1"></div>
                  <div className="h-2 bg-gray-600 rounded w-32"></div>
                </div>
              </div>
              <div className="space-y-2">
                <div className="h-8 bg-gray-600 rounded flex items-center px-3">
                  <div className="w-2 h-2 bg-gray-400 rounded-full"></div>
                </div>
                <div className="h-6 bg-gray-700 rounded"></div>
              </div>
              <div className="mt-3 text-xs text-gray-400 text-center">Dark Mode</div>
            </div>
          </button>

          {/* Light Mode */}
          <button
            onClick={() => handleSelect('monochromatic', 'light')}
            className={`
              w-full p-4 rounded-lg border-2 transition-all duration-200
              ${isSelected('monochromatic', 'light')
                ? 'border-blue-500 shadow-lg shadow-blue-500/20'
                : 'border-gray-700 hover:border-gray-600'
              }
            `}
          >
            <div className="bg-[#F8F9FA] p-6 rounded-lg">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-8 h-8 rounded-full border-2 border-gray-700"></div>
                <div className="flex-1">
                  <div className="h-3 bg-gray-800 rounded w-20 mb-1"></div>
                  <div className="h-2 bg-gray-500 rounded w-32"></div>
                </div>
              </div>
              <div className="space-y-2">
                <div className="h-8 bg-white border border-gray-300 rounded flex items-center px-3">
                  <div className="w-2 h-2 bg-gray-500 rounded-full"></div>
                </div>
                <div className="h-6 bg-gray-200 rounded"></div>
              </div>
              <div className="mt-3 text-xs text-gray-600 text-center">Light Mode</div>
            </div>
          </button>
        </div>

        {/* Vibrant Theme */}
        <div className="space-y-3">
          <h3 className="text-lg font-semibold text-gray-200 text-center">Vibrant</h3>
          <p className="text-xs text-gray-400 text-center mb-3">
            Expressive and brand-forward
          </p>

          {/* Dark Mode */}
          <button
            onClick={() => handleSelect('vibrant', 'dark')}
            className={`
              w-full p-4 rounded-lg border-2 transition-all duration-200
              ${isSelected('vibrant', 'dark')
                ? 'border-pink-500 shadow-lg shadow-pink-500/20'
                : 'border-gray-700 hover:border-gray-600'
              }
            `}
          >
            <div className="bg-[#343A40] p-6 rounded-lg">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-8 h-8 rounded-full border-2 border-pink-500"></div>
                <div className="flex-1">
                  <div className="h-3 bg-gray-400 rounded w-20 mb-1"></div>
                  <div className="h-2 bg-gray-600 rounded w-32"></div>
                </div>
              </div>
              <div className="space-y-2">
                <div className="h-8 bg-gray-600 rounded flex items-center px-3">
                  <div className="w-2 h-2 bg-pink-500 rounded-full"></div>
                </div>
                <div className="h-6 bg-pink-600 rounded"></div>
              </div>
              <div className="mt-3 text-xs text-gray-400 text-center">Dark Mode</div>
            </div>
          </button>

          {/* Light Mode */}
          <button
            onClick={() => handleSelect('vibrant', 'light')}
            className={`
              w-full p-4 rounded-lg border-2 transition-all duration-200
              ${isSelected('vibrant', 'light')
                ? 'border-pink-500 shadow-lg shadow-pink-500/20'
                : 'border-gray-700 hover:border-gray-600'
              }
            `}
          >
            <div className="bg-[#F8F9FA] p-6 rounded-lg">
              <div className="flex items-center gap-3 mb-4">
                <div className="w-8 h-8 rounded-full border-2 border-pink-600"></div>
                <div className="flex-1">
                  <div className="h-3 bg-gray-800 rounded w-20 mb-1"></div>
                  <div className="h-2 bg-gray-500 rounded w-32"></div>
                </div>
              </div>
              <div className="space-y-2">
                <div className="h-8 bg-white border border-gray-300 rounded flex items-center px-3">
                  <div className="w-2 h-2 bg-pink-600 rounded-full"></div>
                </div>
                <div className="h-6 bg-pink-600 rounded"></div>
              </div>
              <div className="mt-3 text-xs text-gray-600 text-center">Light Mode</div>
            </div>
          </button>
        </div>
      </div>

      <div className="mt-6 p-4 bg-gray-800/50 rounded-lg border border-gray-700">
        <p className="text-xs text-gray-400 text-center">
          <strong className="text-gray-300">Current selection:</strong> {selectedTheme === 'monochromatic' ? 'Monochromatic' : 'Vibrant'} · {selectedColorMode === 'dark' ? 'Dark Mode' : 'Light Mode'}
        </p>
      </div>
    </div>
  );
};

export default ThemeSelector;
