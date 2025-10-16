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
import { formatDistanceToNow } from "date-fns";

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

interface ProfileCardProps {
  profile: UserProfile;
  onDelete: (id: number) => void;
}

const ProfileCard: React.FC<ProfileCardProps> = ({ profile, onDelete }) => {
  const [isDeleting, setIsDeleting] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const handleDelete = async () => {
    if (!showDeleteConfirm) {
      setShowDeleteConfirm(true);
      return;
    }

    setIsDeleting(true);
    try {
      await invoke("biometrics_delete_user", { userId: profile.id });
      onDelete(profile.id);
    } catch (error) {
      showErrorToast(`Failed to delete profile: ${error}`);
    } finally {
      setIsDeleting(false);
      setShowDeleteConfirm(false);
    }
  };

  const formatDate = (dateString: string | null) => {
    if (!dateString) return "Never";
    try {
      return formatDistanceToNow(new Date(dateString), { addSuffix: true });
    } catch {
      return "Unknown";
    }
  };

  return (
    <div className="border rounded-lg p-4 space-y-3 bg-white dark:bg-gray-800">
      <div className="flex items-start justify-between">
        <div className="flex items-center space-x-3">
          <div className="w-10 h-10 bg-blue-100 dark:bg-blue-900 rounded-full flex items-center justify-center">
            <span className="text-lg font-semibold text-blue-600 dark:text-blue-300">
              {profile.name.charAt(0).toUpperCase()}
            </span>
          </div>
          <div>
            <h3 className="font-semibold text-lg">{profile.name}</h3>
            <div className="flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-300">
              <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                profile.is_active 
                  ? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200" 
                  : "bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200"
              }`}>
                {profile.is_active ? "✅ Active" : "⏸️ Inactive"}
              </span>
              <span>•</span>
              <span>Enrolled {formatDate(profile.enrollment_date)}</span>
            </div>
          </div>
        </div>
        <div className="text-right">
          {showDeleteConfirm ? (
            <div className="space-x-2">
              <Button 
                variant="outline" 
                size="sm"
                onClick={() => setShowDeleteConfirm(false)}
              >
                Cancel
              </Button>
              <Button 
                variant="destructive" 
                size="sm"
                onClick={handleDelete}
                disabled={isDeleting}
              >
                {isDeleting ? "Deleting..." : "Confirm"}
              </Button>
            </div>
          ) : (
            <Button 
              variant="outline" 
              size="sm"
              onClick={handleDelete}
              className="text-red-600 hover:text-red-700"
            >
              Delete Profile
            </Button>
          )}
        </div>
      </div>

      <div className="flex items-center justify-between text-sm text-gray-600 dark:text-gray-300">
        <div className="flex items-center space-x-4">
          <span>📈 Recognized {profile.recognition_count} times</span>
          {profile.last_recognized && (
            <span>🕐 Last seen {formatDate(profile.last_recognized)}</span>
          )}
        </div>
      </div>
    </div>
  );
};

const UserProfilesSettings: React.FC = () => {
  const [profiles, setProfiles] = useState<UserProfile[]>([]);
  const [status, setStatus] = useState<BiometricsStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [showEnrollmentModal, setShowEnrollmentModal] = useState(false);

  const loadProfiles = async () => {
    try {
      setIsLoading(true);
      const [profilesData, statusData] = await Promise.all([
        invoke<UserProfile[]>("biometrics_list_users"),
        invoke<BiometricsStatus>("biometrics_get_status"),
      ]);
      
      setProfiles(profilesData);
      setStatus(statusData);
    } catch (error) {
      showErrorToast(`Failed to load user profiles: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadProfiles();
  }, []);

  const handleProfileDelete = (deletedId: number) => {
    setProfiles(profiles.filter(p => p.id !== deletedId));
    if (status) {
      setStatus({
        ...status,
        enrolled_user_count: status.enrolled_user_count - 1,
      });
    }
  };

  const handleEnrollmentComplete = () => {
    setShowEnrollmentModal(false);
    loadProfiles(); // Reload to show new profile
  };

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="text-center py-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-300">Loading user profiles...</p>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="space-y-6">
        {/* Header */}
        <div>
          <h2 className="text-2xl font-bold mb-2">User Profiles</h2>
          <p className="text-sm text-gray-600 dark:text-gray-300">
            Manage voice biometric profiles for personalized Aura experience
          </p>
        </div>

        {/* Status Overview */}
        <div className="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 space-y-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <span className="text-2xl">👥</span>
              <span className="font-medium">
                {status?.enrolled_user_count || 0} user{status?.enrolled_user_count !== 1 ? 's' : ''} enrolled
              </span>
            </div>
            {status?.is_enabled && (
              <div className="flex items-center space-x-1 text-sm text-green-600 dark:text-green-400">
                <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                <span>Voice recognition active</span>
              </div>
            )}
          </div>
          <div className="flex items-center space-x-1 text-sm text-gray-600 dark:text-gray-300">
            <span>🔒</span>
            <span>All voice data stored locally and securely</span>
          </div>
        </div>

        {/* User Profiles List */}
        <div className="space-y-4">
          {profiles.length === 0 ? (
            <div className="text-center py-12 space-y-4">
              <div className="text-6xl">🎤</div>
              <div>
                <h3 className="text-lg font-semibold mb-2">No voice profiles yet</h3>
                <p className="text-sm text-gray-600 dark:text-gray-300 mb-4">
                  Create your first voice profile to enable personalized responses
                </p>
                <Button onClick={() => setShowEnrollmentModal(true)}>
                  ➕ Create Voice Profile
                </Button>
              </div>
            </div>
          ) : (
            <>
              {profiles.map((profile) => (
                <ProfileCard
                  key={profile.id}
                  profile={profile}
                  onDelete={handleProfileDelete}
                />
              ))}

              {/* Add New User Button */}
              <div className="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-6 text-center">
                <Button
                  onClick={() => setShowEnrollmentModal(true)}
                  variant="outline"
                  className="space-x-2"
                >
                  <span>➕</span>
                  <span>Add New User</span>
                </Button>
                <p className="text-sm text-gray-600 dark:text-gray-300 mt-2">
                  Enroll additional family members or users
                </p>
              </div>
            </>
          )}
        </div>

        {/* Privacy Information */}
        <div className="bg-blue-50 dark:bg-blue-950 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
          <div className="flex items-start space-x-3">
            <span className="text-xl">🔒</span>
            <div className="space-y-2">
              <h4 className="font-medium text-blue-900 dark:text-blue-100">Your Privacy</h4>
              <ul className="text-sm text-blue-800 dark:text-blue-200 space-y-1">
                <li>• All voice data is stored locally on this device</li>
                <li>• Voice prints never leave your computer</li>
                <li>• You can delete your profile at any time</li>
                <li>• No cloud uploads or external processing</li>
              </ul>
            </div>
          </div>
        </div>

        {/* Help Tip */}
        <div className="bg-yellow-50 dark:bg-yellow-950 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
          <div className="flex items-start space-x-3">
            <span className="text-xl">💡</span>
            <div>
              <h4 className="font-medium text-yellow-900 dark:text-yellow-100 mb-1">Tip</h4>
              <p className="text-sm text-yellow-800 dark:text-yellow-200">
                Aura uses your voice to provide personalized responses and access your specific data 
                (like your Spotify playlists or calendar). The more you use it, the better it gets at recognizing you!
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Enrollment Modal */}
      <EnrollmentModal
        isOpen={showEnrollmentModal}
        onClose={() => setShowEnrollmentModal(false)}
        onComplete={handleEnrollmentComplete}
      />
    </>
  );
};

export default UserProfilesSettings;
