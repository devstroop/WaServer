// API Types matching Rust backend

export interface HealthResponse {
  status: string
  version: string
  timestamp: number
  uptime_seconds: number
  memory_usage_bytes: number
  whatsapp_connection_status: string
  total_messages_sent: number
  total_auth_attempts: number
  error_count: number
  last_activity?: number
  services: Record<string, {
    status: string
    last_check: number
    response_time_ms?: number
    details?: string
  }>
}

export interface QRCodeResponse {
  qrcode: string
}

export interface AuthStatus {
  authenticated: boolean
  status: "authenticated" | "not_authenticated" | "checking"
  phone_number?: string
}

export interface PhoneAuthRequest {
  phone_number: string
}

export interface PhonePairRequest {
  phone_number: string
}

export interface PhonePairResponse {
  pairing_code: string
  expires_in: number
  message: string
}

// =============================================================================
// Local Authentication Types
// =============================================================================

export interface LocalAuthStatusResponse {
  local_auth_enabled: boolean
  logged_in: boolean
  username?: string
}

export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  access_token: string
  refresh_token: string
  token_type: string
  expires_in: number
  username: string
}

export interface RefreshTokenRequest {
  refresh_token: string
}

export interface RefreshTokenResponse {
  access_token: string
  token_type: string
  expires_in: number
}

// =============================================================================
// Chat/Contact Types
// =============================================================================

// Chat/Contact from backend ChatInfo
export interface Contact {
  id: string
  name: string
  phone_number?: string
  is_group: boolean
  avatar_url?: string
  last_message?: string
  timestamp?: string
  unread_count?: number
}

// Backend ChatListResponse
export interface ChatListResponse {
  chats: Contact[]
  total: number
}

// Message from backend MessageInfo
export interface Message {
  id: string
  from_me: boolean
  sender?: string
  text?: string
  message_type: string
  timestamp?: string
  timestamp_unix?: number
  status?: string
  media_info?: string
}

// Backend MessageListResponse
export interface MessageListResponse {
  chat_id: string
  chat_name?: string
  messages: Message[]
  total: number
  has_more: boolean
}

export interface SendMessageRequest {
  phone: string
  text?: string
}

export interface SendMessageResponse {
  status: string
  message_id: string
}

export interface ChatHistory {
  contact: Contact
  messages: Message[]
}

export interface ApiError {
  error: string
  message: string
  status_code?: number
}

// Store types
export interface AuthState {
  isAuthenticated: boolean
  phoneNumber?: string
  userName?: string
  qrCode?: string
  pairingCode?: string
  loading: boolean
  error?: string
}

export interface ChatState {
  contacts: Contact[]
  selectedContact?: Contact
  messages: Message[]
  loading: boolean
  error?: string
}
