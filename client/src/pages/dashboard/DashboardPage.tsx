import {
  MessageSquare,
  CheckCircle,
  XCircle,
  Smartphone,
  Server,
} from 'lucide-react';
import { PageHeader } from '@/components/layout/PageHeader';
import { ContentContainer } from '@/components/layout/ContentContainer';
import { MetricCard } from '@/components/cards/MetricCard';
import { MessageTrafficChart } from '@/components/charts/MessageTrafficChart';
import { DeliveryChart } from '@/components/charts/DeliveryChart';
import { DailyVolumeChart } from '@/components/charts/DailyVolumeChart';
import {
  dashboardStats,
  messageTrafficData,
  deliveryRateData,
  dailyVolumeData,
} from '@/mock';

export function DashboardPage() {
  const formatNumber = (num: number): string => {
    if (num >= 1000) {
      return `${(num / 1000).toFixed(1)}k`;
    }
    return num.toString();
  };

  return (
    <ContentContainer>
      <PageHeader
        title="Dashboard"
        description="Overview of your WhatsApp messaging platform"
      />

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-5 mb-6">
        <MetricCard
          title="Total Messages"
          value={formatNumber(dashboardStats.totalMessages)}
          change={dashboardStats.messageChange}
          changeType="increase"
          description="from last week"
          icon={<MessageSquare className="h-5 w-5" />}
        />
        <MetricCard
          title="Delivered"
          value={formatNumber(dashboardStats.deliveredMessages)}
          change={dashboardStats.deliveryRate}
          changeType="increase"
          description="delivery rate"
          icon={<CheckCircle className="h-5 w-5" />}
        />
        <MetricCard
          title="Failed"
          value={formatNumber(dashboardStats.failedMessages)}
          changeType="neutral"
          icon={<XCircle className="h-5 w-5" />}
        />
        <MetricCard
          title="Active Sessions"
          value={dashboardStats.activeSessions}
          change={dashboardStats.sessionChange}
          changeType="increase"
          description="new this week"
          icon={<Smartphone className="h-5 w-5" />}
        />
        <MetricCard
          title="Online Servers"
          value={dashboardStats.onlineServers}
          changeType="neutral"
          icon={<Server className="h-5 w-5" />}
        />
      </div>

      <div className="grid gap-6 lg:grid-cols-3 mb-6">
        <MessageTrafficChart
          data={messageTrafficData}
          title="Message Traffic (7 Days)"
          className="lg:col-span-2"
        />
        <DeliveryChart
          data={deliveryRateData}
          title="Delivery Rate"
        />
      </div>

      <div className="grid gap-6">
        <DailyVolumeChart
          data={dailyVolumeData}
          title="Daily Message Volume"
        />
      </div>
    </ContentContainer>
  );
}
