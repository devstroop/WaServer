export type CampaignStatus = 'draft' | 'scheduled' | 'running' | 'paused' | 'completed' | 'failed';

export interface Campaign {
  id: string;
  name: string;
  description: string;
  status: CampaignStatus;
  totalRecipients: number;
  sentCount: number;
  deliveredCount: number;
  failedCount: number;
  scheduledAt?: string;
  startedAt?: string;
  completedAt?: string;
  createdAt: string;
  templateId?: string;
}

export interface CampaignPerformanceData {
  date: string;
  sent: number;
  delivered: number;
  failed: number;
  opened: number;
}
