import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { showErrorToast } from "../utils/errorHandler";

interface SpotifyStatus {
  connected: boolean;
  client_id: string;
  auto_play_enabled: boolean;
}

const SpotifySettings: React.FC = () => {
  const [spotifyStatus, setSpotifyStatus] = useState<SpotifyStatus>({
    connected: false,
    client_id: "",
    auto_play_enabled: true,
  });
  const [clientId, setClientId] = useState("");
  const [isConnecting, setIsConnecting] = useState(false);
  const [isDisconnecting, setIsDisconnecting] = useState(false);

  // Load Spotify status on mount
  useEffect(() => {
    loadSpotifyStatus();
  }, []);

  const loadSpotifyStatus = async () => {
    try {
      const status = await invoke<SpotifyStatus>("spotify_get_status");
      setSpotifyStatus(status);
      setClientId(status.client_id);
    } catch (error) {
      console.error("Failed to load Spotify status:", error);
    }
  };

  const handleConnect = async () => {
    if (!clientId.trim()) {
      alert("Please enter your Spotify Client ID");
      return;
    }

    setIsConnecting(true);
    try {
      // Save client ID first
      await invoke("spotify_save_client_id", { clientId: clientId.trim() });

      // Start OAuth flow (will open browser)
      await invoke("spotify_start_auth", { clientId: clientId.trim() });

      // Reload status
      await loadSpotifyStatus();

      alert("Spotify connected successfully! You can now use voice commands to control your music.");
    } catch (error) {
      showErrorToast(error, "Failed to connect Spotify");
      console.error("Spotify connection error:", error);
    } finally {
      setIsConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    if (!confirm("Are you sure you want to disconnect Spotify?")) {
      return;
    }

    setIsDisconnecting(true);
    try {
      await invoke("spotify_disconnect");
      await loadSpotifyStatus();
      alert("Spotify disconnected successfully");
    } catch (error) {
      showErrorToast(error, "Failed to disconnect Spotify");
    } finally {
      setIsDisconnecting(false);
    }
  };

  return (
    <div className="space-y-4">
      {!spotifyStatus.connected ? (
        <>
          {/* Disconnected State */}
          <div className="space-y-3">
            <div className="space-y-2">
              <Label htmlFor="spotify-client-id" className="text-gray-300">
                Spotify Client ID
              </Label>
              <Input
                type="text"
                id="spotify-client-id"
                value={clientId}
                onChange={(e) => setClientId(e.target.value)}
                placeholder="Enter your Spotify app client ID"
                className="bg-gray-800 text-gray-100 border-gray-700 focus:ring-gray-600 placeholder-gray-500 font-mono text-sm"
              />
              <div className="text-xs text-gray-500 space-y-1">
                <p>
                  1. Create a Spotify app at{" "}
                  <a
                    href="https://developer.spotify.com/dashboard"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-green-400 hover:underline"
                  >
                    developer.spotify.com/dashboard
                  </a>
                </p>
                <p>
                  2. Set the redirect URI to:{" "}
                  <code className="bg-gray-800 px-1 py-0.5 rounded text-green-400">
                    http://127.0.0.1:8888/callback
                  </code>
                </p>
                <p>3. Copy the <strong>Client ID</strong> and paste it above</p>
              </div>
            </div>

            <Button
              onClick={handleConnect}
              disabled={!clientId.trim() || isConnecting}
              className="w-full bg-green-600 hover:bg-green-700 text-white font-semibold"
            >
              {isConnecting ? "Connecting..." : "Connect Spotify"}
            </Button>

            {isConnecting && (
              <p className="text-xs text-gray-400 text-center">
                Your browser will open for authorization. Please allow access and return here.
              </p>
            )}
          </div>
        </>
      ) : (
        <>
          {/* Connected State */}
          <div className="space-y-3">
            <div className="flex items-center justify-between bg-gray-800 p-4 rounded-lg border border-gray-700">
              <div className="flex items-center gap-3">
                <div className="w-3 h-3 bg-green-500 rounded-full animate-pulse"></div>
                <div>
                  <p className="text-gray-100 font-medium">Connected to Spotify</p>
                  <p className="text-xs text-gray-400 mt-0.5 font-mono">
                    Client ID: {spotifyStatus.client_id.substring(0, 12)}...
                  </p>
                </div>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={handleDisconnect}
                disabled={isDisconnecting}
                className="text-red-400 border-red-400 hover:bg-red-400 hover:text-white"
              >
                {isDisconnecting ? "Disconnecting..." : "Disconnect"}
              </Button>
            </div>

            <div className="bg-gray-800/50 p-4 rounded-lg space-y-2">
              <h4 className="text-sm font-semibold text-gray-200">Voice Commands</h4>
              <ul className="text-xs text-gray-400 space-y-1">
                <li>• "Play [song] by [artist]" - Play a specific track</li>
                <li>• "Play my [playlist] playlist" - Play your playlist</li>
                <li>• "Pause" / "Resume" - Control playback</li>
                <li>• "Next" / "Previous" - Skip tracks</li>
                <li>• "What's playing?" - Get current track info</li>
              </ul>
            </div>
          </div>
        </>
      )}
    </div>
  );
};

export default SpotifySettings;
