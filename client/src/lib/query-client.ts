import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30 * 1000,
      gcTime: 5 * 60 * 1000,
      retry: (failureCount, error) => {
        if (error instanceof Error && 'status' in error) {
          const status = (error as { status: number }).status;
          if (status === 401 || status === 403 || status === 404) return false;
        }
        return failureCount < 3;
      },
      refetchOnWindowFocus: false,
    },
    mutations: {
      retry: false,
    },
  },
});

export const queryKeys = {
  instances: {
    all: ['instances'] as const,
    detail: (id: string) => ['instances', id] as const,
    stats: ['instances', 'stats'] as const,
  },
  whatsapp: {
    qr: (instanceId: string) => ['whatsapp', instanceId, 'qr'] as const,
    status: (instanceId: string) => ['whatsapp', instanceId, 'status'] as const,
    contacts: (instanceId: string) => ['whatsapp', instanceId, 'contacts'] as const,
    chats: (instanceId: string) => ['whatsapp', instanceId, 'chats'] as const,
  },
  messages: {
    list: (params: { instance_id: string; chat_id?: string | undefined }) =>
      ['messages', params.instance_id, params.chat_id ?? 'all'] as const,
  },
  auth: {
    user: ['auth', 'user'] as const,
  },
  health: ['health'] as const,
};
