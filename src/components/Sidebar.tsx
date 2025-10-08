import React from "react";
import { invoke } from "@tauri-apps/api/core";
import { useChatStore } from "../store";
import { MessageSquare } from "lucide-react";
import {
  isToday,
  isYesterday,
  isThisWeek,
  format,
  parseISO
} from "date-fns";
import SystemStatusIndicator from "./SystemStatusIndicator";

const Sidebar: React.FC = () => {
  const conversations = useChatStore((state) => state.conversations);
  const activeConversationId = useChatStore(
    (state) => state.activeConversationId
  );
  const setActiveConversationId = useChatStore(
    (state) => state.setActiveConversationId
  );
  const setMessages = useChatStore((state) => state.setMessages);
  const addConversation = useChatStore((state) => state.addConversation);
  const clearMessages = useChatStore((state) => state.clearMessages);
  const removeConversation = useChatStore((state) => state.removeConversation);
  const openSettings = useChatStore((state) => state.openSettings);

  /**
   * Format a date as a relative time string
   * - Today: "Today"
   * - Yesterday: "Yesterday"
   * - This week: Day of week (e.g., "Wednesday")
   * - Older: Formatted date (e.g., "Sep 15, 2025")
   */
  const formatRelativeDate = (dateString: string): string => {
    try {
      const date = parseISO(dateString);

      if (isToday(date)) {
        return "Today";
      }

      if (isYesterday(date)) {
        return "Yesterday";
      }

      if (isThisWeek(date, { weekStartsOn: 0 })) {
        // Return day of week (e.g., "Monday", "Tuesday")
        return format(date, "EEEE");
      }

      // For older dates, return a nice formatted date
      return format(date, "MMM d, yyyy");
    } catch (error) {
      console.error("Error formatting date:", error);
      return new Date(dateString).toLocaleDateString();
    }
  };

  const handleNewChat = async () => {
    try {
      // Create new conversation in database
      const newId = await invoke<number>("create_new_conversation");

      // Get the conversation details
      const allConversations = await invoke<any[]>("load_conversations");
      const newConversation = allConversations.find((c) => c.id === newId);

      if (newConversation) {
        // Add to store
        addConversation(newConversation);

        // Set as active
        setActiveConversationId(newId);

        // Clear messages
        clearMessages();

        console.log("Created new conversation:", newId);
      }
    } catch (error) {
      console.error("Failed to create new conversation:", error);
    }
  };

  const handleConversationClick = async (conversationId: number) => {
    try {
      // Set as active
      setActiveConversationId(conversationId);

      // Load messages for this conversation
      const messages = await invoke<any[]>("load_messages", {
        conversationId,
      });

      // Update store with loaded messages
      setMessages(
        messages.map((m) => ({
          role: m.role,
          content: m.content,
          id: m.id,
          timestamp: m.timestamp,
        }))
      );

      console.log(`Loaded ${messages.length} messages for conversation ${conversationId}`);
    } catch (error) {
      console.error("Failed to load conversation:", error);
    }
  };

  const handleDeleteConversation = async (
    conversationId: number,
    e: React.MouseEvent
  ) => {
    // Prevent triggering the conversation click
    e.stopPropagation();

    try {
      // Delete from database
      await invoke("delete_conversation", { conversationId });

      // Remove from store
      removeConversation(conversationId);

      // If this was the active conversation, clear it
      if (activeConversationId === conversationId) {
        setActiveConversationId(null);
        clearMessages();
      }

      console.log(`Deleted conversation ${conversationId}`);
    } catch (error) {
      console.error("Failed to delete conversation:", error);
    }
  };

  return (
    <div className="w-64 bg-gray-900 border-r border-gray-800 flex flex-col">
      {/* New Chat Button */}
      <div className="p-4 border-b border-gray-800">
        <button
          onClick={handleNewChat}
          className="w-full bg-gray-800 hover:bg-gray-700 text-gray-200 font-medium py-2 px-4 rounded-lg transition-colors duration-200 flex items-center justify-center gap-2"
        >
          <span className="text-lg">+</span>
          <span>New Chat</span>
        </button>
      </div>

      {/* Conversation List */}
      <div className="flex-1 overflow-y-auto">
        {conversations.length === 0 ? (
          <div className="p-4 text-center text-gray-500 text-sm">
            No conversations yet
          </div>
        ) : (
          <div className="p-2">
            {conversations.map((conversation) => (
              <div
                key={conversation.id}
                className="relative group mb-1"
              >
                <button
                  onClick={() => handleConversationClick(conversation.id)}
                  className={`w-full text-left p-3 rounded-lg transition-colors duration-150 ${
                    activeConversationId === conversation.id
                      ? "bg-gray-800 text-gray-100"
                      : "text-gray-400 hover:bg-gray-800/50 hover:text-gray-300"
                  }`}
                >
                  {/* Title with Icon */}
                  <div className="flex items-start gap-2 pr-8">
                    <MessageSquare
                      className="w-4 h-4 mt-0.5 flex-shrink-0 text-gray-500"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="font-medium truncate text-sm">
                        {conversation.title}
                      </div>
                      {/* Relative Date */}
                      <div className="text-xs text-gray-500 mt-1">
                        {formatRelativeDate(conversation.created_at)}
                      </div>
                    </div>
                  </div>
                </button>

                {/* Delete button - appears on hover */}
                <button
                  onClick={(e) => handleDeleteConversation(conversation.id, e)}
                  className="absolute right-2 top-3 opacity-0 group-hover:opacity-100 transition-opacity duration-150 p-1 hover:bg-red-600/20 rounded"
                  title="Delete conversation"
                >
                  <svg
                    className="w-4 h-4 text-gray-400 hover:text-red-500"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                    />
                  </svg>
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Settings Button and Status Indicator */}
      <div className="p-4 border-t border-gray-800 space-y-3">
        {/* Status Indicator */}
        <div className="flex items-center gap-2 px-2">
          <SystemStatusIndicator />
          <span className="text-xs text-gray-500">System Status</span>
        </div>

        {/* Settings Button */}
        <button
          onClick={openSettings}
          className="w-full bg-gray-800 hover:bg-gray-700 text-gray-200 font-medium py-2 px-4 rounded-lg transition-colors duration-200 flex items-center justify-center gap-2"
          title="Settings"
        >
          <svg
            className="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
            />
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>
          <span>Settings</span>
        </button>
      </div>
    </div>
  );
};

export default Sidebar;
