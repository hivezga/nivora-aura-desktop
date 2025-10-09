/**
 * Unified error handling for Nivora Aura frontend
 *
 * This module provides centralized error handling with user-friendly
 * toast notifications for all Tauri command errors.
 */

import toast from 'react-hot-toast';

/**
 * Parse error messages from various sources (Tauri errors, exceptions, etc.)
 */
function parseErrorMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }

  if (error instanceof Error) {
    return error.message;
  }

  if (error && typeof error === 'object' && 'message' in error) {
    return String(error.message);
  }

  return 'An unknown error occurred';
}

/**
 * Determine error type from message for better user feedback
 */
function getErrorType(message: string): 'network' | 'config' | 'file' | 'general' {
  const lowerMessage = message.toLowerCase();

  if (lowerMessage.includes('connection') || lowerMessage.includes('network') ||
      lowerMessage.includes('http') || lowerMessage.includes('server') ||
      lowerMessage.includes('offline')) {
    return 'network';
  }

  if (lowerMessage.includes('config') || lowerMessage.includes('settings') ||
      lowerMessage.includes('invalid')) {
    return 'config';
  }

  if (lowerMessage.includes('file') || lowerMessage.includes('model') ||
      lowerMessage.includes('not found') || lowerMessage.includes('missing')) {
    return 'file';
  }

  return 'general';
}

/**
 * Get user-friendly error message based on error type
 */
function getFriendlyMessage(message: string): string {
  const type = getErrorType(message);

  switch (type) {
    case 'network':
      return `Connection Error: ${message}. Please check your server connection and try again.`;
    case 'config':
      return `Configuration Error: ${message}. Please check your settings.`;
    case 'file':
      return `File Error: ${message}. Please ensure all required models are downloaded.`;
    default:
      return message;
  }
}

/**
 * Display error toast notification
 */
export function showErrorToast(error: unknown, context?: string): void {
  const rawMessage = parseErrorMessage(error);
  const friendlyMessage = getFriendlyMessage(rawMessage);
  const fullMessage = context
    ? `${context}: ${friendlyMessage}`
    : friendlyMessage;

  console.error('[Error]', context || 'Unknown context', error);

  toast.error(fullMessage, {
    duration: 5000,
    position: 'top-right',
    style: {
      background: '#ef4444',
      color: '#fff',
      maxWidth: '500px',
    },
  });
}

/**
 * Display success toast notification
 */
export function showSuccessToast(message: string): void {
  toast.success(message, {
    duration: 3000,
    position: 'top-right',
    style: {
      background: '#10b981',
      color: '#fff',
    },
  });
}

/**
 * Display info toast notification
 */
export function showInfoToast(message: string): void {
  toast(message, {
    duration: 3000,
    position: 'top-right',
    icon: 'ℹ️',
  });
}

/**
 * Wrapper for Tauri invoke calls with automatic error handling
 *
 * @example
 * const result = await safeInvoke<string>('greet', { name: 'World' }, 'Greeting');
 */
export async function safeInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
  context?: string
): Promise<T | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<T>(command, args);
  } catch (error) {
    showErrorToast(error, context || `Command '${command}' failed`);
    return null;
  }
}

/**
 * Wrapper for async operations with automatic error handling
 *
 * @example
 * await handleAsync(
 *   async () => { await someOperation(); },
 *   'Operation name'
 * );
 */
export async function handleAsync<T>(
  operation: () => Promise<T>,
  context: string
): Promise<T | null> {
  try {
    return await operation();
  } catch (error) {
    showErrorToast(error, context);
    return null;
  }
}

/**
 * Type guard to check if a value is an error
 */
export function isError(value: unknown): value is Error {
  return value instanceof Error;
}
