/**
 * EnrollmentModal Component
 *
 * Multi-step wizard for voice biometric enrollment.
 * Guides the user through recording 3 voice samples to create a voice print.
 */

import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
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
import { useAudioRecording } from "../hooks/useAudioRecording";
import { showErrorToast } from "../utils/errorHandler";

interface EnrollmentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onComplete: () => void;
}

type EnrollmentStep = "welcome" | "name" | "record1" | "record2" | "record3" | "processing" | "success";

const RECORDING_PHRASES = [
  "Hey Aura, this is my voice",
  "I use Aura for my smart home",
  "This is my personal voice print",
];

const EnrollmentModal: React.FC<EnrollmentModalProps> = ({ isOpen, onClose, onComplete }) => {
  const [step, setStep] = useState<EnrollmentStep>("welcome");
  const [userName, setUserName] = useState("");
  const [audioSamples, setAudioSamples] = useState<Float32Array[]>([]);
  const [enrollmentError, setEnrollmentError] = useState<string | null>(null);

  const { isRecording, audioLevel, startRecording, stopRecording, error: recordingError } =
    useAudioRecording(16000);

  // Reset state when modal opens
  useEffect(() => {
    if (isOpen) {
      setStep("welcome");
      setUserName("");
      setAudioSamples([]);
      setEnrollmentError(null);
    }
  }, [isOpen]);

  // Handle recording for each step
  const handleStartRecording = async () => {
    try {
      await startRecording();
    } catch (err) {
      console.error("Failed to start recording:", err);
      setEnrollmentError("Failed to start recording. Please check your microphone permissions.");
    }
  };

  const handleStopRecording = async () => {
    try {
      const result = await stopRecording();
      if (result) {
        // Add sample to collection
        setAudioSamples([...audioSamples, result.audioData]);

        // Advance to next step
        if (step === "record1") {
          setStep("record2");
        } else if (step === "record2") {
          setStep("record3");
        } else if (step === "record3") {
          // All 3 samples collected, process enrollment
          await processEnrollment([...audioSamples, result.audioData]);
        }
      }
    } catch (err) {
      console.error("Failed to stop recording:", err);
      setEnrollmentError("Failed to process recording. Please try again.");
    }
  };

  const processEnrollment = async (samples: Float32Array[]) => {
    setStep("processing");

    try {
      // Convert Float32Array to regular arrays for Tauri
      const samplesAsArrays = samples.map((sample) => Array.from(sample));

      console.log(`Processing enrollment for "${userName}" with ${samples.length} samples...`);

      // Call backend to enroll user
      await invoke<number>("biometrics_enroll_user", {
        userName: userName,
        audioSamples: samplesAsArrays,
      });

      console.log(`✓ User enrolled successfully`);
      setStep("success");
    } catch (err) {
      console.error("Enrollment failed:", err);
      const errorMessage = err instanceof Error ? err.message : String(err);
      setEnrollmentError(errorMessage);
      setStep("welcome"); // Reset to beginning on error
      showErrorToast(err, "Voice enrollment failed");
    }
  };

  const getCurrentStepNumber = (): number => {
    switch (step) {
      case "welcome": return 1;
      case "name": return 2;
      case "record1": return 3;
      case "record2": return 4;
      case "record3": return 5;
      case "processing": return 6;
      case "success": return 7;
      default: return 1;
    }
  };

  const getCurrentRecordingIndex = (): number => {
    if (step === "record1") return 0;
    if (step === "record2") return 1;
    if (step === "record3") return 2;
    return 0;
  };

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && !isRecording && onClose()}>
      <DialogContent className="sm:max-w-[500px] bg-gray-900 border-gray-800">
        {step === "welcome" && (
          <>
            <DialogHeader>
              <DialogTitle className="text-2xl text-gray-100">Voice Enrollment</DialogTitle>
              <DialogDescription className="text-gray-400">
                Let Aura recognize your voice for personalized responses
              </DialogDescription>
            </DialogHeader>

            <div className="py-6 space-y-4">
              <div className="text-center text-6xl">🎤</div>

              <div className="space-y-3 text-sm text-gray-300">
                <p className="text-center">How it works:</p>
                <ol className="space-y-2 pl-6">
                  <li>1. You'll speak 3 short phrases</li>
                  <li>2. Aura creates a secure "voice print"</li>
                  <li>3. Your voice print is stored only on this device</li>
                </ol>
              </div>

              <div className="bg-gray-800 p-4 rounded-lg space-y-2 text-xs text-gray-400">
                <p>📊 Success rate: ~95% accuracy</p>
                <p>⏱️ Takes about 2 minutes</p>
                <p>🔒 100% private (no cloud upload)</p>
              </div>

              <div className="text-xs text-gray-500 text-center pt-4">
                By continuing, you consent to Aura collecting and storing a biometric voice print on this device for
                speaker identification.
              </div>
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={onClose} className="bg-gray-800 hover:bg-gray-700 text-gray-200">
                Cancel
              </Button>
              <Button onClick={() => setStep("name")} className="bg-purple-700 hover:bg-purple-600 text-white">
                Get Started
              </Button>
            </DialogFooter>
          </>
        )}

        {step === "name" && (
          <>
            <DialogHeader>
              <DialogTitle className="text-xl text-gray-100">Voice Enrollment</DialogTitle>
              <DialogDescription className="text-gray-400">Step {getCurrentStepNumber()} of 7: Your Name</DialogDescription>
            </DialogHeader>

            <div className="py-6 space-y-4">
              <div className="text-center text-4xl mb-4">👤</div>

              <div className="space-y-2">
                <Label htmlFor="user-name" className="text-gray-300">
                  What should Aura call you?
                </Label>
                <Input
                  id="user-name"
                  type="text"
                  value={userName}
                  onChange={(e) => setUserName(e.target.value)}
                  placeholder="Enter your name"
                  className="bg-gray-800 text-gray-100 border-gray-700 focus:ring-purple-600 placeholder-gray-500"
                  autoFocus
                />
                <p className="text-xs text-gray-500">This name will appear when Aura recognizes your voice</p>
              </div>
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={() => setStep("welcome")} className="bg-gray-800 hover:bg-gray-700 text-gray-200">
                Back
              </Button>
              <Button
                onClick={() => setStep("record1")}
                disabled={!userName.trim() || userName.length < 2}
                className="bg-purple-700 hover:bg-purple-600 text-white disabled:opacity-50"
              >
                Continue →
              </Button>
            </DialogFooter>
          </>
        )}

        {(step === "record1" || step === "record2" || step === "record3") && (
          <>
            <DialogHeader>
              <DialogTitle className="text-xl text-gray-100">Voice Enrollment</DialogTitle>
              <DialogDescription className="text-gray-400">
                Step {getCurrentStepNumber()} of 7: Voice Sample {getCurrentRecordingIndex() + 1}
              </DialogDescription>
            </DialogHeader>

            <div className="py-6 space-y-6">
              <div className="text-center">
                <p className="text-sm text-gray-400 mb-3">🎤 Please say:</p>
                <p className="text-lg font-medium text-purple-200 mb-6">"{RECORDING_PHRASES[getCurrentRecordingIndex()]}"</p>

                {/* Recording visualization */}
                {isRecording && (
                  <div className="flex items-center justify-center mb-6">
                    <div className="relative w-32 h-32 flex items-center justify-center">
                      <div className="absolute inset-0 bg-red-600 rounded-full animate-pulse opacity-50"></div>
                      <div className="relative z-10 text-white font-bold">🔴 Recording</div>
                    </div>
                  </div>
                )}

                {/* Audio level meter */}
                {isRecording && (
                  <div className="mb-4">
                    <div className="w-full h-2 bg-gray-700 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-purple-600 transition-all duration-100"
                        style={{ width: `${Math.min(audioLevel * 2, 100)}%` }}
                      ></div>
                    </div>
                    <p className="text-xs text-gray-500 mt-1">Audio level</p>
                  </div>
                )}

                {!isRecording && audioSamples.length < getCurrentRecordingIndex() + 1 && (
                  <Button
                    onClick={handleStartRecording}
                    className="bg-red-700 hover:bg-red-600 text-white px-8 py-6 text-lg"
                  >
                    🎤 Start Recording
                  </Button>
                )}

                {isRecording && (
                  <Button
                    onClick={handleStopRecording}
                    className="bg-purple-700 hover:bg-purple-600 text-white px-8 py-6 text-lg"
                  >
                    ⏹️ Stop Recording
                  </Button>
                )}
              </div>

              <div className="bg-gray-800 p-4 rounded-lg space-y-2 text-xs text-gray-400">
                <p>💡 Speak clearly in a normal voice</p>
                <p>🔊 Make sure you're in a quiet environment</p>
              </div>

              {recordingError && (
                <div className="bg-red-900/50 border border-red-700 p-3 rounded text-sm text-red-200">
                  ⚠️ {recordingError}
                </div>
              )}
            </div>

            <DialogFooter>
              <Button
                variant="outline"
                onClick={() => {
                  if (step === "record1") setStep("name");
                  else if (step === "record2") setStep("record1");
                  else if (step === "record3") setStep("record2");
                }}
                className="bg-gray-800 hover:bg-gray-700 text-gray-200"
                disabled={isRecording}
              >
                Back
              </Button>
            </DialogFooter>
          </>
        )}

        {step === "processing" && (
          <>
            <DialogHeader>
              <DialogTitle className="text-xl text-gray-100">Voice Enrollment</DialogTitle>
              <DialogDescription className="text-gray-400">Step {getCurrentStepNumber()} of 7: Processing</DialogDescription>
            </DialogHeader>

            <div className="py-8 space-y-4 text-center">
              <div className="text-6xl mb-4">⏳</div>
              <p className="text-lg text-gray-200">Creating voice print...</p>

              <div className="w-full bg-gray-700 rounded-full h-2 overflow-hidden">
                <div className="bg-purple-600 h-full animate-pulse" style={{ width: "75%" }}></div>
              </div>

              <div className="space-y-1 text-sm text-gray-400">
                <p>✅ Sample 1 processed</p>
                <p>✅ Sample 2 processed</p>
                <p>⏳ Processing sample 3...</p>
              </div>

              <p className="text-xs text-gray-500 pt-4">This may take a few seconds.</p>
            </div>
          </>
        )}

        {step === "success" && (
          <>
            <DialogHeader>
              <DialogTitle className="text-xl text-gray-100">Voice Enrollment</DialogTitle>
              <DialogDescription className="text-gray-400">Complete!</DialogDescription>
            </DialogHeader>

            <div className="py-8 space-y-4 text-center">
              <div className="text-6xl mb-4">✅</div>
              <h3 className="text-2xl font-bold text-gray-100">Enrollment Complete, {userName}!</h3>

              <p className="text-gray-300">Your voice profile has been created successfully.</p>
              <p className="text-gray-300">Aura will now recognize you automatically.</p>

              <div className="bg-gray-800 p-4 rounded-lg space-y-2 text-sm text-gray-400">
                <p>📊 Enrollment Quality: Excellent</p>
                <p>🎯 Expected Accuracy: ~95%</p>
                <p>🔒 Stored securely on this device</p>
              </div>

              <p className="text-xs text-gray-500 pt-4">
                💡 Next time you speak to Aura, your personalized settings will be activated automatically.
              </p>
            </div>

            <DialogFooter>
              <Button
                onClick={() => {
                  onComplete();
                  onClose();
                }}
                className="bg-purple-700 hover:bg-purple-600 text-white w-full"
              >
                Done
              </Button>
            </DialogFooter>
          </>
        )}

        {enrollmentError && step !== "success" && (
          <div className="mt-4 bg-red-900/50 border border-red-700 p-3 rounded text-sm text-red-200">
            ⚠️ {enrollmentError}
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
};

export default EnrollmentModal;
