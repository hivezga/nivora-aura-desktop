import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface SetupStatus {
  first_run_complete: boolean;
  whisper_model_exists: boolean;
  whisper_model_path: string;
}

interface DownloadProgress {
  downloaded_bytes: number;
  total_bytes: number | null;
  percentage: number;
}

type WizardStep = "checking" | "dependencies" | "downloading" | "complete";

export default function WelcomeWizard({ onComplete }: { onComplete: () => void }) {
  const [step, setStep] = useState<WizardStep>("checking");
  const [status, setStatus] = useState<SetupStatus | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress>({
    downloaded_bytes: 0,
    total_bytes: null,
    percentage: 0,
  });
  const [error, setError] = useState<string | null>(null);

  // Check setup status on mount
  useEffect(() => {
    checkStatus();
  }, []);

  // Listen for download progress events
  useEffect(() => {
    const unlisten = listen<DownloadProgress>("download_progress", (event) => {
      setDownloadProgress(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const checkStatus = async () => {
    try {
      setError(null);
      const setupStatus = await invoke<SetupStatus>("check_setup_status");
      setStatus(setupStatus);

      // If already complete, skip wizard
      if (setupStatus.first_run_complete) {
        onComplete();
        return;
      }

      setStep("dependencies");
    } catch (err) {
      setError(`Failed to check setup status: ${err}`);
      setStep("dependencies");
    }
  };

  const handleDownloadModel = async () => {
    try {
      setDownloading(true);
      setError(null);
      setStep("downloading");

      await invoke("download_whisper_model");

      // Refresh status after download
      await checkStatus();
      setDownloading(false);
      setStep("dependencies");
    } catch (err) {
      setError(`Failed to download model: ${err}`);
      setDownloading(false);
      setStep("dependencies");
    }
  };

  const handleFinish = async () => {
    try {
      setError(null);

      // Check if all dependencies are ready
      if (!status?.whisper_model_exists) {
        setError("Please download the Whisper model before finishing");
        return;
      }

      // Mark setup as complete
      await invoke("mark_setup_complete");

      setStep("complete");

      // Wait a moment then close wizard
      setTimeout(() => {
        onComplete();
      }, 1500);
    } catch (err) {
      setError(`Failed to complete setup: ${err}`);
    }
  };

  const canFinish = status?.whisper_model_exists;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full mx-4 p-8">
        {/* Header */}
        <div className="mb-8 text-center">
          <h1 className="text-3xl font-bold text-white mb-2">Welcome to Nivora Aura</h1>
          <p className="text-gray-400">Let's set up your AI assistant in just a few steps</p>
        </div>

        {/* Error Display */}
        {error && (
          <div className="mb-6 p-4 bg-red-900 bg-opacity-50 border border-red-500 rounded-lg">
            <p className="text-red-200 text-sm">{error}</p>
          </div>
        )}

        {/* Content based on step */}
        {step === "checking" && (
          <div className="text-center py-12">
            <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-purple-500 mx-auto mb-4"></div>
            <p className="text-gray-300">Checking system requirements...</p>
          </div>
        )}

        {step === "dependencies" && status && (
          <div className="space-y-6">
            {/* Whisper Model Status */}
            <div className="flex items-start gap-4 p-4 bg-gray-700 rounded-lg">
              <div className="flex-shrink-0">
                {status.whisper_model_exists ? (
                  <svg className="w-6 h-6 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                  </svg>
                ) : (
                  <svg className="w-6 h-6 text-yellow-500" fill="currentColor" viewBox="0 0 20 20">
                    <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                  </svg>
                )}
              </div>
              <div className="flex-1">
                <h3 className="text-white font-semibold mb-1">Whisper STT Model (Speech-to-Text)</h3>
                <p className="text-gray-400 text-sm mb-2">
                  {status.whisper_model_exists
                    ? "✓ Whisper model is installed and ready"
                    : "Download the Whisper tiny model (~75 MB) for speech recognition"}
                </p>
                {!status.whisper_model_exists && !downloading && (
                  <button
                    onClick={handleDownloadModel}
                    className="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg text-sm transition-colors"
                  >
                    Download Model
                  </button>
                )}
              </div>
            </div>

            {/* Finish Button */}
            <div className="pt-4 flex justify-end">
              <button
                onClick={handleFinish}
                disabled={!canFinish}
                className={`px-6 py-3 rounded-lg font-semibold transition-colors ${
                  canFinish
                    ? "bg-green-600 hover:bg-green-700 text-white"
                    : "bg-gray-600 text-gray-400 cursor-not-allowed"
                }`}
              >
                {canFinish ? "Finish Setup" : "Complete all steps to continue"}
              </button>
            </div>
          </div>
        )}

        {step === "downloading" && (
          <div className="py-12">
            <div className="mb-6 text-center">
              <div className="animate-pulse text-purple-500 mb-4">
                <svg className="w-16 h-16 mx-auto" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clipRule="evenodd" />
                </svg>
              </div>
              <h3 className="text-white text-xl font-semibold mb-2">Downloading Whisper Model</h3>
              <p className="text-gray-400">This may take a few minutes...</p>
            </div>

            {/* Progress Bar */}
            <div className="mb-4">
              <div className="bg-gray-700 rounded-full h-4 overflow-hidden">
                <div
                  className="bg-purple-600 h-full transition-all duration-300 ease-out"
                  style={{ width: `${downloadProgress.percentage}%` }}
                ></div>
              </div>
            </div>

            {/* Progress Stats */}
            <div className="flex justify-between text-sm text-gray-400">
              <span>{downloadProgress.percentage.toFixed(1)}%</span>
              <span>
                {(downloadProgress.downloaded_bytes / 1024 / 1024).toFixed(2)} MB
                {downloadProgress.total_bytes && ` / ${(downloadProgress.total_bytes / 1024 / 1024).toFixed(2)} MB`}
              </span>
            </div>
          </div>
        )}

        {step === "complete" && (
          <div className="text-center py-12">
            <div className="mb-6">
              <svg className="w-20 h-20 text-green-500 mx-auto" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
              </svg>
            </div>
            <h3 className="text-white text-2xl font-bold mb-2">Setup Complete!</h3>
            <p className="text-gray-400">Launching Nivora Aura...</p>
          </div>
        )}
      </div>
    </div>
  );
}
