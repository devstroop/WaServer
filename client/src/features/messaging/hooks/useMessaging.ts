import { useQuery, useMutation, useQueryClient, useInfiniteQuery } from '@tanstack/react-query';
import { queryKeys } from '@/lib/query-client';
import { messagingService } from '@/services/messaging.service';
import { useToast } from '@/hooks/useToast';

interface UseMessagesParams {
  instanceId: string;
  phone: string;
  limit?: number;
}

export function useMessages(params: UseMessagesParams) {
  return useQuery({
    queryKey: queryKeys.messages.list({ instance_id: params.instanceId, chat_id: params.phone }),
    queryFn: () => messagingService.getMessages(params.instanceId, params.phone, { limit: params.limit }),
    enabled: !!params.instanceId && !!params.phone,
  });
}

export function useInfiniteMessages(params: UseMessagesParams) {
  return useInfiniteQuery({
    queryKey: queryKeys.messages.list({ instance_id: params.instanceId, chat_id: params.phone }),
    queryFn: ({ pageParam }) => messagingService.getMessages(
      params.instanceId,
      params.phone,
      { limit: params.limit ?? 50, before: pageParam as string | undefined }
    ),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (lastPage) => {
      if (!lastPage.has_more || lastPage.messages.length === 0) return undefined;
      // Use the oldest message ID as cursor for next page
      const oldestMessage = lastPage.messages[lastPage.messages.length - 1];
      return oldestMessage?.id;
    },
    enabled: !!params.instanceId && !!params.phone,
  });
}

interface SendTextParams {
  instanceId: string;
  phone: string;
  text: string;
}

export function useSendTextMessage() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: ({ instanceId, phone, text }: SendTextParams) =>
      messagingService.sendText(instanceId, phone, text),
    onSuccess: (_, { instanceId, phone }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.messages.list({ instance_id: instanceId, chat_id: phone }) });
      queryClient.invalidateQueries({ queryKey: queryKeys.whatsapp.chats(instanceId) });
      toast({ title: 'Message sent' });
    },
    onError: () => {
      toast({ title: 'Error', description: 'Failed to send message.', variant: 'destructive' });
    },
  });
}

interface SendMediaParams {
  instanceId: string;
  phone: string;
  file: File;
  caption?: string;
}

export function useSendMediaMessage() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: ({ instanceId, phone, file, caption }: SendMediaParams) =>
      messagingService.sendMedia(instanceId, phone, file, caption),
    onSuccess: (_, { instanceId, phone }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.messages.list({ instance_id: instanceId, chat_id: phone }) });
      queryClient.invalidateQueries({ queryKey: queryKeys.whatsapp.chats(instanceId) });
      toast({ title: 'Media sent' });
    },
    onError: () => {
      toast({ title: 'Error', description: 'Failed to send media.', variant: 'destructive' });
    },
  });
}

export function useMarkAsRead() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ instanceId, phone }: { instanceId: string; phone: string }) =>
      messagingService.markAsRead(instanceId, phone),
    onSuccess: (_, { instanceId }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.whatsapp.chats(instanceId) });
    },
  });
}

export function useSendTyping() {
  return useMutation({
    mutationFn: ({ instanceId, phone }: { instanceId: string; phone: string }) =>
      messagingService.sendTyping(instanceId, phone),
  });
}

interface ReactParams {
  instanceId: string;
  phone: string;
  messageId: string;
  emoji: string;
}

export function useReactToMessage() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ instanceId, phone, messageId, emoji }: ReactParams) =>
      messagingService.react(instanceId, phone, messageId, emoji),
    onSuccess: (_, { instanceId, phone }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.messages.list({ instance_id: instanceId, chat_id: phone }) });
    },
  });
}

interface ReplyParams {
  instanceId: string;
  phone: string;
  messageId: string;
  text: string;
}

export function useReplyToMessage() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: ({ instanceId, phone, messageId, text }: ReplyParams) =>
      messagingService.reply(instanceId, phone, messageId, text),
    onSuccess: (_, { instanceId, phone }) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.messages.list({ instance_id: instanceId, chat_id: phone }) });
      queryClient.invalidateQueries({ queryKey: queryKeys.whatsapp.chats(instanceId) });
      toast({ title: 'Reply sent' });
    },
    onError: () => {
      toast({ title: 'Error', description: 'Failed to send reply.', variant: 'destructive' });
    },
  });
}
