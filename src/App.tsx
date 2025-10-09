import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./components/Sidebar";
import ChatView from "./components/ChatView";
import SettingsModal from "./components/SettingsModal";
import { useChatStore } from "./store";
import { showErrorToast } from "./utils/errorHandler";

function App() {
  const setAppStatus = useChatStore((state) => state.setAppStatus);
  const appStatus = useChatStore((state) => state.appStatus);
  const addMessage = useChatStore((state) => state.addMessage);
  const setConversations = useChatStore((state) => state.setConversations);
  const activeConversationId = useChatStore((state) => state.activeConversationId);
  const updateConversationTitle = useChatStore((state) => state.updateConversationTitle);
  const conversations = useChatStore((state) => state.conversations);
  const setSettings = useChatStore((state) => state.setSettings);
  const lastInputMethod = useChatStore((state) => state.lastInputMethod);
  const setLastInputMethod = useChatStore((state) => state.setLastInputMethod);

  // Load conversations on app startup
  useEffect(() => {
    const loadConversations = async () => {
      try {
        const conversations = await invoke<any[]>("load_conversations");
        setConversations(conversations);
        console.log(`Loaded ${conversations.length} conversations`);
      } catch (error) {
        showErrorToast(error, "Failed to load conversations");
        // Non-fatal: app continues to function even if conversations don't load
      }
    };

    // Wrap in try-catch to prevent any uncaught promise rejections
    loadConversations().catch((err) => {
      showErrorToast(err, "Uncaught error in loadConversations");
    });
  }, [setConversations]);

  // Load settings on app startup
  useEffect(() => {
    const loadSettings = async () => {
      try {
        // Load non-sensitive settings from database
        const dbSettings = await invoke<any>("load_settings");

        // Load API key from OS keyring (may fail if not set)
        let apiKey = "";
        try {
          apiKey = await invoke<string>("load_api_key");
        } catch (keyError) {
          console.warn("API key not found (this is OK if not using remote LLM):", keyError);
        }

        // Update store
        setSettings({
          llm_provider: dbSettings.llm_provider,
          server_address: dbSettings.server_address,
          api_key: apiKey,
          wake_word_enabled: dbSettings.wake_word_enabled,
          api_base_url: dbSettings.api_base_url,
          model_name: dbSettings.model_name,
          vad_sensitivity: dbSettings.vad_sensitivity ?? 0.02,
          vad_timeout_ms: dbSettings.vad_timeout_ms ?? 1280,
          stt_model_name: dbSettings.stt_model_name ?? "ggml-base.en.bin",
        });

        console.log("Settings loaded successfully");
      } catch (error) {
        showErrorToast(error, "Failed to load settings");
        // Non-fatal: app uses default settings if load fails
      }
    };

    // Wrap in try-catch to prevent any uncaught promise rejections
    loadSettings().catch((err) => {
      showErrorToast(err, "Uncaught error in loadSettings");
    });
  }, [setSettings]);

  // Sync voice pipeline state with app status to prevent feedback loop
  useEffect(() => {
    const syncVoiceState = async () => {
      try {
        if (appStatus === "speaking") {
          // Disable wake word detection while TTS is speaking to prevent feedback loop
          await invoke("set_voice_state", { state: "speaking" });
          console.log("Voice state: speaking (wake word disabled)");
        } else if (appStatus === "idle") {
          // Re-enable wake word detection when idle
          await invoke("set_voice_state", { state: "listening_for_wake_word" });
          console.log("Voice state: listening_for_wake_word (wake word enabled)");
        }
        // Note: "listening" and "processing" states maintain current voice state
      } catch (error) {
        showErrorToast(error, "Failed to sync voice state");
      }
    };

    syncVoiceState().catch((err) => {
      showErrorToast(err, "Uncaught error in syncVoiceState");
    });
  }, [appStatus]);

  // Listen for wake word detection events
  useEffect(() => {
    // Listen for wake word detection events from Rust backend
    const unlisten = listen<string>("wake_word_detected", async (event) => {
      console.log("Wake word detected:", event.payload);

      // Set input method to voice (using local variable to avoid closure issues)
      const inputMethod = "voice";
      setLastInputMethod(inputMethod);

      // Ensure we have an active conversation
      let conversationId = activeConversationId;
      if (!conversationId) {
        console.log("No active conversation, creating one...");
        try {
          conversationId = await invoke<number>("create_new_conversation");
          console.log("Created new conversation:", conversationId);
        } catch (error) {
          showErrorToast(error, "Failed to create conversation");
          return;
        }
      }

      // Set status to listening
      setAppStatus("listening");

      try {
        // Call STT command to record and transcribe audio
        console.log("Starting audio transcription...");
        const transcription = await invoke<string>("listen_and_transcribe");
        console.log("Transcription received:", transcription);

        if (transcription && transcription.trim()) {
          // Set status to processing
          setAppStatus("processing");

          // Save user message to database
          await invoke("save_message", {
            conversationId,
            role: "user",
            content: transcription,
          });

          // Add user message to UI
          addMessage({ role: "user", content: transcription });

          // Call backend to process the prompt
          const response = await invoke<string>("handle_user_prompt", {
            prompt: transcription,
          });

          // Save assistant response to database
          await invoke("save_message", {
            conversationId,
            role: "assistant",
            content: response,
          });

          // Add assistant response to UI
          addMessage({ role: "assistant", content: response });

          // Auto-title the conversation if it's a new one
          const conversation = conversations.find((c) => c.id === conversationId);
          if (conversation && conversation.title.startsWith("New Chat")) {
            try {
              console.log("Generating title for voice conversation...");
              const generatedTitle = await invoke<string>(
                "generate_conversation_title",
                { prompt: transcription }
              );

              // Update title in database
              await invoke("update_conversation_title", {
                conversationId,
                title: generatedTitle,
              });

              // Update title in store
              updateConversationTitle(conversationId, generatedTitle);

              console.log(`Updated conversation title to: ${generatedTitle}`);
            } catch (titleError) {
              showErrorToast(titleError, "Failed to generate/update title");
              // Non-fatal error - conversation still works
            }
          }

          // Speak the response if input was via voice
          if (inputMethod === "voice") {
            try {
              console.log("Speaking response...");
              setAppStatus("speaking");
              await invoke("speak_text", { text: response });
              console.log("Finished speaking");
            } catch (speakError) {
              showErrorToast(speakError, "Failed to speak response");
              // Non-fatal error - response is still shown in UI
            }
          }
        } else {
          console.warn("Empty transcription received");
        }
      } catch (error) {
        showErrorToast(error, "Error during STT or prompt handling");
        const errorMsg = "Sorry, I couldn't understand that. Please try again.";

        // Save error message if we have a conversation
        if (conversationId) {
          try {
            await invoke("save_message", {
              conversationId,
              role: "assistant",
              content: errorMsg,
            });
          } catch (saveError) {
            showErrorToast(saveError, "Failed to save error message");
          }
        }

        addMessage({
          role: "assistant",
          content: errorMsg,
        });

        // Speak error message if input was via voice
        if (inputMethod === "voice") {
          try {
            setAppStatus("speaking");
            await invoke("speak_text", { text: errorMsg });
          } catch (speakError) {
            showErrorToast(speakError, "Failed to speak error message");
          }
        }
      } finally {
        // Reset to idle
        setAppStatus("idle");
        // Reset input method
        setLastInputMethod(null);
      }
    });

    // Cleanup listener on unmount
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [setAppStatus, addMessage, activeConversationId, conversations, updateConversationTitle, lastInputMethod, setLastInputMethod]);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-gray-950">
      <Sidebar />
      <ChatView />
      <SettingsModal />
    </div>
  );
}

export default App;
