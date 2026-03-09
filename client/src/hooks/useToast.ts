import { useToastStore, toast, dismissToast, type Toast } from '@/stores/toast.store';

/**
 * Hook for managing toast notifications with Zustand store.
 * Provides backward compatibility with the original useToast API.
 */
function useToast() {
  const toasts = useToastStore((state) => state.toasts);
  const addToast = useToastStore((state) => state.addToast);
  const dismiss = useToastStore((state) => state.dismissToast);

  return {
    toasts,
    toast: (props: Omit<Toast, 'id' | 'open'>) => addToast(props),
    dismiss,
  };
}

export { useToast, toast, dismissToast };
