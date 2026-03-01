import { forwardRef, type InputHTMLAttributes, type ReactNode } from 'react';
import { type LucideIcon } from 'lucide-react';
import { cn } from '@/utils/cn';

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  icon?: LucideIcon;
  iconRight?: LucideIcon;
  leftIcon?: ReactNode;
  rightIcon?: ReactNode;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, label, error, icon: Icon, iconRight: IconRight, leftIcon, rightIcon, id, ...props }, ref) => {
    const inputId = id || props.name;
    const hasLeftIcon = Icon || leftIcon;
    const hasRightIcon = IconRight || rightIcon;

    return (
      <div className="w-full">
        {label && (
          <label
            htmlFor={inputId}
            className="block text-sm font-medium text-text-secondary-light dark:text-text-secondary-dark mb-1.5"
          >
            {label}
          </label>
        )}
        <div className="relative">
          {(Icon || leftIcon) && (
            <div className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted-light dark:text-text-muted-dark">
              {leftIcon || (Icon && <Icon className="h-4 w-4" />)}
            </div>
          )}
          <input
            ref={ref}
            id={inputId}
            className={cn(
              'w-full rounded-lg border transition-colors',
              'bg-input-bg-light dark:bg-input-bg-dark',
              'border-input-border-light dark:border-input-border-dark',
              'text-text-light dark:text-text-dark',
              'placeholder:text-text-muted-light dark:placeholder:text-text-muted-dark',
              'focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary',
              'disabled:opacity-50 disabled:cursor-not-allowed',
              'px-3 py-2 text-sm',
              hasLeftIcon && 'pl-10',
              hasRightIcon && 'pr-10',
              error && 'border-error focus:ring-error/50 focus:border-error',
              className
            )}
            {...props}
          />
          {(IconRight || rightIcon) && (
            <div className="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted-light dark:text-text-muted-dark">
              {rightIcon || (IconRight && <IconRight className="h-4 w-4" />)}
            </div>
          )}
        </div>
        {error && (
          <p className="mt-1 text-xs text-error">{error}</p>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';
