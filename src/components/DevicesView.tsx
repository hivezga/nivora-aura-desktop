import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { showErrorToast } from "../utils/errorHandler";

interface EntityAttributes {
  friendly_name?: string;
  area_id?: string;
  device_class?: string;
  brightness?: number;
  temperature?: number;
  current_temperature?: number;
  hvac_mode?: string;
  unit_of_measurement?: string;
  [key: string]: any;
}

interface Entity {
  entity_id: string;
  state: string;
  attributes: EntityAttributes;
  last_changed: string;
  last_updated: string;
}

interface HAStatus {
  connected: boolean;
  base_url: string;
  entity_count: number;
}

const DevicesView: React.FC = () => {
  const [haStatus, setHAStatus] = useState<HAStatus | null>(null);
  const [entities, setEntities] = useState<Entity[]>([]);
  const [filteredEntities, setFilteredEntities] = useState<Entity[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [domainFilter, setDomainFilter] = useState<string>("all");
  const [isLoading, setIsLoading] = useState(true);

  // Load Home Assistant status and entities
  useEffect(() => {
    loadHAStatus();
    loadEntities();

    // Refresh entities every 10 seconds
    const interval = setInterval(() => {
      loadEntities();
    }, 10000);

    return () => clearInterval(interval);
  }, []);

  // Filter entities when search or domain filter changes
  useEffect(() => {
    filterEntities();
  }, [entities, searchQuery, domainFilter]);

  const loadHAStatus = async () => {
    try {
      const status = await invoke<HAStatus>("ha_get_status");
      setHAStatus(status);
    } catch (error) {
      console.error("Failed to load Home Assistant status:", error);
    }
  };

  const loadEntities = async () => {
    try {
      const allEntities = await invoke<Entity[]>("ha_get_entities", {
        domain: null,
        area: null,
      });
      setEntities(allEntities);
      setIsLoading(false);
    } catch (error) {
      console.error("Failed to load entities:", error);
      setIsLoading(false);
    }
  };

  const filterEntities = () => {
    let filtered = entities;

    // Filter by domain
    if (domainFilter !== "all") {
      filtered = filtered.filter((e) => e.entity_id.startsWith(domainFilter + "."));
    }

    // Filter by search query
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (e) =>
          e.entity_id.toLowerCase().includes(query) ||
          (e.attributes.friendly_name &&
            e.attributes.friendly_name.toLowerCase().includes(query)) ||
          (e.attributes.area_id && e.attributes.area_id.toLowerCase().includes(query))
      );
    }

    setFilteredEntities(filtered);
  };

  const callService = async (
    domain: string,
    service: string,
    entityId: string,
    data?: any
  ) => {
    try {
      await invoke("ha_call_service", {
        domain,
        service,
        entityId,
        data: data || null,
      });
      // Refresh entities immediately after action
      setTimeout(() => loadEntities(), 500);
    } catch (error) {
      showErrorToast(error, "Failed to control device");
    }
  };

  const toggleEntity = (entity: Entity) => {
    const domain = entity.entity_id.split(".")[0];
    const isOn = entity.state === "on";
    callService(domain, isOn ? "turn_off" : "turn_on", entity.entity_id);
  };

  const setBrightness = (entity: Entity, brightness: number) => {
    callService("light", "turn_on", entity.entity_id, {
      brightness: Math.round((brightness / 100) * 255),
    });
  };

  const setTemperature = (entity: Entity, temperature: number) => {
    callService("climate", "set_temperature", entity.entity_id, {
      temperature,
    });
  };

  // Group entities by area
  const groupByArea = () => {
    const grouped: { [area: string]: Entity[] } = {};

    filteredEntities.forEach((entity) => {
      const area = entity.attributes.area_id || "Unassigned";
      if (!grouped[area]) {
        grouped[area] = [];
      }
      grouped[area].push(entity);
    });

    return grouped;
  };

  const entityGroups = groupByArea();

  // Get unique domains for filter
  const domains = Array.from(new Set(entities.map((e) => e.entity_id.split(".")[0]))).sort();

  if (!haStatus || !haStatus.connected) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center space-y-4">
          <svg
            className="mx-auto h-12 w-12 text-gray-500"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <h3 className="text-lg font-medium text-gray-300">
            Not Connected to Home Assistant
          </h3>
          <p className="text-sm text-gray-500">
            Open Settings and connect to your Home Assistant instance to see your devices.
          </p>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center space-y-4">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-gray-400 mx-auto"></div>
          <p className="text-sm text-gray-400">Loading devices...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex-shrink-0 border-b border-gray-700 bg-gray-900 p-4">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h2 className="text-xl font-semibold text-gray-200">Smart Home Devices</h2>
            <p className="text-xs text-gray-500">
              {haStatus.entity_count} entities discovered
            </p>
          </div>
          <Button
            onClick={loadEntities}
            size="sm"
            variant="outline"
            className="bg-gray-800 hover:bg-gray-700 text-gray-200 border-gray-700"
          >
            <svg
              className="w-4 h-4 mr-1"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
            Refresh
          </Button>
        </div>

        {/* Filters */}
        <div className="flex gap-3">
          <Input
            type="text"
            placeholder="Search devices..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 bg-gray-800 text-gray-100 border-gray-700 focus:ring-gray-600"
          />
          <Select value={domainFilter} onValueChange={setDomainFilter}>
            <SelectTrigger className="w-[180px] bg-gray-800 text-gray-100 border-gray-700">
              <SelectValue placeholder="All Devices" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Devices</SelectItem>
              {domains.map((domain) => (
                <SelectItem key={domain} value={domain}>
                  {domain.charAt(0).toUpperCase() + domain.slice(1)}s
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>

      {/* Entities List */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6">
        {Object.entries(entityGroups).length === 0 ? (
          <div className="text-center py-12">
            <p className="text-gray-500">No devices found</p>
          </div>
        ) : (
          Object.entries(entityGroups)
            .sort(([a], [b]) => a.localeCompare(b))
            .map(([area, areaEntities]) => (
              <div key={area} className="space-y-2">
                <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wide">
                  {area.replace(/_/g, " ")}
                  <span className="ml-2 text-xs font-normal text-gray-600">
                    ({areaEntities.length})
                  </span>
                </h3>
                <div className="grid grid-cols-1 gap-3">
                  {areaEntities.map((entity) => (
                    <EntityCard
                      key={entity.entity_id}
                      entity={entity}
                      onToggle={toggleEntity}
                      onBrightnessChange={setBrightness}
                      onTemperatureChange={setTemperature}
                    />
                  ))}
                </div>
              </div>
            ))
        )}
      </div>
    </div>
  );
};

// Entity Card Component
interface EntityCardProps {
  entity: Entity;
  onToggle: (entity: Entity) => void;
  onBrightnessChange: (entity: Entity, brightness: number) => void;
  onTemperatureChange: (entity: Entity, temperature: number) => void;
}

const EntityCard: React.FC<EntityCardProps> = ({
  entity,
  onToggle,
  onBrightnessChange,
}) => {
  const domain = entity.entity_id.split(".")[0];
  const friendlyName = entity.attributes.friendly_name || entity.entity_id;
  const isOn = entity.state === "on";
  const isUnavailable = entity.state === "unavailable";

  const getEntityIcon = () => {
    switch (domain) {
      case "light":
        return "💡";
      case "switch":
        return "🔌";
      case "climate":
        return "🌡️";
      case "cover":
        return "🪟";
      case "lock":
        return "🔒";
      case "fan":
        return "💨";
      case "sensor":
        return "📊";
      case "binary_sensor":
        return "🔔";
      default:
        return "⚙️";
    }
  };

  const getStateColor = () => {
    if (isUnavailable) return "text-gray-600";
    if (domain === "sensor" || domain === "binary_sensor") return "text-blue-400";
    return isOn ? "text-green-400" : "text-gray-500";
  };

  const renderControls = () => {
    // Read-only sensors
    if (domain === "sensor" || domain === "binary_sensor") {
      return (
        <div className="text-sm text-gray-400">
          <span className={getStateColor()}>
            {entity.state}
            {entity.attributes.unit_of_measurement && ` ${entity.attributes.unit_of_measurement}`}
          </span>
        </div>
      );
    }

    // Climate controls
    if (domain === "climate") {
      return (
        <div className="flex items-center gap-3">
          <div className="text-sm text-gray-400">
            <span className="text-gray-500">Current:</span>{" "}
            {entity.attributes.current_temperature?.toFixed(1)}°
            {entity.attributes.hvac_mode && (
              <span className="ml-2 text-xs text-gray-600">({entity.attributes.hvac_mode})</span>
            )}
          </div>
          <Button
            onClick={() => onToggle(entity)}
            disabled={isUnavailable}
            size="sm"
            variant="outline"
            className="text-xs bg-gray-800 hover:bg-gray-700 border-gray-700"
          >
            Toggle
          </Button>
        </div>
      );
    }

    // Light with brightness
    if (domain === "light" && entity.attributes.brightness !== undefined && isOn) {
      const brightnessPercent = Math.round((entity.attributes.brightness / 255) * 100);
      return (
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2 flex-1">
            <input
              type="range"
              min="0"
              max="100"
              value={brightnessPercent}
              onChange={(e) => onBrightnessChange(entity, parseInt(e.target.value))}
              disabled={isUnavailable}
              className="flex-1"
            />
            <span className="text-xs text-gray-500 w-10">{brightnessPercent}%</span>
          </div>
          <Button
            onClick={() => onToggle(entity)}
            disabled={isUnavailable}
            size="sm"
            className={`text-xs ${
              isOn
                ? "bg-yellow-600 hover:bg-yellow-700"
                : "bg-gray-700 hover:bg-gray-600"
            }`}
          >
            {isOn ? "ON" : "OFF"}
          </Button>
        </div>
      );
    }

    // Default toggle button
    return (
      <Button
        onClick={() => onToggle(entity)}
        disabled={isUnavailable}
        size="sm"
        className={`text-xs ${
          isOn ? "bg-green-600 hover:bg-green-700" : "bg-gray-700 hover:bg-gray-600"
        }`}
      >
        {isUnavailable ? "Unavailable" : isOn ? "ON" : "OFF"}
      </Button>
    );
  };

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg p-3 hover:border-gray-600 transition-colors">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3 flex-1">
          <span className="text-2xl">{getEntityIcon()}</span>
          <div className="flex-1 min-w-0">
            <h4 className="text-sm font-medium text-gray-200 truncate">{friendlyName}</h4>
            <p className="text-xs text-gray-500 truncate">{entity.entity_id}</p>
          </div>
        </div>
        <div className="flex items-center gap-3">{renderControls()}</div>
      </div>
    </div>
  );
};

export default DevicesView;
