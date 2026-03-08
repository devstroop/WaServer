import { useState, useEffect } from 'react';
import { MessageSquare, Zap, Shield, ArrowRight, Send, Users, Bell, BarChart3 } from 'lucide-react';
import Lottie from 'lottie-react';
import { ThemeToggle } from '@/theme';

interface AuthLayoutProps {
  children: React.ReactNode;
}

const features = ['Multi-session support', 'Webhook integrations', 'Campaign analytics'];

const stats = [
  { value: '99.9%', label: 'Uptime' },
  { value: '10M+', label: 'Messages/day' },
  { value: '50ms', label: 'Avg latency' },
];

const floatingCards = [
  { icon: Send, value: '1,234', label: 'Messages sent', position: 'top-20 right-8' },
  { icon: Users, value: '5,678', label: 'Contacts', position: 'top-40 right-24' },
  { icon: Bell, value: '12', label: 'Active sessions', position: 'bottom-32 right-12' },
  { icon: BarChart3, value: '98.5%', label: 'Delivery rate', position: 'bottom-48 right-28' },
];

// Lottie animation URL (chat/messaging animation)
const LOTTIE_URL = 'https://assets3.lottiefiles.com/packages/lf20_eroqjb7w.json';

export function AuthLayout({ children }: AuthLayoutProps) {
  const [animationData, setAnimationData] = useState<object | null>(null);

  useEffect(() => {
    fetch(LOTTIE_URL)
      .then((res) => res.json())
      .then((data) => setAnimationData(data))
      .catch(() => setAnimationData(null));
  }, []);

  return (
    <div className="min-h-screen flex bg-background">
      {/* Left Panel - Theme aware design */}
      <div className="hidden lg:flex lg:w-1/2 xl:w-[55%] bg-card relative overflow-hidden border-r border-border">
        {/* Subtle gradient overlay using theme colors */}
        <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-transparent to-primary/10" />
        
        {/* Minimal grid lines */}
        <div className="absolute inset-0 opacity-[0.02]">
          <div className="h-full w-full" style={{
            backgroundImage: 'linear-gradient(to right, currentColor 1px, transparent 1px), linear-gradient(to bottom, currentColor 1px, transparent 1px)',
            backgroundSize: '60px 60px'
          }} />
        </div>

        {/* Accent glow - animated */}
        <div className="absolute top-1/2 left-1/3 -translate-x-1/2 -translate-y-1/2 w-[400px] h-[400px] bg-primary/10 rounded-full blur-3xl animate-pulse-glow" />

        <div className="relative z-10 flex flex-col justify-between p-12 xl:p-16 w-full">
          {/* Top Section - Logo */}
          <div className="flex items-center gap-3 animate-on-load animate-fade-in-up">
            <div className="h-10 w-10 rounded-lg bg-primary flex items-center justify-center">
              <MessageSquare className="h-5 w-5 text-primary-foreground" />
            </div>
            <div>
              <h1 className="text-xl font-semibold tracking-tight text-foreground">WAS</h1>
              <p className="text-muted-foreground text-xs">WhatsApp Automation Server</p>
            </div>
          </div>

          {/* Center Section - Two Column Layout */}
          <div className="flex-1 flex items-center gap-8">
            {/* Left Column - Text Content */}
            <div className="flex-1 flex flex-col justify-center">
              {/* Lottie Animation */}
              {animationData && (
                <div className="w-full max-w-[720px] mb-6 animate-on-load animate-fade-in delay-100">
                  <Lottie 
                    animationData={animationData} 
                    loop 
                    style={{ width: '100%', height: 'auto' }}
                  />
                </div>
              )}

              <div className="max-w-md">
                <p className="text-primary text-sm font-medium mb-4 tracking-wide uppercase animate-on-load animate-fade-in-up delay-200">
                  Self-hosted automation
                </p>
                <h2 className="text-3xl xl:text-4xl font-bold leading-[1.1] mb-4 text-foreground animate-on-load animate-fade-in-up delay-300">
                  Scale your WhatsApp messaging
                </h2>
                <p className="text-muted-foreground text-base leading-relaxed mb-6 animate-on-load animate-fade-in-up delay-400">
                  Connect unlimited sessions, automate campaigns, and manage contacts through a powerful REST API.
                </p>
                
                {/* Simple feature list */}
                <div className="space-y-2">
                  {features.map((item, index) => (
                    <div 
                      key={item} 
                      className="flex items-center gap-3 text-muted-foreground animate-on-load animate-slide-in-left"
                      style={{ animationDelay: `${(index + 5) * 100}ms` }}
                    >
                      <ArrowRight className="h-4 w-4 text-primary" />
                      <span>{item}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Right Column - Floating Cards */}
            <div className="hidden xl:flex flex-col gap-4 w-48">
              {floatingCards.map((card, index) => (
                <div 
                  key={card.label}
                  className="bg-background/80 backdrop-blur-sm rounded-xl border border-border p-3 shadow-lg animate-on-load animate-fade-in-up"
                  style={{ animationDelay: `${(index + 3) * 150}ms` }}
                >
                  <div className="flex items-center gap-3">
                    <div className="w-9 h-9 rounded-lg bg-primary/10 flex items-center justify-center">
                      <card.icon className="w-4 h-4 text-primary" />
                    </div>
                    <div>
                      <div className="text-sm font-semibold text-foreground">{card.value}</div>
                      <div className="text-[11px] text-muted-foreground">{card.label}</div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Bottom Section - Stats */}
          <div className="flex gap-12">
            {stats.map((stat, index) => (
              <div 
                key={stat.label} 
                className="animate-on-load animate-fade-in-up"
                style={{ animationDelay: `${(index + 6) * 100}ms` }}
              >
                <div className="text-3xl font-bold text-foreground">{stat.value}</div>
                <div className="text-muted-foreground text-sm">{stat.label}</div>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Right Panel - Login Form */}
      <div className="w-full lg:w-1/2 xl:w-[45%] flex flex-col bg-background">
        <header className="flex justify-between items-center p-6 lg:p-8 animate-on-load animate-fade-in">
          {/* Mobile Logo */}
          <div className="flex items-center gap-2 lg:hidden">
            <div className="h-10 w-10 rounded-lg bg-primary flex items-center justify-center">
              <MessageSquare className="h-5 w-5 text-primary-foreground" />
            </div>
            <span className="text-xl font-semibold">WAS</span>
          </div>
          <div className="ml-auto">
            <ThemeToggle />
          </div>
        </header>

        <main className="flex-1 flex items-center justify-center px-6 lg:px-12 xl:px-20">
          <div className="w-full max-w-md">
            {/* Welcome Section */}
            <div className="mb-8">
              <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary/10 text-primary text-sm font-medium mb-6 animate-on-load animate-fade-in-up delay-100">
                <Zap className="h-3.5 w-3.5" />
                Ready to connect
              </div>
              <h2 className="text-3xl font-bold mb-3 animate-on-load animate-fade-in-up delay-200">Welcome back</h2>
              <p className="text-muted-foreground animate-on-load animate-fade-in-up delay-300">
                Enter your API key to access the dashboard.
              </p>
            </div>

            <div className="animate-on-load animate-fade-in-up delay-400">
              {children}
            </div>

            {/* Security Badge */}
            <div className="mt-8 flex items-center justify-center gap-2 text-sm text-muted-foreground animate-on-load animate-fade-in delay-600">
              <Shield className="h-4 w-4" />
              <span>Secured with end-to-end encryption</span>
            </div>
          </div>
        </main>

        <footer className="py-6 px-6 text-center animate-on-load animate-fade-in delay-700">
          <p className="text-xs text-muted-foreground">
            &copy; {new Date().getFullYear()} WAS. All rights reserved.
            <span className="mx-2">•</span>
            <a href="/docs" className="hover:text-foreground transition-colors">Docs</a>
            <span className="mx-2">•</span>
            <a href="/support" className="hover:text-foreground transition-colors">Support</a>
          </p>
        </footer>
      </div>
    </div>
  );
}
