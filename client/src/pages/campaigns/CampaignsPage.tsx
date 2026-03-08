import { Plus, Play, Pause, MoreVertical, Copy, Trash2 } from 'lucide-react';
import { PageHeader } from '@/components/layout/PageHeader';
import { ContentContainer } from '@/components/layout/ContentContainer';
import { CampaignCard } from '@/components/cards/CampaignCard';
import { CampaignChart } from '@/components/charts/CampaignChart';
import { DataTable, type Column } from '@/components/tables/DataTable';
import { StatusBadge } from '@/components/badges/StatusBadge';
import { Button } from '@/components/ui/Button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/DropdownMenu';
import { campaigns, campaignPerformanceData } from '@/mock';
import { formatDate } from '@/lib/utils';
import type { Campaign } from '@/types';

const campaignColumns: Column<Campaign>[] = [
  { key: 'name', header: 'Campaign' },
  {
    key: 'status',
    header: 'Status',
    render: (campaign) => <StatusBadge status={campaign.status} />,
  },
  { key: 'totalRecipients', header: 'Recipients' },
  {
    key: 'progress',
    header: 'Progress',
    render: (campaign) => {
      const progress =
        campaign.totalRecipients > 0
          ? Math.round((campaign.sentCount / campaign.totalRecipients) * 100)
          : 0;
      return (
        <div className="flex items-center gap-2">
          <div className="w-24 h-1.5 rounded-full bg-secondary">
            <div
              className="h-full rounded-full bg-primary"
              style={{ width: `${progress}%` }}
            />
          </div>
          <span className="text-sm text-muted-foreground">{progress}%</span>
        </div>
      );
    },
  },
  {
    key: 'deliveredCount',
    header: 'Delivered',
    render: (campaign) => campaign.deliveredCount.toLocaleString(),
  },
  {
    key: 'failedCount',
    header: 'Failed',
    render: (campaign) => campaign.failedCount.toLocaleString(),
  },
  {
    key: 'createdAt',
    header: 'Created',
    render: (campaign) => formatDate(campaign.createdAt, 'PP'),
  },
  {
    key: 'actions',
    header: '',
    className: 'w-12',
    render: (campaign) => (
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon">
            <MoreVertical className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          {campaign.status === 'running' && (
            <DropdownMenuItem>
              <Pause className="h-4 w-4 mr-2" />
              Pause
            </DropdownMenuItem>
          )}
          {campaign.status === 'paused' && (
            <DropdownMenuItem>
              <Play className="h-4 w-4 mr-2" />
              Resume
            </DropdownMenuItem>
          )}
          <DropdownMenuItem>
            <Copy className="h-4 w-4 mr-2" />
            Duplicate
          </DropdownMenuItem>
          <DropdownMenuItem className="text-destructive">
            <Trash2 className="h-4 w-4 mr-2" />
            Delete
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    ),
  },
];

export function CampaignsPage() {
  return (
    <ContentContainer>
      <PageHeader
        title="Campaigns"
        description="Create and manage messaging campaigns"
        actions={
          <Button size="sm">
            <Plus className="h-4 w-4 mr-2" />
            New Campaign
          </Button>
        }
      />

      <Tabs defaultValue="all" className="space-y-6">
        <TabsList>
          <TabsTrigger value="all">All Campaigns</TabsTrigger>
          <TabsTrigger value="active">Active</TabsTrigger>
          <TabsTrigger value="scheduled">Scheduled</TabsTrigger>
          <TabsTrigger value="completed">Completed</TabsTrigger>
        </TabsList>

        <TabsContent value="all">
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 mb-8">
            {campaigns.map((campaign) => (
              <CampaignCard key={campaign.id} campaign={campaign} />
            ))}
          </div>

          <CampaignChart
            data={campaignPerformanceData}
            title="Campaign Performance (7 Days)"
          />
        </TabsContent>

        <TabsContent value="active">
          <DataTable
            data={campaigns.filter(
              (c) => c.status === 'running' || c.status === 'paused'
            )}
            columns={campaignColumns}
            emptyMessage="No active campaigns."
          />
        </TabsContent>

        <TabsContent value="scheduled">
          <DataTable
            data={campaigns.filter((c) => c.status === 'scheduled')}
            columns={campaignColumns}
            emptyMessage="No scheduled campaigns."
          />
        </TabsContent>

        <TabsContent value="completed">
          <DataTable
            data={campaigns.filter(
              (c) => c.status === 'completed' || c.status === 'failed'
            )}
            columns={campaignColumns}
            emptyMessage="No completed campaigns."
          />
        </TabsContent>
      </Tabs>
    </ContentContainer>
  );
}
