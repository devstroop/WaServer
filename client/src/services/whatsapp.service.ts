import { api } from '@/lib/axios';

// Base path - all WhatsApp ops are under instances
const BASE = '/v1/instances';

// === Response Types ===

export interface QrCodeResponse {
  qr_code: string;
  expires_at: string;
}

export interface WhatsAppStatus {
  connected: boolean;
  phone_number: string | null;
  battery_level: number | null;
  is_plugged: boolean;
}

export interface PhoneLoginRequest {
  phone_number: string;
}

export interface PhoneAuthResponse {
  pairing_code: string;
  expires_at: string;
}

export interface ProfileInfo {
  name: string | null;
  status: string | null;
  picture_url: string | null;
}

export interface UpdateProfileRequest {
  name?: string;
  status?: string;
}

export interface Contact {
  id: string;
  name: string;
  phone_number: string;
  profile_picture_url: string | null;
  is_group: boolean;
}

export interface Chat {
  id: string;
  contact: Contact;
  last_message: string | null;
  last_message_time: string | null;
  unread_count: number;
}

export interface GroupInfo {
  id: string;
  name: string;
  description: string | null;
  participants: GroupParticipant[];
  created_at: string;
}

export interface GroupParticipant {
  phone: string;
  name: string | null;
  is_admin: boolean;
}

export interface PresenceInfo {
  phone: string;
  is_online: boolean;
  last_seen: string | null;
}

// === Service ===

export const whatsappService = {
  // === Auth & Linking ===

  // Get WhatsApp connection status
  getStatus: (instanceId: string) =>
    api.get<WhatsAppStatus>(`${BASE}/${instanceId}/status`),

  // Get QR code for linking
  getQrCode: (instanceId: string) =>
    api.get<QrCodeResponse>(`${BASE}/${instanceId}/link/qr`),

  // Link via phone number (pairing code)
  linkPhone: (instanceId: string, data: PhoneLoginRequest) =>
    api.post<PhoneAuthResponse>(`${BASE}/${instanceId}/link/phone`, data),

  // Logout / unlink WhatsApp
  logout: (instanceId: string) =>
    api.del<{ success: boolean }>(`${BASE}/${instanceId}/unlink`),

  // === Profile ===

  getProfile: (instanceId: string) =>
    api.get<ProfileInfo>(`${BASE}/${instanceId}/profile`),

  updateProfile: (instanceId: string, data: UpdateProfileRequest) =>
    api.put<ProfileInfo>(`${BASE}/${instanceId}/profile`, data),

  // === Chats ===

  getChats: (instanceId: string) =>
    api.get<Chat[]>(`${BASE}/${instanceId}/chats`),

  // === Contacts & Groups ===

  getContactInfo: (instanceId: string, contactId: string) =>
    api.get<Contact>(`${BASE}/${instanceId}/contacts/${contactId}`),

  getPresence: (instanceId: string, contactId: string) =>
    api.get<PresenceInfo>(`${BASE}/${instanceId}/contacts/${contactId}/presence`),

  getGroupInfo: (instanceId: string, groupId: string) =>
    api.get<GroupInfo>(`${BASE}/${instanceId}/groups/${groupId}`),

  // === Typing & Read Receipts ===

  sendTyping: (instanceId: string, phone: string) =>
    api.post<{ success: boolean }>(`${BASE}/${instanceId}/messages/${phone}/typing`),

  markAsRead: (instanceId: string, phone: string) =>
    api.post<{ success: boolean }>(`${BASE}/${instanceId}/messages/${phone}/read`),
};
