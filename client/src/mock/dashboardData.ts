import type { DashboardStats, MessageTrafficData, DeliveryRateData } from '@/types';

export const dashboardStats: DashboardStats = {
  totalMessages: 48592,
  deliveredMessages: 45123,
  failedMessages: 892,
  activeSessions: 24,
  onlineServers: 8,
  messageChange: 12.5,
  deliveryRate: 92.8,
  sessionChange: 4,
};

export const messageTrafficData: MessageTrafficData[] = [
  { date: 'Mon', sent: 1200, delivered: 1150, failed: 30 },
  { date: 'Tue', sent: 1400, delivered: 1350, failed: 25 },
  { date: 'Wed', sent: 1100, delivered: 1050, failed: 40 },
  { date: 'Thu', sent: 1600, delivered: 1520, failed: 55 },
  { date: 'Fri', sent: 1800, delivered: 1740, failed: 35 },
  { date: 'Sat', sent: 900, delivered: 870, failed: 20 },
  { date: 'Sun', sent: 700, delivered: 680, failed: 15 },
];

export const deliveryRateData: DeliveryRateData[] = [
  { name: 'Delivered', value: 92.8, color: 'hsl(var(--success))' },
  { name: 'Pending', value: 5.4, color: 'hsl(var(--warning))' },
  { name: 'Failed', value: 1.8, color: 'hsl(var(--destructive))' },
];

export const dailyVolumeData = [
  { date: '01', messages: 3200 },
  { date: '02', messages: 2800 },
  { date: '03', messages: 3500 },
  { date: '04', messages: 4100 },
  { date: '05', messages: 3800 },
  { date: '06', messages: 2900 },
  { date: '07', messages: 3100 },
  { date: '08', messages: 3600 },
  { date: '09', messages: 4200 },
  { date: '10', messages: 3900 },
  { date: '11', messages: 4500 },
  { date: '12', messages: 4100 },
  { date: '13', messages: 3700 },
  { date: '14', messages: 3400 },
];

export const monthlyTrafficData = [
  { month: 'Jan', sent: 42000, delivered: 39800 },
  { month: 'Feb', sent: 38000, delivered: 36200 },
  { month: 'Mar', sent: 45000, delivered: 43100 },
  { month: 'Apr', sent: 48000, delivered: 45800 },
  { month: 'May', sent: 52000, delivered: 49600 },
  { month: 'Jun', sent: 49000, delivered: 46700 },
];
