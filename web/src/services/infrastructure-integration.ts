// Infrastructure Integration Service
// Orchestrates scaling across Database, Redis, WebSocket, and Web services

import { predictiveAutoScaling } from './predictive-autoscaling';
import { PHASE_5C_INTEGRATION, AutoScalingIntegrator, SCALING_ORCHESTRATION } from '../config/autoscaling-config';
import type { ResourceRecommendation } from './predictive-autoscaling';

// Note: Database and WebSocket scaling configurations are integrated via PHASE_5C_INTEGRATION

interface ScalingEvent {
  id: string;
  timestamp: Date;
  component: string;
  action: 'scale_up' | 'scale_down' | 'health_check' | 'rollback';
  status: 'initiated' | 'in_progress' | 'completed' | 'failed';
  details: any;
}

interface InfrastructureStatus {
  database: {
    primary: { instances: number; status: string; connections: number };
    replicas: { [region: string]: { instances: number; lag: number } };
  };
  redis: {
    l1: { instances: number; hitRate: number; memory: number };
    l2: { instances: number; hitRate: number; memory: number };
    l3: { instances: number; hitRate: number; memory: number };
  };
  websocket: {
    gateways: { [region: string]: { instances: number; connections: number } };
    kafka: { brokers: number; partitions: number; lag: number };
  };
  webServers: {
    instances: number;
    cpu: number;
    memory: number;
    responseTime: number;
  };
  loadBalancer: {
    activeConnections: number;
    requestRate: number;
    healthyTargets: number;
  };
}

export class InfrastructureIntegrationService {
  private currentStatus: InfrastructureStatus;
  private scalingEvents: ScalingEvent[] = [];
  private isScaling: boolean = false;

  constructor() {
    this.currentStatus = this.initializeStatus();
    this.startMonitoring();
    this.setupScalingEventListeners();
  }

  private initializeStatus(): InfrastructureStatus {
    return {
      database: {
        primary: { instances: 2, status: 'healthy', connections: 450 },
        replicas: {
          'mumbai': { instances: 2, lag: 15 },
          'delhi': { instances: 1, lag: 25 },
          'bangalore': { instances: 1, lag: 20 }
        }
      },
      redis: {
        l1: { instances: 3, hitRate: 0.92, memory: 0.65 },
        l2: { instances: 3, hitRate: 0.88, memory: 0.70 },
        l3: { instances: 2, hitRate: 0.85, memory: 0.60 }
      },
      websocket: {
        gateways: {
          'mumbai': { instances: 2, connections: 3500 },
          'delhi': { instances: 1, connections: 1800 },
          'bangalore': { instances: 1, connections: 2200 },
          'chennai': { instances: 1, connections: 1200 }
        },
        kafka: { brokers: 3, partitions: 24, lag: 150 }
      },
      webServers: {
        instances: 5,
        cpu: 0.68,
        memory: 0.72,
        responseTime: 85
      },
      loadBalancer: {
        activeConnections: 2800,
        requestRate: 450,
        healthyTargets: 5
      }
    };
  }

  private startMonitoring(): void {
    // Monitor infrastructure health every 30 seconds
    setInterval(async () => {
      await this.updateInfrastructureStatus();
      this.checkHealthAndScale();
    }, 30000);

    // Generate detailed reports every 5 minutes
    setInterval(() => {
      this.generateScalingReport();
    }, 300000);
  }

  private setupScalingEventListeners(): void {
    // Listen for scaling events from the predictive auto-scaler
    predictiveAutoScaling.on('scaling-complete', (event: any) => {
      this.handleScalingComplete(event);
    });

    predictiveAutoScaling.on('scaling-failed', (error: any) => {
      this.handleScalingFailure(error);
    });
  }

  private async updateInfrastructureStatus(): Promise<void> {
    try {
      // Update database status
      await this.updateDatabaseStatus();
      
      // Update Redis status
      await this.updateRedisStatus();
      
      // Update WebSocket status
      await this.updateWebSocketStatus();
      
      // Update web server status
      await this.updateWebServerStatus();
      
      // Update load balancer status
      await this.updateLoadBalancerStatus();
      
    } catch (error) {
      console.error('Failed to update infrastructure status:', error);
    }
  }

  private async updateDatabaseStatus(): Promise<void> {
    // Simulate database metrics collection
    const dbMetrics = await this.collectDatabaseMetrics();
    
    this.currentStatus.database = {
      primary: {
        instances: dbMetrics.primary.instanceCount,
        status: dbMetrics.primary.healthStatus,
        connections: dbMetrics.primary.activeConnections
      },
      replicas: dbMetrics.replicas
    };

    // Check if database scaling is needed
    if (dbMetrics.primary.activeConnections > 
        PHASE_5C_INTEGRATION.DATABASE.SCALING_POLICIES.CONNECTIONS_THRESHOLD * 
        this.currentStatus.database.primary.instances) {
      
      await this.triggerDatabaseScaling('up', {
        reason: 'High connection count',
        connections: dbMetrics.primary.activeConnections
      });
    }
  }

  private async collectDatabaseMetrics(): Promise<any> {
    // Simulate collecting real database metrics
    const baseConnections = 400;
    const currentHour = new Date().getHours();
    
    // Simulate peak hour traffic
    let connectionMultiplier = 1.0;
    if (currentHour >= 9 && currentHour < 12) connectionMultiplier = 2.2;
    else if (currentHour >= 19 && currentHour < 23) connectionMultiplier = 2.8;
    
    return {
      primary: {
        instanceCount: this.currentStatus.database.primary.instances,
        healthStatus: 'healthy',
        activeConnections: Math.floor(baseConnections * connectionMultiplier * (0.8 + Math.random() * 0.4))
      },
      replicas: {
        'mumbai': { 
          instances: this.currentStatus.database.replicas['mumbai'].instances, 
          lag: 10 + Math.random() * 20 
        },
        'delhi': { 
          instances: this.currentStatus.database.replicas['delhi'].instances, 
          lag: 20 + Math.random() * 15 
        },
        'bangalore': { 
          instances: this.currentStatus.database.replicas['bangalore'].instances, 
          lag: 15 + Math.random() * 18 
        }
      }
    };
  }

  private async updateRedisStatus(): Promise<void> {
    const redisMetrics = await this.collectRedisMetrics();
    
    this.currentStatus.redis = redisMetrics;
    
    // Check if Redis scaling is needed
    Object.entries(redisMetrics).forEach(async ([layer, metrics]: [string, any]) => {
      if (metrics.hitRate < PHASE_5C_INTEGRATION.REDIS.SCALING_TRIGGERS.HIT_RATIO_THRESHOLD ||
          metrics.memory > PHASE_5C_INTEGRATION.REDIS.SCALING_TRIGGERS.MEMORY_USAGE_THRESHOLD) {
        
        await this.triggerRedisScaling(layer as 'l1' | 'l2' | 'l3', 'up', {
          reason: metrics.hitRate < 0.85 ? 'Low hit rate' : 'High memory usage',
          hitRate: metrics.hitRate,
          memory: metrics.memory
        });
      }
    });
  }

  private async collectRedisMetrics(): Promise<any> {
    const currentHour = new Date().getHours();
    const isPeak = (currentHour >= 9 && currentHour < 12) || (currentHour >= 19 && currentHour < 23);
    
    // Simulate cache performance during peak hours
    const baseHitRate = isPeak ? 0.85 : 0.92;
    const baseMemory = isPeak ? 0.75 : 0.60;
    
    return {
      l1: {
        instances: this.currentStatus.redis.l1.instances,
        hitRate: baseHitRate + Math.random() * 0.1,
        memory: baseMemory + Math.random() * 0.2
      },
      l2: {
        instances: this.currentStatus.redis.l2.instances,
        hitRate: (baseHitRate - 0.05) + Math.random() * 0.1,
        memory: (baseMemory + 0.05) + Math.random() * 0.15
      },
      l3: {
        instances: this.currentStatus.redis.l3.instances,
        hitRate: (baseHitRate - 0.08) + Math.random() * 0.12,
        memory: (baseMemory - 0.05) + Math.random() * 0.2
      }
    };
  }

  private async updateWebSocketStatus(): Promise<void> {
    const wsMetrics = await this.collectWebSocketMetrics();
    
    this.currentStatus.websocket = wsMetrics;
    
    // Check WebSocket scaling needs
    Object.entries(wsMetrics.gateways).forEach(async ([region, metrics]: [string, any]) => {
      const capacity = this.getWebSocketCapacity(region);
      const utilization = metrics.connections / (metrics.instances * capacity);
      
      if (utilization > PHASE_5C_INTEGRATION.WEBSOCKET.SCALING_METRICS.CONNECTION_THRESHOLD) {
        await this.triggerWebSocketScaling(region, 'up', {
          reason: 'High connection utilization',
          utilization,
          connections: metrics.connections
        });
      }
    });
  }

  private async collectWebSocketMetrics(): Promise<any> {
    const currentHour = new Date().getHours();
    const isCollaborationPeak = currentHour >= 10 && currentHour <= 18; // Work/study hours
    
    const baseConnections = {
      'mumbai': 2500,
      'delhi': 1500,
      'bangalore': 1800,
      'chennai': 1000
    };
    
    const multiplier = isCollaborationPeak ? 1.8 : 1.0;
    
    return {
      gateways: Object.entries(baseConnections).reduce((acc, [region, base]) => {
        acc[region] = {
          instances: this.currentStatus.websocket.gateways[region].instances,
          connections: Math.floor(base * multiplier * (0.7 + Math.random() * 0.6))
        };
        return acc;
      }, {} as any),
      kafka: {
        brokers: this.currentStatus.websocket.kafka.brokers,
        partitions: 24,
        lag: 100 + Math.random() * 100
      }
    };
  }

  private async updateWebServerStatus(): Promise<void> {
    const serverMetrics = await this.collectWebServerMetrics();
    
    this.currentStatus.webServers = serverMetrics;
    
    // Trigger web server scaling if needed
    if (serverMetrics.cpu > 0.85 || serverMetrics.responseTime > 150) {
      await this.triggerWebServerScaling('up', {
        reason: serverMetrics.cpu > 0.85 ? 'High CPU usage' : 'High response time',
        cpu: serverMetrics.cpu,
        responseTime: serverMetrics.responseTime
      });
    }
  }

  private async collectWebServerMetrics(): Promise<any> {
    const currentHour = new Date().getHours();
    const isPeak = (currentHour >= 9 && currentHour < 12) || (currentHour >= 19 && currentHour < 23);
    
    const baseCpu = isPeak ? 0.75 : 0.45;
    const baseMemory = isPeak ? 0.80 : 0.55;
    const baseResponseTime = isPeak ? 120 : 60;
    
    return {
      instances: this.currentStatus.webServers.instances,
      cpu: baseCpu + Math.random() * 0.2,
      memory: baseMemory + Math.random() * 0.15,
      responseTime: baseResponseTime + Math.random() * 40
    };
  }

  private async updateLoadBalancerStatus(): Promise<void> {
    const lbMetrics = await this.collectLoadBalancerMetrics();
    this.currentStatus.loadBalancer = lbMetrics;
  }

  private async collectLoadBalancerMetrics(): Promise<any> {
    const totalConnections = Object.values(this.currentStatus.websocket.gateways)
      .reduce((sum, gateway) => sum + gateway.connections, 0);
    
    return {
      activeConnections: totalConnections,
      requestRate: 300 + Math.random() * 200,
      healthyTargets: this.currentStatus.webServers.instances
    };
  }

  // Scaling orchestration methods
  private async triggerDatabaseScaling(direction: 'up' | 'down', _context: any): Promise<void> {
    if (this.isScaling) return;
    
    const scalingEvent: ScalingEvent = {
      id: this.generateEventId(),
      timestamp: new Date(),
      component: 'database',
      action: direction === 'up' ? 'scale_up' : 'scale_down',
      status: 'initiated',
      details: _context
    };
    
    this.scalingEvents.push(scalingEvent);
    console.log(`Database scaling ${direction} initiated:`, _context);
    
    try {
      await this.executeDatabaseScaling(direction, _context);
      scalingEvent.status = 'completed';
    } catch (error) {
      scalingEvent.status = 'failed';
      console.error('Database scaling failed:', error);
    }
  }

  private async executeDatabaseScaling(direction: 'up' | 'down', _context: any): Promise<void> {
    const currentInstances = this.currentStatus.database.primary.instances;
    const maxInstances = PHASE_5C_INTEGRATION.DATABASE.PRIMARY_CLUSTER.maxInstances;
    const minInstances = PHASE_5C_INTEGRATION.DATABASE.PRIMARY_CLUSTER.minInstances;
    
    let newInstances: number;
    if (direction === 'up') {
      newInstances = Math.min(currentInstances + 1, maxInstances);
    } else {
      newInstances = Math.max(currentInstances - 1, minInstances);
    }
    
    if (newInstances !== currentInstances) {
      // Simulate database scaling
      this.currentStatus.database.primary.instances = newInstances;
      
      // Add regional read replicas if scaling up significantly
      if (direction === 'up' && newInstances > 4) {
        this.scaleReadReplicas('up');
      }
    }
  }

  private async scaleReadReplicas(direction: 'up' | 'down'): Promise<void> {
    const replicas = this.currentStatus.database.replicas;
    const increment = direction === 'up' ? 1 : -1;
    
    Object.keys(replicas).forEach(region => {
      const config = PHASE_5C_INTEGRATION.DATABASE.READ_REPLICAS.regions
        .find(r => r.name.includes(region.toLowerCase()));
      
      if (config) {
        const current = replicas[region].instances;
        const newCount = Math.max(1, Math.min(current + increment, config.maxInstances));
        replicas[region].instances = newCount;
      }
    });
  }

  private async triggerRedisScaling(layer: 'l1' | 'l2' | 'l3', direction: 'up' | 'down', context: any): Promise<void> {
    if (this.isScaling) return;
    
    const scalingEvent: ScalingEvent = {
      id: this.generateEventId(),
      timestamp: new Date(),
      component: `redis_${layer}`,
      action: direction === 'up' ? 'scale_up' : 'scale_down',
      status: 'initiated',
      details: context
    };
    
    this.scalingEvents.push(scalingEvent);
    console.log(`Redis ${layer} scaling ${direction} initiated:`, context);
    
    try {
      await this.executeRedisScaling(layer, direction, context);
      scalingEvent.status = 'completed';
    } catch (error) {
      scalingEvent.status = 'failed';
      console.error(`Redis ${layer} scaling failed:`, error);
    }
  }

  private async executeRedisScaling(layer: 'l1' | 'l2' | 'l3', direction: 'up' | 'down', _context: any): Promise<void> {
    const configMap = {
      'l1': PHASE_5C_INTEGRATION.REDIS.L1_CACHE,
      'l2': PHASE_5C_INTEGRATION.REDIS.L2_CACHE,
      'l3': PHASE_5C_INTEGRATION.REDIS.L3_CACHE
    };
    
    const config = configMap[layer];
    const currentInstances = this.currentStatus.redis[layer].instances;
    const increment = direction === 'up' ? 1 : -1;
    
    const newInstances = Math.max(
      config.instances.min,
      Math.min(currentInstances + increment, config.instances.max)
    );
    
    if (newInstances !== currentInstances) {
      this.currentStatus.redis[layer].instances = newInstances;
    }
  }

  private async triggerWebSocketScaling(region: string, direction: 'up' | 'down', _context: any): Promise<void> {
    if (this.isScaling) return;
    
    const scalingEvent: ScalingEvent = {
      id: this.generateEventId(),
      timestamp: new Date(),
      component: `websocket_${region}`,
      action: direction === 'up' ? 'scale_up' : 'scale_down',
      status: 'initiated',
      details: _context
    };
    
    this.scalingEvents.push(scalingEvent);
    console.log(`WebSocket ${region} scaling ${direction} initiated:`, _context);
    
    try {
      await this.executeWebSocketScaling(region, direction, _context);
      scalingEvent.status = 'completed';
    } catch (error) {
      scalingEvent.status = 'failed';
      console.error(`WebSocket ${region} scaling failed:`, error);
    }
  }

  private async executeWebSocketScaling(region: string, direction: 'up' | 'down', _context: any): Promise<void> {
    const currentInstances = this.currentStatus.websocket.gateways[region].instances;
    const config = this.getWebSocketConfig(region);
    const increment = direction === 'up' ? 1 : -1;
    
    const newInstances = Math.max(
      config.instances.min,
      Math.min(currentInstances + increment, config.instances.max)
    );
    
    if (newInstances !== currentInstances) {
      this.currentStatus.websocket.gateways[region].instances = newInstances;
      
      // Scale Kafka brokers if WebSocket traffic increases significantly
      if (direction === 'up' && this.getTotalWebSocketInstances() > 8) {
        await this.scaleKafkaBrokers('up');
      }
    }
  }

  private async triggerWebServerScaling(direction: 'up' | 'down', _context: any): Promise<void> {
    if (this.isScaling) return;
    
    const scalingEvent: ScalingEvent = {
      id: this.generateEventId(),
      timestamp: new Date(),
      component: 'web_servers',
      action: direction === 'up' ? 'scale_up' : 'scale_down',
      status: 'initiated',
      details: _context
    };
    
    this.scalingEvents.push(scalingEvent);
    console.log(`Web server scaling ${direction} initiated:`, _context);
    
    try {
      await this.executeWebServerScaling(direction, _context);
      scalingEvent.status = 'completed';
    } catch (error) {
      scalingEvent.status = 'failed';
      console.error('Web server scaling failed:', error);
    }
  }

  private async executeWebServerScaling(direction: 'up' | 'down', _context: any): Promise<void> {
    const currentInstances = this.currentStatus.webServers.instances;
    const minInstances = 3;
    const maxInstances = 50;
    const increment = direction === 'up' ? 2 : 1; // Scale up faster
    
    let newInstances: number;
    if (direction === 'up') {
      newInstances = Math.min(currentInstances + increment, maxInstances);
    } else {
      newInstances = Math.max(currentInstances - increment, minInstances);
    }
    
    if (newInstances !== currentInstances) {
      this.currentStatus.webServers.instances = newInstances;
      this.currentStatus.loadBalancer.healthyTargets = newInstances;
    }
  }

  private async scaleKafkaBrokers(direction: 'up' | 'down'): Promise<void> {
    const currentBrokers = this.currentStatus.websocket.kafka.brokers;
    const config = PHASE_5C_INTEGRATION.WEBSOCKET.KAFKA_BROKERS;
    const increment = direction === 'up' ? 1 : -1;
    
    const newBrokers = Math.max(
      config.instances.min,
      Math.min(currentBrokers + increment, config.instances.max)
    );
    
    if (newBrokers !== currentBrokers) {
      this.currentStatus.websocket.kafka.brokers = newBrokers;
    }
  }

  // Orchestrated scaling - scales multiple components in sequence
  public async orchestratedScaling(recommendations: ResourceRecommendation, direction: 'up' | 'down'): Promise<void> {
    if (this.isScaling) {
      console.log('Scaling already in progress, skipping orchestrated scaling');
      return;
    }
    
    this.isScaling = true;
    console.log(`Starting orchestrated scaling ${direction}:`, recommendations);
    
    try {
      const sequence = AutoScalingIntegrator.getScalingSequence(direction);
      
      for (const step of sequence) {
        console.log(`Scaling ${step.component} in ${step.delay}ms...`);
        
        await new Promise(resolve => setTimeout(resolve, step.delay * 1000));
        
        switch (step.component) {
          case 'web_servers':
            await this.executeWebServerScaling(direction, { orchestrated: true });
            break;
          case 'database_replicas':
            await this.executeDatabaseScaling(direction, { orchestrated: true });
            break;
          case 'redis_cache':
            await this.executeRedisScaling('l1', direction, { orchestrated: true });
            await this.executeRedisScaling('l2', direction, { orchestrated: true });
            break;
          case 'websocket_gateways':
            for (const region of Object.keys(this.currentStatus.websocket.gateways)) {
              await this.executeWebSocketScaling(region, direction, { orchestrated: true });
            }
            break;
          case 'kafka_brokers':
            await this.scaleKafkaBrokers(direction);
            break;
          case 'load_balancer':
            // Load balancer configuration update
            console.log('Updating load balancer configuration...');
            break;
        }
      }
      
      console.log(`Orchestrated scaling ${direction} completed successfully`);
      
    } catch (error) {
      console.error('Orchestrated scaling failed:', error);
      await this.rollbackScaling();
    } finally {
      this.isScaling = false;
    }
  }

  private async rollbackScaling(): Promise<void> {
    console.log('Initiating scaling rollback...');
    
    const rollbackConfig = SCALING_ORCHESTRATION.ROLLBACK;
    if (!rollbackConfig.enabled) return;
    
    // Implement rollback logic here
    // This would restore the previous known good state
  }

  // Health check and monitoring
  private checkHealthAndScale(): void {
    // const healthChecks = SCALING_ORCHESTRATION.HEALTH_CHECKS;
    
    // Check database health
    const dbHealth = this.checkDatabaseHealth();
    if (!dbHealth.healthy) {
      console.warn('Database health check failed:', dbHealth.issues);
    }
    
    // Check Redis health
    const redisHealth = this.checkRedisHealth();
    if (!redisHealth.healthy) {
      console.warn('Redis health check failed:', redisHealth.issues);
    }
    
    // Check WebSocket health
    const wsHealth = this.checkWebSocketHealth();
    if (!wsHealth.healthy) {
      console.warn('WebSocket health check failed:', wsHealth.issues);
    }
  }

  private checkDatabaseHealth(): { healthy: boolean; issues: string[] } {
    const issues: string[] = [];
    const db = this.currentStatus.database;
    const config = PHASE_5C_INTEGRATION.DATABASE.SCALING_POLICIES;
    
    // Check connection pool utilization
    const connectionUtilization = db.primary.connections / (db.primary.instances * config.CONNECTIONS_THRESHOLD);
    if (connectionUtilization > 0.9) {
      issues.push(`High connection pool utilization: ${(connectionUtilization * 100).toFixed(1)}%`);
    }
    
    // Check replication lag
    Object.entries(db.replicas).forEach(([region, replica]) => {
      if (replica.lag > config.REPLICATION_LAG_THRESHOLD) {
        issues.push(`High replication lag in ${region}: ${replica.lag}ms`);
      }
    });
    
    return { healthy: issues.length === 0, issues };
  }

  private checkRedisHealth(): { healthy: boolean; issues: string[] } {
    const issues: string[] = [];
    const redis = this.currentStatus.redis;
    const triggers = PHASE_5C_INTEGRATION.REDIS.SCALING_TRIGGERS;
    
    Object.entries(redis).forEach(([layer, metrics]) => {
      if (metrics.hitRate < triggers.HIT_RATIO_THRESHOLD) {
        issues.push(`Low hit rate in ${layer}: ${(metrics.hitRate * 100).toFixed(1)}%`);
      }
      if (metrics.memory > triggers.MEMORY_USAGE_THRESHOLD) {
        issues.push(`High memory usage in ${layer}: ${(metrics.memory * 100).toFixed(1)}%`);
      }
    });
    
    return { healthy: issues.length === 0, issues };
  }

  private checkWebSocketHealth(): { healthy: boolean; issues: string[] } {
    const issues: string[] = [];
    const ws = this.currentStatus.websocket;
    
    Object.entries(ws.gateways).forEach(([region, gateway]) => {
      const capacity = this.getWebSocketCapacity(region);
      const utilization = gateway.connections / (gateway.instances * capacity);
      
      if (utilization > 0.9) {
        issues.push(`High connection utilization in ${region}: ${(utilization * 100).toFixed(1)}%`);
      }
    });
    
    if (ws.kafka.lag > 500) {
      issues.push(`High Kafka consumer lag: ${ws.kafka.lag}ms`);
    }
    
    return { healthy: issues.length === 0, issues };
  }

  // Utility methods
  private getWebSocketCapacity(region: string): number {
    if (region === 'mumbai') return 2000;
    if (region === 'delhi' || region === 'bangalore') return 1500;
    return 1000; // chennai and others
  }

  private getWebSocketConfig(region: string): any {
    if (region === 'mumbai') {
      return PHASE_5C_INTEGRATION.WEBSOCKET.GATEWAYS.MUMBAI_PRIMARY;
    }
    return PHASE_5C_INTEGRATION.WEBSOCKET.GATEWAYS.REGIONAL_GATEWAYS
      .find(g => g.region === region) || { instances: { min: 1, max: 3 } };
  }

  private getTotalWebSocketInstances(): number {
    return Object.values(this.currentStatus.websocket.gateways)
      .reduce((sum, gateway) => sum + gateway.instances, 0);
  }

  private generateEventId(): string {
    return `scale_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private generateScalingReport(): void {
    const report = {
      timestamp: new Date().toISOString(),
      infrastructure: this.currentStatus,
      recentEvents: this.scalingEvents.slice(-20),
      recommendations: this.generateRecommendations()
    };
    
    console.log('Infrastructure Scaling Report:', report);
  }

  private generateRecommendations(): string[] {
    const recommendations: string[] = [];
    const status = this.currentStatus;
    
    // Database recommendations
    if (status.database.primary.connections > 800) {
      recommendations.push('Consider adding more database read replicas');
    }
    
    // Redis recommendations
    if (status.redis.l1.hitRate < 0.9) {
      recommendations.push('L1 cache hit rate is below optimal, consider scaling up');
    }
    
    // WebSocket recommendations
    const totalWsConnections = Object.values(status.websocket.gateways)
      .reduce((sum, gw) => sum + gw.connections, 0);
    if (totalWsConnections > 8000) {
      recommendations.push('Consider adding regional WebSocket gateways');
    }
    
    return recommendations;
  }

  // Public interface
  public getInfrastructureStatus(): InfrastructureStatus {
    return { ...this.currentStatus };
  }

  public getScalingEvents(limit = 50): ScalingEvent[] {
    return this.scalingEvents.slice(-limit);
  }

  public async manualScale(component: string, direction: 'up' | 'down', context: any = {}): Promise<void> {
    switch (component) {
      case 'database':
        await this.triggerDatabaseScaling(direction, { ...context, manual: true });
        break;
      case 'redis':
        await this.triggerRedisScaling('l1', direction, { ...context, manual: true });
        break;
      case 'websocket':
        for (const region of Object.keys(this.currentStatus.websocket.gateways)) {
          await this.triggerWebSocketScaling(region, direction, { ...context, manual: true });
        }
        break;
      case 'web_servers':
        await this.triggerWebServerScaling(direction, { ...context, manual: true });
        break;
      default:
        throw new Error(`Unknown component: ${component}`);
    }
  }

  public isCurrentlyScaling(): boolean {
    return this.isScaling;
  }

  private handleScalingComplete(event: any): void {
    console.log('Predictive scaling completed, updating infrastructure:', event);
    // Update infrastructure based on predictive scaling recommendations
  }

  private handleScalingFailure(error: any): void {
    console.error('Predictive scaling failed:', error);
    // Implement fallback or alerting logic
  }
}

// Export singleton instance
export const infrastructureIntegration = new InfrastructureIntegrationService();

export type { ScalingEvent, InfrastructureStatus };