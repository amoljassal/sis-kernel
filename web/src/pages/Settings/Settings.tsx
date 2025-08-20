import React, { useState, useEffect } from 'react';
import SubscriptionManager from '../../components/Business/SubscriptionManager';
import { UserProfile, PaymentMethod, OnboardingProgress } from '../../types/billing';
import { BillingService } from '../../services/billing';

type SettingsTab = 'profile' | 'subscription' | 'billing' | 'preferences' | 'security';

const Settings: React.FC = () => {
  const [activeTab, setActiveTab] = useState<SettingsTab>('profile');
  const [userProfile, setUserProfile] = useState<UserProfile | null>(null);
  const [paymentMethods, setPaymentMethods] = useState<PaymentMethod[]>([]);
  const [onboardingProgress, setOnboardingProgress] = useState<OnboardingProgress | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const billingService = BillingService.getInstance();

  useEffect(() => {
    loadUserData();
  }, []);

  const loadUserData = async () => {
    try {
      const [profile, payments, progress] = await Promise.all([
        billingService.getUserProfile('user_123'),
        billingService.getPaymentMethods('user_123'),
        billingService.getOnboardingProgress('user_123')
      ]);

      setUserProfile(profile);
      setPaymentMethods(payments);
      setOnboardingProgress(progress);
    } catch (error) {
      console.error('Failed to load user data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const ProfileSettings = () => (
    <div className="space-y-6">
      {userProfile && (
        <>
          <div className="card p-6">
            <h3 className="text-lg font-medium text-white mb-4">Profile Information</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label className="label">Display Name</label>
                <input
                  type="text"
                  defaultValue={userProfile.display_name}
                  className="input"
                />
              </div>
              <div>
                <label className="label">Username</label>
                <input
                  type="text"
                  defaultValue={userProfile.username}
                  className="input"
                />
              </div>
              <div>
                <label className="label">Email</label>
                <input
                  type="email"
                  defaultValue={userProfile.email}
                  className="input"
                />
              </div>
              <div>
                <label className="label">Company</label>
                <input
                  type="text"
                  defaultValue={userProfile.company || ''}
                  className="input"
                />
              </div>
              <div>
                <label className="label">Location</label>
                <input
                  type="text"
                  defaultValue={userProfile.location || ''}
                  className="input"
                />
              </div>
              <div>
                <label className="label">Website</label>
                <input
                  type="url"
                  defaultValue={userProfile.website || ''}
                  className="input"
                />
              </div>
            </div>
            <div className="mt-4">
              <label className="label">Bio</label>
              <textarea
                defaultValue={userProfile.bio || ''}
                rows={3}
                className="input resize-none"
              />
            </div>
            <div className="flex justify-end mt-6">
              <button className="btn-primary px-6 py-2">
                Save Changes
              </button>
            </div>
          </div>

          <div className="card p-6">
            <h3 className="text-lg font-medium text-white mb-4">Reputation & Badges</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div className="text-center">
                <div className="text-3xl font-bold text-sis-blue-400">{userProfile.reputation_score}</div>
                <div className="text-sm text-sis-gray-400">Reputation Score</div>
              </div>
              <div className="text-center">
                <div className="text-3xl font-bold text-green-400">${userProfile.total_earned_usd.toFixed(2)}</div>
                <div className="text-sm text-sis-gray-400">Total Earned</div>
              </div>
              <div className="text-center">
                <div className="text-3xl font-bold text-purple-400">{userProfile.badges.length}</div>
                <div className="text-sm text-sis-gray-400">Badges Earned</div>
              </div>
            </div>
            <div className="mt-4">
              <div className="text-sm text-sis-gray-400 mb-2">Badges</div>
              <div className="flex flex-wrap gap-2">
                {userProfile.badges.map(badge => (
                  <span key={badge} className="px-3 py-1 bg-sis-blue-500/20 text-sis-blue-400 rounded-full text-xs">
                    {badge.replace('_', ' ').toUpperCase()}
                  </span>
                ))}
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );

  const BillingSettings = () => (
    <div className="space-y-6">
      <div className="card p-6">
        <h3 className="text-lg font-medium text-white mb-4">Payment Methods</h3>
        {paymentMethods.length > 0 ? (
          <div className="space-y-3">
            {paymentMethods.map(method => (
              <div key={method.id} className="flex items-center justify-between p-4 bg-sis-gray-800 rounded-lg">
                <div className="flex items-center space-x-4">
                  <div className="text-2xl">
                    {method.type === 'card' ? '💳' : method.type === 'bank_account' ? '🏦' : '💰'}
                  </div>
                  <div>
                    <div className="text-white font-medium">
                      {method.brand?.toUpperCase()} ending in {method.last_four}
                    </div>
                    <div className="text-sm text-sis-gray-400">
                      {method.type === 'card' && method.expiry_month && method.expiry_year && (
                        `Expires ${method.expiry_month}/${method.expiry_year}`
                      )}
                      {method.is_default && <span className="text-green-400 ml-2">• Default</span>}
                    </div>
                  </div>
                </div>
                <div className="flex space-x-2">
                  {!method.is_default && (
                    <button className="btn-secondary text-xs px-3 py-1">
                      Make Default
                    </button>
                  )}
                  <button className="text-red-400 hover:text-red-300 text-xs px-3 py-1">
                    Remove
                  </button>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-center py-8 text-sis-gray-400">
            No payment methods configured
          </div>
        )}
        <div className="flex justify-end mt-4">
          <button className="btn-primary px-4 py-2">
            Add Payment Method
          </button>
        </div>
      </div>

      {onboardingProgress && (
        <div className="card p-6">
          <h3 className="text-lg font-medium text-white mb-4">Onboarding Progress</h3>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sis-gray-300">Current Skill Level</span>
              <span className="px-3 py-1 bg-sis-blue-500/20 text-sis-blue-400 rounded-full text-xs uppercase">
                {onboardingProgress.skill_level}
              </span>
            </div>
            <div className="space-y-2">
              <div className="text-sm text-sis-gray-400">Tutorial Progress</div>
              {Object.entries(onboardingProgress.tutorial_progress).map(([key, completed]) => (
                <div key={key} className="flex items-center justify-between">
                  <span className="text-sm text-sis-gray-300 capitalize">
                    {key.replace('_', ' ')}
                  </span>
                  <span className={`text-sm ${completed ? 'text-green-400' : 'text-sis-gray-500'}`}>
                    {completed ? '✓ Complete' : '○ Pending'}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );

  const PreferencesSettings = () => (
    <div className="space-y-6">
      <div className="card p-6">
        <h3 className="text-lg font-medium text-white mb-4">Design Preferences</h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-white font-medium">Auto-save Interval</div>
              <div className="text-sm text-sis-gray-400">Automatically save designs every N seconds</div>
            </div>
            <select className="bg-sis-gray-800 border border-sis-gray-600 rounded-md px-3 py-2 text-white">
              <option value="30">30 seconds</option>
              <option value="60">1 minute</option>
              <option value="300">5 minutes</option>
              <option value="0">Disabled</option>
            </select>
          </div>
          <div className="flex items-center justify-between">
            <div>
              <div className="text-white font-medium">Default Safety Mode</div>
              <div className="text-sm text-sis-gray-400">Safety level for new projects</div>
            </div>
            <select className="bg-sis-gray-800 border border-sis-gray-600 rounded-md px-3 py-2 text-white">
              <option value="beginner">Beginner</option>
              <option value="advanced">Advanced</option>
              <option value="pro">Professional</option>
            </select>
          </div>
          <div className="flex items-center justify-between">
            <div>
              <div className="text-white font-medium">Grid Snap</div>
              <div className="text-sm text-sis-gray-400">Snap components to grid in designer</div>
            </div>
            <input type="checkbox" defaultChecked className="rounded" />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <div className="text-white font-medium">Real-time Validation</div>
              <div className="text-sm text-sis-gray-400">Validate design as you build</div>
            </div>
            <input type="checkbox" defaultChecked className="rounded" />
          </div>
        </div>
      </div>

      <div className="card p-6">
        <h3 className="text-lg font-medium text-white mb-4">Notifications</h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-white font-medium">Synthesis Complete</div>
              <div className="text-sm text-sis-gray-400">Notify when FPGA synthesis finishes</div>
            </div>
            <input type="checkbox" defaultChecked className="rounded" />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <div className="text-white font-medium">Collaboration Invites</div>
              <div className="text-sm text-sis-gray-400">Get notified of new collaboration requests</div>
            </div>
            <input type="checkbox" defaultChecked className="rounded" />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <div className="text-white font-medium">Marketplace Updates</div>
              <div className="text-sm text-sis-gray-400">New IP blocks matching your interests</div>
            </div>
            <input type="checkbox" className="rounded" />
          </div>
        </div>
      </div>
    </div>
  );

  const SecuritySettings = () => (
    <div className="space-y-6">
      <div className="card p-6">
        <h3 className="text-lg font-medium text-white mb-4">Account Security</h3>
        <div className="space-y-4">
          <div>
            <label className="label">Current Password</label>
            <input type="password" className="input" placeholder="Enter current password" />
          </div>
          <div>
            <label className="label">New Password</label>
            <input type="password" className="input" placeholder="Enter new password" />
          </div>
          <div>
            <label className="label">Confirm New Password</label>
            <input type="password" className="input" placeholder="Confirm new password" />
          </div>
          <div className="flex justify-end">
            <button className="btn-primary px-6 py-2">
              Update Password
            </button>
          </div>
        </div>
      </div>

      <div className="card p-6">
        <h3 className="text-lg font-medium text-white mb-4">Two-Factor Authentication</h3>
        <div className="flex items-center justify-between p-4 bg-sis-gray-800 rounded-lg">
          <div>
            <div className="text-white font-medium">Authenticator App</div>
            <div className="text-sm text-sis-gray-400">Use an authenticator app for additional security</div>
          </div>
          <button className="btn-primary px-4 py-2">
            Enable 2FA
          </button>
        </div>
      </div>

      <div className="card p-6">
        <h3 className="text-lg font-medium text-white mb-4">API Access</h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-white font-medium">API Key</div>
              <div className="text-sm text-sis-gray-400">For programmatic access to SIS services</div>
            </div>
            <div className="flex space-x-2">
              <code className="bg-sis-gray-800 px-3 py-1 rounded text-xs">sis_key_***************abc123</code>
              <button className="btn-secondary text-xs px-3 py-1">
                Regenerate
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin w-8 h-8 border-2 border-sis-blue-500 border-t-transparent rounded-full mx-auto mb-4"></div>
          <p className="text-sis-gray-400">Loading settings...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-6 border-b border-sis-gray-700">
        <div>
          <h1 className="text-2xl font-bold text-white">Settings</h1>
          <p className="text-sis-gray-400">Manage your account and preferences</p>
        </div>
        <div className="flex items-center space-x-2">
          <div className="w-2 h-2 bg-green-400 rounded-full"></div>
          <span className="text-sm text-sis-gray-400">Account Active</span>
        </div>
      </div>

      <div className="flex-1 flex">
        {/* Sidebar */}
        <div className="w-64 border-r border-sis-gray-700 p-6">
          <nav className="space-y-2">
            {[
              { key: 'profile', label: 'Profile', icon: '👤' },
              { key: 'subscription', label: 'Subscription', icon: '💳' },
              { key: 'billing', label: 'Billing', icon: '🧾' },
              { key: 'preferences', label: 'Preferences', icon: '⚙️' },
              { key: 'security', label: 'Security', icon: '🔒' }
            ].map(tab => (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key as SettingsTab)}
                className={`w-full flex items-center space-x-3 px-3 py-2 text-sm font-medium rounded-lg transition-colors ${
                  activeTab === tab.key
                    ? 'bg-sis-blue-600 text-white'
                    : 'text-sis-gray-300 hover:text-white hover:bg-sis-gray-800'
                }`}
              >
                <span>{tab.icon}</span>
                <span>{tab.label}</span>
              </button>
            ))}
          </nav>
        </div>

        {/* Content */}
        <div className="flex-1 p-6 overflow-auto">
          {activeTab === 'profile' && <ProfileSettings />}
          {activeTab === 'subscription' && <SubscriptionManager />}
          {activeTab === 'billing' && <BillingSettings />}
          {activeTab === 'preferences' && <PreferencesSettings />}
          {activeTab === 'security' && <SecuritySettings />}
        </div>
      </div>
    </div>
  );
}

export default Settings