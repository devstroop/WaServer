export interface MetricData {
  label: string;
  value: number | string;
  change?: number;
  changeType?: 'increase' | 'decrease' | 'neutral';
  icon?: string;
}

export interface ChartDataPoint {
  name: string;
  value: number;
  [key: string]: string | number;
}

export interface MessageTrafficData {
  date: string;
  sent: number;
  delivered: number;
  failed: number;
}

export interface DeliveryRateData {
  name: string;
  value: number;
  color: string;
}

export interface DashboardStats {
  totalMessages: number;
  deliveredMessages: number;
  failedMessages: number;
  activeSessions: number;
  onlineServers: number;
  messageChange: number;
  deliveryRate: number;
  sessionChange: number;
}
