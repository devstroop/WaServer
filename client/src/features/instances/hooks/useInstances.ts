import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from '@/lib/query-client';
import { instanceService, type CreateInstanceRequest } from '@/services/instance.service';
import { useToast } from '@/hooks/useToast';

export function useInstances() {
  return useQuery({
    queryKey: queryKeys.instances.all,
    queryFn: instanceService.list,
  });
}

export function useInstance(id: string) {
  return useQuery({
    queryKey: queryKeys.instances.detail(id),
    queryFn: () => instanceService.get(id),
    enabled: !!id,
  });
}

/**
 * Derive instance stats from the list response.
 * The backend doesn't have a separate stats endpoint.
 */
export function useInstanceStats() {
  const { data, ...rest } = useInstances();
  
  const stats = data ? {
    total: data.total,
    active: data.instances.filter((i) => i.status === 'active').length,
    inactive: data.instances.filter((i) => i.status === 'inactive').length,
    warming_up: data.instances.filter((i) => i.status === 'warming_up').length,
    error: data.instances.filter((i) => i.status === 'error').length,
  } : null;

  return { data: stats, ...rest };
}

export function useCreateInstance() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (data: CreateInstanceRequest) => instanceService.create(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.instances.all });
      toast({ title: 'Instance created', description: 'Your new instance is ready to use.' });
    },
    onError: () => {
      toast({ title: 'Error', description: 'Failed to create instance.', variant: 'destructive' });
    },
  });
}

export function useDeleteInstance() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (id: string) => instanceService.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.instances.all });
      toast({ title: 'Instance deleted', description: 'Instance has been removed.' });
    },
    onError: () => {
      toast({ title: 'Error', description: 'Failed to delete instance.', variant: 'destructive' });
    },
  });
}

/**
 * Warmup an instance (start its browser)
 */
export function useWarmupInstance() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (id: string) => instanceService.warmup(id),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.instances.detail(id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.instances.all });
      toast({ title: 'Instance warming up', description: 'Browser is starting...' });
    },
    onError: () => {
      toast({ title: 'Error', description: 'Failed to warmup instance.', variant: 'destructive' });
    },
  });
}

/**
 * Reset an instance (wipe session data)
 */
export function useResetInstance() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (id: string) => instanceService.reset(id),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.instances.detail(id) });
      queryClient.invalidateQueries({ queryKey: queryKeys.instances.all });
      toast({ title: 'Instance reset', description: 'Session data has been cleared.' });
    },
    onError: () => {
      toast({ title: 'Error', description: 'Failed to reset instance.', variant: 'destructive' });
    },
  });
}

/**
 * Get instance screenshot
 */
export function useInstanceScreenshot(id: string, enabled = false) {
  return useQuery({
    queryKey: ['instances', id, 'screenshot'],
    queryFn: () => instanceService.getScreenshot(id),
    enabled: enabled && !!id,
    staleTime: 0,
  });
}

/**
 * Get instance config
 */
export function useInstanceConfig(id: string) {
  return useQuery({
    queryKey: ['instances', id, 'config'],
    queryFn: () => instanceService.getConfig(id),
    enabled: !!id,
  });
}

/**
 * Update instance config
 */
export function useUpdateInstanceConfig() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: ({ id, config }: { id: string; config: Parameters<typeof instanceService.updateConfig>[1] }) =>
      instanceService.updateConfig(id, config),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: ['instances', id, 'config'] });
      toast({ title: 'Config updated', description: 'Instance configuration has been saved.' });
    },
    onError: () => {
      toast({ title: 'Error', description: 'Failed to update config.', variant: 'destructive' });
    },
  });
}
