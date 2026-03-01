import { forwardRef, type ButtonHTMLAttributes, type ReactNode, Children, cloneElement, isValidElement } from 'react';
import { type LucideIcon } from 'lucide-react';
import { cn } from '@/utils/cn';

type ButtonVariant = 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger';
type ButtonSize = 'sm' | 'md' | 'lg' | 'icon';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  icon?: LucideIcon;
  iconRight?: LucideIcon;
  loading?: boolean;
  isLoading?: boolean;
  asChild?: boolean;
  children?: ReactNode;
}

const variants: Record<ButtonVariant, string> = {
  primary: 'bg-primary text-white hover:bg-primary-hover shadow-sm',
  secondary: 'bg-primary/10 text-primary hover:bg-primary/20',
  outline: 'border border-border-strong-light dark:border-border-dark bg-transparent text-text-secondary-light dark:text-text-secondary-dark hover:bg-bg-hover-light dark:hover:bg-bg-hover-dark',
  ghost: 'text-text-secondary-light dark:text-text-secondary-dark hover:bg-bg-hover-light dark:hover:bg-bg-hover-dark',
  danger: 'bg-error/10 text-error hover:bg-error/20',
};

const sizes: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-xs',
  md: 'px-4 py-2 text-sm',
  lg: 'px-6 py-3 text-base',
  icon: 'p-2',
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'primary', size = 'md', icon: Icon, iconRight: IconRight, loading, isLoading, asChild, children, disabled, ...props }, ref) => {
    const isLoadingState = loading || isLoading;
    
    const buttonClass = cn(
      'inline-flex items-center justify-center gap-2 rounded-lg font-medium transition-all whitespace-nowrap',
      'focus:outline-none focus:ring-2 focus:ring-primary/50',
      'disabled:opacity-50 disabled:cursor-not-allowed',
      variants[variant],
      sizes[size],
      className
    );
    
    // If asChild is true, clone the child element and merge className
    if (asChild && children) {
      const child = Children.only(children);
      if (isValidElement<{ className?: string }>(child)) {
        return cloneElement(child, {
          className: cn(buttonClass, child.props.className),
        });
      }
      return children;
    }
    
    return (
      <button
        ref={ref}
        className={buttonClass}
        disabled={isLoadingState || disabled}
        {...props}
      >
        {isLoadingState ? (
          <div className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
        ) : (
          Icon && <Icon className={cn('h-4 w-4', size === 'icon' && 'h-5 w-5')} />
        )}
        {children}
        {IconRight && !isLoadingState && <IconRight className="h-4 w-4" />}
      </button>
    );
  }
);

Button.displayName = 'Button';
