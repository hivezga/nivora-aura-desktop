import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useChatStore } from "../store";
import { Mic } from "lucide-react";

const InputBar: React.FC = () => {
  const [input, setInput] = useState("");
  const addMessage = useChatStore((state) => state.addMessage);
  const appStatus = useChatStore((state) => state.appStatus);
  const setAppStatus = useChatStore((state) => state.setAppStatus);
  const activeConversationId = useChatStore((state) => state.activeConversationId);
  const setActiveConversationId = useChatStore((state) => state.setActiveConversationId);
  const addConversation = useChatStore((state) => state.addConversation);
  const updateConversationTitle = useChatStore((state) => state.updateConversationTitle);
  const conversations = useChatStore((state) => state.conversations);
  const setLastInputMethod = useChatStore((state) => state.setLastInputMethod);

  const handleSend = async () => {
    if (!input.trim()) return;

    // Set input method to text (to prevent TTS from triggering)
    setLastInputMethod('text');

    // Ensure we have an active conversation
    let conversationId = activeConversationId;
    if (!conversationId) {
      console.log("No active conversation, creating one...");
      try {
        conversationId = await invoke<number>("create_new_conversation");
        setActiveConversationId(conversationId);

        // Load the new conversation to get its details
        const conversations = await invoke<any[]>("load_conversations");
        const newConv = conversations.find((c) => c.id === conversationId);
        if (newConv) {
          addConversation(newConv);
        }

        console.log("Created new conversation:", conversationId);
      } catch (error) {
        console.error("Failed to create conversation:", error);
        return;
      }
    }

    // Save user prompt
    const userPrompt = input;

    // Clear input field immediately
    setInput("");

    try {
      // Save user message to database
      await invoke("save_message", {
        conversationId,
        role: "user",
        content: userPrompt,
      });

      // Add user message to UI
      addMessage({ role: "user", content: userPrompt });

      // Set status to processing (enables "Stop Generating" button)
      setAppStatus("processing");

      // Call backend command to get response
      const response = await invoke<string>("handle_user_prompt", {
        prompt: userPrompt,
      });

      // Set status back to idle
      setAppStatus("idle");

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
          console.log("Generating title for conversation...");
          const generatedTitle = await invoke<string>(
            "generate_conversation_title",
            { prompt: userPrompt }
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
          console.error("Failed to generate/update title:", titleError);
          // Non-fatal error - conversation still works
        }
      }
    } catch (error) {
      console.error("Error calling backend:", error);

      // Set status back to idle on error
      setAppStatus("idle");

      const errorMsg = "Sorry, there was an error processing your request.";

      // Try to save error message
      if (conversationId) {
        try {
          await invoke("save_message", {
            conversationId,
            role: "assistant",
            content: errorMsg,
          });
        } catch (saveError) {
          console.error("Failed to save error message:", saveError);
        }
      }

      addMessage({
        role: "assistant",
        content: errorMsg,
      });
    }
  };

  const handleStopGenerating = async () => {
    try {
      // Call backend to cancel generation
      await invoke("cancel_generation");
      console.log("Generation cancelled");
    } catch (error) {
      console.error("Failed to cancel generation:", error);
    }
  };

  const handleMicrophoneClick = async () => {
    if (appStatus === "listening") {
      try {
        // Call backend to cancel recording
        await invoke("cancel_recording");
        console.log("Recording cancelled");
        setAppStatus("idle");
      } catch (error) {
        console.error("Failed to cancel recording:", error);
        setAppStatus("idle");
      }
      return;
    }

    if (appStatus !== "idle") {
      return; // Don't allow mic while processing
    }

    try {
      // Set input method to voice (to enable TTS for response)
      setLastInputMethod('voice');

      // Set status to listening
      setAppStatus("listening");

      // Call STT service to record and transcribe
      let transcribedText: string;
      try {
        transcribedText = await invoke<string>("listen_and_transcribe");
      } catch (error) {
        // Check if this was a user cancellation
        const errorMessage = String(error);
        if (errorMessage.includes("cancelled by user")) {
          console.log("Recording cancelled by user");
          setAppStatus("idle");
          return;
        }
        // Re-throw if it's a different error
        throw error;
      }

      // Set status back to idle after transcription
      setAppStatus("idle");

      if (!transcribedText.trim()) {
        console.log("No speech detected");
        return;
      }

      console.log("Transcribed:", transcribedText);

      // Set the transcribed text as input and send it
      setInput(transcribedText);

      // Ensure we have an active conversation
      let conversationId = activeConversationId;
      if (!conversationId) {
        console.log("No active conversation, creating one...");
        conversationId = await invoke<number>("create_new_conversation");
        setActiveConversationId(conversationId);

        const conversations = await invoke<any[]>("load_conversations");
        const newConv = conversations.find((c) => c.id === conversationId);
        if (newConv) {
          addConversation(newConv);
        }
      }

      // Clear input field (it has the transcription)
      setInput("");

      // Save user message
      await invoke("save_message", {
        conversationId,
        role: "user",
        content: transcribedText,
      });

      addMessage({ role: "user", content: transcribedText });

      // Set status to processing
      setAppStatus("processing");

      // Get LLM response
      const response = await invoke<string>("handle_user_prompt", {
        prompt: transcribedText,
      });

      // Save assistant response
      await invoke("save_message", {
        conversationId,
        role: "assistant",
        content: response,
      });

      addMessage({ role: "assistant", content: response });

      // Speak the response since input was via voice (Push-to-Talk)
      try {
        console.log("Speaking response...");
        setAppStatus("speaking");
        await invoke("speak_text", { text: response });
        console.log("Finished speaking");
      } catch (speakError) {
        console.error("Failed to speak response:", speakError);
        // Non-fatal error - response is still shown in UI
      } finally {
        setAppStatus("idle");
      }

      // Auto-title if new conversation
      const conversation = conversations.find((c) => c.id === conversationId);
      if (conversation && conversation.title.startsWith("New Chat")) {
        try {
          const generatedTitle = await invoke<string>(
            "generate_conversation_title",
            { prompt: transcribedText }
          );

          await invoke("update_conversation_title", {
            conversationId,
            title: generatedTitle,
          });

          updateConversationTitle(conversationId, generatedTitle);
        } catch (titleError) {
          console.error("Failed to generate title:", titleError);
        }
      }
    } catch (error) {
      console.error("Error with voice input:", error);
      setAppStatus("idle");

      const errorMsg = "Sorry, there was an error processing your voice input.";
      addMessage({
        role: "assistant",
        content: errorMsg,
      });
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      handleSend();
    }
  };

  // Determine placeholder text based on app status
  const getPlaceholder = () => {
    if (appStatus === "listening") {
      return "Listening... Speak now";
    } else if (appStatus === "processing") {
      return "Processing...";
    }
    return "Type your message or say 'Hey Aura'...";
  };

  return (
    <div className="border-t border-gray-800 bg-gray-900 p-4">
      <div className="max-w-4xl mx-auto flex gap-3">
        <div className="flex-1 relative">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder={getPlaceholder()}
            disabled={appStatus !== "idle"}
            className="w-full bg-gray-800 text-gray-100 placeholder-gray-500 rounded-lg px-4 py-3 focus:outline-none focus:ring-2 focus:ring-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
          />
          {appStatus === "listening" && (
            <div className="absolute right-3 top-1/2 -translate-y-1/2">
              <div className="flex items-center gap-1">
                <div className="w-1 h-3 bg-red-500 animate-pulse rounded-full"></div>
                <div className="w-1 h-4 bg-red-500 animate-pulse rounded-full animation-delay-150"></div>
                <div className="w-1 h-3 bg-red-500 animate-pulse rounded-full animation-delay-300"></div>
              </div>
            </div>
          )}
        </div>

        {/* Microphone button */}
        <button
          onClick={handleMicrophoneClick}
          disabled={appStatus === "processing"}
          className={`p-3 rounded-lg transition-all duration-200 flex items-center justify-center ${
            appStatus === "listening"
              ? "bg-red-500 text-white hover:bg-red-600 animate-pulse"
              : appStatus === "processing"
              ? "bg-gray-800 text-gray-500 cursor-not-allowed"
              : "bg-gray-700 text-gray-100 hover:bg-gray-600"
          }`}
          title={appStatus === "listening" ? "Stop recording" : "Record voice message"}
        >
          <Mic className="w-5 h-5" />
        </button>

        {appStatus === "processing" ? (
          <button
            onClick={handleStopGenerating}
            className="bg-transparent border-2 border-red-500 hover:bg-red-500 text-red-500 hover:text-white font-medium px-6 py-3 rounded-lg transition-all duration-200"
          >
            Stop Generating
          </button>
        ) : (
          <button
            onClick={handleSend}
            disabled={appStatus !== "idle"}
            className="bg-gray-700 hover:bg-gray-600 text-gray-100 font-medium px-6 py-3 rounded-lg transition-colors duration-200 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-gray-700"
          >
            Send
          </button>
        )}
      </div>
    </div>
  );
};

export default InputBar;
