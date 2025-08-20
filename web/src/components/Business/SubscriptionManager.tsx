import React, { useState, useEffect } from 'react';
import { SubscriptionPlan, UserSubscription, Invoice } from '../../types/billing';
import { BillingService } from '../../services/billing';

interface SubscriptionManagerProps {
  className?: string;
  userId?: string;
}

const SubscriptionManager: React.FC<SubscriptionManagerProps> = ({ 
  className = '', 
  userId = 'user_123' 
}) => {
  const [plans, setPlans] = useState<SubscriptionPlan[]>([]);
  const [currentSubscription, setCurrentSubscription] = useState<UserSubscription | null>(null);
  // const [usageCredits] = useState<UsageCredit[]>([]);
  const [recentInvoices, setRecentInvoices] = useState<Invoice[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isUpgrading, setIsUpgrading] = useState(false);
  const [selectedPlan, setSelectedPlan] = useState<string>('');

  const billingService = BillingService.getInstance();

  useEffect(() => {
    loadSubscriptionData();
  }, [userId]);

  const loadSubscriptionData = async () => {
    try {
      const [plansData, subscription, , invoices] = await Promise.all([
        billingService.getSubscriptionPlans(),
        billingService.getUserSubscription(userId),
        billingService.getUsageCredits(userId),
        billingService.getInvoices(userId)
      ]);

      setPlans(plansData);
      setCurrentSubscription(subscription);
      // setUsageCredits(credits); - not used currently
      setRecentInvoices(invoices.slice(0, 3)); // Show last 3 invoices
    } catch (error) {
      console.error('Failed to load subscription data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleUpgrade = async (planId: string) => {
    if (!planId || isUpgrading) return;

    setIsUpgrading(true);
    try {
      const newSubscription = await billingService.upgradeSubscription(userId, planId);
      setCurrentSubscription(newSubscription);
      alert('Successfully upgraded subscription!');
    } catch (error) {
      console.error('Upgrade failed:', error);
      alert('Upgrade failed. Please try again.');
    } finally {
      setIsUpgrading(false);
      setSelectedPlan('');
    }
  };

  const handleCancelSubscription = async () => {
    if (!currentSubscription || currentSubscription.plan.tier === 'community') return;

    const confirmCancel = window.confirm(
      'Are you sure you want to cancel your subscription? It will remain active until the end of your billing period.'
    );

    if (!confirmCancel) return;

    try {
      const cancelledSubscription = await billingService.cancelSubscription(userId, true);
      setCurrentSubscription(cancelledSubscription);
      alert('Subscription cancelled. It will remain active until the end of your billing period.');
    } catch (error) {
      console.error('Cancellation failed:', error);
      alert('Cancellation failed. Please try again.');
    }
  };

  const getTierColor = (tier: string): string => {
    switch (tier) {
      case 'community': return 'text-gray-400 bg-gray-400/10';
      case 'pro': return 'text-sis-blue-400 bg-sis-blue-400/10';
      case 'enterprise': return 'text-purple-400 bg-purple-400/10';
      default: return 'text-sis-gray-400 bg-sis-gray-400/10';
    }
  };

  const formatBillingPeriod = (period: string): string => {
    return period === 'annual' ? 'per year' : 'per month';
  };

  const PlanCard: React.FC<{ plan: SubscriptionPlan; isCurrent?: boolean }> = ({ plan, isCurrent = false }) => (
    <div className={`card p-6 relative ${plan.popular ? 'ring-2 ring-sis-blue-500' : ''} ${isCurrent ? 'border-green-500/50' : ''}`}>
      {plan.popular && (
        <div className="absolute -top-3 left-1/2 transform -translate-x-1/2">
          <div className="bg-sis-blue-500 text-white px-3 py-1 rounded-full text-xs font-medium">
            Most Popular
          </div>
        </div>
      )}
      
      {isCurrent && (
        <div className="absolute -top-3 right-4">
          <div className="bg-green-500 text-white px-3 py-1 rounded-full text-xs font-medium">
            Current Plan
          </div>
        </div>
      )}

      <div className="space-y-4">
        <div>
          <div className={`inline-flex px-3 py-1 rounded-full text-xs font-medium ${getTierColor(plan.tier)}`}>
            {plan.tier.toUpperCase()}
          </div>
          <h3 className="text-xl font-bold text-white mt-2">{plan.name}</h3>
          <div className="flex items-baseline mt-2">
            <span className="text-3xl font-bold text-white">
              {plan.price_usd === 0 ? 'Free' : `$${plan.price_usd}`}
            </span>
            {plan.price_usd > 0 && (
              <span className="text-sm text-sis-gray-400 ml-2">
                {formatBillingPeriod(plan.billing_period)}
              </span>
            )}
          </div>
          {plan.billing_period === 'annual' && plan.tier === 'pro' && (
            <div className="text-sm text-green-400 mt-1">Save 2 months!</div>
          )}
        </div>

        <div className="space-y-3">
          <div className="flex items-center justify-between text-sm">
            <span className="text-sis-gray-300">Private Projects</span>
            <span className="text-white font-medium">
              {plan.features.max_private_projects === null ? 'Unlimited' : plan.features.max_private_projects || 'None'}
            </span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-sis-gray-300">Synthesis Credits/Month</span>
            <span className="text-white font-medium">
              {plan.features.synthesis_credits_per_month === null ? 'Unlimited' : plan.features.synthesis_credits_per_month?.toLocaleString()}
            </span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-sis-gray-300">Collaborators</span>
            <span className="text-white font-medium">
              {plan.features.max_collaborators === null ? 'Unlimited' : plan.features.max_collaborators}
            </span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-sis-gray-300">Cloud FPGA Access</span>
            <span className={`font-medium ${plan.features.cloud_fpga_access ? 'text-green-400' : 'text-red-400'}`}>
              {plan.features.cloud_fpga_access ? '✓ Yes' : '✗ No'}
            </span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-sis-gray-300">Priority Support</span>
            <span className={`font-medium ${plan.features.priority_support ? 'text-green-400' : 'text-red-400'}`}>
              {plan.features.priority_support ? '✓ Yes' : '✗ No'}
            </span>
          </div>
          {plan.tier === 'enterprise' && (
            <>
              <div className="flex items-center justify-between text-sm">
                <span className="text-sis-gray-300">On-premise Deployment</span>
                <span className="text-green-400 font-medium">✓ Yes</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-sis-gray-300">SLA Guarantee</span>
                <span className="text-green-400 font-medium">✓ 99.9%</span>
              </div>
            </>
          )}
        </div>

        <div className="pt-4 border-t border-sis-gray-700">
          {isCurrent ? (
            <div className="space-y-2">
              <button
                disabled
                className="w-full btn-secondary opacity-50 cursor-not-allowed"
              >
                Current Plan
              </button>
              {plan.tier !== 'community' && (
                <button
                  onClick={handleCancelSubscription}
                  className="w-full text-sm text-red-400 hover:text-red-300"
                >
                  Cancel Subscription
                </button>
              )}
            </div>
          ) : plan.tier === 'enterprise' ? (
            <button className="w-full btn-secondary">
              Contact Sales
            </button>
          ) : (
            <button
              onClick={() => handleUpgrade(plan.id)}
              disabled={isUpgrading}
              className="w-full btn-primary"
            >
              {isUpgrading && selectedPlan === plan.id ? 'Upgrading...' : 
               plan.tier === 'community' ? 'Downgrade' : 'Upgrade'}
            </button>
          )}
        </div>
      </div>
    </div>
  );

  if (isLoading) {
    return (
      <div className={`${className}`}>
        <div className="text-center py-8">
          <div className="animate-spin w-8 h-8 border-2 border-sis-blue-500 border-t-transparent rounded-full mx-auto mb-4"></div>
          <p className="text-sis-gray-400">Loading subscription details...</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Current Subscription Overview */}
      {currentSubscription && (
        <div className="card p-6">
          <h2 className="text-xl font-bold text-white mb-4">Current Subscription</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div>
              <div className="text-sm text-sis-gray-400">Plan</div>
              <div className="text-lg font-medium text-white">{currentSubscription.plan.name}</div>
              <div className="text-sm text-sis-gray-500 mt-1">
                {billingService.formatPrice(currentSubscription.plan.price_usd)} {formatBillingPeriod(currentSubscription.plan.billing_period)}
              </div>
            </div>
            <div>
              <div className="text-sm text-sis-gray-400">Next Billing</div>
              <div className="text-lg font-medium text-white">
                {currentSubscription.current_period_end.toLocaleDateString()}
              </div>
              <div className="text-sm text-sis-gray-500 mt-1">
                {Math.ceil((currentSubscription.current_period_end.getTime() - Date.now()) / (1000 * 60 * 60 * 24))} days remaining
              </div>
            </div>
            <div>
              <div className="text-sm text-sis-gray-400">Status</div>
              <div className={`text-lg font-medium ${
                currentSubscription.status === 'active' ? 'text-green-400' : 
                currentSubscription.cancel_at_period_end ? 'text-yellow-400' : 'text-red-400'
              }`}>
                {currentSubscription.cancel_at_period_end ? 'Cancelling' : currentSubscription.status}
              </div>
              {currentSubscription.cancel_at_period_end && (
                <div className="text-sm text-yellow-400 mt-1">Ends {currentSubscription.current_period_end.toLocaleDateString()}</div>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Usage Summary */}
      {currentSubscription && (
        <div className="card p-6">
          <h3 className="text-lg font-medium text-white mb-4">Usage This Period</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-sis-gray-400">Synthesis Credits</span>
                <span className="text-white">
                  {currentSubscription.usage.synthesis_credits_used} / {currentSubscription.plan.features.synthesis_credits_per_month || '∞'}
                </span>
              </div>
              <div className="w-full bg-sis-gray-700 rounded-full h-2">
                <div 
                  className="bg-sis-blue-500 h-2 rounded-full"
                  style={{ 
                    width: `${Math.min(100, (currentSubscription.usage.synthesis_credits_used / (currentSubscription.plan.features.synthesis_credits_per_month || 1000)) * 100)}%` 
                  }}
                />
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-sis-gray-400">Private Projects</span>
                <span className="text-white">
                  {currentSubscription.usage.private_projects_count} / {currentSubscription.plan.features.max_private_projects || '∞'}
                </span>
              </div>
              <div className="w-full bg-sis-gray-700 rounded-full h-2">
                <div 
                  className="bg-green-500 h-2 rounded-full"
                  style={{ 
                    width: `${Math.min(100, (currentSubscription.usage.private_projects_count / (currentSubscription.plan.features.max_private_projects || 100)) * 100)}%` 
                  }}
                />
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-sis-gray-400">Collaborators</span>
                <span className="text-white">
                  {currentSubscription.usage.collaborators_count} / {currentSubscription.plan.features.max_collaborators || '∞'}
                </span>
              </div>
              <div className="w-full bg-sis-gray-700 rounded-full h-2">
                <div 
                  className="bg-purple-500 h-2 rounded-full"
                  style={{ 
                    width: `${Math.min(100, (currentSubscription.usage.collaborators_count / (currentSubscription.plan.features.max_collaborators || 10)) * 100)}%` 
                  }}
                />
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-sis-gray-400">Storage</span>
                <span className="text-white">{currentSubscription.usage.storage_gb_used.toFixed(1)} GB</span>
              </div>
              <div className="w-full bg-sis-gray-700 rounded-full h-2">
                <div 
                  className="bg-yellow-500 h-2 rounded-full"
                  style={{ width: `${Math.min(100, (currentSubscription.usage.storage_gb_used / 10) * 100)}%` }}
                />
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Available Plans */}
      <div>
        <h2 className="text-xl font-bold text-white mb-6">Subscription Plans</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {plans.map(plan => (
            <PlanCard
              key={plan.id}
              plan={plan}
              isCurrent={currentSubscription?.plan_id === plan.id}
            />
          ))}
        </div>
      </div>

      {/* Recent Invoices */}
      {recentInvoices.length > 0 && (
        <div className="card p-6">
          <h3 className="text-lg font-medium text-white mb-4">Recent Invoices</h3>
          <div className="space-y-3">
            {recentInvoices.map(invoice => (
              <div key={invoice.id} className="flex items-center justify-between p-3 bg-sis-gray-800 rounded-lg">
                <div>
                  <div className="text-sm font-medium text-white">
                    {invoice.created_at.toLocaleDateString()}
                  </div>
                  <div className="text-xs text-sis-gray-400">
                    {invoice.items.map(item => item.description).join(', ')}
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-sm font-medium text-white">
                    {billingService.formatPrice(invoice.total_amount_usd)}
                  </div>
                  <div className={`text-xs ${
                    invoice.status === 'paid' ? 'text-green-400' : 
                    invoice.status === 'pending' ? 'text-yellow-400' : 'text-red-400'
                  }`}>
                    {invoice.status.toUpperCase()}
                  </div>
                </div>
                {invoice.pdf_url && (
                  <button className="text-xs btn-secondary px-2 py-1 ml-3">
                    Download PDF
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default SubscriptionManager;