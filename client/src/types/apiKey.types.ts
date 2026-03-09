export type ApiKeyStatus = 'active' | 'revoked' | 'expired';

export interface ApiKey {
  id: string;
  name: string;
  key: string;
  maskedKey: string;
  status: ApiKeyStatus;
  createdAt: string;
  lastUsedAt?: string;
  expiresAt?: string;
  permissions: string[];
}
