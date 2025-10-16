/**
 * useAudioRecording Hook
 *
 * Provides browser-based audio recording functionality for voice enrollment.
 * Records audio at 16kHz (required by speaker recognition models) and returns
 * PCM Float32 samples for processing by the backend.
 *
 * Based on Web Audio API and MediaStream API.
 */

import { useState, useCallback, useRef } from "react";

export interface AudioRecordingResult {
  audioData: Float32Array;
  duration: number;
  sampleRate: number;
}

export interface UseAudioRecordingReturn {
  isRecording: boolean;
  audioLevel: number;
  startRecording: () => Promise<void>;
  stopRecording: () => Promise<AudioRecordingResult | null>;
  hasPermission: boolean;
  error: string | null;
}

/**
 * Hook for recording audio samples via browser microphone
 *
 * @param targetSampleRate - Target sample rate (default: 16000 Hz for speaker recognition)
 * @returns Recording controls and state
 *
 * @example
 * const { isRecording, startRecording, stopRecording, audioLevel } = useAudioRecording();
 *
 * await startRecording();
 * // ... wait for user to speak ...
 * const result = await stopRecording();
 * // result.audioData is Float32Array of PCM samples
 */
export function useAudioRecording(
  targetSampleRate: number = 16000
): UseAudioRecordingReturn {
  const [isRecording, setIsRecording] = useState(false);
  const [audioLevel, setAudioLevel] = useState(0);
  const [hasPermission, setHasPermission] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Refs to hold AudioContext and stream (persist across renders)
  const audioContextRef = useRef<AudioContext | null>(null);
  const mediaStreamRef = useRef<MediaStream | null>(null);
  const audioDataRef = useRef<number[]>([]);
  const startTimeRef = useRef<number>(0);

  /**
   * Start recording audio from microphone
   */
  const startRecording = useCallback(async () => {
    try {
      setError(null);

      // Request microphone permission
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          echoCancellation: false,  // Disable for speaker recognition
          noiseSuppression: false,  // Disable for speaker recognition
          autoGainControl: false,   // Disable for speaker recognition
          sampleRate: targetSampleRate,
        }
      });

      mediaStreamRef.current = stream;
      setHasPermission(true);

      // Create AudioContext with target sample rate
      const audioContext = new AudioContext({ sampleRate: targetSampleRate });
      audioContextRef.current = audioContext;

      // Create media stream source
      const source = audioContext.createMediaStreamSource(stream);

      // Create ScriptProcessor for audio capture
      // Note: ScriptProcessor is deprecated but still widely supported
      // TODO: Migrate to AudioWorklet in future for better performance
      const bufferSize = 4096;
      const processor = audioContext.createScriptProcessor(bufferSize, 1, 1);

      processor.onaudioprocess = (e) => {
        const inputData = e.inputBuffer.getChannelData(0);

        // Store audio data
        audioDataRef.current.push(...Array.from(inputData));

        // Calculate audio level (RMS) for visualization
        let sum = 0;
        for (let i = 0; i < inputData.length; i++) {
          sum += inputData[i] * inputData[i];
        }
        const rms = Math.sqrt(sum / inputData.length);
        setAudioLevel(rms * 100); // Scale to 0-100 range
      };

      // Connect nodes
      source.connect(processor);
      processor.connect(audioContext.destination);

      // Reset audio data buffer
      audioDataRef.current = [];
      startTimeRef.current = Date.now();
      setIsRecording(true);

      console.log(`✓ Audio recording started at ${targetSampleRate} Hz`);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error("Failed to start audio recording:", err);

      // Handle specific permission error
      if (errorMessage.includes("Permission denied") || errorMessage.includes("NotAllowedError")) {
        setError("Microphone permission denied. Please enable microphone access in your browser settings.");
      }
    }
  }, [targetSampleRate]);

  /**
   * Stop recording and return audio data
   */
  const stopRecording = useCallback(async (): Promise<AudioRecordingResult | null> => {
    if (!isRecording || !audioContextRef.current || !mediaStreamRef.current) {
      return null;
    }

    try {
      // Stop all tracks
      mediaStreamRef.current.getTracks().forEach(track => track.stop());

      // Close audio context
      await audioContextRef.current.close();

      // Calculate duration
      const duration = (Date.now() - startTimeRef.current) / 1000;

      // Get recorded audio data
      const audioData = new Float32Array(audioDataRef.current);

      console.log(`✓ Audio recording stopped: ${duration.toFixed(2)}s, ${audioData.length} samples`);

      // Reset state
      setIsRecording(false);
      setAudioLevel(0);
      audioContextRef.current = null;
      mediaStreamRef.current = null;

      return {
        audioData,
        duration,
        sampleRate: targetSampleRate,
      };
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error("Failed to stop audio recording:", err);
      return null;
    }
  }, [isRecording, targetSampleRate]);

  return {
    isRecording,
    audioLevel,
    startRecording,
    stopRecording,
    hasPermission,
    error,
  };
}
