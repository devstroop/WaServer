import * as React from 'react';
import { Label } from '@/components/ui/Label';
import { Textarea } from '@/components/ui/Textarea';
import { cn } from '@/lib/utils';

interface TextAreaFieldProps
  extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  error?: string;
  hint?: string;
}

export const TextAreaField = React.forwardRef<
  HTMLTextAreaElement,
  TextAreaFieldProps
>(({ label, error, hint, className, id, ...props }, ref) => {
  const textareaId = id || React.useId();

  return (
    <div className="space-y-2">
      {label && <Label htmlFor={textareaId}>{label}</Label>}
      <Textarea
        id={textareaId}
        ref={ref}
        className={cn(error && 'border-destructive', className)}
        {...props}
      />
      {hint && !error && (
        <p className="text-sm text-muted-foreground">{hint}</p>
      )}
      {error && <p className="text-sm text-destructive">{error}</p>}
    </div>
  );
});
TextAreaField.displayName = 'TextAreaField';
