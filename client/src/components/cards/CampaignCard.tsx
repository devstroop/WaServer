import type { Campaign } from '@/types';
import { cn } from '@/lib/utils';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Megaphone, Users, CheckCircle, XCircle, Clock } from 'lucide-react';
import { formatDate } from '@/lib/utils';

interface CampaignCardProps {
  campaign: Campaign;
  onClick?: () => void;
  className?: string;
}

function getStatusVariant(status: Campaign['status']) {
  switch (status) {
    case 'completed':
      return 'success';
    case 'running':
      return 'default';
    case 'scheduled':
      return 'secondary';
    case 'paused':
      return 'warning';
    case 'failed':
      return 'destructive';
    case 'draft':
      return 'outline';
    default:
      return 'default';
  }
}

export function CampaignCard({
  campaign,
  onClick,
  className,
}: CampaignCardProps) {
  const deliveryRate =
    campaign.sentCount > 0
      ? Math.round((campaign.deliveredCount / campaign.sentCount) * 100)
      : 0;

  return (
    <Card
      className={cn('card-hover cursor-pointer', className)}
      onClick={onClick}
    >
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="h-8 w-8 rounded-lg bg-primary/10 flex items-center justify-center">
              <Megaphone className="h-4 w-4 text-primary" />
            </div>
            <div>
              <CardTitle className="text-base">{campaign.name}</CardTitle>
              <p className="text-xs text-muted-foreground mt-0.5">
                {campaign.description}
              </p>
            </div>
          </div>
          <Badge variant={getStatusVariant(campaign.status)}>
            {campaign.status}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-3 gap-4 text-center">
          <div className="space-y-1">
            <div className="flex items-center justify-center gap-1 text-muted-foreground">
              <Users className="h-3 w-3" />
            </div>
            <p className="text-lg font-semibold">{campaign.totalRecipients}</p>
            <p className="text-xs text-muted-foreground">Recipients</p>
          </div>
          <div className="space-y-1">
            <div className="flex items-center justify-center gap-1 text-success">
              <CheckCircle className="h-3 w-3" />
            </div>
            <p className="text-lg font-semibold">{campaign.deliveredCount}</p>
            <p className="text-xs text-muted-foreground">Delivered</p>
          </div>
          <div className="space-y-1">
            <div className="flex items-center justify-center gap-1 text-destructive">
              <XCircle className="h-3 w-3" />
            </div>
            <p className="text-lg font-semibold">{campaign.failedCount}</p>
            <p className="text-xs text-muted-foreground">Failed</p>
          </div>
        </div>

        {campaign.sentCount > 0 && (
          <div className="space-y-1">
            <div className="flex items-center justify-between text-xs">
              <span className="text-muted-foreground">Delivery Rate</span>
              <span className="font-medium">{deliveryRate}%</span>
            </div>
            <div className="h-1.5 rounded-full bg-secondary">
              <div
                className="h-full rounded-full bg-primary transition-all"
                style={{ width: `${deliveryRate}%` }}
              />
            </div>
          </div>
        )}

        <div className="flex items-center gap-2 text-xs text-muted-foreground pt-2 border-t">
          <Clock className="h-3 w-3" />
          <span>
            {campaign.scheduledAt
              ? `Scheduled: ${formatDate(campaign.scheduledAt, 'PP')}`
              : `Created: ${formatDate(campaign.createdAt, 'PP')}`}
          </span>
        </div>
      </CardContent>
    </Card>
  );
}
