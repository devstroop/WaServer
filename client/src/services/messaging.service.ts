import { api } from '@/lib/axios';

// Base path - all messaging is under instances
const BASE = '/v1/instances';

// === Types ===

export interface Message {
  id: string;
  from_me: boolean;
  sender: string;
  content: string;
  message_type: 'text' | 'image' | 'video' | 'audio' | 'document' | 'sticker';
  media_url: string | null;
  timestamp: string;
  status: 'pending' | 'sent' | 'delivered' | 'read' | 'failed';
  quoted_message_id: string | null;
}

export interface SendMessageRequest {
  phone: string;
  text?: string;
  file?: File;
  caption?: string;
}

export interface SendMessageResponse {
  message_id: string;
  status: string;
}

export interface ReactionRequest {
  emoji: string;
}

export interface ReplyRequest {
  text: string;
}

export interface MessageListResponse {
  messages: Message[];
  has_more: boolean;
}

export interface MessageQueryParams {
  limit?: number | undefined;
  before?: string | undefined;
}

// === Service ===

export const messagingService = {
  // Get messages for a specific chat
  getMessages: (instanceId: string, phone: string, params?: MessageQueryParams) => {
    const searchParams = new URLSearchParams();
    if (params?.limit) searchParams.append('limit', String(params.limit));
    if (params?.before) searchParams.append('before', params.before);
    const query = searchParams.toString();
    return api.get<MessageListResponse>(
      `${BASE}/${instanceId}/messages/${phone}${query ? `?${query}` : ''}`
    );
  },

  // Send text message
  sendText: (instanceId: string, phone: string, text: string) =>
    api.post<SendMessageResponse>(`${BASE}/${instanceId}/send`, { phone, text }),

  // Send media message
  sendMedia: (instanceId: string, phone: string, file: File, caption?: string) => {
    const formData = new FormData();
    formData.append('phone', phone);
    formData.append('file', file);
    if (caption) formData.append('caption', caption);
    return api.upload<SendMessageResponse>(`${BASE}/${instanceId}/send`, formData);
  },

  // React to a message
  react: (instanceId: string, phone: string, messageId: string, emoji: string) =>
    api.post<{ success: boolean }>(
      `${BASE}/${instanceId}/messages/${phone}/${messageId}/react`,
      { emoji }
    ),

  // Reply to a message (quote)
  reply: (instanceId: string, phone: string, messageId: string, text: string) =>
    api.post<SendMessageResponse>(
      `${BASE}/${instanceId}/messages/${phone}/${messageId}/reply`,
      { text }
    ),

  // Mark messages as read (moved from whatsapp service for consistency)
  markAsRead: (instanceId: string, phone: string) =>
    api.post<{ success: boolean }>(`${BASE}/${instanceId}/messages/${phone}/read`),

  // Send typing indicator
  sendTyping: (instanceId: string, phone: string) =>
    api.post<{ success: boolean }>(`${BASE}/${instanceId}/messages/${phone}/typing`),
};
