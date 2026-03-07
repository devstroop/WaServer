import { create } from 'zustand';
import type { ReactNode } from 'react';

export type ToastVariant = 'default' | 'destructive';
export type ToastAction = ReactNode;

export interface Toast {
  id: string;
  title?: ReactNode;
  description?: ReactNode;
  action?: ToastAction;
  variant?: ToastVariant;
  duration?: number;
  open: boolean;
}

interface ToastState {
  toasts: Toast[];
  addToast: (toast: Omit<Toast, 'id' | 'open'>) => string;
  updateToast: (id: string, toast: Partial<Toast>) => void;
  dismissToast: (id?: string) => void;
  removeToast: (id?: string) => void;
}

const TOAST_LIMIT = 5;
const TOAST_DURATION = 5000;

const toastTimeouts = new Map<string, ReturnType<typeof setTimeout>>();

const scheduleRemoval = (id: string, store: ToastState) => {
  if (toastTimeouts.has(id)) return;
  const timeout = setTimeout(() => {
    toastTimeouts.delete(id);
    store.removeToast(id);
  }, TOAST_DURATION);
  toastTimeouts.set(id, timeout);
};

export const useToastStore = create<ToastState>((set, get) => ({
  toasts: [],

  addToast: (toast) => {
    const id = Math.random().toString(36).slice(2, 11);
    set((state) => ({
      toasts: [{ ...toast, id, open: true }, ...state.toasts].slice(0, TOAST_LIMIT),
    }));
    return id;
  },

  updateToast: (id, toast) => {
    set((state) => ({
      toasts: state.toasts.map((t) => (t.id === id ? { ...t, ...toast } : t)),
    }));
  },

  dismissToast: (id) => {
    const state = get();
    if (id) {
      scheduleRemoval(id, state);
      set((s) => ({
        toasts: s.toasts.map((t) => (t.id === id ? { ...t, open: false } : t)),
      }));
    } else {
      state.toasts.forEach((toast) => scheduleRemoval(toast.id, state));
      set((s) => ({
        toasts: s.toasts.map((t) => ({ ...t, open: false })),
      }));
    }
  },

  removeToast: (id) => {
    if (id === undefined) {
      set({ toasts: [] });
    } else {
      set((state) => ({
        toasts: state.toasts.filter((t) => t.id !== id),
      }));
    }
  },
}));

// Helper hook for easier usage
export const toast = (props: Omit<Toast, 'id' | 'open'>) => {
  return useToastStore.getState().addToast(props);
};

export const dismissToast = (id?: string) => {
  useToastStore.getState().dismissToast(id);
};
