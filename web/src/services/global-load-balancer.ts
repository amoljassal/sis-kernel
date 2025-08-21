// Phase 6B: Global Load Balancer
// Intelligent traffic routing across global regions with health monitoring
// @ts-nocheck

import { GLOBAL_REGIONS } from '../config/global-infrastructure';
import GlobalDeploymentManager from './global-deployment-manager';

export interface LoadBalancerNode {
  regionId: string;
  endpoint: string;
  weight: number; // 0-100
  currentLoad: number; // 0-100
  health: number; // 0-100
  latency: number; // ms
  capacity: number; // requests per second
  activeConnections: number;
  status: 'active' | 'draining' | 'maintenance' | 'failed';
  location: { lat: number; lng: number };
}

export interface RoutingRule {
  id: string;
  name: string;
  priority: number;
  conditions: RoutingCondition[];
  target: string; // regionId
  sticky: boolean; // session affinity
  enabled: boolean;
}

export interface RoutingCondition {
  type: 'geo' | 'header' | 'path' | 'query' | 'ip_range';
  field: string;
  operator: 'equals' | 'contains' | 'starts_with' | 'in_range' | 'matches';
  value: string;
}

export interface TrafficMetrics {
  totalRequests: number;
  requestsPerSecond: number;
  averageLatency: number;
  errorRate: number;
  bandwidthUsage: number; // Mbps
  regionDistribution: { [regionId: string]: number };
  topPaths: { path: string; requests: number }[];
}

export interface HealthCheck {
  regionId: string;
  endpoint: string;
  method: 'GET' | 'POST' | 'HEAD';
  path: string;
  expectedStatus: number;
  timeout: number; // ms
  interval: number; // seconds
  healthyThreshold: number;
  unhealthyThreshold: number;
  lastCheck: Date;
  consecutiveFailures: number;
  responseTime: number;
}

export class GlobalLoadBalancer {
  private nodes: Map<string, LoadBalancerNode> = new Map();
  private routingRules: Map<string, RoutingRule> = new Map();
  private healthChecks: Map<string, HealthCheck> = new Map();
  private trafficHistory: TrafficMetrics[] = [];
  private eventEmitter: any;
  private healthCheckInterval?: NodeJS.Timeout;
  private metricsInterval?: NodeJS.Timeout;
  private deploymentManager?: GlobalDeploymentManager;

  constructor(_deploymentManager?: GlobalDeploymentManager) {
    // this.deploymentManager = deploymentManager;

    // Browser-compatible event emitter
    this.eventEmitter = {
      events: {} as { [event: string]: Function[] },
      on: function(event: string, listener: any): any {
        if (!this.events[event]) this.events[event] = [];
        this.events[event].push(listener);
        return this;
      },
      emit: function(event: string, ...args: any[]): any {
        if (this.events[event]) {
          this.events[event].forEach(listener => listener(...args));
        }
        return this;
      }
    };

    this.initializeNodes();
    this.setupRoutingRules();
    this.setupHealthChecks();
    this.startHealthMonitoring();
    this.startMetricsCollection();
  }

  // =============================================================================
  // INITIALIZATION
  // =============================================================================

  private initializeNodes(): void {
    Object.entries(GLOBAL_REGIONS).forEach(([_continent, regions]) => {
      Object.entries(regions).forEach(([regionId, config]: [string, any]) => {
        const node: LoadBalancerNode = {
          regionId,
          endpoint: this.generateEndpoint(regionId),
          weight: config.tier === 'primary' ? 100 : 70,
          currentLoad: Math.floor(Math.random() * 60) + 20, // 20-80%
          health: 90 + Math.floor(Math.random() * 10), // 90-100%
          latency: this.calculateNodeLatency(regionId),
          capacity: this.calculateNodeCapacity(config.tier),
          activeConnections: Math.floor(Math.random() * 1000),
          status: 'active',
          location: config.location
        };

        this.nodes.set(regionId, node);
      });
    });
  }

  private generateEndpoint(regionId: string): string {
    const subdomains: { [key: string]: string } = {
      'us-east-1': 'us-east',
      'us-west-2': 'us-west',
      'sa-east-1': 'sa-east',
      'eu-west-2': 'eu-west',
      'eu-central-1': 'eu-central',
      'eu-west-1': 'eu-west-1',
      'ap-south-1': 'ap-south',
      'ap-northeast-1': 'ap-northeast',
      'ap-southeast-2': 'ap-southeast',
      'ap-northeast-2': 'ap-northeast-2'
    };
    
    const subdomain = subdomains[regionId] || regionId;
    return `https://${subdomain}.sis-platform.edu`;
  }

  private calculateNodeLatency(regionId: string): number {
    // Base latencies from global perspective
    const latencies: { [key: string]: number } = {
      'us-east-1': 45,
      'us-west-2': 65,
      'sa-east-1': 180,
      'eu-west-2': 25,
      'eu-central-1': 30,
      'eu-west-1': 35,
      'ap-south-1': 15, // Mumbai (closest to current infrastructure)
      'ap-northeast-1': 40,
      'ap-southeast-2': 85,
      'ap-northeast-2': 50
    };
    return latencies[regionId] || 100;
  }

  private calculateNodeCapacity(tier: string): number {
    const capacities = {
      'primary': 5000 + Math.floor(Math.random() * 2000), // 5000-7000 rps
      'secondary': 3000 + Math.floor(Math.random() * 1000), // 3000-4000 rps
      'backup': 2000 + Math.floor(Math.random() * 500) // 2000-2500 rps
    };
    return capacities[tier as keyof typeof capacities] || 2000;
  }

  private setupRoutingRules(): void {
    const rules: RoutingRule[] = [
      {
        id: 'geo-americas',
        name: 'Americas Geographic Routing',
        priority: 10,
        conditions: [
          { type: 'geo', field: 'country', operator: 'equals', value: 'US,CA,MX,BR,AR' }
        ],
        target: 'us-east-1',
        sticky: false,
        enabled: true
      },
      {
        id: 'geo-europe',
        name: 'Europe Geographic Routing',
        priority: 10,
        conditions: [
          { type: 'geo', field: 'country', operator: 'equals', value: 'GB,DE,FR,IT,ES,NL' }
        ],
        target: 'eu-west-2',
        sticky: false,
        enabled: true
      },
      {
        id: 'geo-asia',
        name: 'Asia Pacific Geographic Routing',
        priority: 10,
        conditions: [
          { type: 'geo', field: 'country', operator: 'equals', value: 'IN,JP,AU,SG,KR' }
        ],
        target: 'ap-south-1',
        sticky: false,
        enabled: true
      },
      {
        id: 'api-high-priority',
        name: 'High Priority API Routes',
        priority: 5,
        conditions: [
          { type: 'path', field: 'path', operator: 'starts_with', value: '/api/v1/critical' }
        ],
        target: 'us-east-1', // Primary region
        sticky: true,
        enabled: true
      },
      {
        id: 'ai-processing',
        name: 'AI Processing Routes',
        priority: 7,
        conditions: [
          { type: 'path', field: 'path', operator: 'starts_with', value: '/api/ai' }
        ],
        target: 'ap-south-1', // AI optimized region
        sticky: true,
        enabled: true
      },
      {
        id: 'static-content',
        name: 'Static Content Routing',
        priority: 15,
        conditions: [
          { type: 'path', field: 'path', operator: 'matches', value: '\\.(js|css|png|jpg|gif|ico)$' }
        ],
        target: 'edge', // Will be handled by CDN
        sticky: false,
        enabled: true
      },
      {
        id: 'mobile-app',
        name: 'Mobile App Routing',
        priority: 8,
        conditions: [
          { type: 'header', field: 'User-Agent', operator: 'contains', value: 'Mobile|Android|iOS' }
        ],
        target: 'auto', // Best performance region
        sticky: true,
        enabled: true
      },
      {
        id: 'maintenance-fallback',
        name: 'Maintenance Mode Fallback',
        priority: 1,
        conditions: [
          { type: 'header', field: 'X-Maintenance', operator: 'equals', value: 'true' }
        ],
        target: 'ap-south-1', // Fallback to stable region
        sticky: false,
        enabled: false
      }
    ];

    rules.forEach(rule => {
      this.routingRules.set(rule.id, rule);
    });
  }

  private setupHealthChecks(): void {
    this.nodes.forEach((node, regionId) => {
      const healthCheck: HealthCheck = {
        regionId,
        endpoint: node.endpoint,
        method: 'GET',
        path: '/health',
        expectedStatus: 200,
        timeout: 5000, // 5 seconds
        interval: 30, // 30 seconds
        healthyThreshold: 2,
        unhealthyThreshold: 3,
        lastCheck: new Date(),
        consecutiveFailures: 0,
        responseTime: node.latency
      };

      this.healthChecks.set(regionId, healthCheck);
    });
  }

  // =============================================================================
  // ROUTING LOGIC
  // =============================================================================

  public async routeRequest(request: {
    path: string;
    headers: { [key: string]: string };
    userLocation?: { lat: number; lng: number };
    userIP?: string;
  }): Promise<{ regionId: string; endpoint: string; reason: string }> {
    
    // Apply routing rules in priority order
    const sortedRules = Array.from(this.routingRules.values())
      .filter(rule => rule.enabled)
      .sort((a, b) => a.priority - b.priority);

    for (const rule of sortedRules) {
      if (this.matchesRule(request, rule)) {
        const targetRegion = await this.resolveTarget(rule.target, request);
        const node = this.nodes.get(targetRegion);
        
        if (node && node.status === 'active') {
          return {
            regionId: targetRegion,
            endpoint: node.endpoint,
            reason: `Matched rule: ${rule.name}`
          };
        }
      }
    }

    // Fallback to best available region
    const bestRegion = await this.selectBestRegion(request);
    const node = this.nodes.get(bestRegion);
    
    return {
      regionId: bestRegion,
      endpoint: node?.endpoint || '',
      reason: 'Best available region'
    };
  }

  private matchesRule(request: any, rule: RoutingRule): boolean {
    return rule.conditions.every(condition => {
      switch (condition.type) {
        case 'path':
          return this.matchesPathCondition(request.path, condition);
        case 'header':
          return this.matchesHeaderCondition(request.headers, condition);
        case 'geo':
          return this.matchesGeoCondition(request.userLocation, condition);
        case 'ip_range':
          return this.matchesIPCondition(request.userIP, condition);
        default:
          return false;
      }
    });
  }

  private matchesPathCondition(path: string, condition: RoutingCondition): boolean {
    switch (condition.operator) {
      case 'equals':
        return path === condition.value;
      case 'starts_with':
        return path.startsWith(condition.value);
      case 'contains':
        return path.includes(condition.value);
      case 'matches':
        return new RegExp(condition.value).test(path);
      default:
        return false;
    }
  }

  private matchesHeaderCondition(headers: { [key: string]: string }, condition: RoutingCondition): boolean {
    const headerValue = headers[condition.field.toLowerCase()];
    if (!headerValue) return false;

    switch (condition.operator) {
      case 'equals':
        return headerValue === condition.value;
      case 'contains':
        return headerValue.includes(condition.value);
      default:
        return false;
    }
  }

  private matchesGeoCondition(userLocation: { lat: number; lng: number } | undefined, condition: RoutingCondition): boolean {
    if (!userLocation) return false;
    
    // Simplified geo matching based on coordinates
    // In production, this would use a proper geo-IP database
    const countries = condition.value.split(',');
    const userCountry = this.getCountryFromCoordinates(userLocation);
    
    return countries.includes(userCountry);
  }

  private getCountryFromCoordinates(location: { lat: number; lng: number }): string {
    // Simplified coordinate-to-country mapping
    if (location.lat >= 25 && location.lat <= 50 && location.lng >= -125 && location.lng <= -65) return 'US';
    if (location.lat >= 35 && location.lat <= 70 && location.lng >= -10 && location.lng <= 40) return 'DE';
    if (location.lat >= 6 && location.lat <= 37 && location.lng >= 68 && location.lng <= 97) return 'IN';
    if (location.lat >= 30 && location.lat <= 46 && location.lng >= 130 && location.lng <= 146) return 'JP';
    return 'US'; // Default
  }

  private matchesIPCondition(userIP: string | undefined, condition: RoutingCondition): boolean {
    if (!userIP) return false;
    // Simplified IP range matching
    return condition.value.includes(userIP.split('.')[0]); // Match first octet
  }

  private async resolveTarget(target: string, request: any): Promise<string> {
    switch (target) {
      case 'auto':
        return this.selectBestRegion(request);
      case 'edge':
        return this.selectNearestEdge(request.userLocation);
      default:
        return target; // Direct region ID
    }
  }

  private async selectBestRegion(_request: any): Promise<string> {
    const activeNodes = Array.from(this.nodes.values())
      .filter(node => node.status === 'active');

    if (activeNodes.length === 0) return 'ap-south-1'; // Fallback

    // Score nodes based on multiple factors
    const scoredNodes = activeNodes.map(node => {
      let score = 0;
      
      // Health score (40% weight)
      score += (node.health / 100) * 40;
      
      // Load score (30% weight) - lower load is better
      score += ((100 - node.currentLoad) / 100) * 30;
      
      // Latency score (20% weight) - lower latency is better
      const maxLatency = 200;
      score += ((maxLatency - Math.min(node.latency, maxLatency)) / maxLatency) * 20;
      
      // Weight score (10% weight)
      score += (node.weight / 100) * 10;

      return { node, score };
    });

    // Sort by best score
    scoredNodes.sort((a, b) => b.score - a.score);
    return scoredNodes[0].node.regionId;
  }

  private selectNearestEdge(userLocation?: { lat: number; lng: number }): string {
    if (!userLocation) return 'us-east-1'; // Default

    // Find nearest region based on distance
    let nearestRegion = 'us-east-1';
    let minDistance = Infinity;

    this.nodes.forEach((node, regionId) => {
      const distance = this.calculateDistance(userLocation, node.location);
      if (distance < minDistance) {
        minDistance = distance;
        nearestRegion = regionId;
      }
    });

    return nearestRegion;
  }

  private calculateDistance(point1: { lat: number; lng: number }, point2: { lat: number; lng: number }): number {
    const R = 6371; // Earth's radius in km
    const dLat = (point2.lat - point1.lat) * Math.PI / 180;
    const dLng = (point2.lng - point1.lng) * Math.PI / 180;
    const a = Math.sin(dLat/2) * Math.sin(dLat/2) +
              Math.cos(point1.lat * Math.PI / 180) * Math.cos(point2.lat * Math.PI / 180) *
              Math.sin(dLng/2) * Math.sin(dLng/2);
    const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1-a));
    return R * c;
  }

  // =============================================================================
  // HEALTH MONITORING
  // =============================================================================

  private startHealthMonitoring(): void {
    this.healthCheckInterval = setInterval(() => {
      this.performHealthChecks();
    }, 30000); // Every 30 seconds
  }

  private async performHealthChecks(): Promise<void> {
    const promises = Array.from(this.healthChecks.values()).map(async (check) => {
      await this.performSingleHealthCheck(check);
    });

    await Promise.all(promises);
    this.eventEmitter.emit('healthChecksCompleted', new Date());
  }

  private async performSingleHealthCheck(check: HealthCheck): Promise<void> {
    // const startTime = Date.now();
    
    try {
      // Simulate health check
      const isHealthy = Math.random() > 0.05; // 95% success rate
      const responseTime = check.responseTime + (Math.random() - 0.5) * 20;
      
      check.lastCheck = new Date();
      check.responseTime = Math.max(1, responseTime);

      const node = this.nodes.get(check.regionId);
      if (!node) return;

      if (isHealthy) {
        check.consecutiveFailures = 0;
        
        if (node.status !== 'active' && check.consecutiveFailures === 0) {
          // Mark as healthy if it was unhealthy
          node.status = 'active';
          node.health = Math.min(100, node.health + 10);
          
          this.eventEmitter.emit('nodeRecovered', {
            regionId: check.regionId,
            responseTime: check.responseTime
          });
        }
      } else {
        check.consecutiveFailures++;
        
        if (check.consecutiveFailures >= check.unhealthyThreshold) {
          node.status = 'failed';
          node.health = Math.max(0, node.health - 20);
          
          this.eventEmitter.emit('nodeFailure', {
            regionId: check.regionId,
            consecutiveFailures: check.consecutiveFailures
          });
        }
      }

      this.nodes.set(check.regionId, node);
      this.healthChecks.set(check.regionId, check);

    } catch (error) {
      check.consecutiveFailures++;
      this.eventEmitter.emit('healthCheckError', {
        regionId: check.regionId,
        error: error
      });
    }
  }

  // =============================================================================
  // TRAFFIC MANAGEMENT
  // =============================================================================

  public async adjustNodeWeight(regionId: string, weight: number): Promise<boolean> {
    const node = this.nodes.get(regionId);
    if (!node) return false;

    const oldWeight = node.weight;
    node.weight = Math.max(0, Math.min(100, weight));
    this.nodes.set(regionId, node);

    this.eventEmitter.emit('nodeWeightChanged', {
      regionId,
      oldWeight,
      newWeight: node.weight
    });

    return true;
  }

  public async drainNode(regionId: string): Promise<boolean> {
    const node = this.nodes.get(regionId);
    if (!node) return false;

    node.status = 'draining';
    node.weight = 0; // Stop sending new traffic
    this.nodes.set(regionId, node);

    this.eventEmitter.emit('nodeDraining', { regionId });

    // Simulate draining process
    setTimeout(() => {
      node.status = 'maintenance';
      node.activeConnections = 0;
      this.nodes.set(regionId, node);
      
      this.eventEmitter.emit('nodeDrained', { regionId });
    }, 60000); // 1 minute drain time

    return true;
  }

  public async restoreNode(regionId: string): Promise<boolean> {
    const node = this.nodes.get(regionId);
    if (!node) return false;

    node.status = 'active';
    node.weight = 100; // Restore full weight
    node.health = 100;
    this.nodes.set(regionId, node);

    this.eventEmitter.emit('nodeRestored', { regionId });
    return true;
  }

  // =============================================================================
  // METRICS AND ANALYTICS
  // =============================================================================

  private startMetricsCollection(): void {
    this.metricsInterval = setInterval(() => {
      this.updateTrafficMetrics();
      this.updateNodeMetrics();
    }, 60000); // Every minute
  }

  private updateTrafficMetrics(): void {
    const metrics: TrafficMetrics = {
      totalRequests: this.calculateTotalRequests(),
      requestsPerSecond: this.calculateRequestsPerSecond(),
      averageLatency: this.calculateAverageLatency(),
      errorRate: Math.random() * 2, // 0-2% error rate
      bandwidthUsage: this.calculateBandwidthUsage(),
      regionDistribution: this.calculateRegionDistribution(),
      topPaths: this.getTopPaths()
    };

    this.trafficHistory.push(metrics);
    
    // Keep only last 100 entries
    if (this.trafficHistory.length > 100) {
      this.trafficHistory.shift();
    }

    this.eventEmitter.emit('trafficMetricsUpdated', metrics);
  }

  private calculateTotalRequests(): number {
    return Array.from(this.nodes.values())
      .reduce((sum, node) => sum + node.activeConnections * 60, 0); // Rough estimate
  }

  private calculateRequestsPerSecond(): number {
    return Array.from(this.nodes.values())
      .reduce((sum, node) => sum + (node.activeConnections / 60), 0);
  }

  private calculateAverageLatency(): number {
    const activeNodes = Array.from(this.nodes.values()).filter(n => n.status === 'active');
    if (activeNodes.length === 0) return 0;
    
    return activeNodes.reduce((sum, node) => sum + node.latency, 0) / activeNodes.length;
  }

  private calculateBandwidthUsage(): number {
    return Array.from(this.nodes.values())
      .reduce((sum, node) => sum + (node.activeConnections * 0.5), 0); // 0.5 Mbps per connection
  }

  private calculateRegionDistribution(): { [regionId: string]: number } {
    const distribution: { [regionId: string]: number } = {};
    let total = 0;

    this.nodes.forEach((node, regionId) => {
      distribution[regionId] = node.activeConnections;
      total += node.activeConnections;
    });

    // Convert to percentages
    Object.keys(distribution).forEach(regionId => {
      distribution[regionId] = total > 0 ? (distribution[regionId] / total) * 100 : 0;
    });

    return distribution;
  }

  private getTopPaths(): { path: string; requests: number }[] {
    // Mock data for top paths
    return [
      { path: '/dashboard', requests: 12500 },
      { path: '/api/circuits', requests: 8900 },
      { path: '/hardware', requests: 6700 },
      { path: '/api/ai/generate', requests: 5400 },
      { path: '/settings', requests: 3200 }
    ];
  }

  private updateNodeMetrics(): void {
    this.nodes.forEach((node, regionId) => {
      // Simulate load changes
      node.currentLoad += (Math.random() - 0.5) * 10;
      node.currentLoad = Math.max(10, Math.min(95, node.currentLoad));
      
      // Simulate connection changes
      node.activeConnections += Math.floor((Math.random() - 0.5) * 100);
      node.activeConnections = Math.max(0, node.activeConnections);
      
      // Update latency based on load
      const loadFactor = node.currentLoad / 100;
      node.latency = node.latency * (1 + loadFactor * 0.2);
      
      this.nodes.set(regionId, node);
    });
  }

  // =============================================================================
  // PUBLIC API
  // =============================================================================

  public getGlobalStatus(): any {
    const nodes = Array.from(this.nodes.values());
    const activeNodes = nodes.filter(n => n.status === 'active');
    
    return {
      totalNodes: nodes.length,
      activeNodes: activeNodes.length,
      totalCapacity: nodes.reduce((sum, n) => sum + n.capacity, 0),
      totalLoad: nodes.reduce((sum, n) => sum + n.activeConnections, 0),
      averageHealth: activeNodes.reduce((sum, n) => sum + n.health, 0) / activeNodes.length,
      averageLatency: activeNodes.reduce((sum, n) => sum + n.latency, 0) / activeNodes.length,
      nodes: Array.from(this.nodes.values())
    };
  }

  public getNodeStatus(regionId: string): LoadBalancerNode | undefined {
    return this.nodes.get(regionId);
  }

  public getRoutingRules(): RoutingRule[] {
    return Array.from(this.routingRules.values());
  }

  public getTrafficMetrics(): TrafficMetrics[] {
    return this.trafficHistory;
  }

  public getCurrentTrafficMetrics(): TrafficMetrics | undefined {
    return this.trafficHistory[this.trafficHistory.length - 1];
  }

  public getHealthCheckStatus(): HealthCheck[] {
    return Array.from(this.healthChecks.values());
  }

  // Event subscription methods
  public onNodeEvent(callback: Function): void {
    this.eventEmitter.on('nodeFailure', callback);
    this.eventEmitter.on('nodeRecovered', callback);
    this.eventEmitter.on('nodeDraining', callback);
    this.eventEmitter.on('nodeRestored', callback);
  }

  public onTrafficEvent(callback: Function): void {
    this.eventEmitter.on('trafficMetricsUpdated', callback);
    this.eventEmitter.on('nodeWeightChanged', callback);
  }

  public onHealthEvent(callback: Function): void {
    this.eventEmitter.on('healthChecksCompleted', callback);
    this.eventEmitter.on('healthCheckError', callback);
  }

  // Cleanup
  public destroy(): void {
    if (this.healthCheckInterval) {
      clearInterval(this.healthCheckInterval);
    }
    if (this.metricsInterval) {
      clearInterval(this.metricsInterval);
    }
  }
}

export default GlobalLoadBalancer;