/**
 * Infrastructure Monitor
 * Real-time monitoring of training infrastructure health and performance
 */

import React, { useState, useEffect, useRef } from 'react';
import {
  Activity,
  Cpu,
  HardDrive,
  Wifi,
  Thermometer,
  Zap,
  AlertTriangle,
  CheckCircle,
  XCircle,
  TrendingUp,
  Clock,
  Database,
  Network,
  Shield
} from 'lucide-react';

interface SystemMetrics {
  timestamp: number;
  cpu: number;
  memory: number;
  gpu: number;
  network: number;
  storage: number;
  temperature: number;
  power: number;
}

interface Alert {
  id: string;
  type: 'critical' | 'warning' | 'info';
  component: string;
  message: string;
  timestamp: Date;
  acknowledged: boolean;
}

interface InfrastructureStatus {
  overall: 'healthy' | 'warning' | 'critical';
  services: {
    database: 'up' | 'down' | 'degraded';
    messageQueue: 'up' | 'down' | 'degraded';
    storage: 'up' | 'down' | 'degraded';
    networking: 'up' | 'down' | 'degraded';
    security: 'up' | 'down' | 'degraded';
  };
  uptime: number;
  lastCheck: Date;
}

const SAMPLE_STATUS: InfrastructureStatus = {
  overall: 'healthy',
  services: {
    database: 'up',
    messageQueue: 'up',
    storage: 'up',
    networking: 'up',
    security: 'up'
  },
  uptime: 168.5,
  lastCheck: new Date()
};

const SAMPLE_ALERTS: Alert[] = [
  {
    id: 'alert-001',
    type: 'warning',
    component: 'Training Node 3',
    message: 'GPU temperature above 75°C',
    timestamp: new Date(Date.now() - 1000 * 60 * 15),
    acknowledged: false
  },
  {
    id: 'alert-002',
    type: 'info',
    component: 'Load Balancer',
    message: 'Traffic spike detected - auto-scaling triggered',
    timestamp: new Date(Date.now() - 1000 * 60 * 32),
    acknowledged: true
  }
];

export const InfrastructureMonitor: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [metrics, setMetrics] = useState<SystemMetrics[]>([]);
  const [alerts, setAlerts] = useState<Alert[]>(SAMPLE_ALERTS);
  const [status, setStatus] = useState<InfrastructureStatus>(SAMPLE_STATUS);
  const [selectedMetric, setSelectedMetric] = useState<'cpu' | 'memory' | 'gpu' | 'network'>('cpu');
  const [timeRange, setTimeRange] = useState<'1h' | '6h' | '24h' | '7d'>('1h');

  // Generate real-time metrics
  useEffect(() => {
    const generateMetric = (): SystemMetrics => ({
      timestamp: Date.now(),
      cpu: 30 + Math.random() * 40 + Math.sin(Date.now() / 10000) * 15,
      memory: 45 + Math.random() * 35 + Math.sin(Date.now() / 8000) * 10,
      gpu: 60 + Math.random() * 30 + Math.sin(Date.now() / 12000) * 20,
      network: 20 + Math.random() * 60 + Math.sin(Date.now() / 6000) * 15,
      storage: 15 + Math.random() * 20,
      temperature: 35 + Math.random() * 25 + Math.sin(Date.now() / 15000) * 8,
      power: 150 + Math.random() * 100 + Math.sin(Date.now() / 9000) * 30
    });

    const interval = setInterval(() => {
      setMetrics(prev => {
        const newMetric = generateMetric();
        const updated = [...prev, newMetric].slice(-60); // Keep last 60 points
        return updated;
      });
    }, 2000);

    // Initialize with some data
    const initialMetrics = Array.from({ length: 20 }, generateMetric);
    setMetrics(initialMetrics);

    return () => clearInterval(interval);
  }, []);

  // Draw real-time chart
  useEffect(() => {
    if (!canvasRef.current || metrics.length === 0) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    canvas.width = canvas.offsetWidth;
    canvas.height = canvas.offsetHeight;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    const padding = 40;
    const chartWidth = canvas.width - 2 * padding;
    const chartHeight = canvas.height - 2 * padding;

    // Get data for selected metric
    const data = metrics.map(m => m[selectedMetric]);
    const maxValue = Math.max(...data, 100);
    const minValue = Math.min(...data, 0);
    const valueRange = maxValue - minValue || 1;

    // Draw grid
    ctx.strokeStyle = '#374151';
    ctx.lineWidth = 1;

    for (let i = 0; i <= 5; i++) {
      const x = padding + (i * chartWidth) / 5;
      const y = padding + (i * chartHeight) / 5;
      
      ctx.beginPath();
      ctx.moveTo(x, padding);
      ctx.lineTo(x, canvas.height - padding);
      ctx.stroke();
      
      ctx.beginPath();
      ctx.moveTo(padding, y);
      ctx.lineTo(canvas.width - padding, y);
      ctx.stroke();
    }

    // Draw line
    if (data.length > 1) {
      const colors = {
        cpu: '#3B82F6',
        memory: '#10B981',
        gpu: '#F59E0B',
        network: '#8B5CF6'
      };
      
      ctx.strokeStyle = colors[selectedMetric];
      ctx.lineWidth = 2;
      ctx.beginPath();

      data.forEach((value, index) => {
        const x = padding + (index * chartWidth) / (data.length - 1);
        const y = canvas.height - padding - ((value - minValue) / valueRange) * chartHeight;

        if (index === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      });

      ctx.stroke();

      // Draw points
      ctx.fillStyle = colors[selectedMetric];
      data.forEach((value, index) => {
        const x = padding + (index * chartWidth) / (data.length - 1);
        const y = canvas.height - padding - ((value - minValue) / valueRange) * chartHeight;
        
        ctx.beginPath();
        ctx.arc(x, y, 2, 0, 2 * Math.PI);
        ctx.fill();
      });
    }

    // Draw labels
    ctx.fillStyle = '#9CA3AF';
    ctx.font = '12px monospace';
    ctx.textAlign = 'right';

    for (let i = 0; i <= 5; i++) {
      const value = maxValue - (i * valueRange) / 5;
      const y = padding + (i * chartHeight) / 5;
      ctx.fillText(value.toFixed(0), padding - 10, y + 4);
    }

    // Current value
    ctx.fillStyle = '#FFFFFF';
    ctx.font = '14px Inter';
    ctx.textAlign = 'left';
    ctx.fillText(
      `${selectedMetric.toUpperCase()}: ${data[data.length - 1]?.toFixed(1)}%`,
      padding,
      padding - 10
    );

  }, [metrics, selectedMetric]);

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'up': case 'healthy': return <CheckCircle className="w-4 h-4 text-green-400" />;
      case 'degraded': case 'warning': return <AlertTriangle className="w-4 h-4 text-yellow-400" />;
      case 'down': case 'critical': return <XCircle className="w-4 h-4 text-red-400" />;
    }
  };

  const getAlertIcon = (type: Alert['type']) => {
    switch (type) {
      case 'critical': return <XCircle className="w-4 h-4 text-red-400" />;
      case 'warning': return <AlertTriangle className="w-4 h-4 text-yellow-400" />;
      case 'info': return <CheckCircle className="w-4 h-4 text-blue-400" />;
    }
  };

  const getAlertColor = (type: Alert['type']) => {
    switch (type) {
      case 'critical': return 'bg-red-900/30 border-red-500/30';
      case 'warning': return 'bg-yellow-900/30 border-yellow-500/30';
      case 'info': return 'bg-blue-900/30 border-blue-500/30';
    }
  };

  const currentMetric = metrics[metrics.length - 1];

  return (
    <div className="space-y-6">
      {/* System Overview */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="card p-4">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-medium text-sis-gray-300 flex items-center space-x-2">
              <Activity className="w-4 h-4 text-blue-400" />
              <span>System Health</span>
            </h3>
            {getStatusIcon(status.overall)}
          </div>
          <div className="text-2xl font-bold text-white capitalize">{status.overall}</div>
          <div className="text-xs text-sis-gray-400">Overall Status</div>
        </div>

        <div className="card p-4">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-medium text-sis-gray-300 flex items-center space-x-2">
              <Clock className="w-4 h-4 text-green-400" />
              <span>Uptime</span>
            </h3>
          </div>
          <div className="text-2xl font-bold text-white">{status.uptime.toFixed(1)}h</div>
          <div className="text-xs text-sis-gray-400">System Uptime</div>
        </div>

        <div className="card p-4">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-medium text-sis-gray-300 flex items-center space-x-2">
              <AlertTriangle className="w-4 h-4 text-yellow-400" />
              <span>Active Alerts</span>
            </h3>
          </div>
          <div className="text-2xl font-bold text-white">{alerts.filter(a => !a.acknowledged).length}</div>
          <div className="text-xs text-sis-gray-400">Unacknowledged</div>
        </div>

        <div className="card p-4">
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-medium text-sis-gray-300 flex items-center space-x-2">
              <TrendingUp className="w-4 h-4 text-purple-400" />
              <span>Performance</span>
            </h3>
          </div>
          <div className="text-2xl font-bold text-white">
            {currentMetric ? currentMetric[selectedMetric].toFixed(0) : '0'}
            <span className="text-sm text-sis-gray-400">%</span>
          </div>
          <div className="text-xs text-sis-gray-400 capitalize">{selectedMetric} Usage</div>
        </div>
      </div>

      {/* Real-time Metrics Chart */}
      <div className="card p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Real-time Metrics</h3>
          
          <div className="flex items-center space-x-3">
            <div className="flex items-center space-x-2">
              {['cpu', 'memory', 'gpu', 'network'].map(metric => (
                <button
                  key={metric}
                  onClick={() => setSelectedMetric(metric as any)}
                  className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${
                    selectedMetric === metric
                      ? 'bg-sis-blue-600 text-white'
                      : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
                  }`}
                >
                  {metric.toUpperCase()}
                </button>
              ))}
            </div>
            
            <select
              value={timeRange}
              onChange={(e) => setTimeRange(e.target.value as any)}
              className="bg-sis-gray-800 border border-sis-gray-600 rounded text-white text-sm px-2 py-1"
            >
              <option value="1h">Last Hour</option>
              <option value="6h">Last 6 Hours</option>
              <option value="24h">Last 24 Hours</option>
              <option value="7d">Last 7 Days</option>
            </select>
          </div>
        </div>

        <div className="h-64 bg-sis-gray-800 rounded-lg">
          <canvas
            ref={canvasRef}
            className="w-full h-full"
            style={{ background: '#1F2937' }}
          />
        </div>
      </div>

      {/* Service Status */}
      <div className="card p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Infrastructure Services</h3>
        
        <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
          {Object.entries(status.services).map(([service, serviceStatus]) => {
            const icons = {
              database: Database,
              messageQueue: Activity,
              storage: HardDrive,
              networking: Network,
              security: Shield
            };
            
            const IconComponent = icons[service as keyof typeof icons];
            
            return (
              <div key={service} className="text-center p-4 bg-sis-gray-800 rounded-lg">
                <div className="flex items-center justify-center mb-2">
                  <IconComponent className="w-6 h-6 text-sis-blue-400" />
                </div>
                <h4 className="text-sm font-medium text-white mb-1 capitalize">
                  {service.replace(/([A-Z])/g, ' $1')}
                </h4>
                <div className="flex items-center justify-center space-x-1">
                  {getStatusIcon(serviceStatus)}
                  <span className={`text-xs font-medium capitalize ${
                    serviceStatus === 'up' ? 'text-green-400' : 
                    serviceStatus === 'degraded' ? 'text-yellow-400' : 'text-red-400'
                  }`}>
                    {serviceStatus}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Active Alerts */}
      <div className="card p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Active Alerts</h3>
          <button className="btn-secondary text-sm px-3 py-1">
            Acknowledge All
          </button>
        </div>
        
        <div className="space-y-3">
          {alerts.map(alert => (
            <div
              key={alert.id}
              className={`p-4 rounded-lg border ${getAlertColor(alert.type)} ${
                alert.acknowledged ? 'opacity-60' : ''
              }`}
            >
              <div className="flex items-start justify-between">
                <div className="flex items-start space-x-3">
                  <div className="mt-1">
                    {getAlertIcon(alert.type)}
                  </div>
                  <div>
                    <h4 className="text-white font-medium">{alert.component}</h4>
                    <p className="text-sm text-sis-gray-300 mt-1">{alert.message}</p>
                    <p className="text-xs text-sis-gray-400 mt-2">
                      {alert.timestamp.toLocaleString()}
                    </p>
                  </div>
                </div>
                
                <div className="flex items-center space-x-2">
                  {alert.acknowledged && (
                    <span className="text-xs text-green-400 bg-green-400/20 px-2 py-1 rounded">
                      Acknowledged
                    </span>
                  )}
                  {!alert.acknowledged && (
                    <button className="text-xs btn-primary px-3 py-1">
                      Acknowledge
                    </button>
                  )}
                </div>
              </div>
            </div>
          ))}
          
          {alerts.length === 0 && (
            <div className="text-center py-8 text-sis-gray-400">
              <CheckCircle className="w-12 h-12 text-green-400 mx-auto mb-4" />
              <h4 className="text-lg font-medium text-white mb-2">All Clear</h4>
              <p>No active alerts at this time</p>
            </div>
          )}
        </div>
      </div>

      {/* Resource Utilization */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="card p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Current Resource Usage</h3>
          
          {currentMetric && (
            <div className="space-y-4">
              {[
                { key: 'cpu', label: 'CPU Usage', icon: Cpu, color: 'blue' },
                { key: 'memory', label: 'Memory Usage', icon: HardDrive, color: 'green' },
                { key: 'gpu', label: 'GPU Usage', icon: Activity, color: 'yellow' },
                { key: 'network', label: 'Network Usage', icon: Wifi, color: 'purple' }
              ].map(({ key, label, icon: Icon, color }) => {
                const value = currentMetric[key as keyof SystemMetrics] as number;
                return (
                  <div key={key}>
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center space-x-2">
                        <Icon className={`w-4 h-4 text-${color}-400`} />
                        <span className="text-sm text-sis-gray-300">{label}</span>
                      </div>
                      <span className="text-sm text-white">{value.toFixed(1)}%</span>
                    </div>
                    <div className="w-full bg-sis-gray-700 rounded-full h-2">
                      <div 
                        className={`h-2 rounded-full bg-${color}-500`}
                        style={{ width: `${Math.min(100, value)}%` }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        <div className="card p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Environmental Metrics</h3>
          
          {currentMetric && (
            <div className="space-y-4">
              <div className="text-center p-4 bg-sis-gray-800 rounded-lg">
                <Thermometer className="w-8 h-8 text-red-400 mx-auto mb-2" />
                <div className="text-2xl font-bold text-white">
                  {currentMetric.temperature.toFixed(1)}°C
                </div>
                <div className="text-sm text-sis-gray-400">System Temperature</div>
              </div>
              
              <div className="text-center p-4 bg-sis-gray-800 rounded-lg">
                <Zap className="w-8 h-8 text-yellow-400 mx-auto mb-2" />
                <div className="text-2xl font-bold text-white">
                  {currentMetric.power.toFixed(0)}W
                </div>
                <div className="text-sm text-sis-gray-400">Power Consumption</div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default InfrastructureMonitor;