import type { Server } from '@/types';
import { cn } from '@/lib/utils';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Server as ServerIcon, Wifi, HardDrive, Cpu, MemoryStick } from 'lucide-react';

interface ServerCardProps {
  server: Server;
  onClick?: () => void;
  className?: string;
}

function getStatusVariant(status: Server['status']) {
  switch (status) {
    case 'online':
      return 'success';
    case 'offline':
      return 'destructive';
    case 'warning':
      return 'warning';
    case 'maintenance':
      return 'secondary';
    default:
      return 'default';
  }
}

function ProgressBar({
  value,
  label,
  icon: Icon,
}: {
  value: number;
  label: string;
  icon: typeof Cpu;
}) {
  const getColor = () => {
    if (value >= 80) return 'bg-destructive';
    if (value >= 60) return 'bg-warning';
    return 'bg-primary';
  };

  return (
    <div className="space-y-1">
      <div className="flex items-center justify-between text-xs">
        <div className="flex items-center gap-1 text-muted-foreground">
          <Icon className="h-3 w-3" />
          <span>{label}</span>
        </div>
        <span className="font-medium">{value}%</span>
      </div>
      <div className="h-1.5 rounded-full bg-secondary">
        <div
          className={cn('h-full rounded-full transition-all', getColor())}
          style={{ width: `${value}%` }}
        />
      </div>
    </div>
  );
}

export function ServerCard({ server, onClick, className }: ServerCardProps) {
  return (
    <Card
      className={cn('card-hover cursor-pointer', className)}
      onClick={onClick}
    >
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="h-8 w-8 rounded-lg bg-primary/10 flex items-center justify-center">
              <ServerIcon className="h-4 w-4 text-primary" />
            </div>
            <CardTitle className="text-base">{server.name}</CardTitle>
          </div>
          <Badge variant={getStatusVariant(server.status)}>
            {server.status}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center gap-4 text-sm text-muted-foreground">
          <div className="flex items-center gap-1">
            <Wifi className="h-3 w-3" />
            <span>{server.ipAddress}</span>
          </div>
          <span>{server.location}</span>
        </div>

        <div className="space-y-3">
          <ProgressBar value={server.cpuUsage} label="CPU" icon={Cpu} />
          <ProgressBar value={server.ramUsage} label="RAM" icon={MemoryStick} />
          <ProgressBar value={server.diskUsage} label="Disk" icon={HardDrive} />
        </div>

        <div className="flex justify-between text-xs text-muted-foreground pt-2 border-t">
          <span>Sessions: {server.activeSessions}</span>
          <span>Uptime: {server.uptime}</span>
        </div>
      </CardContent>
    </Card>
  );
}
