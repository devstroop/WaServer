import type {
  HealthResponse,
  QRCodeResponse,
  AuthStatus,
  PhonePairResponse,
  Contact,
  Message,
  SendMessageRequest,
  SendMessageResponse,
  ChatListResponse,
  MessageListResponse,
  LocalAuthStatusResponse,
  LoginRequest,
  LoginResponse,
  RefreshTokenRequest,
  RefreshTokenResponse,
} from "@/types"
import { useSettingsStore } from "@/store"

const API_BASE = "/api/v1"

class ApiClient {
  private getHeaders(): HeadersInit {
    const token = useSettingsStore.getState().getActiveToken()
    return {
      "Content-Type": "application/json",
      ...(token && { Authorization: `Bearer ${token}` }),
    }
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {},
    skipAuth = false
  ): Promise<T> {
    const headers = skipAuth
      ? { "Content-Type": "application/json", ...options.headers }
      : { ...this.getHeaders(), ...options.headers }

    const response = await fetch(`${API_BASE}${endpoint}`, {
      ...options,
      headers,
    })

    if (!response.ok) {
      // If 401 and we have refresh token, try to refresh
      if (response.status === 401 && !skipAuth) {
        const refreshed = await this.tryRefreshToken()
        if (refreshed) {
          // Retry the request with new token
          return this.request<T>(endpoint, options, false)
        }
      }

      const error = await response.json().catch(() => ({
        error: "Request failed",
        message: response.statusText,
      }))
      throw new Error(error.message || error.error || "Request failed")
    }

    return response.json()
  }

  private async tryRefreshToken(): Promise<boolean> {
    const state = useSettingsStore.getState()
    if (!state.authTokens?.refreshToken) {
      return false
    }

    try {
      const response = await this.refreshToken({
        refresh_token: state.authTokens.refreshToken,
      })

      // Update tokens
      state.setAuthTokens({
        ...state.authTokens,
        accessToken: response.access_token,
        expiresAt: Date.now() / 1000 + response.expires_in,
      })

      return true
    } catch {
      // Refresh failed, clear tokens
      state.clearAuthTokens()
      return false
    }
  }

  // Health
  async getHealth(): Promise<HealthResponse> {
    const response = await fetch("/health")
    return response.json()
  }

  // =============================================================================
  // Local Authentication
  // =============================================================================

  async getLocalAuthStatus(): Promise<LocalAuthStatusResponse> {
    return this.request<LocalAuthStatusResponse>("/auth/local-status", {}, true)
  }

  async login(request: LoginRequest): Promise<LoginResponse> {
    return this.request<LoginResponse>(
      "/auth/login",
      {
        method: "POST",
        body: JSON.stringify(request),
      },
      true
    )
  }

  async refreshToken(request: RefreshTokenRequest): Promise<RefreshTokenResponse> {
    return this.request<RefreshTokenResponse>(
      "/auth/refresh",
      {
        method: "POST",
        body: JSON.stringify(request),
      },
      true
    )
  }

  async localLogout(refreshToken: string): Promise<void> {
    await this.request(
      "/auth/local-logout",
      {
        method: "POST",
        body: JSON.stringify({ refresh_token: refreshToken }),
      }
    )
  }

  // =============================================================================
  // WhatsApp Auth
  // =============================================================================

  async getQRCode(): Promise<QRCodeResponse> {
    return this.request<QRCodeResponse>("/auth/qr")
  }

  async getAuthStatus(): Promise<AuthStatus> {
    return this.request<AuthStatus>("/auth/status")
  }

  async requestPhonePairing(phoneNumber: string): Promise<PhonePairResponse> {
    return this.request<PhonePairResponse>("/auth/phone", {
      method: "POST",
      body: JSON.stringify({ phone: phoneNumber }),
    })
  }

  async logout(): Promise<void> {
    await this.request("/auth/logout", { method: "POST" })
  }

  // =============================================================================
  // Contacts
  // =============================================================================

  async getContacts(): Promise<Contact[]> {
    return this.request<Contact[]>("/contacts")
  }

  async searchContacts(query: string): Promise<Contact[]> {
    return this.request<Contact[]>(`/contacts/search?q=${encodeURIComponent(query)}`)
  }

  // =============================================================================
  // Messages
  // =============================================================================

  async getMessages(chatId: string, limit = 50): Promise<Message[]> {
    const response = await this.request<MessageListResponse>(
      `/chats/${encodeURIComponent(chatId)}?limit=${limit}`
    )
    return response.messages
  }

  async sendMessage(request: SendMessageRequest): Promise<SendMessageResponse> {
    // Backend expects multipart form data
    const token = useSettingsStore.getState().getActiveToken()
    const formData = new FormData()
    formData.append("phone", request.phone)
    if (request.text) {
      formData.append("text", request.text)
    }
    
    const response = await fetch(`${API_BASE}/messages`, {
      method: "POST",
      headers: {
        ...(token && { Authorization: `Bearer ${token}` }),
      },
      body: formData,
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({
        error: "Request failed",
        message: response.statusText,
      }))
      throw new Error(error.message || error.error || "Request failed")
    }

    return response.json()
  }

  // =============================================================================
  // Chats
  // =============================================================================

  async getChats(): Promise<Contact[]> {
    const response = await this.request<ChatListResponse>("/chats")
    return response.chats
  }
}

export const apiClient = new ApiClient()
