import { cn } from '@/lib/utils';

type StatusType =
  | 'success'
  | 'warning'
  | 'error'
  | 'info'
  | 'pending'
  | 'default';

interface StatusBadgeProps {
  status: string;
  type?: StatusType;
  className?: string;
}

const statusTypeMap: Record<string, StatusType> = {
  // Server statuses
  online: 'success',
  offline: 'error',
  maintenance: 'warning',
  warning: 'warning',
  // Session statuses
  connected: 'success',
  disconnected: 'error',
  connecting: 'pending',
  qr_pending: 'info',
  // Message statuses
  delivered: 'success',
  read: 'success',
  sent: 'info',
  pending: 'pending',
  failed: 'error',
  // Campaign statuses
  completed: 'success',
  running: 'info',
  scheduled: 'pending',
  paused: 'warning',
  draft: 'default',
  // API Key statuses
  active: 'success',
  revoked: 'error',
  expired: 'warning',
  // Contact statuses
  blocked: 'error',
  unsubscribed: 'warning',
};

const typeStyles: Record<StatusType, string> = {
  success: 'bg-success/10 text-success border-success/20',
  warning: 'bg-warning/10 text-warning border-warning/20',
  error: 'bg-destructive/10 text-destructive border-destructive/20',
  info: 'bg-primary/10 text-primary border-primary/20',
  pending: 'bg-muted text-muted-foreground border-border',
  default: 'bg-secondary text-secondary-foreground border-border',
};

export function StatusBadge({ status, type, className }: StatusBadgeProps) {
  const resolvedType = type || statusTypeMap[status.toLowerCase()] || 'default';
  const styles = typeStyles[resolvedType];

  return (
    <span
      className={cn(
        'inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-medium capitalize',
        styles,
        className
      )}
    >
      <span
        className={cn(
          'mr-1.5 h-1.5 w-1.5 rounded-full',
          resolvedType === 'success' && 'bg-success',
          resolvedType === 'warning' && 'bg-warning',
          resolvedType === 'error' && 'bg-destructive',
          resolvedType === 'info' && 'bg-primary',
          resolvedType === 'pending' && 'bg-muted-foreground animate-pulse',
          resolvedType === 'default' && 'bg-secondary-foreground'
        )}
      />
      {status.replace('_', ' ')}
    </span>
  );
}
