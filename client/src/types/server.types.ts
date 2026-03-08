export type ServerStatus = 'online' | 'offline' | 'maintenance' | 'warning';

export interface Server {
  id: string;
  name: string;
  ipAddress: string;
  status: ServerStatus;
  cpuUsage: number;
  ramUsage: number;
  diskUsage: number;
  uptime: string;
  lastPing: string;
  location: string;
  activeSessions: number;
  messagesPerMinute: number;
}

export interface ServerPerformanceData {
  time: string;
  cpu: number;
  ram: number;
  network: number;
}
