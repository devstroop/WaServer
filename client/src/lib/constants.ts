export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api';

export const POLLING_INTERVAL = 10000;
export const STATUS_POLLING_INTERVAL = 3000;
export const QR_REFRESH_INTERVAL = 20000;

export const ROUTES = {
  LOGIN: '/login',
  DASHBOARD: '/dashboard',
  SERVERS: '/servers',
  SESSIONS: '/sessions',
  MESSAGES: '/messages',
  CAMPAIGNS: '/campaigns',
  CONTACTS: '/contacts',
  ANALYTICS: '/analytics',
  API_KEYS: '/api-keys',
  LOGS: '/logs',
  SETTINGS: '/settings',
  // Legacy routes (kept for compatibility)
  INSTANCES: '/instances',
} as const;

export const STORAGE_KEYS = {
  AUTH_TOKEN: 'was-auth-token',
  THEME: 'was-theme',
  SIDEBAR_COLLAPSED: 'was-sidebar-collapsed',
} as const;

export const INSTANCE_STATUS = {
  ACTIVE: 'active',
  INACTIVE: 'inactive',
  WARMING_UP: 'warming_up',
  ERROR: 'error',
} as const;

export type InstanceStatusType = (typeof INSTANCE_STATUS)[keyof typeof INSTANCE_STATUS];

export const STATUS_DISPLAY: Record<InstanceStatusType, { label: string; variant: 'default' | 'success' | 'warning' | 'destructive' | 'secondary' }> = {
  [INSTANCE_STATUS.ACTIVE]: { label: 'Active', variant: 'success' },
  [INSTANCE_STATUS.INACTIVE]: { label: 'Inactive', variant: 'secondary' },
  [INSTANCE_STATUS.WARMING_UP]: { label: 'Starting', variant: 'warning' },
  [INSTANCE_STATUS.ERROR]: { label: 'Error', variant: 'destructive' },
};

export const FILE_LIMITS = {
  MAX_SIZE: 16 * 1024 * 1024,
  ALLOWED_IMAGE_TYPES: ['image/jpeg', 'image/png', 'image/gif', 'image/webp'],
  ALLOWED_DOCUMENT_TYPES: ['application/pdf', 'application/msword', 'application/vnd.openxmlformats-officedocument.wordprocessingml.document', 'text/plain'],
} as const;
