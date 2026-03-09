export type LogLevel = 'info' | 'warning' | 'error' | 'debug';

export interface LogEntry {
  id: string;
  timestamp: string;
  level: LogLevel;
  message: string;
  source: string;
  metadata?: Record<string, unknown>;
}
