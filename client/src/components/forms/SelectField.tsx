import * as React from 'react';
import { Label } from '@/components/ui/Label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/Select';
import { cn } from '@/lib/utils';

interface SelectOption {
  value: string;
  label: string;
}

interface SelectFieldProps {
  label?: string;
  error?: string;
  hint?: string;
  placeholder?: string;
  options: SelectOption[];
  value?: string;
  onValueChange?: (value: string) => void;
  disabled?: boolean;
  className?: string;
}

export function SelectField({
  label,
  error,
  hint,
  placeholder = 'Select an option',
  options,
  value,
  onValueChange,
  disabled,
  className,
}: SelectFieldProps) {
  const id = React.useId();

  return (
    <div className={cn('space-y-2', className)}>
      {label && <Label htmlFor={id}>{label}</Label>}
      <Select
        {...(value !== undefined && { value })}
        {...(onValueChange !== undefined && { onValueChange })}
        {...(disabled === true && { disabled: true })}
      >
        <SelectTrigger
          id={id}
          className={cn(error && 'border-destructive')}
        >
          <SelectValue placeholder={placeholder} />
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {hint && !error && (
        <p className="text-sm text-muted-foreground">{hint}</p>
      )}
      {error && <p className="text-sm text-destructive">{error}</p>}
    </div>
  );
}
