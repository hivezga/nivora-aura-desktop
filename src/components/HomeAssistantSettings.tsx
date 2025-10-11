import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { showErrorToast } from "../utils/errorHandler";

interface HAStatus {
  connected: boolean;
  base_url: string;
  entity_count: number;
}

const HomeAssistantSettings: React.FC = () => {
  const [haStatus, setHAStatus] = useState<HAStatus>({
    connected: false,
    base_url: "",
    entity_count: 0,
  });
  const [baseUrl, setBaseUrl] = useState("");
  const [accessToken, setAccessToken] = useState("");
  const [isConnecting, setIsConnecting] = useState(false);
  const [isDisconnecting, setIsDisconnecting] = useState(false);

  // Load Home Assistant status on mount
  useEffect(() => {
    loadHAStatus();
  }, []);

  const loadHAStatus = async () => {
    try {
      const status = await invoke<HAStatus>("ha_get_status");
      setHAStatus(status);
      setBaseUrl(status.base_url);
    } catch (error) {
      console.error("Failed to load Home Assistant status:", error);
    }
  };

  const handleConnect = async () => {
    if (!baseUrl.trim()) {
      alert("Please enter your Home Assistant URL");
      return;
    }

    if (!accessToken.trim()) {
      alert("Please enter your Home Assistant Access Token");
      return;
    }

    setIsConnecting(true);
    try {
      // Connect to Home Assistant
      await invoke("ha_connect", {
        baseUrl: baseUrl.trim(),
        token: accessToken.trim(),
      });

      // Reload status
      await loadHAStatus();

      // Clear token input for security
      setAccessToken("");

      alert(
        "Home Assistant connected successfully! You can now control your smart home with voice commands."
      );
    } catch (error) {
      showErrorToast(error, "Failed to connect to Home Assistant");
      console.error("Home Assistant connection error:", error);
    } finally {
      setIsConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    if (!confirm("Are you sure you want to disconnect Home Assistant?")) {
      return;
    }

    setIsDisconnecting(true);
    try {
      await invoke("ha_disconnect");
      await loadHAStatus();
      alert("Home Assistant disconnected successfully");
    } catch (error) {
      showErrorToast(error, "Failed to disconnect Home Assistant");
    } finally {
      setIsDisconnecting(false);
    }
  };

  return (
    <div className="space-y-4">
      {!haStatus.connected ? (
        <>
          {/* Disconnected State */}
          <div className="space-y-3">
            <div className="space-y-2">
              <Label htmlFor="ha-base-url" className="text-gray-300">
                Home Assistant URL
              </Label>
              <Input
                type="text"
                id="ha-base-url"
                value={baseUrl}
                onChange={(e) => setBaseUrl(e.target.value)}
                placeholder="http://homeassistant.local:8123"
                className="bg-gray-800 text-gray-100 border-gray-700 focus:ring-gray-600 placeholder-gray-500 font-mono text-sm"
              />
              <div className="text-xs text-gray-500">
                The URL of your Home Assistant instance (e.g., http://192.168.1.100:8123)
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="ha-access-token" className="text-gray-300">
                Long-Lived Access Token
              </Label>
              <Input
                type="password"
                id="ha-access-token"
                value={accessToken}
                onChange={(e) => setAccessToken(e.target.value)}
                placeholder="Enter your access token"
                className="bg-gray-800 text-gray-100 border-gray-700 focus:ring-gray-600 placeholder-gray-500 font-mono text-sm"
              />
              <div className="text-xs text-gray-500 space-y-1">
                <p>
                  <strong>How to create an access token:</strong>
                </p>
                <ol className="list-decimal list-inside space-y-1 ml-2">
                  <li>Open your Home Assistant web interface</li>
                  <li>Click on your profile (bottom left)</li>
                  <li>
                    Scroll down to "Long-Lived Access Tokens"
                  </li>
                  <li>Click "Create Token"</li>
                  <li>Give it a name (e.g., "Nivora Aura")</li>
                  <li>Copy the token and paste it here</li>
                </ol>
              </div>
            </div>

            <Button
              onClick={handleConnect}
              disabled={isConnecting}
              className="w-full bg-blue-600 hover:bg-blue-700 text-white"
            >
              {isConnecting ? (
                <span className="flex items-center justify-center">
                  <svg
                    className="animate-spin -ml-1 mr-2 h-4 w-4 text-white"
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                    ></circle>
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    ></path>
                  </svg>
                  Connecting...
                </span>
              ) : (
                "Connect Home Assistant"
              )}
            </Button>
          </div>
        </>
      ) : (
        <>
          {/* Connected State */}
          <div className="space-y-3">
            <div className="rounded-md bg-green-900/20 border border-green-700 p-3">
              <div className="flex items-start">
                <svg
                  className="h-5 w-5 text-green-400 mt-0.5"
                  fill="none"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path d="M5 13l4 4L19 7"></path>
                </svg>
                <div className="ml-3 flex-1">
                  <h3 className="text-sm font-medium text-green-400">
                    Connected to Home Assistant
                  </h3>
                  <div className="mt-2 text-xs text-green-300 space-y-1">
                    <p>
                      <span className="text-gray-400">URL:</span>{" "}
                      <span className="font-mono">{haStatus.base_url}</span>
                    </p>
                    <p>
                      <span className="text-gray-400">Entities discovered:</span>{" "}
                      <span className="font-semibold">{haStatus.entity_count}</span>
                    </p>
                  </div>
                </div>
              </div>
            </div>

            <div className="rounded-md bg-gray-800 border border-gray-700 p-3">
              <h4 className="text-sm font-medium text-gray-300 mb-2">
                Voice Commands
              </h4>
              <div className="text-xs text-gray-400 space-y-1">
                <p className="font-mono">"Turn on the kitchen lights"</p>
                <p className="font-mono">"Set bedroom to 72 degrees"</p>
                <p className="font-mono">"Dim the living room lights to 50%"</p>
                <p className="font-mono">"What's the temperature in the office?"</p>
                <p className="font-mono">"Open the garage door"</p>
              </div>
            </div>

            <Button
              onClick={handleDisconnect}
              disabled={isDisconnecting}
              variant="destructive"
              className="w-full"
            >
              {isDisconnecting ? "Disconnecting..." : "Disconnect"}
            </Button>
          </div>
        </>
      )}

      {/* Privacy Notice */}
      <div className="rounded-md bg-gray-800 border border-gray-700 p-3 text-xs text-gray-400">
        <p className="font-medium text-gray-300 mb-1">Privacy & Security</p>
        <ul className="space-y-1 list-disc list-inside">
          <li>Your access token is stored securely in your system's keyring</li>
          <li>All communication is direct to your local Home Assistant instance</li>
          <li>No data is sent to any cloud service</li>
          <li>Voice processing happens 100% locally on your device</li>
        </ul>
      </div>
    </div>
  );
};

export default HomeAssistantSettings;
