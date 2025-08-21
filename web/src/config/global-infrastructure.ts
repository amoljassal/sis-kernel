// Phase 6B: Global Infrastructure Expansion
// Multi-region deployment with edge computing capabilities

export const GLOBAL_REGIONS = {
  // Americas
  AMERICAS: {
    'us-east-1': {
      name: 'US East (Virginia)',
      location: { lat: 38.9072, lng: -77.0369 },
      provider: 'aws',
      tier: 'primary',
      capabilities: ['compute', 'storage', 'ai', 'edge'],
      compliance: ['SOC2', 'HIPAA', 'PCI-DSS'],
      dataCenter: 'us-east-1a'
    },
    'us-west-2': {
      name: 'US West (Oregon)',
      location: { lat: 45.5152, lng: -122.6784 },
      provider: 'aws',
      tier: 'primary',
      capabilities: ['compute', 'storage', 'ai', 'edge'],
      compliance: ['SOC2', 'HIPAA', 'PCI-DSS'],
      dataCenter: 'us-west-2a'
    },
    'sa-east-1': {
      name: 'South America (São Paulo)',
      location: { lat: -23.5505, lng: -46.6333 },
      provider: 'aws',
      tier: 'secondary',
      capabilities: ['compute', 'storage', 'edge'],
      compliance: ['LGPD'],
      dataCenter: 'sa-east-1a'
    }
  },

  // Europe
  EUROPE: {
    'eu-west-2': {
      name: 'Europe (London)',
      location: { lat: 51.5074, lng: -0.1278 },
      provider: 'aws',
      tier: 'primary',
      capabilities: ['compute', 'storage', 'ai', 'edge'],
      compliance: ['GDPR', 'ISO27001'],
      dataCenter: 'eu-west-2a'
    },
    'eu-central-1': {
      name: 'Europe (Frankfurt)',
      location: { lat: 50.1109, lng: 8.6821 },
      provider: 'aws',
      tier: 'primary',
      capabilities: ['compute', 'storage', 'ai', 'edge'],
      compliance: ['GDPR', 'ISO27001'],
      dataCenter: 'eu-central-1a'
    },
    'eu-west-1': {
      name: 'Europe (Amsterdam)',
      location: { lat: 52.3676, lng: 4.9041 },
      provider: 'aws',
      tier: 'secondary',
      capabilities: ['compute', 'storage', 'edge'],
      compliance: ['GDPR', 'ISO27001'],
      dataCenter: 'eu-west-1a'
    }
  },

  // Asia-Pacific
  ASIA_PACIFIC: {
    'ap-northeast-1': {
      name: 'Asia Pacific (Tokyo)',
      location: { lat: 35.6762, lng: 139.6503 },
      provider: 'aws',
      tier: 'primary',
      capabilities: ['compute', 'storage', 'ai', 'edge'],
      compliance: ['PIPL', 'ISO27001'],
      dataCenter: 'ap-northeast-1a'
    },
    'ap-southeast-2': {
      name: 'Asia Pacific (Sydney)',
      location: { lat: -33.8688, lng: 151.2093 },
      provider: 'aws',
      tier: 'secondary',
      capabilities: ['compute', 'storage', 'edge'],
      compliance: ['Privacy Act', 'ISO27001'],
      dataCenter: 'ap-southeast-2a'
    },
    'ap-northeast-2': {
      name: 'Asia Pacific (Seoul)',
      location: { lat: 37.5665, lng: 126.9780 },
      provider: 'aws',
      tier: 'secondary',
      capabilities: ['compute', 'storage', 'edge'],
      compliance: ['PIPA', 'ISO27001'],
      dataCenter: 'ap-northeast-2a'
    },
    // Existing India infrastructure (from Phase 5C)
    'ap-south-1': {
      name: 'Asia Pacific (Mumbai)',
      location: { lat: 19.0760, lng: 72.8777 },
      provider: 'aws',
      tier: 'primary',
      capabilities: ['compute', 'storage', 'ai', 'edge'],
      compliance: ['DPDP', 'ISO27001'],
      dataCenter: 'ap-south-1a'
    }
  }
};

// Edge CDN Deployment (50+ locations)
export const EDGE_LOCATIONS = {
  NORTH_AMERICA: [
    { city: 'New York', code: 'JFK', country: 'US', provider: 'cloudflare' },
    { city: 'Los Angeles', code: 'LAX', country: 'US', provider: 'cloudflare' },
    { city: 'Chicago', code: 'ORD', country: 'US', provider: 'aws' },
    { city: 'Dallas', code: 'DFW', country: 'US', provider: 'aws' },
    { city: 'Seattle', code: 'SEA', country: 'US', provider: 'aws' },
    { city: 'Miami', code: 'MIA', country: 'US', provider: 'cloudflare' },
    { city: 'Toronto', code: 'YYZ', country: 'CA', provider: 'aws' },
    { city: 'Montreal', code: 'YUL', country: 'CA', provider: 'cloudflare' },
    { city: 'Mexico City', code: 'MEX', country: 'MX', provider: 'cloudflare' }
  ],

  SOUTH_AMERICA: [
    { city: 'São Paulo', code: 'GRU', country: 'BR', provider: 'aws' },
    { city: 'Rio de Janeiro', code: 'GIG', country: 'BR', provider: 'cloudflare' },
    { city: 'Buenos Aires', code: 'EZE', country: 'AR', provider: 'cloudflare' },
    { city: 'Lima', code: 'LIM', country: 'PE', provider: 'cloudflare' },
    { city: 'Bogotá', code: 'BOG', country: 'CO', provider: 'cloudflare' }
  ],

  EUROPE: [
    { city: 'London', code: 'LHR', country: 'GB', provider: 'aws' },
    { city: 'Frankfurt', code: 'FRA', country: 'DE', provider: 'aws' },
    { city: 'Amsterdam', code: 'AMS', country: 'NL', provider: 'aws' },
    { city: 'Paris', code: 'CDG', country: 'FR', provider: 'cloudflare' },
    { city: 'Madrid', code: 'MAD', country: 'ES', provider: 'cloudflare' },
    { city: 'Rome', code: 'FCO', country: 'IT', provider: 'cloudflare' },
    { city: 'Zurich', code: 'ZUR', country: 'CH', provider: 'cloudflare' },
    { city: 'Stockholm', code: 'ARN', country: 'SE', provider: 'aws' },
    { city: 'Warsaw', code: 'WAW', country: 'PL', provider: 'cloudflare' },
    { city: 'Vienna', code: 'VIE', country: 'AT', provider: 'cloudflare' }
  ],

  ASIA_PACIFIC: [
    { city: 'Mumbai', code: 'BOM', country: 'IN', provider: 'aws' },
    { city: 'Delhi', code: 'DEL', country: 'IN', provider: 'aws' },
    { city: 'Bangalore', code: 'BLR', country: 'IN', provider: 'aws' },
    { city: 'Chennai', code: 'MAA', country: 'IN', provider: 'aws' },
    { city: 'Tokyo', code: 'NRT', country: 'JP', provider: 'aws' },
    { city: 'Seoul', code: 'ICN', country: 'KR', provider: 'aws' },
    { city: 'Sydney', code: 'SYD', country: 'AU', provider: 'aws' },
    { city: 'Singapore', code: 'SIN', country: 'SG', provider: 'aws' },
    { city: 'Hong Kong', code: 'HKG', country: 'HK', provider: 'cloudflare' },
    { city: 'Taipei', code: 'TPE', country: 'TW', provider: 'cloudflare' },
    { city: 'Bangkok', code: 'BKK', country: 'TH', provider: 'cloudflare' },
    { city: 'Jakarta', code: 'CGK', country: 'ID', provider: 'cloudflare' },
    { city: 'Manila', code: 'MNL', country: 'PH', provider: 'cloudflare' }
  ],

  MIDDLE_EAST_AFRICA: [
    { city: 'Dubai', code: 'DXB', country: 'AE', provider: 'aws' },
    { city: 'Tel Aviv', code: 'TLV', country: 'IL', provider: 'cloudflare' },
    { city: 'Cape Town', code: 'CPT', country: 'ZA', provider: 'aws' },
    { city: 'Lagos', code: 'LOS', country: 'NG', provider: 'cloudflare' },
    { city: 'Cairo', code: 'CAI', country: 'EG', provider: 'cloudflare' }
  ]
};

// Global Infrastructure Configuration
export const GLOBAL_INFRASTRUCTURE_CONFIG = {
  // Data replication strategy
  DATA_REPLICATION: {
    strategy: 'multi-master',
    consistency: 'eventual',
    syncInterval: 30, // seconds
    conflictResolution: 'timestamp-based',
    regions: {
      primary: ['us-east-1', 'eu-west-2', 'ap-south-1', 'ap-northeast-1'],
      secondary: ['us-west-2', 'eu-central-1', 'ap-southeast-2'],
      backup: ['sa-east-1', 'eu-west-1', 'ap-northeast-2']
    }
  },

  // Load balancing configuration
  LOAD_BALANCING: {
    algorithm: 'geo-proximity',
    healthCheckInterval: 30, // seconds
    failoverTime: 5, // seconds
    algorithms: {
      'geo-proximity': { weight: 0.6 },
      'latency-based': { weight: 0.3 },
      'load-based': { weight: 0.1 }
    },
    routing: {
      americas: ['us-east-1', 'us-west-2', 'sa-east-1'],
      europe: ['eu-west-2', 'eu-central-1', 'eu-west-1'],
      asia: ['ap-south-1', 'ap-northeast-1', 'ap-southeast-2', 'ap-northeast-2']
    }
  },

  // Edge computing configuration
  EDGE_COMPUTING: {
    enabled: true,
    cacheStrategy: 'intelligent',
    cacheTTL: {
      static: 86400, // 24 hours
      dynamic: 300,  // 5 minutes
      api: 60        // 1 minute
    },
    edgeFunctions: [
      'user-authentication',
      'request-routing',
      'content-optimization',
      'security-filtering'
    ],
    bandwidth: {
      tier1: '100 Gbps', // Primary regions
      tier2: '50 Gbps',  // Secondary regions
      edge: '10 Gbps'    // Edge locations
    }
  },

  // GDPR and compliance configuration
  COMPLIANCE: {
    GDPR: {
      enabled: true,
      regions: ['eu-west-2', 'eu-central-1', 'eu-west-1'],
      dataResidency: 'eu-only',
      rightToErasure: true,
      dataPortability: true,
      consentManagement: true,
      auditLogging: true
    },
    CCPA: {
      enabled: true,
      regions: ['us-east-1', 'us-west-2'],
      rightToDelete: true,
      rightToKnow: true,
      optOut: true
    },
    LGPD: {
      enabled: true,
      regions: ['sa-east-1'],
      dataMinimization: true,
      consentRequired: true
    },
    DPDP: {
      enabled: true,
      regions: ['ap-south-1'],
      dataLocalization: true,
      consentFramework: true
    }
  },

  // Disaster recovery
  DISASTER_RECOVERY: {
    rpo: 60, // Recovery Point Objective (seconds)
    rto: 300, // Recovery Time Objective (seconds)
    backupFrequency: 'continuous',
    crossRegionBackup: true,
    failoverStrategy: 'automatic',
    regions: {
      'us-east-1': { backup: 'us-west-2', tertiary: 'eu-west-2' },
      'eu-west-2': { backup: 'eu-central-1', tertiary: 'us-east-1' },
      'ap-south-1': { backup: 'ap-northeast-1', tertiary: 'ap-southeast-2' }
    }
  },

  // Performance monitoring
  MONITORING: {
    metrics: [
      'latency',
      'throughput',
      'error-rate',
      'availability',
      'resource-utilization'
    ],
    alerting: {
      latency: { threshold: 100, unit: 'ms' },
      errorRate: { threshold: 1, unit: 'percent' },
      availability: { threshold: 99.9, unit: 'percent' }
    },
    reporting: {
      frequency: 'real-time',
      aggregation: 'regional',
      retention: '90 days'
    }
  }
};

// Regional traffic patterns for global scaling
export const GLOBAL_TRAFFIC_PATTERNS = {
  AMERICAS: {
    peakHours: {
      est: [9, 17], // 9 AM - 5 PM EST
      pst: [9, 17], // 9 AM - 5 PM PST
      brt: [8, 16]  // 8 AM - 4 PM BRT
    },
    expectedLoad: {
      students: 50000,
      institutions: 500,
      concurrent: 15000
    },
    languages: ['english', 'spanish', 'portuguese'],
    preferences: {
      lowLatency: true,
      highThroughput: true,
      aiFeatures: 0.8
    }
  },

  EUROPE: {
    peakHours: {
      gmt: [8, 18], // 8 AM - 6 PM GMT
      cet: [8, 18]  // 8 AM - 6 PM CET
    },
    expectedLoad: {
      students: 75000,
      institutions: 800,
      concurrent: 20000
    },
    languages: ['english', 'german', 'french', 'spanish', 'italian'],
    preferences: {
      dataPrivacy: true,
      gdprCompliance: true,
      aiFeatures: 0.7
    }
  },

  ASIA_PACIFIC: {
    peakHours: {
      ist: [9, 23], // 9 AM - 11 PM IST (existing)
      jst: [9, 17], // 9 AM - 5 PM JST
      aest: [9, 17] // 9 AM - 5 PM AEST
    },
    expectedLoad: {
      students: 200000, // Including existing India base
      institutions: 1500,
      concurrent: 50000
    },
    languages: ['english', 'hindi', 'japanese', 'korean', 'mandarin'],
    preferences: {
      mobileFirst: true,
      voiceFeatures: 0.9,
      aiFeatures: 0.85
    }
  }
};

export default {
  GLOBAL_REGIONS,
  EDGE_LOCATIONS,
  GLOBAL_INFRASTRUCTURE_CONFIG,
  GLOBAL_TRAFFIC_PATTERNS
};