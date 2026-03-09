# Frontend API Integration Guide

**Version:** 0.3.0  
**Base URL:** `http://localhost:3000`  
**Last Updated:** March 7, 2026

---

## Table of Contents

1. [Authentication](#authentication)
2. [API Overview](#api-overview)
3. [Health APIs](#health-apis)
4. [Instance APIs](#instance-apis)
5. [WhatsApp APIs](#whatsapp-apis)
6. [Messaging APIs](#messaging-apis)
7. [TypeScript Types](#typescript-types)
8. [React Hooks Examples](#react-hooks-examples)
9. [Error Handling](#error-handling)
10. [API Service Implementation](#api-service-implementation)

---

## Authentication

All API requests (except health endpoints) require Bearer token authentication.

```typescript
const API_BASE = 'http://localhost:3000';
const API_KEY = 'your-secret-key';

const headers = {
  'Authorization': `Bearer ${API_KEY}`,
  'Content-Type': 'application/json',
};
```

---

## API Overview

| Category | Endpoints | Auth Required |
|----------|-----------|---------------|
| Health | 4 | No |
| Instances | 5 | Yes |
| WhatsApp | 5 | Yes |
| Messaging | 4 | Yes |
| **Total** | **18** | |

---

## Health APIs

### GET /api/health
**Description:** Overall system health check

```typescript
interface HealthResponse {
  status: 'healthy' | 'unhealthy';
  timestamp: number;
  version: string;
  uptime_seconds: number;
  instances_count: number;
  services: {
    [key: string]: {
      status: string;
      last_check: number;
      response_time_ms?: number;
      details?: string;
    };
  };
}
```

### GET /api/ready
**Description:** Kubernetes readiness probe

```typescript
interface StatusResponse {
  status: 'ready' | 'not_ready';
}
```

### GET /api/live
**Description:** Kubernetes liveness probe

```typescript
interface StatusResponse {
  status: 'alive';
}
```

### GET /api/metrics
**Description:** System and instance metrics

```typescript
interface MetricsResponse {
  timestamp: number;
  uptime_seconds: number;
  memory_usage_bytes: number;
  instances_count: number;
  instances: InstanceMetrics[];
}

interface InstanceMetrics {
  id: string;
  status: 'sleeping' | 'warming_up' | 'active' | 'error';
  authorized: boolean;
  total_messages_sent: number;
  error_count: number;
}
```

---

## Instance APIs

### GET /api/v1/instances
**Description:** List all WhatsApp instances

**Response:**
```typescript
interface InstanceListResponse {
  instances: InstanceInfo[];
  total: number;
}

interface InstanceInfo {
  id: string;              // UUID
  name: string;            // Instance name
  phone_number?: string;   // E.164 format
  status: InstanceStatus;
  authorized: boolean;
  created_at: string;      // ISO 8601
  updated_at: string;      // ISO 8601
}

type InstanceStatus = 'sleeping' | 'warming_up' | 'active' | 'error';
```

**Frontend Usage:**
```typescript
const fetchInstances = async (): Promise<InstanceListResponse> => {
  const response = await fetch(`${API_BASE}/api/v1/instances`, { headers });
  return response.json();
};
```

---

### POST /api/v1/instances
**Description:** Create a new WhatsApp instance

**Request:**
```typescript
interface CreateInstanceRequest {
  name: string;           // Required: Unique instance name
  phone_number?: string;  // Optional: For phone linking
}
```

**Response (201):**
```typescript
interface CreateInstanceResponse {
  instance_id: string;
  name: string;
  phone_number?: string;
  status: InstanceStatus;
  created_at: string;
}
```

**Frontend Usage:**
```typescript
const createInstance = async (name: string, phone?: string) => {
  const response = await fetch(`${API_BASE}/api/v1/instances`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ name, phone_number: phone }),
  });
  return response.json();
};
```

---

### GET /api/v1/instances/{instance_id}
**Description:** Get instance details

**Path Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| instance_id | string | Instance UUID or phone number |

**Response:**
```typescript
interface InstanceInfo {
  id: string;
  name: string;
  phone_number?: string;
  status: InstanceStatus;
  authorized: boolean;
  created_at: string;
  updated_at: string;
}
```

---

### DELETE /api/v1/instances/{instance_id}
**Description:** Delete an instance

**Query Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| delete_data | boolean | false | Delete all instance data |

**Response:**
```typescript
interface DeleteInstanceResponse {
  message: string;
  instance_id: string;
  data_deleted: boolean;
}
```

**Frontend Usage:**
```typescript
const deleteInstance = async (id: string, deleteData = false) => {
  const response = await fetch(
    `${API_BASE}/api/v1/instances/${id}?delete_data=${deleteData}`,
    { method: 'DELETE', headers }
  );
  return response.json();
};
```

---

### POST /api/v1/instances/{instance_id}/warmup
**Description:** Pre-warm instance browser

**Response:**
```typescript
interface InstanceActionResponse {
  success: boolean;
  message: string;
  instance_id: string;
}
```

---

## WhatsApp APIs

### GET /api/v1/instances/{instance_id}/status
**Description:** Get WhatsApp authentication status

**Response:**
```typescript
interface WhatsAppStatusResponse {
  instance_id: string;
  phone_number?: string;
  status: 'sleeping' | 'warming_up' | 'active' | 'error';
  authorized: boolean;
}
```

**Frontend Usage:**
```typescript
const getStatus = async (instanceId: string) => {
  const response = await fetch(
    `${API_BASE}/api/v1/instances/${instanceId}/status`,
    { headers }
  );
  return response.json();
};
```

---

### GET /api/v1/instances/{instance_id}/link/qr
**Description:** Get QR code for WhatsApp linking

**Response:** PNG image (image/png)

**Frontend Usage:**
```typescript
const QrCodeDisplay: React.FC<{ instanceId: string }> = ({ instanceId }) => {
  const qrUrl = `${API_BASE}/api/v1/instances/${instanceId}/link/qr`;
  
  return (
    <img 
      src={qrUrl}
      alt="WhatsApp QR Code"
      onError={() => console.log('QR not available')}
    />
  );
};

// Or fetch as blob
const fetchQrCode = async (instanceId: string): Promise<string> => {
  const response = await fetch(
    `${API_BASE}/api/v1/instances/${instanceId}/link/qr`,
    { headers }
  );
  const blob = await response.blob();
  return URL.createObjectURL(blob);
};
```

**Error Responses:**
- `409 Conflict`: Already authorized
- `503 Service Unavailable`: Browser not running

---

### POST /api/v1/instances/{instance_id}/link/phone
**Description:** Initiate phone number linking (alternative to QR)

**Response:**
```typescript
interface PhoneLinkResponse {
  success: boolean;
  phone_number: string;
  linking_code?: string;  // 8-digit code to enter on phone
}
```

**Frontend Usage:**
```typescript
const linkPhone = async (instanceId: string) => {
  const response = await fetch(
    `${API_BASE}/api/v1/instances/${instanceId}/link/phone`,
    { method: 'POST', headers }
  );
  return response.json();
};
```

---

### DELETE /api/v1/instances/{instance_id}/unlink
**Description:** Disconnect WhatsApp session

**Response:**
```typescript
interface UnlinkResponse {
  success: boolean;
  message: string;
}
```

---

### GET /api/v1/instances/{instance_id}/profile
**Description:** Get WhatsApp profile info (name, about, picture)

**Response:**
```typescript
interface ProfileInfo {
  name?: string;
  about?: string;
  picture_url?: string;
}
```

---

## Messaging APIs

### POST /api/v1/instances/{instance_id}/send
**Description:** Send text message or file attachment

**Query Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| phone | string | Yes | Recipient phone (e.g., 919876543210) |
| text | string | No* | Message text |

*At least one of `text` or `file` must be provided.

**Request (Multipart for file upload):**
```typescript
// Text-only message
const sendTextMessage = async (
  instanceId: string,
  phone: string,
  text: string
) => {
  const response = await fetch(
    `${API_BASE}/api/v1/instances/${instanceId}/send?phone=${phone}&text=${encodeURIComponent(text)}`,
    { method: 'POST', headers }
  );
  return response.json();
};

// File attachment
const sendFileMessage = async (
  instanceId: string,
  phone: string,
  file: File,
  caption?: string
) => {
  const formData = new FormData();
  formData.append('file', file);
  
  const url = new URL(`${API_BASE}/api/v1/instances/${instanceId}/send`);
  url.searchParams.set('phone', phone);
  if (caption) url.searchParams.set('text', caption);
  
  const response = await fetch(url.toString(), {
    method: 'POST',
    headers: { 'Authorization': `Bearer ${API_KEY}` }, // No Content-Type for FormData
    body: formData,
  });
  return response.json();
};
```

**Response:**
```typescript
interface SendMessageResponse {
  success: boolean;
  message_id?: string;
  timestamp: string;
  recipient: string;
  has_attachment: boolean;
}
```

---

### GET /api/v1/instances/{instance_id}/chats
**Description:** List all visible chats

**Response:**
```typescript
interface ChatListResponse {
  chats: ChatInfo[];
  total: number;
}

interface ChatInfo {
  id: string;
  name: string;
  phone_number?: string;
  is_group: boolean;
  last_message?: string;
  last_message_time?: string;
  unread_count: number;
  avatar_url?: string;
}
```

---

### GET /api/v1/instances/{instance_id}/messages/{phone}
**Description:** Get messages from a conversation

**Path Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| instance_id | string | Instance UUID |
| phone | string | Phone number (e.g., 919876543210) |

**Query Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| limit | number | 50 | Max messages to retrieve |
| load_more | boolean | false | Load older messages |

**Response:**
```typescript
interface MessageListResponse {
  messages: Message[];
  total: number;
  has_more: boolean;
}

interface Message {
  id: string;
  content: string;
  timestamp: string;
  is_outgoing: boolean;
  is_read: boolean;
  sender_name?: string;
  sender_phone?: string;
  media_type?: 'image' | 'video' | 'audio' | 'document';
  media_url?: string;
}
```

---

### POST /api/v1/instances/{instance_id}/messages/{phone}/typing
**Description:** Send typing indicator

**Request:**
```typescript
interface TypingRequest {
  state: 'composing' | 'paused';
}
```

**Response:**
```typescript
interface TypingResponse {
  success: boolean;
  chat_id: string;
  state: 'composing' | 'paused';
}
```

---

## TypeScript Types

Complete type definitions for your frontend:

```typescript
// src/types/api.ts

// ============ Status Types ============
export type InstanceStatus = 'sleeping' | 'warming_up' | 'active' | 'error';
export type TypingState = 'composing' | 'paused';

// ============ Instance Types ============
export interface InstanceInfo {
  id: string;
  name: string;
  phone_number?: string;
  status: InstanceStatus;
  authorized: boolean;
  created_at: string;
  updated_at: string;
}

export interface InstanceListResponse {
  instances: InstanceInfo[];
  total: number;
}

export interface CreateInstanceRequest {
  name: string;
  phone_number?: string;
}

export interface CreateInstanceResponse {
  instance_id: string;
  name: string;
  phone_number?: string;
  status: InstanceStatus;
  created_at: string;
}

export interface DeleteInstanceResponse {
  message: string;
  instance_id: string;
  data_deleted: boolean;
}

// ============ WhatsApp Types ============
export interface WhatsAppStatusResponse {
  instance_id: string;
  phone_number?: string;
  status: string;
  authorized: boolean;
}

export interface PhoneLinkResponse {
  success: boolean;
  phone_number: string;
  linking_code?: string;
}

export interface ProfileInfo {
  name?: string;
  about?: string;
  picture_url?: string;
}

// ============ Chat Types ============
export interface ChatInfo {
  id: string;
  name: string;
  phone_number?: string;
  is_group: boolean;
  last_message?: string;
  last_message_time?: string;
  unread_count: number;
  avatar_url?: string;
}

export interface ChatListResponse {
  chats: ChatInfo[];
  total: number;
}

// ============ Message Types ============
export interface Message {
  id: string;
  content: string;
  timestamp: string;
  is_outgoing: boolean;
  is_read: boolean;
  sender_name?: string;
  sender_phone?: string;
  media_type?: 'image' | 'video' | 'audio' | 'document';
  media_url?: string;
}

export interface MessageListResponse {
  messages: Message[];
  total: number;
  has_more: boolean;
}

export interface SendMessageResponse {
  success: boolean;
  message_id?: string;
  timestamp: string;
  recipient: string;
  has_attachment: boolean;
}

// ============ Health Types ============
export interface HealthResponse {
  status: string;
  timestamp: number;
  version: string;
  uptime_seconds: number;
  instances_count: number;
  services: Record<string, ServiceHealth>;
}

export interface ServiceHealth {
  status: string;
  last_check: number;
  response_time_ms?: number;
  details?: string;
}

export interface MetricsResponse {
  timestamp: number;
  uptime_seconds: number;
  memory_usage_bytes: number;
  instances_count: number;
  instances: InstanceMetrics[];
}

export interface InstanceMetrics {
  id: string;
  status: string;
  authorized: boolean;
  total_messages_sent: number;
  error_count: number;
}

// ============ Error Types ============
export interface ApiError {
  error: string;
  message: string;
}
```

---

## React Hooks Examples

### useInstances Hook

```typescript
// src/hooks/useInstances.ts
import { useState, useEffect, useCallback } from 'react';
import { instanceApi } from '../services/api';
import type { InstanceInfo } from '../types/api';

export const useInstances = (pollInterval = 10000) => {
  const [instances, setInstances] = useState<InstanceInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchInstances = useCallback(async () => {
    try {
      const data = await instanceApi.list();
      setInstances(data.instances);
      setError(null);
    } catch (err) {
      setError('Failed to fetch instances');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchInstances();
    const interval = setInterval(fetchInstances, pollInterval);
    return () => clearInterval(interval);
  }, [fetchInstances, pollInterval]);

  return { instances, loading, error, refetch: fetchInstances };
};
```

### useInstanceStatus Hook

```typescript
// src/hooks/useInstanceStatus.ts
import { useState, useEffect, useCallback } from 'react';
import { whatsappApi } from '../services/api';
import type { WhatsAppStatusResponse } from '../types/api';

export const useInstanceStatus = (instanceId: string, pollInterval = 5000) => {
  const [status, setStatus] = useState<WhatsAppStatusResponse | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchStatus = useCallback(async () => {
    if (!instanceId) return;
    try {
      const data = await whatsappApi.getStatus(instanceId);
      setStatus(data);
    } catch (err) {
      // Handle error
    } finally {
      setLoading(false);
    }
  }, [instanceId]);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, pollInterval);
    return () => clearInterval(interval);
  }, [fetchStatus, pollInterval]);

  return { status, loading, refetch: fetchStatus };
};
```

---

## Error Handling

### HTTP Status Codes

| Code | Meaning | Action |
|------|---------|--------|
| 200 | Success | Process response |
| 201 | Created | Resource created |
| 400 | Bad Request | Show validation error |
| 401 | Unauthorized | Redirect to login |
| 404 | Not Found | Show not found message |
| 409 | Conflict | Show conflict message |
| 503 | Service Unavailable | Show retry message |
| 500 | Server Error | Show generic error |

### Error Response Format

```typescript
interface ApiError {
  error: string;   // Error code (e.g., "instance_not_found")
  message: string; // Human-readable message
}
```

### Error Handling Example

```typescript
const handleApiError = async (response: Response) => {
  if (!response.ok) {
    const error: ApiError = await response.json();
    
    switch (response.status) {
      case 401:
        // Redirect to login
        window.location.href = '/login';
        break;
      case 404:
        throw new Error(`Not found: ${error.message}`);
      case 409:
        throw new Error(`Conflict: ${error.message}`);
      case 503:
        throw new Error('Service temporarily unavailable. Please try again.');
      default:
        throw new Error(error.message || 'An unexpected error occurred');
    }
  }
  return response;
};
```

---

## API Service Implementation

Complete API service for your React frontend:

```typescript
// src/services/api.ts
const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000';
const API_KEY = import.meta.env.VITE_API_KEY || '';

const getHeaders = () => ({
  'Authorization': `Bearer ${API_KEY}`,
  'Content-Type': 'application/json',
});

const handleResponse = async <T>(response: Response): Promise<T> => {
  if (!response.ok) {
    const error = await response.json().catch(() => ({ message: 'Request failed' }));
    throw new Error(error.message || `HTTP ${response.status}`);
  }
  return response.json();
};

// ============ Instance API ============
export const instanceApi = {
  list: async () => {
    const res = await fetch(`${API_BASE}/api/v1/instances`, { headers: getHeaders() });
    return handleResponse<InstanceListResponse>(res);
  },

  get: async (id: string) => {
    const res = await fetch(`${API_BASE}/api/v1/instances/${id}`, { headers: getHeaders() });
    return handleResponse<InstanceInfo>(res);
  },

  create: async (data: CreateInstanceRequest) => {
    const res = await fetch(`${API_BASE}/api/v1/instances`, {
      method: 'POST',
      headers: getHeaders(),
      body: JSON.stringify(data),
    });
    return handleResponse<CreateInstanceResponse>(res);
  },

  delete: async (id: string, deleteData = false) => {
    const res = await fetch(`${API_BASE}/api/v1/instances/${id}?delete_data=${deleteData}`, {
      method: 'DELETE',
      headers: getHeaders(),
    });
    return handleResponse<DeleteInstanceResponse>(res);
  },

  warmup: async (id: string) => {
    const res = await fetch(`${API_BASE}/api/v1/instances/${id}/warmup`, {
      method: 'POST',
      headers: getHeaders(),
    });
    return handleResponse<InstanceActionResponse>(res);
  },
};

// ============ WhatsApp API ============
export const whatsappApi = {
  getStatus: async (instanceId: string) => {
    const res = await fetch(`${API_BASE}/api/v1/instances/${instanceId}/status`, {
      headers: getHeaders(),
    });
    return handleResponse<WhatsAppStatusResponse>(res);
  },

  getQrCodeUrl: (instanceId: string) => {
    return `${API_BASE}/api/v1/instances/${instanceId}/link/qr`;
  },

  linkPhone: async (instanceId: string) => {
    const res = await fetch(`${API_BASE}/api/v1/instances/${instanceId}/link/phone`, {
      method: 'POST',
      headers: getHeaders(),
    });
    return handleResponse<PhoneLinkResponse>(res);
  },

  unlink: async (instanceId: string) => {
    const res = await fetch(`${API_BASE}/api/v1/instances/${instanceId}/unlink`, {
      method: 'DELETE',
      headers: getHeaders(),
    });
    return handleResponse<{ success: boolean }>(res);
  },

  getProfile: async (instanceId: string) => {
    const res = await fetch(`${API_BASE}/api/v1/instances/${instanceId}/profile`, {
      headers: getHeaders(),
    });
    return handleResponse<ProfileInfo>(res);
  },
};

// ============ Messaging API ============
export const messagingApi = {
  sendMessage: async (instanceId: string, phone: string, text: string) => {
    const url = `${API_BASE}/api/v1/instances/${instanceId}/send?phone=${phone}&text=${encodeURIComponent(text)}`;
    const res = await fetch(url, {
      method: 'POST',
      headers: getHeaders(),
    });
    return handleResponse<SendMessageResponse>(res);
  },

  sendFile: async (instanceId: string, phone: string, file: File, caption?: string) => {
    const formData = new FormData();
    formData.append('file', file);

    const url = new URL(`${API_BASE}/api/v1/instances/${instanceId}/send`);
    url.searchParams.set('phone', phone);
    if (caption) url.searchParams.set('text', caption);

    const res = await fetch(url.toString(), {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${API_KEY}` },
      body: formData,
    });
    return handleResponse<SendMessageResponse>(res);
  },

  getChats: async (instanceId: string) => {
    const res = await fetch(`${API_BASE}/api/v1/instances/${instanceId}/chats`, {
      headers: getHeaders(),
    });
    return handleResponse<ChatListResponse>(res);
  },

  getMessages: async (instanceId: string, phone: string, limit = 50) => {
    const res = await fetch(
      `${API_BASE}/api/v1/instances/${instanceId}/messages/${phone}?limit=${limit}`,
      { headers: getHeaders() }
    );
    return handleResponse<MessageListResponse>(res);
  },

  sendTyping: async (instanceId: string, phone: string, state: 'composing' | 'paused') => {
    const res = await fetch(
      `${API_BASE}/api/v1/instances/${instanceId}/messages/${phone}/typing`,
      {
        method: 'POST',
        headers: getHeaders(),
        body: JSON.stringify({ state }),
      }
    );
    return handleResponse<TypingResponse>(res);
  },
};

// ============ Health API ============
export const healthApi = {
  check: async () => {
    const res = await fetch(`${API_BASE}/api/health`);
    return handleResponse<HealthResponse>(res);
  },

  metrics: async () => {
    const res = await fetch(`${API_BASE}/api/metrics`);
    return handleResponse<MetricsResponse>(res);
  },
};
```

---

## Quick Reference

### All Endpoints Summary

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | System health check |
| GET | `/api/ready` | Readiness probe |
| GET | `/api/live` | Liveness probe |
| GET | `/api/metrics` | System metrics |
| GET | `/api/v1/instances` | List all instances |
| POST | `/api/v1/instances` | Create instance |
| GET | `/api/v1/instances/{id}` | Get instance |
| DELETE | `/api/v1/instances/{id}` | Delete instance |
| POST | `/api/v1/instances/{id}/warmup` | Warm up instance |
| GET | `/api/v1/instances/{id}/status` | Get WhatsApp status |
| GET | `/api/v1/instances/{id}/link/qr` | Get QR code (PNG) |
| POST | `/api/v1/instances/{id}/link/phone` | Link via phone |
| DELETE | `/api/v1/instances/{id}/unlink` | Unlink session |
| GET | `/api/v1/instances/{id}/profile` | Get profile |
| POST | `/api/v1/instances/{id}/send` | Send message |
| GET | `/api/v1/instances/{id}/chats` | List chats |
| GET | `/api/v1/instances/{id}/messages/{phone}` | Get messages |
| POST | `/api/v1/instances/{id}/messages/{phone}/typing` | Send typing |

---

**Related Documentation:**
- [API Reference](API_REFERENCE.md) - Full API documentation
- [Frontend Wireframes](FRONTEND_WIREFRAMES.md) - UI specifications
- [PRD](prd/README.md) - Product requirements
