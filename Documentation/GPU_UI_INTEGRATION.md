# GPU Status UI Integration Guide

## Overview

This guide shows how to integrate GPU status display in the Aura Desktop frontend Settings modal.

## Backend API

The backend provides a `get_gpu_info` Tauri command:

```rust
#[tauri::command]
async fn get_gpu_info(
    ollama_sidecar: State<'_, Arc<StdMutex<OllamaSidecar>>>
) -> Result<ollama_sidecar::GpuInfo, AuraError>
```

**Returns:**
```typescript
interface GpuInfo {
  backend: 'Cuda' | 'Rocm' | 'Metal' | 'Cpu'
  available: boolean
  device_name: string | null
}
```

## Frontend Integration

### Step 1: Create Type Definition

Add to your types file (e.g., `src/types.ts`):

```typescript
export interface GpuInfo {
  backend: 'Cuda' | 'Rocm' | 'Metal' | 'Cpu'
  available: boolean
  device_name: string | null
}

export type GpuBackend = GpuInfo['backend']
```

### Step 2: Create GPU Status Hook

Create `src/hooks/useGpuInfo.ts`:

```typescript
import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { GpuInfo } from '../types'

export function useGpuInfo() {
  const [gpuInfo, setGpuInfo] = useState<GpuInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    async function fetchGpuInfo() {
      try {
        const info = await invoke<GpuInfo>('get_gpu_info')
        setGpuInfo(info)
        setError(null)
      } catch (err) {
        console.error('Failed to fetch GPU info:', err)
        setError(err instanceof Error ? err.message : 'Unknown error')
      } finally {
        setLoading(false)
      }
    }

    fetchGpuInfo()
  }, [])

  return { gpuInfo, loading, error }
}
```

### Step 3: Create GPU Status Component

Create `src/components/GpuStatus.tsx`:

```typescript
import React from 'react'
import { useGpuInfo } from '../hooks/useGpuInfo'
import type { GpuBackend } from '../types'

// GPU backend display names
const GPU_BACKEND_NAMES: Record<GpuBackend, string> = {
  Cuda: 'CUDA (NVIDIA)',
  Rocm: 'ROCm (AMD)',
  Metal: 'Metal (Apple)',
  Cpu: 'CPU',
}

// GPU backend icons/colors
const GPU_BACKEND_COLORS: Record<GpuBackend, string> = {
  Cuda: 'text-green-600 dark:text-green-400',
  Rocm: 'text-red-600 dark:text-red-400',
  Metal: 'text-blue-600 dark:text-blue-400',
  Cpu: 'text-gray-600 dark:text-gray-400',
}

export function GpuStatus() {
  const { gpuInfo, loading, error } = useGpuInfo()

  if (loading) {
    return (
      <div className="flex items-center gap-2 text-sm text-gray-500">
        <span className="animate-pulse">Detecting GPU...</span>
      </div>
    )
  }

  if (error || !gpuInfo) {
    return (
      <div className="flex items-center gap-2 text-sm text-gray-500">
        <span>GPU detection failed</span>
      </div>
    )
  }

  const backendName = GPU_BACKEND_NAMES[gpuInfo.backend]
  const colorClass = GPU_BACKEND_COLORS[gpuInfo.backend]

  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
          Inference Device:
        </span>
        <span className={`text-sm font-semibold ${colorClass}`}>
          {backendName}
        </span>
        {gpuInfo.available && (
          <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
            Accelerated
          </span>
        )}
      </div>
      {gpuInfo.device_name && (
        <span className="text-xs text-gray-500 dark:text-gray-400">
          {gpuInfo.device_name}
        </span>
      )}
    </div>
  )
}
```

### Step 4: Add to Settings Modal

Update your Settings component to include the GPU status:

```typescript
import { GpuStatus } from './components/GpuStatus'

function SettingsModal() {
  return (
    <div className="settings-modal">
      {/* Other settings sections */}

      {/* GPU Status Section */}
      <div className="settings-section">
        <h3 className="text-lg font-semibold mb-3">System Information</h3>
        <GpuStatus />
      </div>

      {/* More settings */}
    </div>
  )
}
```

## Alternative: Inline Display

For a simpler inline display without a separate component:

```typescript
import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'

function SettingsModal() {
  const [gpuBackend, setGpuBackend] = useState<string>('Detecting...')

  useEffect(() => {
    invoke<GpuInfo>('get_gpu_info')
      .then(info => {
        const backend = info.available
          ? `${info.backend} (${info.device_name || 'GPU'})`
          : 'CPU (No GPU detected)'
        setGpuBackend(backend)
      })
      .catch(() => setGpuBackend('Unknown'))
  }, [])

  return (
    <div className="settings-section">
      <div className="setting-row">
        <span>Inference Device:</span>
        <span className="font-semibold">{gpuBackend}</span>
      </div>
    </div>
  )
}
```

## Styling Examples

### Minimal Style
```tsx
<div className="bg-gray-100 dark:bg-gray-800 rounded-lg p-3">
  <div className="text-sm">
    <span className="text-gray-600 dark:text-gray-400">Inference: </span>
    <span className="font-medium">{backendName}</span>
  </div>
</div>
```

### Card Style
```tsx
<div className="border border-gray-200 dark:border-gray-700 rounded-lg p-4">
  <h4 className="text-sm font-medium mb-2">Hardware Acceleration</h4>
  <div className="flex items-center justify-between">
    <span className="text-sm">{backendName}</span>
    {gpuInfo.available && (
      <svg className="w-5 h-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
      </svg>
    )}
  </div>
</div>
```

### Badge Style
```tsx
<div className="inline-flex items-center gap-2 px-3 py-2 rounded-full bg-blue-50 dark:bg-blue-900/20">
  <div className="w-2 h-2 rounded-full bg-blue-500 animate-pulse"></div>
  <span className="text-sm font-medium text-blue-700 dark:text-blue-300">
    {backendName}
  </span>
</div>
```

## Performance Indicators

Add visual feedback for GPU performance:

```typescript
function GpuStatusWithPerformance() {
  const { gpuInfo } = useGpuInfo()

  const getPerformanceLabel = (backend: GpuBackend) => {
    switch (backend) {
      case 'Cuda':
      case 'Rocm':
      case 'Metal':
        return '⚡ High Performance'
      case 'Cpu':
        return '🐢 CPU Mode'
    }
  }

  return (
    <div className="flex items-center gap-3">
      <div className="flex-1">
        <div className="text-sm font-medium">{backendName}</div>
        {gpuInfo?.device_name && (
          <div className="text-xs text-gray-500">{gpuInfo.device_name}</div>
        )}
      </div>
      <div className="text-xs font-medium">
        {getPerformanceLabel(gpuInfo.backend)}
      </div>
    </div>
  )
}
```

## Real-Time Updates

If you want to refresh GPU status (e.g., after GPU becomes available):

```typescript
function useGpuInfo(refreshInterval?: number) {
  const [gpuInfo, setGpuInfo] = useState<GpuInfo | null>(null)

  useEffect(() => {
    const fetchGpuInfo = async () => {
      const info = await invoke<GpuInfo>('get_gpu_info')
      setGpuInfo(info)
    }

    fetchGpuInfo()

    if (refreshInterval) {
      const interval = setInterval(fetchGpuInfo, refreshInterval)
      return () => clearInterval(interval)
    }
  }, [refreshInterval])

  return { gpuInfo }
}

// Usage: Refresh every 30 seconds
const { gpuInfo } = useGpuInfo(30000)
```

## Error Handling

Handle potential errors gracefully:

```typescript
function GpuStatusWithErrors() {
  const { gpuInfo, loading, error } = useGpuInfo()

  if (loading) {
    return <LoadingSpinner />
  }

  if (error) {
    return (
      <div className="text-sm text-red-600 dark:text-red-400">
        <span>⚠️ GPU detection failed</span>
        <button
          onClick={() => window.location.reload()}
          className="ml-2 underline"
        >
          Retry
        </button>
      </div>
    )
  }

  // Render GPU info...
}
```

## Accessibility

Ensure the GPU status is accessible:

```tsx
<div
  className="gpu-status"
  role="status"
  aria-label={`Inference device: ${backendName}${gpuInfo.device_name ? `, ${gpuInfo.device_name}` : ''}`}
>
  <span aria-hidden="true">🎯</span>
  <span className="sr-only">GPU Status:</span>
  <span>{backendName}</span>
</div>
```

## Testing

Test GPU status display:

```typescript
// Mock GPU info for testing
const mockGpuInfo: GpuInfo = {
  backend: 'Cuda',
  available: true,
  device_name: 'NVIDIA GeForce RTX 3060'
}

// Test component
describe('GpuStatus', () => {
  it('displays GPU info correctly', () => {
    // Mock the invoke function
    vi.mock('@tauri-apps/api/core', () => ({
      invoke: vi.fn(() => Promise.resolve(mockGpuInfo))
    }))

    render(<GpuStatus />)

    expect(screen.getByText(/CUDA/)).toBeInTheDocument()
    expect(screen.getByText(/RTX 3060/)).toBeInTheDocument()
  })
})
```

## Complete Example

Here's a complete, production-ready implementation:

```tsx
// src/components/Settings/GpuStatusCard.tsx
import React from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'

interface GpuInfo {
  backend: 'Cuda' | 'Rocm' | 'Metal' | 'Cpu'
  available: boolean
  device_name: string | null
}

export function GpuStatusCard() {
  const [gpuInfo, setGpuInfo] = useState<GpuInfo | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    invoke<GpuInfo>('get_gpu_info')
      .then(setGpuInfo)
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [])

  if (loading) {
    return <div className="animate-pulse">Detecting GPU...</div>
  }

  if (!gpuInfo) {
    return <div className="text-gray-500">GPU info unavailable</div>
  }

  const getBackendIcon = () => {
    switch (gpuInfo.backend) {
      case 'Cuda': return '🟢'
      case 'Rocm': return '🔴'
      case 'Metal': return '🔵'
      case 'Cpu': return '⚪'
    }
  }

  const getBackendName = () => {
    switch (gpuInfo.backend) {
      case 'Cuda': return 'NVIDIA CUDA'
      case 'Rocm': return 'AMD ROCm'
      case 'Metal': return 'Apple Metal'
      case 'Cpu': return 'CPU'
    }
  }

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4">
      <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
        Hardware Acceleration
      </h3>

      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-2xl">{getBackendIcon()}</span>
          <div>
            <div className="text-sm font-semibold text-gray-900 dark:text-white">
              {getBackendName()}
            </div>
            {gpuInfo.device_name && (
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {gpuInfo.device_name}
              </div>
            )}
          </div>
        </div>

        {gpuInfo.available && (
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
            Active
          </span>
        )}
      </div>

      <div className="mt-3 text-xs text-gray-500 dark:text-gray-400">
        {gpuInfo.available
          ? '⚡ GPU acceleration is active for faster inference'
          : 'ℹ️ Running on CPU - GPU not detected'}
      </div>
    </div>
  )
}
```

## Summary

The GPU status UI integration provides:
- ✅ Automatic GPU detection display
- ✅ Visual indicators for different backends
- ✅ Device name display when available
- ✅ Graceful error handling
- ✅ Accessible and responsive design
- ✅ Easy to integrate into existing Settings modal

Simply add `<GpuStatusCard />` to your Settings modal and users will see their GPU acceleration status!
