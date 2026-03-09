import type { ReactNode } from 'react';
import { cn } from '@/lib/utils';

interface ContentContainerProps {
  children: ReactNode;
  className?: string;
}

export function ContentContainer({
  children,
  className,
}: ContentContainerProps) {
  return (
    <div
      className={cn(
        'w-full px-4 sm:px-6 lg:px-8 py-6',
        className
      )}
    >
      {children}
    </div>
  );
}
