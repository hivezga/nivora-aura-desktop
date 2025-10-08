import { create } from "zustand";

export interface Message {
  role: "user" | "assistant";
  content: string;
  id?: number;
  timestamp?: string;
}

export interface Conversation {
  id: number;
  title: string;
  created_at: string;
}

export interface Settings {
  llm_provider: string;         // "local" or "api" (kept for backward compatibility)
  server_address: string;       // Remote server address (legacy field)
  api_key: string;              // API key (stored in keyring, but cached here)
  wake_word_enabled: boolean;   // Enable/disable wake word detection
  api_base_url: string;         // Base URL for OpenAI-compatible API (e.g., "http://localhost:11434/v1")
  model_name: string;           // Model name to use (e.g., "llama3", "phi3:instruct")
  vad_sensitivity: number;      // Voice activity detection sensitivity (0.001-1.0)
  vad_timeout_ms: number;       // Silence timeout in milliseconds (100-10000)
  stt_model_name: string;       // STT (Whisper) model filename (e.g., "ggml-base.en.bin")
}

export type AppStatus = "idle" | "listening" | "processing" | "speaking";
export type InputMethod = "voice" | "text";

export interface ServiceStatus {
  wake_word: boolean;
  stt: boolean;
  llm: boolean;
}

interface ChatStore {
  // Conversation management
  conversations: Conversation[];
  activeConversationId: number | null;
  setConversations: (conversations: Conversation[]) => void;
  setActiveConversationId: (id: number | null) => void;
  addConversation: (conversation: Conversation) => void;
  removeConversation: (id: number) => void;
  updateConversationTitle: (id: number, title: string) => void;

  // Message management
  messages: Message[];
  setMessages: (messages: Message[]) => void;
  addMessage: (message: Message) => void;
  clearMessages: () => void;

  // App status
  appStatus: AppStatus;
  setAppStatus: (status: AppStatus) => void;

  // Input method tracking (for TTS triggering)
  lastInputMethod: InputMethod | null;
  setLastInputMethod: (method: InputMethod | null) => void;

  // Settings management
  settings: Settings;
  setSettings: (settings: Settings) => void;
  isSettingsOpen: boolean;
  openSettings: () => void;
  closeSettings: () => void;

  // Service status
  serviceStatus: ServiceStatus;
  setServiceStatus: (service: "wake_word" | "stt" | "llm", connected: boolean) => void;
  setSystemStatus: (status: { stt_connected: boolean; llm_connected: boolean }) => void;
}

export const useChatStore = create<ChatStore>((set) => ({
  // Conversation state
  conversations: [],
  activeConversationId: null,
  setConversations: (conversations) => set({ conversations }),
  setActiveConversationId: (id) => set({ activeConversationId: id }),
  addConversation: (conversation) =>
    set((state) => ({ conversations: [conversation, ...state.conversations] })),
  removeConversation: (id) =>
    set((state) => ({
      conversations: state.conversations.filter((c) => c.id !== id),
    })),
  updateConversationTitle: (id, title) =>
    set((state) => ({
      conversations: state.conversations.map((c) =>
        c.id === id ? { ...c, title } : c
      ),
    })),

  // Message state
  messages: [],
  setMessages: (messages) => set({ messages }),
  addMessage: (message) =>
    set((state) => ({ messages: [...state.messages, message] })),
  clearMessages: () => set({ messages: [] }),

  // App status
  appStatus: "idle",
  setAppStatus: (status) => set({ appStatus: status }),

  // Input method tracking
  lastInputMethod: null,
  setLastInputMethod: (method) => set({ lastInputMethod: method }),

  // Settings state
  settings: {
    llm_provider: "local",
    server_address: "",
    api_key: "",
    wake_word_enabled: false,
    api_base_url: "http://localhost:11434/v1",
    model_name: "llama3",
    vad_sensitivity: 0.02,
    vad_timeout_ms: 1280,
    stt_model_name: "ggml-base.en.bin",
  },
  setSettings: (settings) => set({ settings }),
  isSettingsOpen: false,
  openSettings: () => set({ isSettingsOpen: true }),
  closeSettings: () => set({ isSettingsOpen: false }),

  // Service status state
  serviceStatus: {
    wake_word: false,
    stt: false,
    llm: false,
  },
  setServiceStatus: (service, connected) =>
    set((state) => ({
      serviceStatus: {
        ...state.serviceStatus,
        [service]: connected,
      },
    })),
  setSystemStatus: (status) =>
    set((state) => ({
      serviceStatus: {
        ...state.serviceStatus,
        stt: status.stt_connected,
        llm: status.llm_connected,
      },
    })),
}));
