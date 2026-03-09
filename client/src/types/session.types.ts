export type SessionStatus = 'connected' | 'disconnected' | 'connecting' | 'qr_pending' | 'error';

export interface Session {
  id: string;
  name: string;
  phoneNumber: string;
  status: SessionStatus;
  connectedAt?: string;
  lastActivity?: string;
  messagesCount: number;
  serverId: string;
  qrCode?: string;
}

export interface QRConnectionState {
  sessionId: string;
  qrCode: string;
  expiresAt: string;
  status: 'pending' | 'scanned' | 'expired';
}
