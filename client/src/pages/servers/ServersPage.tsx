import { useState } from 'react';
import { 
  Plus, 
  RefreshCw, 
  Server, 
  Activity, 
  Clock, 
  Filter,
  RotateCw,
  Eye,
  Terminal,
  Zap,
  Play,
  Snowflake
} from 'lucide-react';
import { ContentContainer } from '@/components/layout/ContentContainer';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Input } from '@/components/ui/Input';

// Mock data for nodes
const nodes = [
  {
    id: 1,
    name: 'WA-US-EAST-1',
    ip: '192.168.1.189',
    status: 'online',
    cpuUsage: 42,
    ramUsage: 2.4,
    ramTotal: 8,
    latency: 12,
  },
  {
    id: 2,
    name: 'WA-EU-WEST-2',
    ip: '10.0.4.12',
    status: 'online',
    cpuUsage: 88,
    ramUsage: 6.8,
    ramTotal: 8,
    latency: 58,
  },
  {
    id: 3,
    name: 'WA-ASIA-SOUTH-1',
    ip: '10.24.6.89',
    status: 'offline',
    cpuUsage: 0,
    ramUsage: 0,
    ramTotal: 8,
    latency: 0,
  },
];

const clusters = [
  { name: 'US-EAST Cluster', traffic: '45% Traffic', color: 'bg-success' },
  { name: 'EU-WEST Cluster', traffic: '32% Traffic', color: 'bg-primary' },
  { name: 'ASIA-PAC Cluster', traffic: '23% Traffic', color: 'bg-warning' },
];

export function ServersPage() {
  const [searchQuery, setSearchQuery] = useState('');

  const getStatusColor = (status: string) => {
    return status === 'online' ? 'bg-success' : 'bg-muted-foreground';
  };

  const getCpuBarColor = (usage: number) => {
    if (usage >= 80) return 'bg-destructive';
    if (usage >= 60) return 'bg-warning';
    return 'bg-primary';
  };

  const getRamBarColor = (usage: number, total: number) => {
    const percent = (usage / total) * 100;
    if (percent >= 80) return 'bg-destructive';
    if (percent >= 60) return 'bg-warning';
    return 'bg-warning';
  };

  return (
    <ContentContainer>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="relative w-64">
          <Input 
            placeholder="Search servers..." 
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
          <Server className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        </div>
        <Button>
          <Plus className="h-4 w-4 mr-2" />
          Add New Server
        </Button>
      </div>

      {/* Stats Cards */}
      <div className="grid gap-4 md:grid-cols-3 mb-6">
        <Card>
          <CardContent className="p-6">
            <p className="text-xs text-muted-foreground uppercase tracking-wider mb-2">Total Nodes</p>
            <div className="flex items-baseline gap-2">
              <span className="text-4xl font-bold">24</span>
              <span className="text-sm text-success">+2 today</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-6">
            <p className="text-xs text-muted-foreground uppercase tracking-wider mb-2">Network Traffic</p>
            <div className="flex items-baseline gap-2">
              <span className="text-4xl font-bold">1.2 TB/s</span>
              <span className="text-sm text-primary">Peak: 1.8</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="p-6">
            <p className="text-xs text-muted-foreground uppercase tracking-wider mb-2">Global Latency</p>
            <div className="flex items-baseline gap-2">
              <span className="text-4xl font-bold">42ms</span>
              <span className="text-sm text-success">Optimized</span>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Server Load Balancing */}
      <Card className="mb-6">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-lg">
            <Zap className="h-5 w-5 text-primary" />
            Server Load Balancing
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-8">
            {/* Gateway LB */}
            <div className="flex flex-col items-center">
              <div className="w-20 h-20 rounded-full border-2 border-primary bg-primary/10 flex items-center justify-center">
                <Snowflake className="h-8 w-8 text-primary" />
              </div>
              <span className="text-xs text-muted-foreground mt-2">GATEWAY LB</span>
            </div>

            {/* Connections */}
            <div className="flex-1">
              <svg className="w-full h-24" viewBox="0 0 400 100">
                {/* Connection lines */}
                <path d="M 0,50 C 100,50 100,20 200,20" stroke="currentColor" className="text-primary/30" strokeWidth="2" fill="none" strokeDasharray="4,4" />
                <path d="M 0,50 C 100,50 100,50 200,50" stroke="currentColor" className="text-primary/30" strokeWidth="2" fill="none" strokeDasharray="4,4" />
                <path d="M 0,50 C 100,50 100,80 200,80" stroke="currentColor" className="text-primary/30" strokeWidth="2" fill="none" strokeDasharray="4,4" />
                {/* Animated dots */}
                <circle r="4" fill="currentColor" className="text-primary">
                  <animateMotion dur="2s" repeatCount="indefinite" path="M 0,50 C 100,50 100,20 200,20" />
                </circle>
                <circle r="4" fill="currentColor" className="text-primary">
                  <animateMotion dur="2.5s" repeatCount="indefinite" path="M 0,50 C 100,50 100,50 200,50" />
                </circle>
                <circle r="4" fill="currentColor" className="text-primary">
                  <animateMotion dur="3s" repeatCount="indefinite" path="M 0,50 C 100,50 100,80 200,80" />
                </circle>
              </svg>
            </div>

            {/* Clusters */}
            <div className="space-y-2">
              {clusters.map((cluster, idx) => (
                <div key={idx} className="flex items-center gap-3 px-4 py-2 bg-card/50 rounded-lg border border-border">
                  <span className={`w-2 h-2 rounded-full ${cluster.color}`} />
                  <span className="text-sm font-medium">{cluster.name}</span>
                  <span className="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded">
                    {cluster.traffic}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Active Nodes */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="flex items-center gap-2 text-lg font-semibold">
          <Activity className="h-5 w-5 text-success" />
          Active Nodes
        </h2>
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="icon">
            <Filter className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="icon">
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {nodes.map((node) => (
          <Card key={node.id} className={node.status === 'offline' ? 'opacity-60' : ''}>
            <CardContent className="p-5">
              {/* Header */}
              <div className="flex items-center justify-between mb-4">
                <div>
                  <div className="flex items-center gap-2">
                    <h3 className="font-semibold">{node.name}</h3>
                    <span className="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded">
                      {node.ip}
                    </span>
                  </div>
                  <div className="flex items-center gap-1.5 mt-1">
                    <span className={`w-2 h-2 rounded-full ${getStatusColor(node.status)}`} />
                    <span className={`text-xs uppercase font-medium ${node.status === 'online' ? 'text-success' : 'text-muted-foreground'}`}>
                      {node.status}
                    </span>
                  </div>
                </div>
              </div>

              {node.status === 'online' ? (
                <>
                  {/* CPU Usage */}
                  <div className="mb-3">
                    <div className="flex items-center justify-between text-sm mb-1">
                      <span className="text-muted-foreground">CPU USAGE</span>
                      <span className="font-medium">{node.cpuUsage}%</span>
                    </div>
                    <div className="h-1.5 bg-muted rounded-full overflow-hidden">
                      <div 
                        className={`h-full rounded-full ${getCpuBarColor(node.cpuUsage)}`}
                        style={{ width: `${node.cpuUsage}%` }}
                      />
                    </div>
                  </div>

                  {/* RAM Usage */}
                  <div className="mb-4">
                    <div className="flex items-center justify-between text-sm mb-1">
                      <span className="text-muted-foreground">RAM USAGE</span>
                      <span className="font-medium">{node.ramUsage} / {node.ramTotal} GB</span>
                    </div>
                    <div className="h-1.5 bg-muted rounded-full overflow-hidden">
                      <div 
                        className={`h-full rounded-full ${getRamBarColor(node.ramUsage, node.ramTotal)}`}
                        style={{ width: `${(node.ramUsage / node.ramTotal) * 100}%` }}
                      />
                    </div>
                  </div>

                  {/* Latency */}
                  <div className="flex items-center gap-1 text-sm mb-4">
                    <Clock className="h-3 w-3 text-muted-foreground" />
                    <span className="text-muted-foreground">Latency:</span>
                    <span className={`font-medium ${node.latency > 50 ? 'text-warning' : 'text-success'}`}>
                      {node.latency}ms
                    </span>
                  </div>

                  {/* Action Buttons */}
                  <div className="flex items-center gap-2">
                    <Button variant="ghost" size="icon" className="h-9 w-9">
                      <RotateCw className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="icon" className="h-9 w-9">
                      <Eye className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="icon" className="h-9 w-9">
                      <Terminal className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="icon" className="h-9 w-9">
                      <Zap className="h-4 w-4" />
                    </Button>
                  </div>
                </>
              ) : (
                <div className="py-8 text-center">
                  <p className="text-muted-foreground text-sm mb-4">NODE UNREACHABLE</p>
                  <Button variant="outline" size="sm">
                    <Play className="h-4 w-4 mr-2" />
                    START
                  </Button>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </ContentContainer>
  );
}
