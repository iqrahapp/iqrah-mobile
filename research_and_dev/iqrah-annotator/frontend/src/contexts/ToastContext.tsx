/**
 * Toast Notification Context
 * Provides a clean way to show non-blocking notifications throughout the app
 * Replaces intrusive alert() calls with MUI Snackbar
 */

import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { Snackbar, Alert, IconButton } from '@mui/material';
import { Close } from '@mui/icons-material';

type AlertColor = 'success' | 'info' | 'warning' | 'error';

interface Toast {
  id: string;
  message: string;
  severity: AlertColor;
  duration?: number;
}

interface ToastContextValue {
  /** Show a success toast */
  showSuccess: (message: string, duration?: number) => void;
  /** Show an error toast */
  showError: (message: string, duration?: number) => void;
  /** Show a warning toast */
  showWarning: (message: string, duration?: number) => void;
  /** Show an info toast */
  showInfo: (message: string, duration?: number) => void;
  /** Show a generic toast */
  showToast: (message: string, severity?: AlertColor, duration?: number) => void;
}

const ToastContext = createContext<ToastContextValue | undefined>(undefined);

interface ToastProviderProps {
  children: ReactNode;
  /** Maximum number of toasts to show at once */
  maxToasts?: number;
  /** Default duration in milliseconds */
  defaultDuration?: number;
}

export const ToastProvider: React.FC<ToastProviderProps> = ({
  children,
  maxToasts = 3,
  defaultDuration = 5000,
}) => {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const showToast = useCallback(
    (message: string, severity: AlertColor = 'info', duration: number = defaultDuration) => {
      const id = `toast-${Date.now()}-${Math.random()}`;
      const newToast: Toast = { id, message, severity, duration };

      setToasts((prev) => {
        // Limit number of toasts
        const updated = [...prev, newToast];
        if (updated.length > maxToasts) {
          return updated.slice(-maxToasts);
        }
        return updated;
      });

      // Auto-remove after duration
      if (duration > 0) {
        setTimeout(() => {
          setToasts((prev) => prev.filter((t) => t.id !== id));
        }, duration);
      }
    },
    [maxToasts, defaultDuration]
  );

  const showSuccess = useCallback(
    (message: string, duration?: number) => showToast(message, 'success', duration),
    [showToast]
  );

  const showError = useCallback(
    (message: string, duration?: number) => showToast(message, 'error', duration),
    [showToast]
  );

  const showWarning = useCallback(
    (message: string, duration?: number) => showToast(message, 'warning', duration),
    [showToast]
  );

  const showInfo = useCallback(
    (message: string, duration?: number) => showToast(message, 'info', duration),
    [showToast]
  );

  const handleClose = (id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  return (
    <ToastContext.Provider value={{ showSuccess, showError, showWarning, showInfo, showToast }}>
      {children}
      {/* Render toasts in a stack (bottom to top) */}
      {toasts.map((toast, index) => (
        <Snackbar
          key={toast.id}
          open={true}
          anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
          style={{
            bottom: 16 + index * 70, // Stack toasts vertically
          }}
        >
          <Alert
            severity={toast.severity}
            variant="filled"
            onClose={() => handleClose(toast.id)}
            action={
              <IconButton size="small" color="inherit" onClick={() => handleClose(toast.id)}>
                <Close fontSize="small" />
              </IconButton>
            }
            sx={{
              minWidth: 300,
              maxWidth: 500,
              boxShadow: 3,
            }}
          >
            {toast.message}
          </Alert>
        </Snackbar>
      ))}
    </ToastContext.Provider>
  );
};

/**
 * Hook to use toast notifications
 *
 * Usage:
 * ```tsx
 * const toast = useToast();
 *
 * toast.showSuccess('Saved successfully!');
 * toast.showError('Failed to load data');
 * toast.showWarning('This action cannot be undone');
 * toast.showInfo('Processing in background...');
 * ```
 */
export const useToast = (): ToastContextValue => {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
};

/**
 * Helper function to replace window.alert() with toast notifications
 * Can be used for gradual migration from alert() to toast
 */
export const createToastAlert = (toast: ToastContextValue) => {
  return (message: string, severity: AlertColor = 'info') => {
    toast.showToast(message, severity);
  };
};

/**
 * Helper function to replace window.confirm() with a promise-based dialog
 * Note: This still uses window.confirm() but logs a warning
 * For true non-blocking confirm, use a Dialog component instead
 */
export const createToastConfirm = (toast: ToastContextValue) => {
  return (message: string): Promise<boolean> => {
    console.warn('[ToastConfirm] Using blocking window.confirm(). Consider using a Dialog component.');
    return Promise.resolve(window.confirm(message));
  };
};
