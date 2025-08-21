// Auto-Scaling Configuration for Phase 5C Integration
// Integrates with Database, Redis, and WebSocket infrastructure

import type { ResourceRecommendation } from '../services/predictive-autoscaling';

// Integration with existing Phase 5C infrastructure
export const PHASE_5C_INTEGRATION = {
  // Database scaling integration
  DATABASE: {
    PRIMARY_CLUSTER: {
      name: 'mumbai-primary',
      region: 'ap-south-1',
      zones: ['ap-south-1a', 'ap-south-1b', 'ap-south-1c'],
      maxInstances: 10,
      minInstances: 2
    },
    READ_REPLICAS: {
      regions: [
        { name: 'ap-south-1', maxInstances: 6 }, // Mumbai
        { name: 'ap-southeast-1', maxInstances: 4 }, // Singapore (backup)
        { name: 'us-east-1', maxInstances: 2 } // US (global sync)
      ]
    },
    SCALING_POLICIES: {
      CONNECTIONS_THRESHOLD: 1000, // per instance
      CPU_THRESHOLD: 75,
      MEMORY_THRESHOLD: 80,
      REPLICATION_LAG_THRESHOLD: 100 // ms
    }
  },

  // Redis caching integration
  REDIS: {
    L1_CACHE: { // Edge cache
      instances: { min: 3, max: 12 },
      memory: '2GB',
      regions: ['mumbai', 'delhi', 'bangalore', 'chennai']
    },
    L2_CACHE: { // Regional cache
      instances: { min: 3, max: 8 },
      memory: '8GB',
      regions: ['mumbai', 'singapore']
    },
    L3_CACHE: { // Global cache
      instances: { min: 2, max: 4 },
      memory: '16GB',
      regions: ['mumbai', 'us-east']
    },
    SCALING_TRIGGERS: {
      HIT_RATIO_THRESHOLD: 0.85,
      MEMORY_USAGE_THRESHOLD: 0.75,
      LATENCY_THRESHOLD: 5 // ms
    }
  },

  // WebSocket infrastructure integration
  WEBSOCKET: {
    GATEWAYS: {
      MUMBAI_PRIMARY: {
        instances: { min: 2, max: 10 },
        capacity: 2000, // connections per instance
        regions: ['ap-south-1a', 'ap-south-1b']
      },
      REGIONAL_GATEWAYS: [
        { region: 'delhi', instances: { min: 1, max: 5 }, capacity: 1500 },
        { region: 'bangalore', instances: { min: 1, max: 5 }, capacity: 1500 },
        { region: 'chennai', instances: { min: 1, max: 3 }, capacity: 1000 }
      ]
    },
    KAFKA_BROKERS: {
      instances: { min: 3, max: 12 },
      partitions: 24,
      replication: 3,
      regions: ['mumbai', 'delhi', 'bangalore']
    },
    SCALING_METRICS: {
      CONNECTION_THRESHOLD: 0.8, // 80% capacity
      MESSAGE_RATE_THRESHOLD: 1000, // per second
      LATENCY_THRESHOLD: 50 // ms
    }
  },

  // Load balancer configuration
  LOAD_BALANCERS: {
    APPLICATION: {
      algorithm: 'least_connections',
      health_check: {
        path: '/health',
        interval: 30,
        timeout: 5,
        unhealthy_threshold: 3
      }
    },
    DATABASE: {
      read_preference: 'secondary_preferred',
      max_connections: 100
    }
  }
};

// Auto-scaling orchestration configuration
export const SCALING_ORCHESTRATION = {
  // Scaling sequence to prevent cascading failures
  SCALE_UP_SEQUENCE: [
    { component: 'load_balancer', delay: 0 },
    { component: 'redis_cache', delay: 10 },
    { component: 'websocket_gateways', delay: 15 },
    { component: 'web_servers', delay: 20 },
    { component: 'kafka_brokers', delay: 25 },
    { component: 'database_replicas', delay: 30 }
  ],
  
  SCALE_DOWN_SEQUENCE: [
    { component: 'database_replicas', delay: 0 },
    { component: 'kafka_brokers', delay: 5 },
    { component: 'web_servers', delay: 10 },
    { component: 'websocket_gateways', delay: 15 },
    { component: 'redis_cache', delay: 20 },
    { component: 'load_balancer', delay: 25 }
  ],

  // Health check configuration
  HEALTH_CHECKS: {
    DATABASE: {
      query: 'SELECT 1',
      timeout: 5000,
      retry_count: 3
    },
    REDIS: {
      command: 'PING',
      timeout: 1000,
      retry_count: 2
    },
    WEBSOCKET: {
      path: '/ws/health',
      timeout: 3000,
      retry_count: 2
    },
    WEB_SERVER: {
      path: '/api/health',
      timeout: 5000,
      retry_count: 3
    }
  },

  // Rollback configuration
  ROLLBACK: {
    enabled: true,
    conditions: {
      error_rate_threshold: 0.05, // 5%
      response_time_threshold: 1000, // ms
      health_check_failure_threshold: 0.2 // 20%
    },
    timeout: 300000 // 5 minutes
  }
};

// Indian-specific scaling patterns
export const INDIAN_SCALING_PATTERNS = {
  // Peak hour scaling multipliers
  PEAK_MULTIPLIERS: {
    MORNING: { // 9 AM - 12 PM IST
      web_servers: 2.5,
      database_replicas: 2.0,
      redis_cache: 2.2,
      websocket_gateways: 3.0, // Higher for collaboration
      kafka_brokers: 2.0
    },
    AFTERNOON: { // 2 PM - 5 PM IST
      web_servers: 2.0,
      database_replicas: 1.8,
      redis_cache: 2.0,
      websocket_gateways: 2.5,
      kafka_brokers: 1.8
    },
    EVENING: { // 7 PM - 11 PM IST
      web_servers: 3.0,
      database_replicas: 2.5,
      redis_cache: 2.8,
      websocket_gateways: 3.5, // Peak collaboration time
      kafka_brokers: 2.5
    }
  },

  // Regional scaling preferences
  REGIONAL_PRIORITY: {
    MUMBAI: { weight: 0.35, max_capacity: 0.4 },
    DELHI: { weight: 0.25, max_capacity: 0.3 },
    BANGALORE: { weight: 0.25, max_capacity: 0.3 },
    CHENNAI: { weight: 0.15, max_capacity: 0.2 }
  },

  // Educational calendar integration
  EDUCATIONAL_CALENDAR: {
    EXAM_SEASONS: [
      { months: [2, 3], multiplier: 2.5, name: 'Board Exams' },
      { months: [10, 11], multiplier: 2.2, name: 'Mid-term Exams' },
      { months: [4, 5], multiplier: 2.0, name: 'Final Exams' }
    ],
    ADMISSION_SEASONS: [
      { months: [6, 7], multiplier: 2.8, name: 'College Admissions' },
      { months: [12, 1], multiplier: 1.8, name: 'School Admissions' }
    ],
    VACATION_PERIODS: [
      { months: [5, 6], multiplier: 0.7, name: 'Summer Break' },
      { months: [12], multiplier: 0.8, name: 'Winter Break' }
    ]
  },

  // Festival impact scaling
  FESTIVAL_SCALING: {
    MAJOR_FESTIVALS: [
      { name: 'Diwali', duration: 5, multiplier: 1.5, regions: ['all'] },
      { name: 'Holi', duration: 2, multiplier: 1.3, regions: ['north', 'west'] },
      { name: 'Durga Puja', duration: 10, multiplier: 1.8, regions: ['east'] },
      { name: 'Onam', duration: 10, multiplier: 1.6, regions: ['south'] },
      { name: 'Ganesh Chaturthi', duration: 11, multiplier: 1.7, regions: ['west'] }
    ],
    PRE_FESTIVAL_SCALING: {
      enabled: true,
      advance_days: 2,
      multiplier: 1.2
    }
  }
};

// Cost optimization configuration
export const COST_OPTIMIZATION = {
  // Spot instance configuration for non-critical workloads
  SPOT_INSTANCES: {
    enabled: true,
    max_percentage: 0.7, // 70% of instances can be spot
    fallback_timeout: 120, // seconds
    preferred_types: ['t3.medium', 't3.large', 'c5.large']
  },

  // Reserved instance planning
  RESERVED_INSTANCES: {
    baseline_percentage: 0.3, // 30% reserved for baseline
    commitment_months: 12,
    regions: ['ap-south-1', 'ap-southeast-1']
  },

  // Cost thresholds and alerts
  COST_CONTROLS: {
    hourly_budget: 500, // USD
    daily_budget: 10000, // USD
    monthly_budget: 250000, // USD
    alert_thresholds: [0.7, 0.8, 0.9], // % of budget
    emergency_shutdown_threshold: 1.2 // 120% of budget
  },

  // Resource efficiency targets
  EFFICIENCY_TARGETS: {
    cpu_utilization: { min: 60, max: 85 },
    memory_utilization: { min: 65, max: 80 },
    network_utilization: { min: 40, max: 75 },
    storage_utilization: { min: 70, max: 90 }
  }
};

// Monitoring and alerting configuration
export const MONITORING_CONFIG = {
  // Metrics collection
  METRICS: {
    collection_interval: 30, // seconds
    retention_period: 90, // days
    aggregation_intervals: [60, 300, 3600], // 1m, 5m, 1h
    custom_metrics: [
      'indian_user_sessions',
      'regional_load_distribution',
      'educational_platform_usage',
      'collaboration_session_count',
      'ai_inference_requests'
    ]
  },

  // Alert rules
  ALERTS: {
    CRITICAL: [
      {
        name: 'high_error_rate',
        condition: 'error_rate > 0.05',
        duration: '2m',
        channels: ['slack', 'email', 'sms']
      },
      {
        name: 'scaling_failure',
        condition: 'scaling_failed == true',
        duration: '1m',
        channels: ['slack', 'email', 'pagerduty']
      },
      {
        name: 'database_connection_pool_exhausted',
        condition: 'db_connections > 0.95',
        duration: '1m',
        channels: ['slack', 'email']
      }
    ],
    WARNING: [
      {
        name: 'prediction_accuracy_low',
        condition: 'prediction_accuracy < 0.8',
        duration: '10m',
        channels: ['slack']
      },
      {
        name: 'regional_imbalance',
        condition: 'regional_variance > 0.3',
        duration: '5m',
        channels: ['slack']
      }
    ]
  }
};

// Export utility functions
export class AutoScalingIntegrator {
  static calculateRegionalDistribution(
    totalInstances: number,
    currentHour: number
  ): { [region: string]: number } {
    const priorities = INDIAN_SCALING_PATTERNS.REGIONAL_PRIORITY;
    const distribution: { [region: string]: number } = {};
    
    // Adjust for peak hours
    let peakMultiplier = 1.0;
    if (currentHour >= 9 && currentHour < 12) peakMultiplier = 1.2;
    else if (currentHour >= 19 && currentHour < 23) peakMultiplier = 1.4;
    
    Object.entries(priorities).forEach(([region, config]) => {
      const baseInstances = Math.floor(totalInstances * config.weight);
      const peakInstances = Math.floor(baseInstances * peakMultiplier);
      const maxInstances = Math.floor(totalInstances * config.max_capacity);
      
      distribution[region] = Math.min(peakInstances, maxInstances);
    });
    
    return distribution;
  }

  static getEducationalMultiplier(): number {
    const currentMonth = new Date().getMonth() + 1;
    const calendar = INDIAN_SCALING_PATTERNS.EDUCATIONAL_CALENDAR;
    
    // Check exam seasons
    for (const season of calendar.EXAM_SEASONS) {
      if (season.months.includes(currentMonth)) {
        return season.multiplier;
      }
    }
    
    // Check admission seasons
    for (const season of calendar.ADMISSION_SEASONS) {
      if (season.months.includes(currentMonth)) {
        return season.multiplier;
      }
    }
    
    // Check vacation periods
    for (const period of calendar.VACATION_PERIODS) {
      if (period.months.includes(currentMonth)) {
        return period.multiplier;
      }
    }
    
    return 1.0; // Normal multiplier
  }

  static calculateCostOptimizedInstances(
    recommendedInstances: ResourceRecommendation,
    currentBudget: number
  ): ResourceRecommendation {
    const costConfig = COST_OPTIMIZATION;
    const maxBudget = costConfig.COST_CONTROLS.hourly_budget;
    
    if (currentBudget > maxBudget * 0.9) {
      // Reduce instances by 20% if near budget limit
      return {
        webServers: Math.floor(recommendedInstances.webServers * 0.8),
        databaseReplicas: Math.floor(recommendedInstances.databaseReplicas * 0.8),
        redisNodes: Math.floor(recommendedInstances.redisNodes * 0.8),
        websocketGateways: Math.floor(recommendedInstances.websocketGateways * 0.8),
        kafkaBrokers: Math.floor(recommendedInstances.kafkaBrokers * 0.8)
      };
    }
    
    return recommendedInstances;
  }

  static getScalingSequence(direction: 'up' | 'down'): any[] {
    return direction === 'up' 
      ? SCALING_ORCHESTRATION.SCALE_UP_SEQUENCE
      : SCALING_ORCHESTRATION.SCALE_DOWN_SEQUENCE;
  }
}

export default {
  PHASE_5C_INTEGRATION,
  SCALING_ORCHESTRATION,
  INDIAN_SCALING_PATTERNS,
  COST_OPTIMIZATION,
  MONITORING_CONFIG,
  AutoScalingIntegrator
};