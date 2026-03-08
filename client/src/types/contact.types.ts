export interface Contact {
  id: string;
  name: string;
  phoneNumber: string;
  email?: string;
  tags: string[];
  createdAt: string;
  lastMessageAt?: string;
  messagesCount: number;
  status: 'active' | 'blocked' | 'unsubscribed';
}

export interface ContactTag {
  id: string;
  name: string;
  color: string;
  contactCount: number;
}
