import { PageHeader } from '@/components/layout/PageHeader';
import { ContentContainer } from '@/components/layout/ContentContainer';
import { MetricCard } from '@/components/cards/MetricCard';
import { MessageTrafficChart } from '@/components/charts/MessageTrafficChart';
import { DailyVolumeChart } from '@/components/charts/DailyVolumeChart';
import { CampaignChart } from '@/components/charts/CampaignChart';
import { ServerPerformanceChart } from '@/components/charts/ServerPerformanceChart';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import {
  BarChart3,
  TrendingUp,
  MessageSquare,
  Users,
} from 'lucide-react';
import {
  dashboardStats,
  messageTrafficData,
  dailyVolumeData,
  monthlyTrafficData,
  campaignPerformanceData,
  serverPerformanceData,
} from '@/mock';

export function AnalyticsPage() {
  return (
    <ContentContainer>
      <PageHeader
        title="Analytics"
        description="Detailed insights and performance metrics"
      />

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4 mb-6">
        <MetricCard
          title="Total Messages (30d)"
          value={`${(dashboardStats.totalMessages * 4.2).toFixed(0)}k`}
          change={15.3}
          changeType="increase"
          icon={<MessageSquare className="h-5 w-5" />}
        />
        <MetricCard
          title="Delivery Rate"
          value={`${dashboardStats.deliveryRate}%`}
          change={2.1}
          changeType="increase"
          icon={<TrendingUp className="h-5 w-5" />}
        />
        <MetricCard
          title="Active Contacts"
          value="12.5k"
          change={8.7}
          changeType="increase"
          icon={<Users className="h-5 w-5" />}
        />
        <MetricCard
          title="Campaigns Sent"
          value="24"
          change={20}
          changeType="increase"
          icon={<BarChart3 className="h-5 w-5" />}
        />
      </div>

      <Tabs defaultValue="messages" className="space-y-6">
        <TabsList>
          <TabsTrigger value="messages">Messages</TabsTrigger>
          <TabsTrigger value="campaigns">Campaigns</TabsTrigger>
          <TabsTrigger value="servers">Servers</TabsTrigger>
        </TabsList>

        <TabsContent value="messages" className="space-y-6">
          <div className="grid gap-6 lg:grid-cols-2">
            <MessageTrafficChart
              data={messageTrafficData}
              title="Weekly Message Traffic"
            />
            <DailyVolumeChart
              data={dailyVolumeData}
              title="Daily Volume (14 Days)"
            />
          </div>

          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Monthly Traffic</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid gap-4 md:grid-cols-6">
                {monthlyTrafficData.map((month) => (
                  <div key={month.month} className="text-center p-4 rounded-lg bg-muted/50">
                    <p className="text-sm text-muted-foreground mb-2">{month.month}</p>
                    <p className="text-2xl font-bold">{(month.sent / 1000).toFixed(0)}k</p>
                    <p className="text-xs text-muted-foreground mt-1">
                      {((month.delivered / month.sent) * 100).toFixed(1)}% delivered
                    </p>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="campaigns" className="space-y-6">
          <CampaignChart
            data={campaignPerformanceData}
            title="Campaign Performance (7 Days)"
          />

          <div className="grid gap-4 md:grid-cols-3">
            <Card>
              <CardContent className="p-6 text-center">
                <p className="text-4xl font-bold text-primary">24</p>
                <p className="text-sm text-muted-foreground mt-2">Total Campaigns</p>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-6 text-center">
                <p className="text-4xl font-bold text-success">89.2%</p>
                <p className="text-sm text-muted-foreground mt-2">Avg. Delivery Rate</p>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-6 text-center">
                <p className="text-4xl font-bold">45.2k</p>
                <p className="text-sm text-muted-foreground mt-2">Total Recipients</p>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <TabsContent value="servers" className="space-y-6">
          <ServerPerformanceChart
            data={serverPerformanceData}
            title="Server Performance (24h)"
          />

          <div className="grid gap-4 md:grid-cols-4">
            <Card>
              <CardContent className="p-6 text-center">
                <p className="text-4xl font-bold text-success">8</p>
                <p className="text-sm text-muted-foreground mt-2">Online Servers</p>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-6 text-center">
                <p className="text-4xl font-bold">42%</p>
                <p className="text-sm text-muted-foreground mt-2">Avg. CPU Usage</p>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-6 text-center">
                <p className="text-4xl font-bold">58%</p>
                <p className="text-sm text-muted-foreground mt-2">Avg. RAM Usage</p>
              </CardContent>
            </Card>
            <Card>
              <CardContent className="p-6 text-center">
                <p className="text-4xl font-bold text-primary">99.9%</p>
                <p className="text-sm text-muted-foreground mt-2">Uptime</p>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </ContentContainer>
  );
}
