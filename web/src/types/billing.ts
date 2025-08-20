// Business platform types for pricing, billing, and marketplace
export type SubscriptionTier = 'community' | 'pro' | 'enterprise';
export type PaymentStatus = 'pending' | 'paid' | 'failed' | 'refunded';
export type LicenseType = 'free' | 'commercial' | 'enterprise' | 'custom';

export interface SubscriptionPlan {
  id: string;
  tier: SubscriptionTier;
  name: string;
  price_usd: number;
  billing_period: 'monthly' | 'annual';
  features: {
    max_private_projects: number | null; // null = unlimited
    synthesis_credits_per_month: number | null; // null = unlimited
    max_collaborators: number | null;
    cloud_fpga_access: boolean;
    priority_support: boolean;
    advanced_collaboration: boolean;
    on_premise_deployment: boolean;
    sla_guarantee: boolean;
    saml_sso: boolean;
  };
  popular?: boolean;
}

export interface UserSubscription {
  id: string;
  user_id: string;
  plan_id: string;
  plan: SubscriptionPlan;
  status: 'active' | 'cancelled' | 'past_due' | 'trialing';
  current_period_start: Date;
  current_period_end: Date;
  cancel_at_period_end: boolean;
  trial_end?: Date;
  usage: {
    synthesis_credits_used: number;
    private_projects_count: number;
    collaborators_count: number;
    storage_gb_used: number;
  };
}

export interface PaymentMethod {
  id: string;
  user_id: string;
  type: 'card' | 'bank_account' | 'paypal';
  last_four: string;
  brand?: string; // for cards: visa, mastercard, etc.
  expiry_month?: number;
  expiry_year?: number;
  is_default: boolean;
  created_at: Date;
}

export interface Invoice {
  id: string;
  user_id: string;
  subscription_id?: string;
  amount_usd: number;
  tax_amount_usd: number;
  total_amount_usd: number;
  currency: string;
  status: PaymentStatus;
  payment_method_id?: string;
  created_at: Date;
  due_date: Date;
  paid_at?: Date;
  items: InvoiceItem[];
  pdf_url?: string;
}

export interface InvoiceItem {
  id: string;
  description: string;
  quantity: number;
  unit_price_usd: number;
  total_price_usd: number;
  period_start?: Date;
  period_end?: Date;
}

export interface UsageCredit {
  id: string;
  user_id: string;
  type: 'synthesis' | 'cloud_fpga_hour' | 'storage_gb_month';
  amount: number;
  remaining: number;
  expires_at?: Date;
  purchased_at: Date;
  source: 'subscription' | 'purchase' | 'promotional';
}

// Marketplace types
export interface IPBlock {
  id: string;
  name: string;
  description: string;
  category: 'processor' | 'memory' | 'io' | 'dsp' | 'communication' | 'custom';
  author_id: string;
  author_name: string;
  version: string;
  tags: string[];
  license_type: LicenseType;
  price_usd?: number; // undefined for free blocks
  download_count: number;
  rating: number; // 1-5 stars
  review_count: number;
  created_at: Date;
  updated_at: Date;
  verified: boolean; // verified by SIS team
  featured: boolean;
  compatibility: {
    fpga_vendors: string[];
    min_logic_cells: number;
    min_block_ram_kb: number;
    min_dsp_blocks: number;
  };
  files: {
    verilog_url?: string;
    vhdl_url?: string;
    documentation_url?: string;
    example_url?: string;
    testbench_url?: string;
  };
  preview_image_url?: string;
  demo_video_url?: string;
}

export interface MarketplacePurchase {
  id: string;
  user_id: string;
  ip_block_id: string;
  ip_block: IPBlock;
  price_paid_usd: number;
  license_terms: string;
  purchased_at: Date;
  expires_at?: Date; // for time-limited licenses
  download_count: number;
  max_downloads?: number;
}

export interface MarketplaceReview {
  id: string;
  ip_block_id: string;
  user_id: string;
  user_name: string;
  rating: number; // 1-5 stars
  title: string;
  content: string;
  helpful_count: number;
  created_at: Date;
  verified_purchase: boolean;
}

export interface RevenueShare {
  id: string;
  author_id: string;
  ip_block_id: string;
  period_start: Date;
  period_end: Date;
  total_sales: number;
  total_revenue_usd: number;
  platform_fee_percent: number;
  platform_fee_usd: number;
  author_share_usd: number;
  status: 'pending' | 'paid' | 'processing';
  paid_at?: Date;
}

export interface UserProfile {
  id: string;
  email: string;
  username: string;
  display_name: string;
  avatar_url?: string;
  bio?: string;
  company?: string;
  location?: string;
  website?: string;
  github_username?: string;
  linkedin_username?: string;
  created_at: Date;
  email_verified: boolean;
  subscription?: UserSubscription;
  payment_methods: PaymentMethod[];
  total_earned_usd: number; // from marketplace sales
  reputation_score: number;
  badges: string[];
}

export interface OnboardingProgress {
  user_id: string;
  steps_completed: string[];
  current_step: string;
  tutorial_progress: {
    first_design_created: boolean;
    first_simulation_run: boolean;
    first_hardware_deployment: boolean;
    first_ip_block_used: boolean;
    first_collaboration: boolean;
  };
  skill_level: 'beginner' | 'intermediate' | 'advanced' | 'expert';
  completed_at?: Date;
}

export interface SupportTicket {
  id: string;
  user_id: string;
  subject: string;
  content: string;
  category: 'technical' | 'billing' | 'marketplace' | 'feature_request' | 'bug_report';
  priority: 'low' | 'medium' | 'high' | 'critical';
  status: 'open' | 'in_progress' | 'waiting_user' | 'resolved' | 'closed';
  assigned_to?: string;
  created_at: Date;
  updated_at: Date;
  resolved_at?: Date;
  satisfaction_rating?: number; // 1-5 stars
}