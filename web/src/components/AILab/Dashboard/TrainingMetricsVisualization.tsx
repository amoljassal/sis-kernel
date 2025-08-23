/**
 * Training Metrics Visualization
 * Real-time charts and visualizations for training metrics
 */

import React, { useState, useEffect, useRef } from 'react';
import { TrendingUp, Activity, Eye, Settings } from 'lucide-react';

interface MetricPoint {
  epoch: number;
  loss: number;
  accuracy: number;
  learningRate: number;
  timestamp: number;
}

interface TrainingMetricsVisualizationProps {
  sessionId: string;
  isActive: boolean;
  className?: string;
}

export const TrainingMetricsVisualization: React.FC<TrainingMetricsVisualizationProps> = ({
  sessionId,
  isActive,
  className = ''
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [metrics, setMetrics] = useState<MetricPoint[]>([]);
  const [selectedMetric, setSelectedMetric] = useState<'loss' | 'accuracy' | 'learning_rate'>('loss');
  const [timeRange, setTimeRange] = useState<'last_hour' | 'last_day' | 'all'>('last_hour');

  // Simulate real-time metrics generation
  useEffect(() => {
    if (!isActive) return;

    const interval = setInterval(() => {
      const now = Date.now();
      const epoch = metrics.length + 1;
      
      const newPoint: MetricPoint = {
        epoch,
        loss: Math.max(0.1, 2.0 - (epoch / 100) * 1.8 + Math.random() * 0.2),
        accuracy: Math.min(0.95, (epoch / 100) * 0.85 + Math.random() * 0.05),
        learningRate: 0.001 * Math.pow(0.95, Math.floor(epoch / 10)),
        timestamp: now
      };

      setMetrics(prev => [...prev.slice(-99), newPoint]); // Keep last 100 points
    }, 2000);

    return () => clearInterval(interval);
  }, [isActive, metrics.length]);

  // Canvas drawing
  useEffect(() => {
    if (!canvasRef.current || metrics.length === 0) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Set canvas size
    canvas.width = canvas.offsetWidth;
    canvas.height = canvas.offsetHeight;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Setup
    const padding = 40;
    const chartWidth = canvas.width - 2 * padding;
    const chartHeight = canvas.height - 2 * padding;

    // Get data for selected metric
    const data = metrics.map(point => {
      switch (selectedMetric) {
        case 'accuracy': return point.accuracy;
        case 'learning_rate': return point.learningRate;
        default: return point.loss;
      }
    });

    const minValue = Math.min(...data);
    const maxValue = Math.max(...data);
    const valueRange = maxValue - minValue || 1;

    // Draw grid
    ctx.strokeStyle = '#374151';
    ctx.lineWidth = 1;

    // Vertical lines
    for (let i = 0; i <= 5; i++) {
      const x = padding + (i * chartWidth) / 5;
      ctx.beginPath();
      ctx.moveTo(x, padding);
      ctx.lineTo(x, canvas.height - padding);
      ctx.stroke();
    }

    // Horizontal lines
    for (let i = 0; i <= 5; i++) {
      const y = padding + (i * chartHeight) / 5;
      ctx.beginPath();
      ctx.moveTo(padding, y);
      ctx.lineTo(canvas.width - padding, y);
      ctx.stroke();
    }

    // Draw chart line
    if (data.length > 1) {
      ctx.strokeStyle = selectedMetric === 'loss' ? '#EF4444' : 
                        selectedMetric === 'accuracy' ? '#10B981' : '#F59E0B';
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
      ctx.fillStyle = ctx.strokeStyle;
      data.forEach((value, index) => {
        const x = padding + (index * chartWidth) / (data.length - 1);
        const y = canvas.height - padding - ((value - minValue) / valueRange) * chartHeight;
        
        ctx.beginPath();
        ctx.arc(x, y, 3, 0, 2 * Math.PI);
        ctx.fill();
      });
    }

    // Draw labels
    ctx.fillStyle = '#9CA3AF';
    ctx.font = '12px monospace';
    ctx.textAlign = 'center';

    // Y-axis labels
    ctx.textAlign = 'right';
    for (let i = 0; i <= 5; i++) {
      const value = maxValue - (i * valueRange) / 5;
      const y = padding + (i * chartHeight) / 5;
      ctx.fillText(value.toFixed(4), padding - 10, y + 4);
    }

    // X-axis labels
    ctx.textAlign = 'center';
    for (let i = 0; i <= 5; i++) {
      const epochIndex = Math.floor((i * (data.length - 1)) / 5);
      const epoch = metrics[epochIndex]?.epoch || 0;
      const x = padding + (i * chartWidth) / 5;
      ctx.fillText(`E${epoch}`, x, canvas.height - padding + 20);
    }

    // Title
    ctx.fillStyle = '#FFFFFF';
    ctx.font = '14px Inter';
    ctx.textAlign = 'left';
    ctx.fillText(
      selectedMetric.charAt(0).toUpperCase() + selectedMetric.slice(1).replace('_', ' '),
      padding,
      padding - 10
    );

    // Current value
    ctx.textAlign = 'right';
    const currentValue = data[data.length - 1];
    if (currentValue !== undefined) {
      ctx.fillText(
        `Current: ${currentValue.toFixed(4)}`,
        canvas.width - padding,
        padding - 10
      );
    }

  }, [metrics, selectedMetric]);

  const getMetricColor = (metric: string) => {
    switch (metric) {
      case 'loss': return 'text-red-400 bg-red-900/30';
      case 'accuracy': return 'text-green-400 bg-green-900/30';
      case 'learning_rate': return 'text-yellow-400 bg-yellow-900/30';
      default: return 'text-gray-400 bg-gray-900/30';
    }
  };

  const getCurrentValue = () => {
    if (metrics.length === 0) return 'N/A';
    const latest = metrics[metrics.length - 1];
    
    switch (selectedMetric) {
      case 'accuracy':
        return (latest.accuracy * 100).toFixed(2) + '%';
      case 'learning_rate':
        return latest.learningRate.toExponential(2);
      default:
        return latest.loss.toFixed(4);
    }
  };

  return (
    <div className={`bg-sis-gray-900 rounded-lg border border-sis-gray-700 ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-sis-gray-700">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold text-white flex items-center space-x-2">
            <TrendingUp className="w-5 h-5 text-sis-blue-400" />
            <span>Training Metrics</span>
          </h3>
          
          <div className="flex items-center space-x-2">
            <select
              value={timeRange}
              onChange={(e) => setTimeRange(e.target.value as any)}
              className="bg-sis-gray-800 border border-sis-gray-600 rounded text-white text-sm px-2 py-1"
            >
              <option value="last_hour">Last Hour</option>
              <option value="last_day">Last Day</option>
              <option value="all">All Time</option>
            </select>
            
            <button className="p-1 text-sis-gray-400 hover:text-white transition-colors">
              <Settings className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Metric Selector */}
        <div className="flex items-center space-x-2 mt-4">
          {['loss', 'accuracy', 'learning_rate'].map(metric => (
            <button
              key={metric}
              onClick={() => setSelectedMetric(metric as any)}
              className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${
                selectedMetric === metric
                  ? getMetricColor(metric)
                  : 'text-sis-gray-400 hover:text-white'
              }`}
            >
              {metric.charAt(0).toUpperCase() + metric.slice(1).replace('_', ' ')}
            </button>
          ))}
        </div>
      </div>

      {/* Chart Area */}
      <div className="p-4">
        {isActive && metrics.length > 0 ? (
          <div className="space-y-4">
            {/* Current Value Display */}
            <div className="text-center">
              <div className="text-3xl font-bold text-white mb-2">
                {getCurrentValue()}
              </div>
              <div className="text-sm text-sis-gray-400">
                Current {selectedMetric.replace('_', ' ')} value
              </div>
            </div>

            {/* Canvas Chart */}
            <div className="w-full h-64 relative">
              <canvas
                ref={canvasRef}
                className="w-full h-full"
                style={{ background: '#1F2937' }}
              />
            </div>

            {/* Stats Summary */}
            <div className="grid grid-cols-3 gap-4 text-sm">
              <div className="text-center">
                <div className="text-white font-mono">{metrics.length}</div>
                <div className="text-sis-gray-400">Total Epochs</div>
              </div>
              
              <div className="text-center">
                <div className="text-white font-mono">
                  {isActive ? (
                    <span className="flex items-center justify-center space-x-1">
                      <Activity className="w-3 h-3 text-green-400 animate-pulse" />
                      <span>Live</span>
                    </span>
                  ) : (
                    'Stopped'
                  )}
                </div>
                <div className="text-sis-gray-400">Status</div>
              </div>
              
              <div className="text-center">
                <div className="text-white font-mono">
                  {metrics.length > 0 ? 
                    new Date(metrics[metrics.length - 1].timestamp).toLocaleTimeString() : 
                    'N/A'
                  }
                </div>
                <div className="text-sis-gray-400">Last Update</div>
              </div>
            </div>
          </div>
        ) : (
          <div className="text-center py-12">
            <Eye className="w-12 h-12 text-sis-gray-600 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-white mb-2">
              {isActive ? 'Waiting for metrics...' : 'Training not active'}
            </h3>
            <p className="text-sis-gray-400">
              {isActive ? 
                'Metrics will appear once training begins' : 
                'Start a training session to view real-time metrics'
              }
            </p>
          </div>
        )}
      </div>
    </div>
  );
};

export default TrainingMetricsVisualization;