import { cn } from '@/utils/cn';

type BadgeVariant = 'default' | 'primary' | 'success' | 'warning' | 'error' | 'info';

export interface BadgeProps {
  children: React.ReactNode;
  variant?: BadgeVariant;
  className?: string;
}

const variants: Record<BadgeVariant, string> = {
  default: 'bg-bg-subtle-light dark:bg-bg-subtle-dark text-text-secondary-light dark:text-text-secondary-dark',
  primary: 'bg-primary/10 text-primary',
  success: 'bg-success-light dark:bg-success/20 text-success-dark dark:text-success',
  warning: 'bg-warning-light dark:bg-warning/20 text-warning-dark dark:text-warning',
  error: 'bg-error-light dark:bg-error/20 text-error-dark dark:text-error',
  info: 'bg-info-light dark:bg-info/20 text-info-dark dark:text-info',
};

export function Badge({ children, variant = 'default', className }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium',
        variants[variant],
        className
      )}
    >
      {children}
    </span>
  );
}
