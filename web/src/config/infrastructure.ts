/**
 * SIS Global Infrastructure Configuration
 * Phase 5A: India-Primary Global Platform Setup
 * Primary Region: AP-SOUTH-1 (Mumbai)
 */

export interface RegionConfig {
  name: string;
  code: string;
  isPrimary: boolean;
  maxConcurrentUsers: number;
  targetLatency: number; // milliseconds
  edgeLocations: string[];
  ispPartners: string[];
  dataResidency: boolean;
  complianceRequirements: string[];
}

export const GLOBAL_INFRASTRUCTURE_CONFIG = {
  // India as Primary Region (60% of global users)
  primary: {
    name: 'Asia Pacific (Mumbai)',
    code: 'ap-south-1',
    isPrimary: true,
    maxConcurrentUsers: 6000,
    targetLatency: 50,
    edgeLocations: [
      'mumbai-edge-1',
      'delhi-edge-1', 
      'bangalore-edge-1',
      'chennai-edge-1',
      'hyderabad-edge-1',
      'pune-edge-1'
    ],
    ispPartners: [
      'bharti-airtel',
      'reliance-jio',
      'bsnl',
      'tata-communications',
      'vodafone-idea'
    ],
    dataResidency: true, // Indian user data stays in India
    complianceRequirements: [
      'PDPB', // Personal Data Protection Bill
      'IT_ACT_2000', // Information Technology Act
      'GST_COMPLIANCE',
      'RBI_GUIDELINES'
    ]
  },

  // Secondary Regions
  secondary: [
    {
      name: 'US East (N. Virginia)',
      code: 'us-east-1', 
      isPrimary: false,
      maxConcurrentUsers: 2500,
      targetLatency: 200,
      edgeLocations: ['virginia', 'ohio', 'oregon'],
      ispPartners: ['aws-cloudfront', 'cloudflare'],
      dataResidency: false,
      complianceRequirements: ['SOC2', 'GDPR', 'CCPA']
    },
    {
      name: 'EU West (Ireland)',
      code: 'eu-west-1',
      isPrimary: false, 
      maxConcurrentUsers: 1500,
      targetLatency: 200,
      edgeLocations: ['dublin', 'frankfurt', 'london'],
      ispPartners: ['aws-cloudfront', 'cloudflare'],
      dataResidency: true, // EU users data stays in EU
      complianceRequirements: ['GDPR', 'SOC2']
    }
  ]
} as const;

// Performance monitoring thresholds for Indian infrastructure
export const PERFORMANCE_TARGETS = {
  india: {
    latency: {
      p50: 25, // milliseconds
      p95: 50, // milliseconds 
      p99: 100 // milliseconds
    },
    availability: {
      target: 99.95, // 99.95% uptime
      measurement: 'monthly'
    },
    throughput: {
      requestsPerSecond: 10000,
      concurrentUsers: 6000,
      peakCapacity: 8000 // 33% headroom
    }
  },
  
  global: {
    latency: {
      p50: 100,
      p95: 200, 
      p99: 500
    },
    availability: {
      target: 99.9,
      measurement: 'monthly' 
    },
    throughput: {
      requestsPerSecond: 15000,
      concurrentUsers: 10000,
      peakCapacity: 13000
    }
  }
} as const;

// Infrastructure scaling configuration
export const AUTO_SCALING_CONFIG = {
  india: {
    normalHours: {
      minInstances: 8,
      maxInstances: 25,
      targetCPU: 70,
      scaleUpCooldown: 300, // 5 minutes
      scaleDownCooldown: 600 // 10 minutes  
    },
    peakHours: { // 9 AM - 11 PM IST
      minInstances: 15,
      maxInstances: 40,
      targetCPU: 60,
      anticipatoryScaling: true
    },
    examSeason: { // November, May
      minInstances: 25,
      maxInstances: 60,
      targetCPU: 50,
      anticipatoryScaling: true,
      duration: '4_weeks'
    }
  }
} as const;

export const INDIAN_MARKET_CONFIG = {
  // Pricing tiers in INR
  pricing: {
    currency: 'INR',
    tiers: {
      free: {
        price: 0,
        features: ['public_projects', 'community_support', 'basic_tutorials'],
        limits: { projects: 3, storage_gb: 1, ai_queries: 10 }
      },
      student: {
        price: 999, // 67% discount from Pro
        billing: 'monthly',
        verification: 'edu_email_required',
        features: ['pro_features', 'educational_content', 'placement_prep'],
        limits: { projects: 10, storage_gb: 5, ai_queries: 100 }
      },
      pro: {
        price: 2999, // ~$36/month optimized for Indian market
        billing: 'monthly',
        features: [
          'unlimited_private_projects',
          'ai_design_assistant', 
          'advanced_simulation',
          'priority_support',
          'collaboration_tools'
        ],
        limits: { projects: 'unlimited', storage_gb: 50, ai_queries: 1000 }
      },
      enterprise: {
        price: 25000, // ~$300/month
        billing: 'monthly',
        features: [
          'sso_integration',
          'advanced_compliance',
          'dedicated_support',
          'custom_integrations',
          'on_premise_option',
          'bulk_licensing'
        ],
        limits: { 
          projects: 'unlimited', 
          storage_gb: 'unlimited', 
          ai_queries: 'unlimited',
          users: 'unlimited'
        }
      }
    },
    
    // Geographic pricing adjustments
    regionalDiscounts: {
      tier1Cities: ['mumbai', 'delhi', 'bangalore', 'hyderabad', 'chennai', 'pune', 'kolkata', 'ahmedabad'],
      tier2Cities: { discount: 0.20 }, // 20% discount
      tier3Cities: { discount: 0.40 }, // 40% discount for accessibility
      ruralAreas: { discount: 0.50 }   // 50% discount with government partnerships
    },
    
    // Payment methods popular in India
    paymentMethods: {
      primary: ['upi', 'netbanking', 'razorpay'],
      secondary: ['credit_card', 'debit_card', 'wallet'],
      emi: { 
        available: true,
        durations: [3, 6, 12], // months
        minAmount: 10000 // ₹10,000 minimum for EMI
      }
    },
    
    // GST configuration
    taxation: {
      gst: {
        rate: 0.18, // 18% GST on digital services
        applicable: true,
        automaticCalculation: true,
        filingRequired: true
      },
      tds: {
        rate: 0.02, // 2% TDS on digital services
        applicable: true,
        threshold: 250000 // ₹2.5 Lakh per year
      }
    }
  },

  // Indian educational system integration
  education: {
    targetInstitutions: {
      tier1: {
        iits: 23, // All IITs
        nits: 31, // All NITs  
        iiits: 25, // IIITs
        priority: ['iit_bombay', 'iit_delhi', 'iit_madras', 'iit_kanpur', 'iit_kharagpur']
      },
      tier2: {
        stateUniversities: 400,
        deemedUniversities: 126,
        priority: ['vit', 'srm', 'manipal', 'amity', 'bennett']
      },
      tier3: {
        privateColleges: 3000,
        affiliatedColleges: 5000
      }
    },
    
    academicCalendar: {
      oddSemester: { start: 'july', end: 'november' },
      evenSemester: { start: 'january', end: 'may' },
      examPeriods: ['november', 'may'],
      admissions: ['june', 'december']
    },
    
    competitiveExams: {
      gate: { 
        subjects: ['ec', 'ee', 'cs', 'in'], 
        preparation: true,
        integration: 'curriculum_aligned'
      },
      jee: {
        advanced: { preparation: true },
        mains: { preparation: true }
      },
      placement: {
        companies: ['tcs', 'infosys', 'wipro', 'hcl', 'accenture', 'cognizant'],
        preparation: true,
        certification_weightage: '25_percent_salary_premium'
      }
    }
  },

  // Indian language and localization
  localization: {
    languages: {
      primary: 'en-IN', // Indian English
      secondary: ['hi-IN'], // Hindi
      planned: ['ta-IN', 'te-IN', 'mr-IN'] // Tamil, Telugu, Marathi
    },
    
    cultural: {
      festivals: ['diwali', 'dussehra', 'holi', 'ganesh_chaturthi'],
      workingDays: { pattern: 'monday_to_saturday', holidays: 'indian_calendar' },
      businessHours: { start: '09:00', end: '18:00', timezone: 'Asia/Kolkata' }
    }
  }
} as const;

export default {
  GLOBAL_INFRASTRUCTURE_CONFIG,
  INDIAN_MARKET_CONFIG,
  PERFORMANCE_TARGETS,
  AUTO_SCALING_CONFIG
};