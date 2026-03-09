export type MessageStatus = 'pending' | 'sent' | 'delivered' | 'read' | 'failed';
export type MessageType = 'text' | 'image' | 'document' | 'audio' | 'video' | 'template';

export interface Message {
  id: string;
  recipient: string;
  content: string;
  type: MessageType;
  status: MessageStatus;
  sentAt: string;
  deliveredAt?: string;
  readAt?: string;
  sessionId: string;
  templateId?: string;
  error?: string;
}

export interface MessageTemplate {
  id: string;
  name: string;
  content: string;
  variables: string[];
  category: string;
  createdAt: string;
}
