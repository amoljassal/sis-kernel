import { SubscriptionPlan, UserSubscription, PaymentMethod, Invoice, UsageCredit, UserProfile, OnboardingProgress } from '../types/billing';

// Mock billing and subscription service
export class BillingService {
  private static instance: BillingService;
  private subscriptionPlans: Map<string, SubscriptionPlan> = new Map();
  private userSubscriptions: Map<string, UserSubscription> = new Map();
  private userProfiles: Map<string, UserProfile> = new Map();

  static getInstance(): BillingService {
    if (!BillingService.instance) {
      BillingService.instance = new BillingService();
      BillingService.instance.initializePlans();
    }
    return BillingService.instance;
  }

  private initializePlans(): void {
    const plans: SubscriptionPlan[] = [
      {
        id: 'community_monthly',
        tier: 'community',
        name: 'Community',
        price_usd: 0,
        billing_period: 'monthly',
        features: {
          max_private_projects: 0,
          synthesis_credits_per_month: 100,
          max_collaborators: 3,
          cloud_fpga_access: false,
          priority_support: false,
          advanced_collaboration: false,
          on_premise_deployment: false,
          sla_guarantee: false,
          saml_sso: false
        }
      },
      {
        id: 'pro_monthly',
        tier: 'pro',
        name: 'Pro',
        price_usd: 99,
        billing_period: 'monthly',
        popular: true,
        features: {
          max_private_projects: null,
          synthesis_credits_per_month: 1000,
          max_collaborators: 10,
          cloud_fpga_access: true,
          priority_support: true,
          advanced_collaboration: true,
          on_premise_deployment: false,
          sla_guarantee: false,
          saml_sso: false
        }
      },
      {
        id: 'pro_annual',
        tier: 'pro',
        name: 'Pro Annual',
        price_usd: 999, // 2 months free
        billing_period: 'annual',
        features: {
          max_private_projects: null,
          synthesis_credits_per_month: 1000,
          max_collaborators: 10,
          cloud_fpga_access: true,
          priority_support: true,
          advanced_collaboration: true,
          on_premise_deployment: false,
          sla_guarantee: false,
          saml_sso: false
        }
      },
      {
        id: 'enterprise_custom',
        tier: 'enterprise',
        name: 'Enterprise',
        price_usd: 0, // Custom pricing
        billing_period: 'monthly',
        features: {
          max_private_projects: null,
          synthesis_credits_per_month: null,
          max_collaborators: null,
          cloud_fpga_access: true,
          priority_support: true,
          advanced_collaboration: true,
          on_premise_deployment: true,
          sla_guarantee: true,
          saml_sso: true
        }
      }
    ];

    plans.forEach(plan => this.subscriptionPlans.set(plan.id, plan));

    // Create mock user profile and subscription
    const mockUser: UserProfile = {
      id: 'user_123',
      email: 'demo@sislab.ai',
      username: 'demo_user',
      display_name: 'Demo User',
      avatar_url: '/avatars/demo.png',
      bio: 'Hardware engineer exploring AI-powered design tools',
      company: 'SIS Demo Corp',
      location: 'Silicon Valley, CA',
      website: 'https://demo.sislab.ai',
      github_username: 'demo_user_sis',
      created_at: new Date('2024-01-15'),
      email_verified: true,
      payment_methods: [],
      total_earned_usd: 1250.75,
      reputation_score: 850,
      badges: ['early_adopter', 'contributor', 'verified_engineer']
    };

    const mockSubscription: UserSubscription = {
      id: 'sub_123',
      user_id: 'user_123',
      plan_id: 'pro_monthly',
      plan: plans[1], // Pro plan
      status: 'active',
      current_period_start: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000), // 15 days ago
      current_period_end: new Date(Date.now() + 15 * 24 * 60 * 60 * 1000), // 15 days from now
      cancel_at_period_end: false,
      usage: {
        synthesis_credits_used: 750,
        private_projects_count: 12,
        collaborators_count: 5,
        storage_gb_used: 4.2
      }
    };

    mockUser.subscription = mockSubscription;
    this.userProfiles.set('user_123', mockUser);
    this.userSubscriptions.set('sub_123', mockSubscription);
  }

  async getSubscriptionPlans(): Promise<SubscriptionPlan[]> {
    await new Promise(resolve => setTimeout(resolve, 500)); // Simulate API delay
    return Array.from(this.subscriptionPlans.values());
  }

  async getUserSubscription(userId: string): Promise<UserSubscription | null> {
    await new Promise(resolve => setTimeout(resolve, 300));
    
    // Find subscription by user_id
    for (const subscription of this.userSubscriptions.values()) {
      if (subscription.user_id === userId) {
        return subscription;
      }
    }
    return null;
  }

  async getUserProfile(userId: string): Promise<UserProfile | null> {
    await new Promise(resolve => setTimeout(resolve, 400));
    return this.userProfiles.get(userId) || null;
  }

  async upgradeSubscription(userId: string, planId: string): Promise<UserSubscription> {
    await new Promise(resolve => setTimeout(resolve, 2000)); // Simulate payment processing

    const plan = this.subscriptionPlans.get(planId);
    if (!plan) {
      throw new Error(`Plan ${planId} not found`);
    }

    const existingSubscription = await this.getUserSubscription(userId);
    const now = new Date();
    const nextMonth = new Date(now);
    nextMonth.setMonth(nextMonth.getMonth() + 1);

    const newSubscription: UserSubscription = {
      id: existingSubscription?.id || `sub_${Date.now()}`,
      user_id: userId,
      plan_id: planId,
      plan,
      status: 'active',
      current_period_start: now,
      current_period_end: plan.billing_period === 'annual' 
        ? new Date(now.getFullYear() + 1, now.getMonth(), now.getDate())
        : nextMonth,
      cancel_at_period_end: false,
      usage: existingSubscription?.usage || {
        synthesis_credits_used: 0,
        private_projects_count: 0,
        collaborators_count: 0,
        storage_gb_used: 0
      }
    };

    this.userSubscriptions.set(newSubscription.id, newSubscription);

    // Update user profile
    const userProfile = this.userProfiles.get(userId);
    if (userProfile) {
      userProfile.subscription = newSubscription;
      this.userProfiles.set(userId, userProfile);
    }

    return newSubscription;
  }

  async cancelSubscription(userId: string, cancelAtPeriodEnd: boolean = true): Promise<UserSubscription> {
    await new Promise(resolve => setTimeout(resolve, 1000));

    const subscription = await this.getUserSubscription(userId);
    if (!subscription) {
      throw new Error('No active subscription found');
    }

    subscription.cancel_at_period_end = cancelAtPeriodEnd;
    if (!cancelAtPeriodEnd) {
      subscription.status = 'cancelled';
    }

    this.userSubscriptions.set(subscription.id, subscription);

    // Update user profile
    const userProfile = this.userProfiles.get(userId);
    if (userProfile) {
      userProfile.subscription = subscription;
      this.userProfiles.set(userId, userProfile);
    }

    return subscription;
  }

  async getUsageCredits(userId: string): Promise<UsageCredit[]> {
    await new Promise(resolve => setTimeout(resolve, 400));

    const subscription = await this.getUserSubscription(userId);
    if (!subscription) return [];

    // Mock usage credits based on subscription
    const credits: UsageCredit[] = [
      {
        id: 'credit_synthesis_001',
        user_id: userId,
        type: 'synthesis',
        amount: subscription.plan.features.synthesis_credits_per_month || 0,
        remaining: Math.max(0, (subscription.plan.features.synthesis_credits_per_month || 0) - subscription.usage.synthesis_credits_used),
        expires_at: subscription.current_period_end,
        purchased_at: subscription.current_period_start,
        source: 'subscription'
      }
    ];

    if (subscription.plan.features.cloud_fpga_access) {
      credits.push({
        id: 'credit_cloud_fpga_001',
        user_id: userId,
        type: 'cloud_fpga_hour',
        amount: 10, // 10 hours per month for Pro
        remaining: 7, // 7 hours remaining
        expires_at: subscription.current_period_end,
        purchased_at: subscription.current_period_start,
        source: 'subscription'
      });
    }

    return credits;
  }

  async getInvoices(userId: string): Promise<Invoice[]> {
    await new Promise(resolve => setTimeout(resolve, 600));

    const subscription = await this.getUserSubscription(userId);
    if (!subscription || subscription.plan.price_usd === 0) return [];

    // Mock recent invoices
    const invoices: Invoice[] = [
      {
        id: 'inv_001',
        user_id: userId,
        subscription_id: subscription.id,
        amount_usd: subscription.plan.price_usd,
        tax_amount_usd: subscription.plan.price_usd * 0.08, // 8% tax
        total_amount_usd: subscription.plan.price_usd * 1.08,
        currency: 'USD',
        status: 'paid',
        created_at: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000), // 15 days ago
        due_date: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000),
        paid_at: new Date(Date.now() - 15 * 24 * 60 * 60 * 1000),
        items: [
          {
            id: 'item_001',
            description: `${subscription.plan.name} Plan - ${subscription.plan.billing_period}`,
            quantity: 1,
            unit_price_usd: subscription.plan.price_usd,
            total_price_usd: subscription.plan.price_usd,
            period_start: subscription.current_period_start,
            period_end: subscription.current_period_end
          }
        ],
        pdf_url: '/invoices/inv_001.pdf'
      },
      {
        id: 'inv_002',
        user_id: userId,
        subscription_id: subscription.id,
        amount_usd: subscription.plan.price_usd,
        tax_amount_usd: subscription.plan.price_usd * 0.08,
        total_amount_usd: subscription.plan.price_usd * 1.08,
        currency: 'USD',
        status: 'paid',
        created_at: new Date(Date.now() - 45 * 24 * 60 * 60 * 1000), // 45 days ago
        due_date: new Date(Date.now() - 45 * 24 * 60 * 60 * 1000),
        paid_at: new Date(Date.now() - 45 * 24 * 60 * 60 * 1000),
        items: [
          {
            id: 'item_002',
            description: `${subscription.plan.name} Plan - ${subscription.plan.billing_period}`,
            quantity: 1,
            unit_price_usd: subscription.plan.price_usd,
            total_price_usd: subscription.plan.price_usd,
            period_start: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000),
            period_end: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)
          }
        ],
        pdf_url: '/invoices/inv_002.pdf'
      }
    ];

    return invoices;
  }

  async getPaymentMethods(userId: string): Promise<PaymentMethod[]> {
    await new Promise(resolve => setTimeout(resolve, 400));

    // Mock payment methods
    const paymentMethods: PaymentMethod[] = [
      {
        id: 'pm_001',
        user_id: userId,
        type: 'card',
        last_four: '4242',
        brand: 'visa',
        expiry_month: 12,
        expiry_year: 2025,
        is_default: true,
        created_at: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000)
      },
      {
        id: 'pm_002',
        user_id: userId,
        type: 'card',
        last_four: '0005',
        brand: 'mastercard',
        expiry_month: 8,
        expiry_year: 2026,
        is_default: false,
        created_at: new Date(Date.now() - 60 * 24 * 60 * 60 * 1000)
      }
    ];

    return paymentMethods;
  }

  async addPaymentMethod(userId: string, paymentMethod: Omit<PaymentMethod, 'id' | 'user_id' | 'created_at'>): Promise<PaymentMethod> {
    await new Promise(resolve => setTimeout(resolve, 2000)); // Simulate card processing

    const newPaymentMethod: PaymentMethod = {
      id: `pm_${Date.now()}`,
      user_id: userId,
      created_at: new Date(),
      ...paymentMethod
    };

    return newPaymentMethod;
  }

  async getOnboardingProgress(userId: string): Promise<OnboardingProgress> {
    await new Promise(resolve => setTimeout(resolve, 300));

    return {
      user_id: userId,
      steps_completed: [
        'account_created',
        'email_verified',
        'first_login',
        'profile_completed'
      ],
      current_step: 'first_design',
      tutorial_progress: {
        first_design_created: false,
        first_simulation_run: false,
        first_hardware_deployment: false,
        first_ip_block_used: false,
        first_collaboration: false
      },
      skill_level: 'beginner'
    };
  }

  async updateOnboardingProgress(userId: string, step: string, completed: boolean = true): Promise<OnboardingProgress> {
    await new Promise(resolve => setTimeout(resolve, 200));

    const progress = await this.getOnboardingProgress(userId);
    
    if (completed && !progress.steps_completed.includes(step)) {
      progress.steps_completed.push(step);
    }

    // Update tutorial progress based on step
    switch (step) {
      case 'first_design_created':
        progress.tutorial_progress.first_design_created = completed;
        break;
      case 'first_simulation_run':
        progress.tutorial_progress.first_simulation_run = completed;
        break;
      case 'first_hardware_deployment':
        progress.tutorial_progress.first_hardware_deployment = completed;
        break;
      case 'first_ip_block_used':
        progress.tutorial_progress.first_ip_block_used = completed;
        break;
      case 'first_collaboration':
        progress.tutorial_progress.first_collaboration = completed;
        break;
    }

    // Update skill level based on progress
    const completedTutorials = Object.values(progress.tutorial_progress).filter(Boolean).length;
    if (completedTutorials >= 4) {
      progress.skill_level = 'advanced';
    } else if (completedTutorials >= 2) {
      progress.skill_level = 'intermediate';
    }

    // Check if onboarding is complete
    if (Object.values(progress.tutorial_progress).every(Boolean)) {
      progress.completed_at = new Date();
    }

    return progress;
  }

  // Utility methods
  formatPrice(amount: number): string {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD'
    }).format(amount);
  }

  calculateNextBillingDate(subscription: UserSubscription): Date {
    return subscription.current_period_end;
  }

  calculateProration(currentPlan: SubscriptionPlan, newPlan: SubscriptionPlan, daysRemaining: number): number {
    if (currentPlan.billing_period !== newPlan.billing_period) {
      // Different billing periods, calculate based on daily rates
      const currentDailyRate = currentPlan.price_usd / (currentPlan.billing_period === 'annual' ? 365 : 30);
      const newDailyRate = newPlan.price_usd / (newPlan.billing_period === 'annual' ? 365 : 30);
      return (newDailyRate - currentDailyRate) * daysRemaining;
    }
    
    // Same billing period
    const dailyDifference = (newPlan.price_usd - currentPlan.price_usd) / 
                           (currentPlan.billing_period === 'annual' ? 365 : 30);
    return dailyDifference * daysRemaining;
  }
}