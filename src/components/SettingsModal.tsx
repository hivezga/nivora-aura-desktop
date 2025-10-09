import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useChatStore } from "../store";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { showErrorToast } from "../utils/errorHandler";

const SettingsModal: React.FC = () => {
  const isOpen = useChatStore((state) => state.isSettingsOpen);
  const closeSettings = useChatStore((state) => state.closeSettings);
  const settings = useChatStore((state) => state.settings);
  const setSettings = useChatStore((state) => state.setSettings);

  const [llmProvider, setLlmProvider] = useState(settings.llm_provider);
  const [apiKey, setApiKey] = useState(settings.api_key);
  const [serverAddress, setServerAddress] = useState(settings.server_address);
  const [wakeWordEnabled, setWakeWordEnabled] = useState(settings.wake_word_enabled);
  const [apiBaseUrl, setApiBaseUrl] = useState(settings.api_base_url);
  const [modelName, setModelName] = useState(settings.model_name);
  const [vadSensitivity, setVadSensitivity] = useState(settings.vad_sensitivity);
  const [vadTimeoutMs, setVadTimeoutMs] = useState(settings.vad_timeout_ms);
  const [sttModelName, setSttModelName] = useState(settings.stt_model_name);
  const [voicePreference, setVoicePreference] = useState(settings.voice_preference);
  const [isSaving, setIsSaving] = useState(false);
  const [availableModels, setAvailableModels] = useState<string[]>([]);
  const [isLoadingModels, setIsLoadingModels] = useState(false);

  // Fetch available models when modal opens
  useEffect(() => {
    const fetchModels = async () => {
      if (!isOpen) return;

      setIsLoadingModels(true);
      try {
        const models = await invoke<string[]>("fetch_available_models");
        setAvailableModels(models);
        console.log("Fetched available models:", models);
      } catch (error) {
        console.error("Failed to fetch models:", error);
        // Fallback to current model if fetch fails
        setAvailableModels([settings.model_name]);
      } finally {
        setIsLoadingModels(false);
      }
    };

    fetchModels();
  }, [isOpen, settings.model_name]);

  // Update local state when settings change
  useEffect(() => {
    setLlmProvider(settings.llm_provider);
    setApiKey(settings.api_key);
    setServerAddress(settings.server_address);
    setWakeWordEnabled(settings.wake_word_enabled);
    setApiBaseUrl(settings.api_base_url);
    setModelName(settings.model_name);
    setVadSensitivity(settings.vad_sensitivity);
    setVadTimeoutMs(settings.vad_timeout_ms);
    setSttModelName(settings.stt_model_name);
    setVoicePreference(settings.voice_preference);
  }, [settings]);

  const handleSave = async () => {
    setIsSaving(true);
    try {
      // Save API key to OS keyring if provided (must be done separately)
      if (apiKey.trim()) {
        await invoke("save_api_key", {
          apiKey: apiKey.trim(),
        });
      }

      // Save all settings to database (including voice preference)
      await invoke("save_settings", {
        llmProvider,
        serverAddress,
        wakeWordEnabled,
        apiBaseUrl,
        modelName,
        vadSensitivity,
        vadTimeoutMs,
        sttModelName,
        voicePreference,
      });

      // Reload voice pipeline with new settings (restarts pipeline with new VAD/STT settings)
      await invoke("reload_voice_pipeline", {
        llmProvider,
        serverAddress,
        wakeWordEnabled,
        apiBaseUrl,
        modelName,
        vadSensitivity,
        vadTimeoutMs,
        sttModelName,
      });

      // Update store
      setSettings({
        llm_provider: llmProvider,
        server_address: serverAddress,
        api_key: apiKey,
        wake_word_enabled: wakeWordEnabled,
        api_base_url: apiBaseUrl,
        model_name: modelName,
        vad_sensitivity: vadSensitivity,
        vad_timeout_ms: vadTimeoutMs,
        stt_model_name: sttModelName,
        voice_preference: voicePreference,
      });

      console.log("Settings saved and voice pipeline reloaded successfully");
      alert("Settings applied successfully! Voice changes are now active.");

      closeSettings();
    } catch (error) {
      showErrorToast(error, "Failed to save settings");
      alert(`Failed to save settings: ${error}`);
    } finally {
      setIsSaving(false);
    }
  };

  const handleClose = (open: boolean) => {
    if (!open) {
      // Reset to current settings without saving
      setLlmProvider(settings.llm_provider);
      setApiKey(settings.api_key);
      setServerAddress(settings.server_address);
      setWakeWordEnabled(settings.wake_word_enabled);
      setApiBaseUrl(settings.api_base_url);
      setModelName(settings.model_name);
      setVadSensitivity(settings.vad_sensitivity);
      setVadTimeoutMs(settings.vad_timeout_ms);
      setSttModelName(settings.stt_model_name);
      setVoicePreference(settings.voice_preference);
      closeSettings();
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[500px] bg-gray-900 border-gray-800">
        <DialogHeader>
          <DialogTitle className="text-2xl text-gray-100">Settings</DialogTitle>
          <DialogDescription className="text-gray-400">
            Configure your LLM provider and connection settings
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-5 py-4">
          {/* LLM Provider */}
          <div className="space-y-2">
            <Label htmlFor="llm-provider" className="text-gray-300">
              LLM Provider
            </Label>
            <Select value={llmProvider} onValueChange={setLlmProvider}>
              <SelectTrigger
                id="llm-provider"
                className="w-full bg-gray-800 text-gray-100 border-gray-700 focus:ring-gray-600"
              >
                <SelectValue placeholder="Select provider" />
              </SelectTrigger>
              <SelectContent className="bg-gray-800 border-gray-700">
                <SelectItem
                  value="local"
                  className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                >
                  Local Model (Default)
                </SelectItem>
                <SelectItem
                  value="api"
                  className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                >
                  Third-Party API
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* API Key - only show if Third-Party API is selected */}
          {llmProvider === "api" && (
            <div className="space-y-2">
              <Label htmlFor="api-key" className="text-gray-300">
                API Key
              </Label>
              <Input
                type="password"
                id="api-key"
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="Enter your API key"
                className="bg-gray-800 text-gray-100 border-gray-700 focus:ring-gray-600 placeholder-gray-500"
              />
              <p className="text-xs text-gray-500">
                Stored securely in your system keychain
              </p>
            </div>
          )}

          {/* API Base URL */}
          <div className="space-y-2">
            <Label htmlFor="api-base-url" className="text-gray-300">
              API Base URL
            </Label>
            <Input
              type="text"
              id="api-base-url"
              value={apiBaseUrl}
              onChange={(e) => setApiBaseUrl(e.target.value)}
              placeholder="e.g., http://localhost:11434/v1"
              className="bg-gray-800 text-gray-100 border-gray-700 focus:ring-gray-600 placeholder-gray-500"
            />
            <p className="text-xs text-gray-500">
              Base URL of your OpenAI-compatible API server (Ollama, LM Studio, etc.)
            </p>
          </div>

          {/* Model Name */}
          <div className="space-y-2">
            <Label htmlFor="model-name" className="text-gray-300">
              Model Selection
            </Label>
            <Select value={modelName} onValueChange={setModelName} disabled={isLoadingModels}>
              <SelectTrigger
                id="model-name"
                className="w-full bg-gray-800 text-gray-100 border-gray-700 focus:ring-gray-600"
              >
                <SelectValue placeholder={isLoadingModels ? "Loading models..." : "Select a model"} />
              </SelectTrigger>
              <SelectContent className="bg-gray-800 border-gray-700">
                {availableModels.length > 0 ? (
                  availableModels.map((model) => (
                    <SelectItem
                      key={model}
                      value={model}
                      className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                    >
                      {model}
                    </SelectItem>
                  ))
                ) : (
                  <SelectItem
                    value={modelName}
                    className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                  >
                    {modelName}
                  </SelectItem>
                )}
              </SelectContent>
            </Select>
            <p className="text-xs text-gray-500">
              {availableModels.length > 0
                ? `${availableModels.length} model(s) available on your system`
                : "Select from locally available models"}
            </p>
          </div>

          {/* Divider */}
          <div className="border-t border-gray-800"></div>

          {/* Wake Word Settings */}
          <div className="space-y-4">
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label htmlFor="wake-word-enabled" className="text-gray-300">
                    Enable Wake Word ("Hey Aura")
                  </Label>
                  <p className="text-xs text-gray-500">
                    Activate voice input hands-free with energy-based voice detection (100% offline)
                  </p>
                </div>
                <Switch
                  id="wake-word-enabled"
                  checked={wakeWordEnabled}
                  onCheckedChange={setWakeWordEnabled}
                />
              </div>
              {wakeWordEnabled && (
                <p className="text-xs text-gray-400 mt-2">
                  Wake word detection uses energy-based VAD - no additional models needed.
                  Adjust microphone sensitivity in Voice Settings below if needed.
                </p>
              )}
            </div>
          </div>

          {/* Divider */}
          <div className="border-t border-gray-800"></div>

          {/* Voice Activity Detection Settings */}
          <div className="space-y-4">
            <div>
              <h3 className="text-lg font-semibold text-gray-200 mb-3">Voice Settings</h3>
              <p className="text-xs text-gray-500 mb-4">
                Configure transcription model and fine-tune voice detection settings
              </p>
            </div>

            {/* STT Model Selection */}
            <div className="space-y-2">
              <Label htmlFor="stt-model" className="text-gray-300">
                Transcription Model
              </Label>
              <Select value={sttModelName} onValueChange={setSttModelName}>
                <SelectTrigger
                  id="stt-model"
                  className="w-full bg-gray-800 text-gray-100 border-gray-700 focus:ring-gray-600"
                >
                  <SelectValue placeholder="Select model" />
                </SelectTrigger>
                <SelectContent className="bg-gray-800 border-gray-700">
                  <SelectItem
                    value="ggml-tiny.en.bin"
                    className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                  >
                    Tiny English (Fast, 75 MB)
                  </SelectItem>
                  <SelectItem
                    value="ggml-base.en.bin"
                    className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                  >
                    Base English (Default, 142 MB)
                  </SelectItem>
                  <SelectItem
                    value="ggml-small.en.bin"
                    className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                  >
                    Small English (Better, 466 MB)
                  </SelectItem>
                  <SelectItem
                    value="ggml-medium.en.bin"
                    className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                  >
                    Medium English (Best, 1.5 GB)
                  </SelectItem>
                  <SelectItem
                    value="ggml-tiny.bin"
                    className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                  >
                    Tiny Multilingual (75 MB)
                  </SelectItem>
                  <SelectItem
                    value="ggml-base.bin"
                    className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                  >
                    Base Multilingual (142 MB)
                  </SelectItem>
                  <SelectItem
                    value="ggml-small.bin"
                    className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                  >
                    Small Multilingual (466 MB)
                  </SelectItem>
                  <SelectItem
                    value="ggml-medium.bin"
                    className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                  >
                    Medium Multilingual (1.5 GB)
                  </SelectItem>
                </SelectContent>
              </Select>
              <p className="text-xs text-gray-500">
                Download your chosen model and place it in the models directory. Restart required after changing.
              </p>
            </div>

            {/* TTS Voice Selection */}
            <div className="space-y-2">
              <Label htmlFor="voice-preference" className="text-gray-300">
                Voice Preference
              </Label>
              <Select value={voicePreference} onValueChange={setVoicePreference}>
                <SelectTrigger
                  id="voice-preference"
                  className="w-full bg-gray-800 text-gray-100 border-gray-700 focus:ring-gray-600"
                >
                  <SelectValue placeholder="Select voice" />
                </SelectTrigger>
                <SelectContent className="bg-gray-800 border-gray-700">
                  <SelectItem
                    value="male"
                    className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                  >
                    Male Voice (Lessac)
                  </SelectItem>
                  <SelectItem
                    value="female"
                    className="text-gray-100 focus:bg-gray-700 focus:text-gray-100"
                  >
                    Female Voice (Amy)
                  </SelectItem>
                </SelectContent>
              </Select>
              <p className="text-xs text-gray-500">
                Select the voice for text-to-speech output. Changes apply after clicking Apply.
              </p>
            </div>

            {/* Microphone Sensitivity Slider */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="vad-sensitivity" className="text-gray-300">
                  Microphone Sensitivity
                </Label>
                <span className="text-sm text-gray-400">{(vadSensitivity * 100).toFixed(1)}%</span>
              </div>
              <input
                type="range"
                id="vad-sensitivity"
                min="0.005"
                max="0.15"
                step="0.001"
                value={vadSensitivity}
                onChange={(e) => setVadSensitivity(parseFloat(e.target.value))}
                className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-gray-600"
              />
              <p className="text-xs text-gray-500">
                Lower values = more sensitive (picks up quieter speech). Default: 2.0%
              </p>
            </div>

            {/* Silence Timeout Slider */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="vad-timeout" className="text-gray-300">
                  Silence Timeout
                </Label>
                <span className="text-sm text-gray-400">{(vadTimeoutMs / 1000).toFixed(2)}s</span>
              </div>
              <input
                type="range"
                id="vad-timeout"
                min="500"
                max="3000"
                step="100"
                value={vadTimeoutMs}
                onChange={(e) => setVadTimeoutMs(parseInt(e.target.value))}
                className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-gray-600"
              />
              <p className="text-xs text-gray-500">
                How long to wait for silence before ending recording. Default: 1.28s
              </p>
            </div>
          </div>
        </div>

        <DialogFooter className="gap-2">
          <Button
            variant="outline"
            onClick={() => handleClose(false)}
            className="bg-gray-800 hover:bg-gray-700 text-gray-200 border-gray-700"
          >
            Cancel
          </Button>
          <Button
            onClick={handleSave}
            disabled={isSaving}
            className="bg-gray-700 hover:bg-gray-600 text-gray-100"
          >
            {isSaving ? "Saving..." : "Save"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default SettingsModal;
