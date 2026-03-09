import { cn } from '@/lib/utils';
import type { LogLevel } from '@/types';

interface LogLevelBadgeProps {
  level: LogLevel;
  className?: string;
}

const levelStyles: Record<LogLevel, string> = {
  info: 'bg-primary/10 text-primary',
  warning: 'bg-warning/10 text-warning',
  error: 'bg-destructive/10 text-destructive',
  debug: 'bg-muted text-muted-foreground',
};

export function LogLevelBadge({ level, className }: LogLevelBadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center rounded px-2 py-0.5 text-xs font-medium uppercase',
        levelStyles[level],
        className
      )}
    >
      {level}
    </span>
  );
}
