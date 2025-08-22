// Hybrid CDN Manager - Grok's recommendation for 95% cache hit rate
// Combines Cloudflare + CloudFront for optimal global performance

interface CDNProvider {
  name: string;
  regions: string[];
  capabilities: string[];
  costPerGB: number;
  latency: number;
}

interface CacheRule {
  pattern: string;
  ttl: number;
  provider: 'cloudflare' | 'cloudfront' | 'both';
  compression: boolean;
  edgeCompute: boolean;
}

interface PerformanceMetrics {
  hitRate: number;
  avgLatency: number;
  bandwidthSaved: number;
  costReduction: number;
  uptime: number;
}

interface EdgeLocation {
  id: string;
  provider: 'cloudflare' | 'cloudfront';
  region: string;
  country: string;
  pop: string;
  capacity: number;
  utilization: number;
  latency: number;
}

interface CacheStats {
  hits: number;
  misses: number;
  invalidations: number;
  bandwidth: number;
  requests: number;
  errors: number;
}

// Content categorization for optimal CDN routing
const CONTENT_CATEGORIES = {
  static: {
    patterns: ['*.js', '*.css', '*.png', '*.jpg', '*.svg', '*.woff2'],
    ttl: 31536000, // 1 year
    compression: true,
    edgeCompute: false,
    preferredProvider: 'cloudflare' as const
  },
  dynamic: {
    patterns: ['/api/*', '/auth/*', '/ws/*'],
    ttl: 0, // No cache
    compression: false,
    edgeCompute: true,
    preferredProvider: 'cloudfront' as const
  },
  semi_static: {
    patterns: ['/docs/*', '/help/*', '/examples/*'],
    ttl: 3600, // 1 hour
    compression: true,
    edgeCompute: false,
    preferredProvider: 'both' as const
  },
  educational: {
    patterns: ['/courses/*', '/labs/*', '/tutorials/*'],
    ttl: 1800, // 30 minutes
    compression: true,
    edgeCompute: true,
    preferredProvider: 'both' as const
  },
  media: {
    patterns: ['*.mp4', '*.webm', '*.mp3', '*.wav'],
    ttl: 86400, // 24 hours
    compression: false,
    edgeCompute: false,
    preferredProvider: 'cloudfront' as const
  }
};

export class HybridCDNManager {
  private providers: Map<string, CDNProvider> = new Map();
  private cacheRules: CacheRule[] = [];
  private edgeLocations: Map<string, EdgeLocation> = new Map();
  private performanceMetrics: Map<string, PerformanceMetrics> = new Map();
  private cacheStats: Map<string, CacheStats> = new Map();
  private failoverChain: string[] = [];
  getFailoverChain() { return this.failoverChain; }

  constructor() {
    this.initializeProviders();
    this.setupCacheRules();
    this.initializeEdgeLocations();
    this.setupFailoverChain();
    this.startPerformanceMonitoring();
  }

  private initializeProviders(): void {
    // Cloudflare configuration
    this.providers.set('cloudflare', {
      name: 'Cloudflare',
      regions: [
        'us-east', 'us-west', 'eu-west', 'eu-central', 'ap-south', 
        'ap-southeast', 'ap-northeast', 'sa-east', 'af-south', 'me-south'
      ],
      capabilities: [
        'edge-compute', 'workers', 'ddos-protection', 'waf', 
        'load-balancing', 'dns', 'ssl-termination'
      ],
      costPerGB: 0.085,
      latency: 15 // Average global latency in ms
    });

    // AWS CloudFront configuration
    this.providers.set('cloudfront', {
      name: 'AWS CloudFront',
      regions: [
        'us-east-1', 'us-west-2', 'eu-west-1', 'eu-central-1', 
        'ap-south-1', 'ap-southeast-1', 'ap-northeast-1', 'sa-east-1'
      ],
      capabilities: [
        'lambda-edge', 'origin-shield', 'real-time-logs', 
        'field-level-encryption', 'signed-urls', 'geo-restriction'
      ],
      costPerGB: 0.12,
      latency: 25 // Average global latency in ms
    });
  }

  private setupCacheRules(): void {
    // Convert content categories to cache rules
    for (const [, config] of Object.entries(CONTENT_CATEGORIES)) {
      for (const pattern of config.patterns) {
        this.cacheRules.push({
          pattern,
          ttl: config.ttl,
          provider: config.preferredProvider,
          compression: config.compression,
          edgeCompute: config.edgeCompute
        });
      }
    }

    // Special rules for educational content
    this.cacheRules.push(
      {
        pattern: '/ai-lab/*',
        ttl: 300, // 5 minutes for real-time collaboration
        provider: 'both',
        compression: true,
        edgeCompute: true
      },
      {
        pattern: '/design-validator/*',
        ttl: 60, // 1 minute for validation results
        provider: 'cloudflare', // Faster edge compute
        compression: true,
        edgeCompute: true
      },
      {
        pattern: '/collaboration/*',
        ttl: 0, // No cache for real-time features
        provider: 'both',
        compression: false,
        edgeCompute: true
      }
    );
  }

  private initializeEdgeLocations(): void {
    // Cloudflare edge locations (major ones)
    const cloudflareLocations = [
      { region: 'us-east', country: 'US', pop: 'NYC', capacity: 100, latency: 10 },
      { region: 'us-west', country: 'US', pop: 'LAX', capacity: 100, latency: 12 },
      { region: 'eu-west', country: 'UK', pop: 'LHR', capacity: 80, latency: 15 },
      { region: 'eu-central', country: 'DE', pop: 'FRA', capacity: 80, latency: 14 },
      { region: 'ap-south', country: 'IN', pop: 'BOM', capacity: 60, latency: 20 },
      { region: 'ap-southeast', country: 'SG', pop: 'SIN', capacity: 60, latency: 18 },
      { region: 'ap-northeast', country: 'JP', pop: 'NRT', capacity: 60, latency: 16 },
      { region: 'sa-east', country: 'BR', pop: 'GRU', capacity: 40, latency: 25 }
    ];

    // CloudFront edge locations
    const cloudfrontLocations = [
      { region: 'us-east-1', country: 'US', pop: 'IAD', capacity: 120, latency: 12 },
      { region: 'us-west-2', country: 'US', pop: 'SEA', capacity: 120, latency: 14 },
      { region: 'eu-west-1', country: 'IE', pop: 'DUB', capacity: 100, latency: 16 },
      { region: 'eu-central-1', country: 'DE', pop: 'FRA', capacity: 100, latency: 15 },
      { region: 'ap-south-1', country: 'IN', pop: 'BOM', capacity: 80, latency: 22 },
      { region: 'ap-southeast-1', country: 'SG', pop: 'SIN', capacity: 80, latency: 19 },
      { region: 'ap-northeast-1', country: 'JP', pop: 'NRT', capacity: 80, latency: 17 },
      { region: 'sa-east-1', country: 'BR', pop: 'GRU', capacity: 60, latency: 28 }
    ];

    // Initialize Cloudflare locations
    cloudflareLocations.forEach((loc, index) => {
      this.edgeLocations.set(`cf-${index}`, {
        id: `cf-${index}`,
        provider: 'cloudflare',
        region: loc.region,
        country: loc.country,
        pop: loc.pop,
        capacity: loc.capacity,
        utilization: Math.random() * 0.7, // 0-70% utilization
        latency: loc.latency
      });
    });

    // Initialize CloudFront locations
    cloudfrontLocations.forEach((loc, index) => {
      this.edgeLocations.set(`aws-${index}`, {
        id: `aws-${index}`,
        provider: 'cloudfront',
        region: loc.region,
        country: loc.country,
        pop: loc.pop,
        capacity: loc.capacity,
        utilization: Math.random() * 0.6, // 0-60% utilization
        latency: loc.latency
      });
    });
  }

  private setupFailoverChain(): void {
    // Intelligent failover based on performance and cost
    this.failoverChain = [
      'cloudflare', // Primary - faster, cheaper
      'cloudfront', // Secondary - more features, AWS ecosystem
      'origin' // Final fallback
    ];
  }

  private startPerformanceMonitoring(): void {
    // Initialize performance metrics for each provider
    this.providers.forEach((provider, name) => {
      this.performanceMetrics.set(name, {
        hitRate: 0.85, // Starting estimate
        avgLatency: provider.latency,
        bandwidthSaved: 0,
        costReduction: 0,
        uptime: 0.999
      });

      this.cacheStats.set(name, {
        hits: 0,
        misses: 0,
        invalidations: 0,
        bandwidth: 0,
        requests: 0,
        errors: 0
      });
    });

    // Start real-time monitoring
    setInterval(() => {
      this.updatePerformanceMetrics();
    }, 30000); // Update every 30 seconds
  }

  async routeRequest(url: string, userLocation: { lat: number; lon: number }): Promise<{
    provider: string;
    edgeLocation: EdgeLocation;
    cacheRule: CacheRule;
    estimatedLatency: number;
  }> {
    // Find matching cache rule
    const cacheRule = this.findMatchingCacheRule(url);
    
    // Find optimal edge location based on user location and provider preference
    const candidates = this.findOptimalEdgeLocations(userLocation, cacheRule.provider);
    
    // Select best candidate based on latency, utilization, and performance
    const selectedEdge = this.selectBestEdgeLocation(candidates);
    
    // Calculate estimated latency
    const estimatedLatency = this.calculateEstimatedLatency(selectedEdge, userLocation);

    return {
      provider: selectedEdge.provider,
      edgeLocation: selectedEdge,
      cacheRule,
      estimatedLatency
    };
  }

  private findMatchingCacheRule(url: string): CacheRule {
    // Find the most specific matching rule
    for (const rule of this.cacheRules) {
      if (this.matchesPattern(url, rule.pattern)) {
        return rule;
      }
    }

    // Default rule for unmatched content
    return {
      pattern: '*',
      ttl: 300,
      provider: 'both',
      compression: true,
      edgeCompute: false
    };
  }

  private matchesPattern(url: string, pattern: string): boolean {
    // Convert glob pattern to regex
    const regexPattern = pattern
      .replace(/\*/g, '.*')
      .replace(/\?/g, '.')
      .replace(/\./g, '\\.');
    
    const regex = new RegExp(`^${regexPattern}$`);
    return regex.test(url);
  }

  private findOptimalEdgeLocations(
    userLocation: { lat: number; lon: number }, 
    preferredProvider: 'cloudflare' | 'cloudfront' | 'both'
  ): EdgeLocation[] {
    const candidates: EdgeLocation[] = [];

    for (const location of this.edgeLocations.values()) {
      // Filter by provider preference
      if (preferredProvider !== 'both' && location.provider !== preferredProvider) {
        continue;
      }

      // Calculate geographic distance (simplified)
      const distance = this.calculateDistance(userLocation, location);
      
      // Add to candidates if reasonable distance and not overloaded
      if (distance < 5000 && location.utilization < 0.9) { // 5000km max, <90% utilization
        candidates.push(location);
      }
    }

    // Sort by performance score
    return candidates.sort((a, b) => {
      const scoreA = this.calculateLocationScore(a, userLocation);
      const scoreB = this.calculateLocationScore(b, userLocation);
      return scoreB - scoreA; // Higher score is better
    });
  }

  private calculateDistance(
    userLocation: { lat: number; lon: number }, 
    edgeLocation: EdgeLocation
  ): number {
    // Simplified distance calculation (using major city coordinates)
    const cityCoords: Record<string, { lat: number; lon: number }> = {
      'NYC': { lat: 40.7128, lon: -74.0060 },
      'LAX': { lat: 34.0522, lon: -118.2437 },
      'LHR': { lat: 51.4700, lon: -0.4543 },
      'FRA': { lat: 50.0379, lon: 8.5622 },
      'BOM': { lat: 19.0760, lon: 72.8777 },
      'SIN': { lat: 1.3521, lon: 103.8198 },
      'NRT': { lat: 35.7720, lon: 140.3929 },
      'GRU': { lat: -23.5505, lon: -46.6333 },
      'IAD': { lat: 38.9445, lon: -77.4558 },
      'SEA': { lat: 47.4502, lon: -122.3088 },
      'DUB': { lat: 53.4213, lon: -6.2707 }
    };

    const edgeCoords = cityCoords[edgeLocation.pop];
    if (!edgeCoords) return 10000; // Unknown location, assign high distance

    // Haversine formula for great circle distance
    const R = 6371; // Earth's radius in km
    const dLat = this.toRad(edgeCoords.lat - userLocation.lat);
    const dLon = this.toRad(edgeCoords.lon - userLocation.lon);
    
    const a = Math.sin(dLat / 2) * Math.sin(dLat / 2) +
              Math.cos(this.toRad(userLocation.lat)) * Math.cos(this.toRad(edgeCoords.lat)) *
              Math.sin(dLon / 2) * Math.sin(dLon / 2);
    
    const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
    return R * c;
  }

  private toRad(degrees: number): number {
    return degrees * (Math.PI / 180);
  }

  private calculateLocationScore(
    location: EdgeLocation, 
    userLocation: { lat: number; lon: number }
  ): number {
    const distance = this.calculateDistance(userLocation, location);
    const metrics = this.performanceMetrics.get(location.provider);
    
    // Scoring factors (higher is better)
    const distanceScore = Math.max(0, 1000 - distance) / 1000; // 0-1 based on distance
    const utilizationScore = Math.max(0, 1 - location.utilization); // 0-1 based on available capacity
    const latencyScore = Math.max(0, 100 - location.latency) / 100; // 0-1 based on latency
    const uptimeScore = metrics?.uptime || 0.99; // Provider uptime
    const hitRateScore = metrics?.hitRate || 0.85; // Cache hit rate

    // Weighted composite score
    return (
      distanceScore * 0.3 +
      utilizationScore * 0.2 +
      latencyScore * 0.2 +
      uptimeScore * 0.15 +
      hitRateScore * 0.15
    );
  }

  private selectBestEdgeLocation(candidates: EdgeLocation[]): EdgeLocation {
    if (candidates.length === 0) {
      // Fallback to any available location
      return Array.from(this.edgeLocations.values())[0];
    }

    // Return the highest scoring candidate
    return candidates[0];
  }

  private calculateEstimatedLatency(
    edgeLocation: EdgeLocation, 
    userLocation: { lat: number; lon: number }
  ): number {
    const distance = this.calculateDistance(userLocation, edgeLocation);
    const baseLatency = edgeLocation.latency;
    const distanceLatency = Math.min(distance / 100, 50); // ~1ms per 100km, max 50ms
    const utilizationPenalty = edgeLocation.utilization * 10; // 0-10ms based on load

    return Math.round(baseLatency + distanceLatency + utilizationPenalty);
  }

  async invalidateCache(pattern: string, provider?: string): Promise<void> {
    const providers = provider ? [provider] : ['cloudflare', 'cloudfront'];
    
    const invalidationPromises = providers.map(async (prov) => {
      try {
        // Simulate invalidation API call
        console.log(`Invalidating cache pattern "${pattern}" on ${prov}`);
        
        // Update stats
        const stats = this.cacheStats.get(prov);
        if (stats) {
          stats.invalidations++;
          this.cacheStats.set(prov, stats);
        }
        
        // Simulate API call delay
        await new Promise(resolve => setTimeout(resolve, 100));
        
      } catch (error) {
        console.error(`Cache invalidation failed for ${prov}:`, error);
        throw error;
      }
    });

    await Promise.all(invalidationPromises);
  }

  async preloadContent(urls: string[], regions?: string[]): Promise<void> {
    const targetRegions = regions || ['us-east', 'eu-west', 'ap-south'];
    
    for (const url of urls) {
      const cacheRule = this.findMatchingCacheRule(url);
      
      // Preload on both providers if rule allows
      const providers = cacheRule.provider === 'both' 
        ? ['cloudflare', 'cloudfront'] 
        : [cacheRule.provider];

      for (const provider of providers) {
        for (const region of targetRegions) {
          // Simulate preloading
          console.log(`Preloading ${url} to ${provider} in ${region}`);
          
          // Update cache stats
          const stats = this.cacheStats.get(provider);
          if (stats) {
            stats.requests++;
            this.cacheStats.set(provider, stats);
          }
        }
      }
    }
  }

  private updatePerformanceMetrics(): void {
    // Simulate real-time metrics updates
    for (const [provider, metrics] of this.performanceMetrics) {
      const stats = this.cacheStats.get(provider);
      if (!stats) continue;

      // Calculate hit rate
      const totalRequests = stats.hits + stats.misses;
      if (totalRequests > 0) {
        metrics.hitRate = stats.hits / totalRequests;
      }

      // Update average latency with some randomness
      metrics.avgLatency += (Math.random() - 0.5) * 2; // ±1ms variation
      metrics.avgLatency = Math.max(10, Math.min(50, metrics.avgLatency)); // 10-50ms range

      // Calculate bandwidth saved (estimate)
      metrics.bandwidthSaved = stats.hits * 0.5; // Assume 500KB average per hit

      // Calculate cost reduction (estimate)
      const provider_info = this.providers.get(provider);
      if (provider_info) {
        metrics.costReduction = metrics.bandwidthSaved * provider_info.costPerGB * 0.8; // 80% cost reduction
      }

      // Simulate uptime variations
      metrics.uptime = Math.max(0.95, Math.min(0.9999, metrics.uptime + (Math.random() - 0.5) * 0.001));

      this.performanceMetrics.set(provider, metrics);
    }

    // Update edge location utilization
    for (const [id, location] of this.edgeLocations) {
      location.utilization += (Math.random() - 0.5) * 0.1; // ±5% variation
      location.utilization = Math.max(0.1, Math.min(0.9, location.utilization)); // 10-90% range
      this.edgeLocations.set(id, location);
    }
  }

  getPerformanceReport(): {
    overall: {
      totalRequests: number;
      avgHitRate: number;
      avgLatency: number;
      totalBandwidthSaved: number;
      totalCostReduction: number;
    };
    byProvider: Record<string, PerformanceMetrics & CacheStats>;
    edgeLocations: EdgeLocation[];
  } {
    let totalRequests = 0;
    let totalHits = 0;
    let totalLatency = 0;
    let totalBandwidthSaved = 0;
    let totalCostReduction = 0;
    let providerCount = 0;

    const byProvider: Record<string, PerformanceMetrics & CacheStats> = {};

    // Aggregate metrics
    for (const [provider, metrics] of this.performanceMetrics) {
      const stats = this.cacheStats.get(provider);
      if (!stats) continue;

      totalRequests += stats.requests;
      totalHits += stats.hits;
      totalLatency += metrics.avgLatency;
      totalBandwidthSaved += metrics.bandwidthSaved;
      totalCostReduction += metrics.costReduction;
      providerCount++;

      byProvider[provider] = { ...metrics, ...stats };
    }

    return {
      overall: {
        totalRequests,
        avgHitRate: totalRequests > 0 ? totalHits / totalRequests : 0,
        avgLatency: providerCount > 0 ? totalLatency / providerCount : 0,
        totalBandwidthSaved,
        totalCostReduction
      },
      byProvider,
      edgeLocations: Array.from(this.edgeLocations.values())
    };
  }

  // Grok's intelligent cost optimization
  async optimizeCosts(): Promise<{
    recommendations: string[];
    potentialSavings: number;
  }> {
    const report = this.getPerformanceReport();
    const recommendations: string[] = [];
    let potentialSavings = 0;

    // Analyze cache hit rates
    for (const [provider, data] of Object.entries(report.byProvider)) {
      if (data.hitRate < 0.8) {
        recommendations.push(`Improve cache strategy for ${provider} (current hit rate: ${(data.hitRate * 100).toFixed(1)}%)`);
        potentialSavings += data.bandwidth * 0.2; // 20% potential savings
      }
    }

    // Analyze edge location efficiency
    const underutilizedLocations = report.edgeLocations.filter(loc => loc.utilization < 0.3);
    if (underutilizedLocations.length > 0) {
      recommendations.push(`Consider reducing capacity in ${underutilizedLocations.length} underutilized edge locations`);
      potentialSavings += underutilizedLocations.length * 100; // $100 per location potential savings
    }

    // Analyze provider cost efficiency
    const cloudflareMetrics = report.byProvider['cloudflare'];
    const cloudfrontMetrics = report.byProvider['cloudfront'];
    
    if (cloudflareMetrics && cloudfrontMetrics) {
      const cloudflareCost = cloudflareMetrics.bandwidth * 0.085;
      const cloudfrontCost = cloudfrontMetrics.bandwidth * 0.12;
      
      if (cloudfrontCost > cloudflareCost * 1.2) {
        recommendations.push('Consider routing more static content through Cloudflare for cost savings');
        potentialSavings += (cloudfrontCost - cloudflareCost) * 0.3; // 30% of the difference
      }
    }

    return {
      recommendations,
      potentialSavings
    };
  }

  async cleanup(): Promise<void> {
    // Clear all caches and stop monitoring
    this.providers.clear();
    this.cacheRules.length = 0;
    this.edgeLocations.clear();
    this.performanceMetrics.clear();
    this.cacheStats.clear();
    
    console.log('Hybrid CDN Manager cleaned up');
  }
}

// Export singleton instance
export const hybridCDN = new HybridCDNManager();

// Export types
export type {
  CDNProvider,
  CacheRule,
  PerformanceMetrics,
  EdgeLocation,
  CacheStats
};