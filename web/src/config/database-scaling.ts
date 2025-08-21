/**
 * Phase 5C Database Scaling Configuration
 * Target: 25,000+ concurrent Indian users
 * Based on Multi-AI Consultation Synthesis
 */

export interface DatabaseClusterConfig {
  region: string;
  isPrimary: boolean;
  maxConnections: number;
  instanceType: string;
  replicationFactor: number;
  shardingStrategy: string;
  connectionPooling: {
    maxConnections: number;
    poolSize: number;
    timeout: number;
  };
}

// Phase 5C: India-Primary Database Architecture
export const DATABASE_SCALING_CONFIG = {
  // Primary Cluster - Mumbai (AP-SOUTH-1)
  primary: {
    region: 'ap-south-1',
    isPrimary: true,
    maxConnections: 15000, // Support 25,000+ concurrent users
    instanceType: 'r6g.8xlarge', // Upgraded from r6g.4xlarge
    monthlyCapacity: '₹1,60,000', // ~$2,000
    
    // Master-Replica Setup
    master: {
      instanceType: 'r6g.8xlarge',
      storage: '2TB',
      iops: 20000,
      backupRetention: 30, // days
      multiAZ: true
    },
    
    readReplicas: [
      {
        id: 'mumbai-read-1',
        instanceType: 'r6g.4xlarge',
        purpose: 'indian_traffic_primary',
        maxConnections: 4000
      },
      {
        id: 'mumbai-read-2', 
        instanceType: 'r6g.4xlarge',
        purpose: 'indian_traffic_secondary',
        maxConnections: 4000
      },
      {
        id: 'mumbai-read-3',
        instanceType: 'r6g.4xlarge', 
        purpose: 'analytics_reporting',
        maxConnections: 2000
      },
      {
        id: 'mumbai-read-4', // New replica for 25K scaling
        instanceType: 'r6g.4xlarge',
        purpose: 'educational_workload',
        maxConnections: 5000
      }
    ],
    
    // Connection Pooling (PgBouncer)
    connectionPooling: {
      instances: 4, // Increased from 2
      maxConnections: 3750, // 15,000 / 4 instances
      poolSize: 100,
      timeout: 30,
      poolMode: 'transaction'
    },
    
    // Data Residency Compliance
    dataResidency: {
      enabled: true,
      enforcement: 'strict',
      auditLogging: true,
      encryptionAtRest: true,
      backupLocation: 'ap-south-1-only'
    }
  },
  
  // Sharding Strategy for 25,000+ Users
  sharding: {
    strategy: 'horizontal_by_org_geography',
    shards: [
      {
        id: 'shard_india_primary',
        region: 'ap-south-1', 
        userRange: '0-15000', // Primary Indian users
        organizations: 'indian_orgs',
        instanceType: 'r6g.4xlarge'
      },
      {
        id: 'shard_india_educational',
        region: 'ap-south-1',
        userRange: '15001-25000', // Educational institutions
        organizations: 'educational_orgs',
        instanceType: 'r6g.4xlarge'
      },
      {
        id: 'shard_global_users',
        region: 'us-east-1',
        userRange: '25001+', // Global users
        organizations: 'global_orgs',
        instanceType: 'r6g.2xlarge'
      }
    ],
    
    // Shard Key Strategy
    shardKey: {
      primary: 'organization_id',
      secondary: 'user_geography',
      algorithm: 'consistent_hashing'
    }
  }
} as const;

// Multi-Layer Redis Caching for Indian Traffic
export const REDIS_SCALING_CONFIG = {
  // L1 - Application Cache (Primary - Mumbai)
  l1_application_cache: {
    region: 'ap-south-1',
    cluster: {
      nodes: 8, // Increased from 6 for 25K users
      instanceType: 'r6g.2xlarge', // Upgraded
      totalMemory: '256GB', // 32GB per node
      evictionPolicy: 'allkeys-lru'
    },
    
    // Optimized for Indian Traffic Patterns
    caching: {
      userSessions: { ttl: 3600 }, // 1 hour
      designProjects: { ttl: 1800 }, // 30 minutes (active editing)
      componentLibrary: { ttl: 86400 }, // 24 hours
      educationalContent: { ttl: 43200 }, // 12 hours
      certificationData: { ttl: 604800 } // 7 days
    },
    
    // Indian Peak Hours Optimization
    peakHours: {
      schedule: '09:00-23:00 IST',
      preloadingEnabled: true,
      anticipatoryCache: true,
      festivalsOptimization: ['diwali', 'dussehra', 'exam_season']
    }
  },
  
  // L2 - Session Store (High Availability)
  l2_session_store: {
    region: 'ap-south-1',
    sentinel: {
      nodes: 5, // Increased from 3 for better HA
      instanceType: 'r6g.large',
      quorum: 3
    },
    
    // Session Management for 25K Users
    sessions: {
      maxConcurrent: 25000,
      timeout: 7200, // 2 hours
      indianComplianceMode: true, // PDPB compliance
      encryptionEnabled: true
    }
  },
  
  // L3 - Real-time Collaboration
  l3_collaboration: {
    region: 'ap-south-1',
    streams: {
      nodes: 6,
      instanceType: 'r6g.xlarge',
      maxConnections: 30000, // 25K users + overhead
      
      // Optimized for Indian Network Conditions
      networkOptimization: {
        bufferSize: '64MB',
        compressionEnabled: true,
        offlineSync: true, // Handle unreliable connections
        conflictResolution: 'indian_timezone_priority'
      }
    },
    
    // Real-time Channels
    channels: {
      design_collaboration: { 
        maxUsers: 50,
        persistence: '7_days',
        compression: true
      },
      educational_sessions: {
        maxUsers: 500, // Large classroom support
        persistence: '30_days',
        recording: true
      },
      marketplace_updates: {
        broadcast: true,
        persistence: '1_day'
      }
    }
  }
} as const;

// WebSocket Infrastructure for 25K Concurrent Connections
export const WEBSOCKET_SCALING_CONFIG = {
  // Primary Gateway - Mumbai
  mumbai_primary: {
    region: 'ap-south-1',
    capacity: 20000, // Increased from 6,000 for 25K users
    instances: 10, // Scaled up from 6
    instanceType: 'c6i.4xlarge',
    
    // Sticky Sessions for Collaboration
    stickySessionConfig: {
      enabled: true,
      algorithm: 'ip_hash',
      fallbackEnabled: true,
      sessionPersistence: '2_hours'
    },
    
    // ISP-Optimized Connections
    ispOptimization: {
      directPeering: ['bharti-airtel', 'reliance-jio', 'tata-communications'],
      cdnIntegration: true,
      edgeTermination: ['delhi', 'bangalore', 'chennai', 'hyderabad']
    }
  },
  
  // Kafka Event Streaming (Scaled)
  kafkaCluster: {
    region: 'ap-south-1',
    brokers: 9, // Increased from 6
    partitionStrategy: 'user_id_hash',
    replicationFactor: 3,
    
    topics: {
      design_events_india: {
        partitions: 60, // Doubled for 25K users
        replication: 3,
        retention: '7_days',
        compression: 'lz4'
      },
      collaboration_events_india: {
        partitions: 40,
        replication: 3,
        retention: '3_days',
        compression: 'lz4'
      },
      analytics_events_india: {
        partitions: 30,
        replication: 2,
        retention: '30_days',
        compression: 'gzip'
      },
      educational_events: { // New topic for education
        partitions: 20,
        replication: 3,
        retention: '90_days',
        compression: 'gzip'
      }
    }
  },
  
  // CRDT for Collaborative Editing
  crdtConfiguration: {
    algorithm: 'yjs_optimized',
    
    // Indian Network Optimization
    networkAdaptation: {
      lowBandwidthMode: true,
      offlineFirstEnabled: true,
      conflictResolutionStrategy: 'timestamp_with_indian_timezone',
      deltaCompression: true
    },
    
    // Educational Use Cases
    educationalFeatures: {
      professorOverrideEnabled: true,
      studentGroupCollaboration: true,
      examModeIsolation: true,
      progressTracking: true
    }
  }
} as const;

// Auto-Scaling for 25K Users (Enhanced)
export const ENHANCED_SCALING_CONFIG = {
  // EKS Cluster - Mumbai (Scaled)
  mumbai_cluster: {
    nodeGroups: [
      {
        name: 'primary-compute',
        instanceTypes: ['c6i.4xlarge', 'c6i.8xlarge'],
        minSize: 15, // Increased from 8
        maxSize: 80, // Doubled from 40
        targetCapacity: 'ON_DEMAND:30%, SPOT:70%'
      },
      {
        name: 'memory-optimized', // New node group for caching
        instanceTypes: ['r6g.2xlarge', 'r6g.4xlarge'],
        minSize: 8,
        maxSize: 25,
        targetCapacity: 'ON_DEMAND:50%, SPOT:50%'
      }
    ],
    
    // Predictive Scaling (ML-Enhanced)
    predictiveScaling: {
      enabled: true,
      mlModel: 'indian_usage_patterns_v2',
      
      // Indian-Specific Patterns
      patterns: {
        dailyPeaks: ['09:00-12:00', '14:00-17:00', '19:00-23:00'], // IST
        weeklyPeaks: ['monday', 'tuesday', 'wednesday'],
        monthlyPeaks: ['exam_preparation_periods', 'project_submission_weeks'],
        festivals: {
          diwali: { scalingFactor: 2.5, duration: '5_days' },
          examSeason: { scalingFactor: 3.0, duration: '4_weeks' }
        }
      }
    }
  }
} as const;

// Cost Optimization for 25K Scale
export const COST_OPTIMIZATION_CONFIG = {
  targetCosts: {
    monthlyBudget: '₹12,00,000', // ₹1.2 Cr for 25K users
    costPerUser: '₹480', // ₹240/user target × 2 for safety margin
    breakdown: {
      database: '₹4,80,000', // 40%
      compute: '₹3,60,000', // 30% 
      networking: '₹2,40,000', // 20%
      storage: '₹1,20,000' // 10%
    }
  },
  
  optimization: {
    spotInstanceUtilization: 70, // 70% spot instances
    reservedInstanceCommitment: 30, // 30% reserved for stability
    autoScalingAggressive: true,
    coldStorageAfter: '90_days',
    compressionEnabled: true
  }
} as const;

export default {
  DATABASE_SCALING_CONFIG,
  REDIS_SCALING_CONFIG, 
  WEBSOCKET_SCALING_CONFIG,
  ENHANCED_SCALING_CONFIG,
  COST_OPTIMIZATION_CONFIG
};