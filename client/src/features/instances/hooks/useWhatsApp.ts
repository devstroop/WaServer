import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { queryKeys } from '@/lib/query-client';
import { whatsappService, type PhoneLoginRequest } from '@/services/whatsapp.service';
import { QR_REFRESH_INTERVAL, STATUS_POLLING_INTERVAL } from '@/lib/constants';
import { useToast } from '@/hooks/useToast';

export function useQrCode(instanceId: string, enabled = true) {
  return useQuery({
    queryKey: queryKeys.whatsapp.qr(instanceId),
    queryFn: () => whatsappService.getQrCode(instanceId),
    enabled: enabled && !!instanceId,
    refetchInterval: QR_REFRESH_INTERVAL,
    staleTime: QR_REFRESH_INTERVAL - 5000,
  });
}

export function useWhatsAppStatus(instanceId: string, enabled = true) {
  return useQuery({
    queryKey: queryKeys.whatsapp.status(instanceId),
    queryFn: () => whatsappService.getStatus(instanceId),
    enabled: enabled && !!instanceId,
    refetchInterval: STATUS_POLLING_INTERVAL,
  });
}

export function useChats(instanceId: string, enabled = true) {
  return useQuery({
    queryKey: queryKeys.whatsapp.chats(instanceId),
    queryFn: () => whatsappService.getChats(instanceId),
    enabled: enabled && !!instanceId,
  });
}

export function useProfile(instanceId: string, enabled = true) {
  return useQuery({
    queryKey: ['whatsapp', instanceId, 'profile'],
    queryFn: () => whatsappService.getProfile(instanceId),
    enabled: enabled && !!instanceId,
  });
}

export function useUpdateProfile() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: ({ instanceId, data }: { instanceId: string; data: Parameters<typeof whatsappService.updateProfile>[1] }) =>
      whatsappService.updateProfile(instanceId, data),
    onSuccess: (_, { instanceId }) => {
      queryClient.invalidateQueries({ queryKey: ['whatsapp', instanceId, 'profile'] });
      toast({ title: 'Profile updated', description: 'Your WhatsApp profile has been updated.' });
    },
    onError: () => {
      toast({ title: 'Error', description: 'Failed to update profile.', variant: 'destructive' });
    },
  });
}

export function useLinkPhone() {
  const { toast } = useToast();

  return useMutation({
    mutationFn: ({ instanceId, data }: { instanceId: string; data: PhoneLoginRequest }) =>
      whatsappService.linkPhone(instanceId, data),
    onSuccess: () => {
      toast({ title: 'Pairing code generated', description: 'Enter the code in WhatsApp to link.' });
    },
    onError: () => {
      toast({ title: 'Error', description: 'Failed to generate pairing code.', variant: 'destructive' });
    },
  });
}

/**
 * Logout from WhatsApp (unlink the session)
 * Named useWhatsAppLogout to avoid conflict with auth useLogout
 */
export function useWhatsAppLogout() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (instanceId: string) => whatsappService.logout(instanceId),
    onSuccess: (_, instanceId) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.whatsapp.status(instanceId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.instances.detail(instanceId) });
      toast({ title: 'Logged out', description: 'WhatsApp session has been terminated.' });
    },
    onError: () => {
      toast({ title: 'Error', description: 'Failed to logout from WhatsApp.', variant: 'destructive' });
    },
  });
}

export function useContactInfo(instanceId: string, contactId: string, enabled = true) {
  return useQuery({
    queryKey: ['whatsapp', instanceId, 'contacts', contactId],
    queryFn: () => whatsappService.getContactInfo(instanceId, contactId),
    enabled: enabled && !!instanceId && !!contactId,
  });
}

export function usePresence(instanceId: string, contactId: string, enabled = true) {
  return useQuery({
    queryKey: ['whatsapp', instanceId, 'contacts', contactId, 'presence'],
    queryFn: () => whatsappService.getPresence(instanceId, contactId),
    enabled: enabled && !!instanceId && !!contactId,
    refetchInterval: 30000,
  });
}

export function useGroupInfo(instanceId: string, groupId: string, enabled = true) {
  return useQuery({
    queryKey: ['whatsapp', instanceId, 'groups', groupId],
    queryFn: () => whatsappService.getGroupInfo(instanceId, groupId),
    enabled: enabled && !!instanceId && !!groupId,
  });
}
