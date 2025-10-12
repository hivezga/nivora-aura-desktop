/**
 * UserProfilesSettings Component
 *
 * Main settings panel for managing voice biometric user profiles.
 * Displays enrolled users, their statistics, and provides enrollment UI.
 */

import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { showErrorToast } from "../utils/errorHandler";
import EnrollmentModal from "./EnrollmentModal";

export interface UserProfile {
  id: number;
  name: string;
  enrollment_date: string;
  last_recognized: string | null;
  recognition_count: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

interface BiometricsStatus {
  enrolled_user_count: number;
  is_enabled: boolean;
}

const UserProfilesSettings: React.FC = () => {
  const [profiles, setProfiles] = useState<UserProfile[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [showEnrollmentModal, setShowEnrollmentModal] = useState(false);
  const [status, setStatus] = useState<BiometricsStatus>({
    enrolled_user_count: 0,
    is_enabled: true,
  });

  // Load profiles on mount
  useEffect(() => {
    loadProfiles();
    loadStatus();
  }, []);

  const loadProfiles = async () => {
    setIsLoading(true);
    try {
      const userProfiles = await invoke<UserProfile[]>("biometrics_list_users");
      setProfiles(userProfiles);
      console.log("✓ Loaded voice profiles:", userProfiles);
    } catch (error) {
      console.error("Failed to load voice profiles:", error);
      showErrorToast(error, "Failed to load voice profiles");
    } finally {
      setIsLoading(false);
    }
  };

  const loadStatus = async () => {
    try {
      const bioStatus = await invoke<BiometricsStatus>("biometrics_get_status");
      setStatus(bioStatus);
    } catch (error) {
      console.error("Failed to load biometrics status:", error);
    }
  };

  const handleDeleteProfile = async (profileId: number, profileName: string) => {
    if (!confirm(`Are you sure you want to delete the voice profile for "${profileName}"? This action cannot be undone.`)) {
      return;
    }

    try {
      await invoke("biometrics_delete_user", { userId: profileId });
      console.log(`✓ Deleted voice profile: ${profileName}`);
      alert(`Voice profile for "${profileName}" has been deleted.`);
      await loadProfiles();
      await loadStatus();
    } catch (error) {
      showErrorToast(error, `Failed to delete profile for ${profileName}`);
    }
  };

  const handleEnrollmentComplete = async () => {
    setShowEnrollmentModal(false);
    await loadProfiles();
    await loadStatus();
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "Today";
    if (diffDays === 1) return "Yesterday";
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
    return `${Math.floor(diffDays / 365)} years ago`;
  };

  return (
    <>
      <div className="space-y-4">
        {/* Header with stats */}
        <div className="bg-gray-800 p-4 rounded-lg border border-gray-700">
          <div className="flex items-center justify-between mb-2">
            <h4 className="text-sm font-medium text-gray-300">Voice Biometrics</h4>
            <span className="text-xs bg-purple-900 text-purple-200 px-2 py-1 rounded">
              {status.enrolled_user_count} {status.enrolled_user_count === 1 ? "user" : "users"} enrolled
            </span>
          </div>
          <p className="text-xs text-gray-500">
            🔒 All voice data stored locally and securely • Privacy-first design
          </p>
        </div>

        {/* Loading state */}
        {isLoading && (
          <div className="text-center py-6 text-gray-400">
            <div className="animate-pulse">Loading profiles...</div>
          </div>
        )}

        {/* Empty state */}
        {!isLoading && profiles.length === 0 && (
          <div className="bg-gray-800 p-6 rounded-lg border border-gray-700 text-center">
            <div className="text-4xl mb-3">🎤</div>
            <h4 className="text-lg font-medium text-gray-200 mb-2">No Users Enrolled</h4>
            <p className="text-sm text-gray-400 mb-4">
              Enroll your voice to enable personalized responses and automatic user recognition.
            </p>
            <Button
              onClick={() => setShowEnrollmentModal(true)}
              className="bg-purple-700 hover:bg-purple-600 text-white"
            >
              + Enroll Your Voice
            </Button>
          </div>
        )}

        {/* Profile list */}
        {!isLoading && profiles.length > 0 && (
          <div className="space-y-3">
            {profiles.map((profile) => (
              <div
                key={profile.id}
                className="bg-gray-800 p-4 rounded-lg border border-gray-700 hover:border-gray-600 transition-colors"
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-2xl">👤</span>
                      <h5 className="text-base font-medium text-gray-100">{profile.name}</h5>
                      {profile.is_active && (
                        <span className="text-xs bg-green-900 text-green-200 px-2 py-0.5 rounded">
                          Active
                        </span>
                      )}
                    </div>

                    <div className="ml-9 space-y-1">
                      <p className="text-xs text-gray-400">
                        Enrolled {formatDate(profile.enrollment_date)}
                      </p>

                      {profile.last_recognized && (
                        <p className="text-xs text-gray-400">
                          Last recognized: {formatDate(profile.last_recognized)}
                        </p>
                      )}

                      <p className="text-xs text-gray-500">
                        📈 Recognized {profile.recognition_count} times
                      </p>
                    </div>
                  </div>

                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleDeleteProfile(profile.id, profile.name)}
                    className="ml-4 bg-gray-700 hover:bg-red-900 text-gray-300 hover:text-red-200 border-gray-600"
                  >
                    Delete
                  </Button>
                </div>
              </div>
            ))}

            {/* Add new user button */}
            <Button
              onClick={() => setShowEnrollmentModal(true)}
              variant="outline"
              className="w-full bg-gray-800 hover:bg-gray-700 text-gray-300 border-gray-700"
            >
              + Add Another User
            </Button>
          </div>
        )}

        {/* Info box */}
        <div className="bg-purple-900/20 border border-purple-800/30 p-4 rounded-lg">
          <h4 className="text-sm font-medium text-purple-200 mb-2">💡 How it works</h4>
          <ul className="text-xs text-purple-300/80 space-y-1">
            <li>• Aura uses your voice to identify who's speaking</li>
            <li>• Personalized responses based on your preferences</li>
            <li>• Access to your Spotify playlists and smart home devices</li>
            <li>• 100% offline processing - no cloud uploads</li>
          </ul>
        </div>
      </div>

      {/* Enrollment Modal */}
      {showEnrollmentModal && (
        <EnrollmentModal
          isOpen={showEnrollmentModal}
          onClose={() => setShowEnrollmentModal(false)}
          onComplete={handleEnrollmentComplete}
        />
      )}
    </>
  );
};

export default UserProfilesSettings;
