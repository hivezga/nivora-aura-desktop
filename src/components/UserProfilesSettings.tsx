import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { showErrorToast } from "../utils/errorHandler";

// Type definitions matching Rust backend
interface UserProfileWithSpotify {
  id: number;
  name: string;
  enrollment_date: string;
  last_recognized: string | null;
  recognition_count: number;
  is_active: boolean;
  spotify_connected: boolean;
  spotify_display_name: string | null;
  spotify_email: string | null;
  spotify_connected_at: string | null;
}

interface MigrationStatus {
  global_tokens_exist: boolean;
  can_migrate: boolean;
  user_count: number;
}

// Home Assistant shortcuts and preferences types
interface UserHAShortcut {
  id: number;
  user_id: number;
  shortcut_name: string;
  ha_entity_id: string;
  entity_type: string; // "scene" or "script"
  created_at: string;
}

interface UserHAPreferences {
  user_id: number;
  default_room: string | null;
  default_light_entity: string | null;
  default_climate_entity: string | null;
  default_media_player_entity: string | null;
  updated_at: string;
}

interface HAEntity {
  entity_id: string;
  state: string;
  attributes: {
    friendly_name?: string;
    [key: string]: any;
  };
}

interface HAStatus {
  connected: boolean;
  base_url: string;
}

const UserProfilesSettings: React.FC = () => {
  const [userProfiles, setUserProfiles] = useState<UserProfileWithSpotify[]>([]);
  const [migrationStatus, setMigrationStatus] = useState<MigrationStatus | null>(null);
  const [showMigrationBanner, setShowMigrationBanner] = useState(true);
  const [isLoading, setIsLoading] = useState(true);
  const [connectingUserId, setConnectingUserId] = useState<number | null>(null);
  const [disconnectingUserId, setDisconnectingUserId] = useState<number | null>(null);
  const [migratingToUserId, setMigratingToUserId] = useState<number | null>(null);
  const [clientId, setClientId] = useState("");

  // Home Assistant state
  const [haStatus, setHaStatus] = useState<HAStatus | null>(null);
  const [userShortcuts, setUserShortcuts] = useState<Record<number, UserHAShortcut[]>>({});
  const [userPreferences, setUserPreferences] = useState<Record<number, UserHAPreferences>>({});
  const [availableScenes, setAvailableScenes] = useState<HAEntity[]>([]);
  const [availableScripts, setAvailableScripts] = useState<HAEntity[]>([]);
  const [expandedUserId, setExpandedUserId] = useState<number | null>(null);
  const [newShortcutName, setNewShortcutName] = useState("");
  const [selectedEntity, setSelectedEntity] = useState("");
  const [isCreatingShortcut, setIsCreatingShortcut] = useState(false);

  // Load user profiles and migration status on mount
  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setIsLoading(true);
    try {
      // Load user profiles with Spotify status
      const profiles = await invoke<UserProfileWithSpotify[]>("list_user_profiles_with_spotify");
      setUserProfiles(profiles);

      // Check for migration opportunities
      const migration = await invoke<MigrationStatus>("check_global_spotify_migration");
      setMigrationStatus(migration);

      // Load Spotify client ID from settings
      const settings = await invoke<any>("load_settings");
      setClientId(settings.spotify_client_id || "");

      // Load Home Assistant status
      try {
        const haConnected = await invoke<HAStatus>("ha_get_status");
        setHaStatus(haConnected);

        // If HA is connected, load available scenes/scripts
        if (haConnected.connected) {
          const entities = await invoke<HAEntity[]>("ha_get_entities");
          const scenes = entities.filter(e => e.entity_id.startsWith("scene."));
          const scripts = entities.filter(e => e.entity_id.startsWith("script."));
          setAvailableScenes(scenes);
          setAvailableScripts(scripts);

          // Load shortcuts and preferences for all users
          const shortcutsMap: Record<number, UserHAShortcut[]> = {};
          const preferencesMap: Record<number, UserHAPreferences> = {};

          for (const profile of profiles) {
            const shortcuts = await invoke<UserHAShortcut[]>("list_user_ha_shortcuts", { userId: profile.id });
            shortcutsMap[profile.id] = shortcuts;

            const prefs = await invoke<UserHAPreferences | null>("get_user_ha_preferences", { userId: profile.id });
            if (prefs) {
              preferencesMap[profile.id] = prefs;
            }
          }

          setUserShortcuts(shortcutsMap);
          setUserPreferences(preferencesMap);
        }
      } catch (haError) {
        console.warn("Home Assistant not connected or error loading HA data:", haError);
        setHaStatus({ connected: false, base_url: "" });
      }
    } catch (error) {
      console.error("Failed to load user profiles:", error);
      showErrorToast(error, "Failed to load user profiles");
    } finally {
      setIsLoading(false);
    }
  };

  const handleConnectSpotify = async (userId: number) => {
    if (!clientId.trim()) {
      alert("Please configure your Spotify Client ID in the Spotify Settings section first.");
      return;
    }

    setConnectingUserId(userId);
    try {
      await invoke("user_spotify_start_auth", {
        userId: userId,
        clientId: clientId.trim(),
      });

      // Reload user profiles to show updated status
      await loadData();

      alert("Spotify connected successfully!");
    } catch (error) {
      showErrorToast(error, "Failed to connect Spotify");
      console.error("Spotify connection error:", error);
    } finally {
      setConnectingUserId(null);
    }
  };

  const handleDisconnectSpotify = async (userId: number, userName: string) => {
    if (!confirm(`Are you sure you want to disconnect Spotify for ${userName}?`)) {
      return;
    }

    setDisconnectingUserId(userId);
    try {
      await invoke("user_spotify_disconnect", { userId: userId });

      // Reload user profiles to show updated status
      await loadData();

      alert(`Spotify disconnected for ${userName}`);
    } catch (error) {
      showErrorToast(error, "Failed to disconnect Spotify");
    } finally {
      setDisconnectingUserId(null);
    }
  };

  const handleMigrate = async (userId: number, userName: string) => {
    if (!confirm(`Migrate your existing Spotify connection to ${userName}? This will remove the global connection.`)) {
      return;
    }

    setMigratingToUserId(userId);
    try {
      await invoke("migrate_global_spotify_to_user", { userId: userId });

      // Reload data to reflect migration
      await loadData();

      // Hide migration banner
      setShowMigrationBanner(false);

      alert(`Successfully migrated Spotify connection to ${userName}!`);
    } catch (error) {
      showErrorToast(error, "Failed to migrate Spotify connection");
    } finally {
      setMigratingToUserId(null);
    }
  };

  // Home Assistant shortcut handlers
  const handleCreateShortcut = async (userId: number) => {
    if (!newShortcutName.trim() || !selectedEntity) {
      alert("Please enter a shortcut name and select a scene/script");
      return;
    }

    const entityType = selectedEntity.startsWith("scene.") ? "scene" : "script";

    setIsCreatingShortcut(true);
    try {
      await invoke("create_user_ha_shortcut", {
        userId,
        shortcutName: newShortcutName.trim(),
        haEntityId: selectedEntity,
        entityType,
      });

      // Reload shortcuts
      const shortcuts = await invoke<UserHAShortcut[]>("list_user_ha_shortcuts", { userId });
      setUserShortcuts(prev => ({ ...prev, [userId]: shortcuts }));

      // Reset form
      setNewShortcutName("");
      setSelectedEntity("");

      alert(`Shortcut "${newShortcutName}" created successfully!`);
    } catch (error) {
      showErrorToast(error, "Failed to create shortcut");
    } finally {
      setIsCreatingShortcut(false);
    }
  };

  const handleDeleteShortcut = async (userId: number, shortcutId: number, shortcutName: string) => {
    if (!confirm(`Delete shortcut "${shortcutName}"?`)) {
      return;
    }

    try {
      await invoke("delete_user_ha_shortcut", { shortcutId });

      // Reload shortcuts
      const shortcuts = await invoke<UserHAShortcut[]>("list_user_ha_shortcuts", { userId });
      setUserShortcuts(prev => ({ ...prev, [userId]: shortcuts }));

      alert(`Shortcut "${shortcutName}" deleted`);
    } catch (error) {
      showErrorToast(error, "Failed to delete shortcut");
    }
  };

  const handleUpdatePreferences = async (userId: number, field: string, value: string | null) => {
    const currentPrefs = userPreferences[userId] || {
      user_id: userId,
      default_room: null,
      default_light_entity: null,
      default_climate_entity: null,
      default_media_player_entity: null,
      updated_at: new Date().toISOString(),
    };

    const updatedPrefs = { ...currentPrefs, [field]: value || null };

    try {
      await invoke("update_user_ha_preferences", {
        userId,
        defaultRoom: updatedPrefs.default_room,
        defaultLightEntity: updatedPrefs.default_light_entity,
        defaultClimateEntity: updatedPrefs.default_climate_entity,
        defaultMediaPlayerEntity: updatedPrefs.default_media_player_entity,
      });

      setUserPreferences(prev => ({ ...prev, [userId]: updatedPrefs }));
    } catch (error) {
      showErrorToast(error, "Failed to update preferences");
    }
  };

  if (isLoading) {
    return (
      <div className="space-y-4">
        <div className="text-gray-400 text-center py-8">Loading user profiles...</div>
      </div>
    );
  }

  if (userProfiles.length === 0) {
    return (
      <div className="space-y-4">
        <div className="bg-gray-800/50 p-6 rounded-lg border border-gray-700">
          <div className="text-center">
            <p className="text-gray-300 font-medium mb-2">No User Profiles Enrolled</p>
            <p className="text-sm text-gray-500">
              You need to enroll users with voice biometrics before you can connect their Spotify accounts.
            </p>
            <p className="text-xs text-gray-600 mt-3">
              Voice enrollment is currently done via the test enrollment command. Full UI coming soon.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Migration Banner */}
      {migrationStatus?.can_migrate && showMigrationBanner && (
        <div className="bg-gradient-to-r from-yellow-900/30 to-yellow-800/30 border border-yellow-700/50 rounded-lg p-4">
          <div className="flex items-start justify-between">
            <div className="flex-1">
              <h4 className="text-yellow-400 font-semibold mb-2 flex items-center gap-2">
                <span>⚠️</span>
                <span>Migration Available</span>
              </h4>
              <p className="text-sm text-gray-300 mb-3">
                You have a global Spotify connection from before multi-user support.
                Select a user profile to migrate it to:
              </p>
              <div className="flex flex-wrap gap-2">
                {userProfiles.map((profile) => (
                  <Button
                    key={profile.id}
                    onClick={() => handleMigrate(profile.id, profile.name)}
                    disabled={migratingToUserId !== null || profile.spotify_connected}
                    size="sm"
                    className="bg-yellow-600 hover:bg-yellow-700 text-white disabled:opacity-50"
                  >
                    {migratingToUserId === profile.id ? (
                      "Migrating..."
                    ) : profile.spotify_connected ? (
                      `${profile.name} (Already Connected)`
                    ) : (
                      `Migrate to ${profile.name}`
                    )}
                  </Button>
                ))}
              </div>
            </div>
            <button
              onClick={() => setShowMigrationBanner(false)}
              className="text-gray-400 hover:text-gray-200 ml-4"
              title="Dismiss"
            >
              ✕
            </button>
          </div>
        </div>
      )}

      {/* User Profiles List */}
      <div className="space-y-3">
        {userProfiles.map((profile) => (
          <div
            key={profile.id}
            className="bg-gray-800 p-4 rounded-lg border border-gray-700 hover:border-gray-600 transition-colors"
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                {/* User Name */}
                <div className="flex items-center gap-2 mb-2">
                  <span className="text-xl">👤</span>
                  <h4 className="text-lg font-semibold text-gray-100">{profile.name}</h4>
                </div>

                {/* Voice Recognition Status */}
                <div className="flex items-center gap-2 mb-1">
                  <span className="text-xs text-gray-500">Voice Recognition:</span>
                  <span className="text-xs text-green-400 flex items-center gap-1">
                    <span>✓</span>
                    <span>Enrolled</span>
                  </span>
                  <span className="text-xs text-gray-600">
                    ({profile.recognition_count} recognitions)
                  </span>
                </div>

                {/* Spotify Status */}
                {profile.spotify_connected ? (
                  <div className="mt-3 space-y-1">
                    <div className="flex items-center gap-2">
                      <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                      <span className="text-sm font-medium text-green-400">
                        Spotify Connected
                      </span>
                    </div>
                    {profile.spotify_email && (
                      <p className="text-xs text-gray-400 ml-4">
                        {profile.spotify_email}
                      </p>
                    )}
                    {profile.spotify_display_name && (
                      <p className="text-xs text-gray-500 ml-4">
                        Display Name: {profile.spotify_display_name}
                      </p>
                    )}
                    {profile.spotify_connected_at && (
                      <p className="text-xs text-gray-600 ml-4">
                        Connected: {new Date(profile.spotify_connected_at).toLocaleString()}
                      </p>
                    )}
                  </div>
                ) : (
                  <div className="mt-3">
                    <div className="flex items-center gap-2">
                      <span className="text-sm text-gray-400">Spotify:</span>
                      <span className="text-sm text-red-400">✗ Not Connected</span>
                    </div>
                    <p className="text-xs text-gray-500 mt-1">
                      Connect Spotify to use personalized music commands
                    </p>
                  </div>
                )}

                {/* Home Assistant Shortcuts */}
                {haStatus?.connected && (
                  <div className="mt-4 pt-4 border-t border-gray-700">
                    <button
                      onClick={() => setExpandedUserId(expandedUserId === profile.id ? null : profile.id)}
                      className="flex items-center gap-2 text-sm font-medium text-gray-200 hover:text-white transition-colors"
                    >
                      <span>🏠</span>
                      <span>Smart Home Shortcuts</span>
                      <span className="text-xs text-gray-500">
                        ({userShortcuts[profile.id]?.length || 0})
                      </span>
                      <span className="ml-auto text-gray-400">
                        {expandedUserId === profile.id ? "▼" : "▶"}
                      </span>
                    </button>

                    {expandedUserId === profile.id && (
                      <div className="mt-3 space-y-4">
                        {/* Existing Shortcuts */}
                        {userShortcuts[profile.id]?.length > 0 && (
                          <div>
                            <p className="text-xs text-gray-400 mb-2">Your Personal Shortcuts:</p>
                            <div className="space-y-2">
                              {userShortcuts[profile.id].map((shortcut) => (
                                <div
                                  key={shortcut.id}
                                  className="flex items-center justify-between bg-gray-700/50 p-2 rounded border border-gray-600"
                                >
                                  <div className="flex-1">
                                    <p className="text-sm text-gray-200 font-medium">
                                      "{shortcut.shortcut_name}"
                                    </p>
                                    <p className="text-xs text-gray-500">
                                      → {shortcut.ha_entity_id}
                                    </p>
                                  </div>
                                  <button
                                    onClick={() => handleDeleteShortcut(profile.id, shortcut.id, shortcut.shortcut_name)}
                                    className="text-xs text-red-400 hover:text-red-300 px-2 py-1"
                                  >
                                    Delete
                                  </button>
                                </div>
                              ))}
                            </div>
                          </div>
                        )}

                        {/* Create New Shortcut */}
                        <div className="bg-gray-700/30 p-3 rounded border border-gray-600">
                          <p className="text-xs text-gray-400 mb-2 font-medium">Create New Shortcut:</p>
                          <div className="space-y-2">
                            <input
                              type="text"
                              placeholder='Shortcut name (e.g., "morning routine")'
                              value={expandedUserId === profile.id ? newShortcutName : ""}
                              onChange={(e) => setNewShortcutName(e.target.value)}
                              className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            />
                            <select
                              value={expandedUserId === profile.id ? selectedEntity : ""}
                              onChange={(e) => setSelectedEntity(e.target.value)}
                              className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-2 text-sm text-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            >
                              <option value="">Select a scene or script...</option>
                              {availableScenes.length > 0 && (
                                <optgroup label="Scenes">
                                  {availableScenes.map((scene) => (
                                    <option key={scene.entity_id} value={scene.entity_id}>
                                      {scene.attributes.friendly_name || scene.entity_id}
                                    </option>
                                  ))}
                                </optgroup>
                              )}
                              {availableScripts.length > 0 && (
                                <optgroup label="Scripts">
                                  {availableScripts.map((script) => (
                                    <option key={script.entity_id} value={script.entity_id}>
                                      {script.attributes.friendly_name || script.entity_id}
                                    </option>
                                  ))}
                                </optgroup>
                              )}
                            </select>
                            <Button
                              size="sm"
                              onClick={() => handleCreateShortcut(profile.id)}
                              disabled={isCreatingShortcut || !newShortcutName.trim() || !selectedEntity}
                              className="w-full bg-blue-600 hover:bg-blue-700 text-white disabled:opacity-50"
                            >
                              {isCreatingShortcut ? "Creating..." : "Create Shortcut"}
                            </Button>
                          </div>
                          <p className="text-xs text-gray-500 mt-2">
                            💡 You can say "activate my {newShortcutName || "shortcut"}" to trigger it
                          </p>
                        </div>

                        {/* Default Preferences */}
                        <div className="bg-gray-700/30 p-3 rounded border border-gray-600">
                          <p className="text-xs text-gray-400 mb-2 font-medium">Contextual Defaults:</p>
                          <p className="text-xs text-gray-500 mb-3">
                            Set defaults for ambiguous commands like "turn on the lights"
                          </p>
                          <div className="space-y-2">
                            <div>
                              <label className="text-xs text-gray-400 block mb-1">Default Room:</label>
                              <input
                                type="text"
                                placeholder='e.g., "bedroom", "living_room"'
                                value={userPreferences[profile.id]?.default_room || ""}
                                onChange={(e) => handleUpdatePreferences(profile.id, "default_room", e.target.value)}
                                className="w-full bg-gray-800 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                              />
                            </div>
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                )}
              </div>

              {/* Action Button */}
              <div className="ml-4">
                {profile.spotify_connected ? (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleDisconnectSpotify(profile.id, profile.name)}
                    disabled={disconnectingUserId === profile.id}
                    className="text-red-400 border-red-400 hover:bg-red-400 hover:text-white"
                  >
                    {disconnectingUserId === profile.id ? "Disconnecting..." : "Disconnect"}
                  </Button>
                ) : (
                  <Button
                    size="sm"
                    onClick={() => handleConnectSpotify(profile.id)}
                    disabled={connectingUserId === profile.id}
                    className="bg-green-600 hover:bg-green-700 text-white"
                  >
                    {connectingUserId === profile.id ? "Connecting..." : "Connect Spotify"}
                  </Button>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Info Box */}
      <div className="bg-gray-800/50 p-4 rounded-lg border border-gray-700">
        <h4 className="text-sm font-semibold text-gray-200 mb-2">How Personalization Works</h4>
        <div className="space-y-3">
          <div>
            <p className="text-xs font-medium text-gray-300 mb-1">🎵 Spotify:</p>
            <ul className="text-xs text-gray-400 space-y-1">
              <li>• Each enrolled user can connect their own Spotify account</li>
              <li>• Voice commands use the identified speaker's Spotify connection</li>
              <li>• Say "play my playlist" and Aura will use your personal playlists</li>
              <li>• Unknown speakers fall back to the global account (if any)</li>
            </ul>
          </div>
          {haStatus?.connected && (
            <div>
              <p className="text-xs font-medium text-gray-300 mb-1">🏠 Smart Home:</p>
              <ul className="text-xs text-gray-400 space-y-1">
                <li>• Create personal shortcuts for scenes/scripts (e.g., "my morning routine")</li>
                <li>• Set default room for ambiguous commands ("turn on the lights" → your bedroom)</li>
                <li>• Voice commands automatically use your personalized settings</li>
                <li>• Unknown speakers fall back to global Home Assistant control</li>
              </ul>
            </div>
          )}
          <p className="text-xs text-gray-500 italic mt-2">
            All credentials are stored securely in your system keyring. Privacy-first, always.
          </p>
        </div>
      </div>

      {/* Client ID Check */}
      {!clientId && (
        <div className="bg-yellow-900/20 border border-yellow-700/50 rounded-lg p-3">
          <p className="text-xs text-yellow-400">
            ⚠️ No Spotify Client ID configured. Please configure it in the Spotify Settings section above.
          </p>
        </div>
      )}
    </div>
  );
};

export default UserProfilesSettings;
