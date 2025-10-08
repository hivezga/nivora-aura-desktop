import React, { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useChatStore } from "../store";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./ui/tooltip";

interface ServiceStatusPayload {
  service: "satellite" | "stt";  // "satellite" is the Wyoming Satellite service (handles STT + wake word)
  connected: boolean;
}

interface SystemStatusPayload {
  stt_connected: boolean;
  llm_connected: boolean;
}

const SystemStatusIndicator: React.FC = () => {
  const serviceStatus = useChatStore((state) => state.serviceStatus);
  const setServiceStatus = useChatStore((state) => state.setServiceStatus);
  const setSystemStatus = useChatStore((state) => state.setSystemStatus);

  useEffect(() => {
    // Listen for individual service status events from backend
    const unlistenService = listen<ServiceStatusPayload>("service_status", (event) => {
      const { service, connected } = event.payload;
      // Map "satellite" service to "stt" status (satellite handles both STT and wake word)
      const statusKey = service === "satellite" ? "stt" : service;
      setServiceStatus(statusKey as "wake_word" | "stt" | "llm", connected);
    });

    // Listen for system-wide status updates (from periodic checker)
    const unlistenSystem = listen<SystemStatusPayload>("system_status_update", (event) => {
      setSystemStatus(event.payload);
    });

    // Cleanup listeners on unmount
    return () => {
      unlistenService.then((fn) => fn());
      unlistenSystem.then((fn) => fn());
    };
  }, [setServiceStatus, setSystemStatus]);

  // Determine overall status color
  const getStatusColor = (): string => {
    const { stt, llm } = serviceStatus;

    // Core services are STT and LLM (wake_word is optional)
    if (stt && llm) {
      return "bg-green-500"; // Core services connected
    } else if (!stt && !llm) {
      return "bg-red-500"; // Both core services disconnected
    } else {
      return "bg-yellow-500"; // One core service disconnected
    }
  };

  // Get status text for tooltip
  const getStatusText = (service: "wake_word" | "stt" | "llm", connected: boolean): string => {
    const serviceName =
      service === "wake_word" ? "Wake Word" :
      service === "stt" ? "Speech-to-Text" :
      "LLM Server";
    return `${serviceName}: ${connected ? "🟢 Connected" : "🔴 Disconnected"}`;
  };

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <div className="flex items-center justify-center">
            <div
              className={`w-3 h-3 rounded-full ${getStatusColor()} transition-colors duration-300`}
              aria-label="System Status"
            />
          </div>
        </TooltipTrigger>
        <TooltipContent side="right" className="bg-gray-800 text-gray-200 border-gray-700">
          <div className="text-xs space-y-1">
            <div>{getStatusText("stt", serviceStatus.stt)}</div>
            <div>{getStatusText("llm", serviceStatus.llm)}</div>
            <div className="text-gray-400 text-[10px] mt-1 pt-1 border-t border-gray-700">
              Wake word integrated with STT
            </div>
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
};

export default SystemStatusIndicator;
