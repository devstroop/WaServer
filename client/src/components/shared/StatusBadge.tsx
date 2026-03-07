import { Badge } from '@/components/ui/Badge';
import { INSTANCE_STATUS, STATUS_DISPLAY, type InstanceStatusType } from '@/lib/constants';

interface StatusBadgeProps {
  status: InstanceStatusType;
  authorized?: boolean;
}

export function StatusBadge({ status, authorized }: StatusBadgeProps) {
  if (status === INSTANCE_STATUS.ACTIVE && authorized) {
    return <Badge variant="success">Connected</Badge>;
  }
  if (status === INSTANCE_STATUS.ACTIVE && !authorized) {
    return <Badge variant="warning">Awaiting QR</Badge>;
  }
  const display = STATUS_DISPLAY[status];
  return <Badge variant={display.variant}>{display.label}</Badge>;
}
