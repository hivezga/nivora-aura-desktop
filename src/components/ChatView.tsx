import React, { useEffect, useRef } from "react";
import ReactMarkdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import InputBar from "./InputBar";
import { useChatStore } from "../store";

const ChatView: React.FC = () => {
  const messages = useChatStore((state) => state.messages);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  return (
    <div className="flex-1 flex flex-col bg-gray-950">
      <div className="flex-1 overflow-y-auto p-4">
        {messages.length === 0 ? (
          <div className="h-full flex items-center justify-center">
            <h1 className="text-3xl font-light text-gray-300">
              Welcome to Nivora Aura
            </h1>
          </div>
        ) : (
          <div className="max-w-4xl mx-auto space-y-4">
            {messages.map((message, index) => (
              <div
                key={index}
                className={`flex ${
                  message.role === "user" ? "justify-end" : "justify-start"
                }`}
              >
                <div
                  className={`max-w-[80%] rounded-lg px-4 py-3 ${
                    message.role === "user"
                      ? "bg-blue-600 text-white"
                      : "bg-gray-800 text-gray-100"
                  }`}
                >
                  {message.role === "user" ? (
                    // User messages: render as plain text (no markdown needed)
                    <p className="whitespace-pre-wrap break-words">
                      {message.content}
                    </p>
                  ) : (
                    // Assistant messages: render with markdown
                    <div className="markdown-content">
                      <ReactMarkdown rehypePlugins={[rehypeHighlight]}>
                        {message.content}
                      </ReactMarkdown>
                    </div>
                  )}
                </div>
              </div>
            ))}
            <div ref={messagesEndRef} />
          </div>
        )}
      </div>
      <InputBar />
    </div>
  );
};

export default ChatView;
