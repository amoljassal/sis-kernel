/**
 * Partnership Onboarding Dashboard
 * Phase 5B: Management interface for IIT/NIT partnerships
 */

import React, { useState, useEffect } from 'react';
import { 
  AcademicCapIcon, 
  BuildingLibraryIcon,
  ChartBarIcon,
  CheckCircleIcon,
  ClockIcon,
  CurrencyRupeeIcon,
  DocumentTextIcon,
  UserGroupIcon,
  MapPinIcon,
  ArrowTrendingUpIcon as TrendingUpIcon
} from '@heroicons/react/24/outline';

interface PartnershipDashboardProps {
  userRole: 'admin' | 'partnership_manager' | 'technical_lead';
}

interface DashboardMetrics {
  applications: {
    total: number;
    pending: number;
    approved: number;
    rejected: number;
    byInstitutionType: Record<string, number>;
  };
  activePartnerships: {
    total: number;
    byRegion: Record<string, number>;
    byInstitutionType: Record<string, number>;
    totalStudents: number;
    totalFaculty: number;
  };
  revenue: {
    totalARR: number;
    averageContractValue: number;
    renewalRate: number;
    growthRate: number;
  };
  onboardingStatus: {
    inProgress: number;
    completedThisMonth: number;
    averageOnboardingTime: number;
  };
}

const PartnershipOnboardingDashboard: React.FC<PartnershipDashboardProps> = ({ userRole }) => {
  const [metrics, setMetrics] = useState<DashboardMetrics | null>(null);
  const [selectedTab, setSelectedTab] = useState<'overview' | 'applications' | 'active' | 'onboarding' | 'revenue'>('overview');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadDashboardMetrics();
  }, []);

  const loadDashboardMetrics = async () => {
    setLoading(true);
    try {
      // In production, this would call the actual API
      const mockMetrics: DashboardMetrics = {
        applications: {
          total: 47,
          pending: 12,
          approved: 28,
          rejected: 7,
          byInstitutionType: {
            'IIT': 15,
            'NIT': 23,
            'IIIT': 6,
            'Other': 3
          }
        },
        activePartnerships: {
          total: 23,
          byRegion: {
            'north': 8,
            'south': 7,
            'west': 5,
            'east': 2,
            'central': 1
          },
          byInstitutionType: {
            'IIT': 9,
            'NIT': 12,
            'IIIT': 2
          },
          totalStudents: 18750,
          totalFaculty: 645
        },
        revenue: {
          totalARR: 45000000, // ₹4.5 Crores
          averageContractValue: 1956521, // ~₹19.6 Lakhs
          renewalRate: 0.87,
          growthRate: 0.52
        },
        onboardingStatus: {
          inProgress: 8,
          completedThisMonth: 4,
          averageOnboardingTime: 42 // days
        }
      };

      // Simulate API delay
      setTimeout(() => {
        setMetrics(mockMetrics);
        setLoading(false);
      }, 1000);
    } catch (error) {
      console.error('Failed to load dashboard metrics:', error);
      setLoading(false);
    }
  };

  const formatIndianCurrency = (amount: number): string => {
    if (amount >= 10000000) { // 1 Crore
      return `₹${(amount / 10000000).toFixed(1)} Cr`;
    } else if (amount >= 100000) { // 1 Lakh
      return `₹${(amount / 100000).toFixed(1)} L`;
    } else {
      return `₹${amount.toLocaleString('en-IN')}`;
    }
  };

  const formatPercentage = (value: number): string => {
    return `${(value * 100).toFixed(1)}%`;
  };

  const getInstitutionTypeIcon = (type: string) => {
    switch (type) {
      case 'IIT': return '🏛️';
      case 'NIT': return '🏢';
      case 'IIIT': return '💻';
      default: return '🏫';
    }
  };

  const getRegionName = (region: string): string => {
    const regionNames: Record<string, string> = {
      'north': 'North India',
      'south': 'South India',
      'west': 'West India',
      'east': 'East India',
      'central': 'Central India',
      'northeast': 'Northeast India'
    };
    return regionNames[region] || region;
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="animate-pulse">
            <div className="h-8 bg-gray-300 rounded w-1/3 mb-6"></div>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
              {[1, 2, 3, 4].map(i => (
                <div key={i} className="bg-white rounded-xl shadow-sm p-6">
                  <div className="h-6 bg-gray-300 rounded w-1/2 mb-4"></div>
                  <div className="h-10 bg-gray-300 rounded w-3/4"></div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (!metrics) {
    return (
      <div className="min-h-screen bg-gray-50 p-6 flex items-center justify-center">
        <div className="text-center">
          <div className="text-red-600 text-xl mb-2">Failed to load dashboard</div>
          <button 
            onClick={loadDashboardMetrics}
            className="btn btn-primary"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">Partnership Dashboard</h1>
              <p className="text-gray-600 mt-1">
                Phase 5B: Educational Partnership Management & Growth Tracking
              </p>
            </div>
            <div className="flex items-center space-x-4">
              <div className="text-right">
                <div className="text-sm text-gray-500">Target: 15 Institutions</div>
                <div className="text-2xl font-bold text-green-600">
                  {metrics.activePartnerships.total}/15
                </div>
              </div>
              <div className="text-right">
                <div className="text-sm text-gray-500">Target: 25K Students</div>
                <div className="text-2xl font-bold text-blue-600">
                  {(metrics.activePartnerships.totalStudents / 1000).toFixed(1)}K
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Key Metrics Cards */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {/* Applications */}
          <div className="bg-white rounded-xl shadow-sm p-6 border-l-4 border-blue-500">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total Applications</p>
                <p className="text-3xl font-bold text-gray-900">{metrics.applications.total}</p>
                <div className="flex items-center mt-2">
                  <ClockIcon className="h-4 w-4 text-orange-500 mr-1" />
                  <span className="text-sm text-orange-600">{metrics.applications.pending} Pending</span>
                </div>
              </div>
              <DocumentTextIcon className="h-12 w-12 text-blue-500" />
            </div>
          </div>

          {/* Active Partnerships */}
          <div className="bg-white rounded-xl shadow-sm p-6 border-l-4 border-green-500">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Active Partnerships</p>
                <p className="text-3xl font-bold text-gray-900">{metrics.activePartnerships.total}</p>
                <div className="flex items-center mt-2">
                  <UserGroupIcon className="h-4 w-4 text-green-500 mr-1" />
                  <span className="text-sm text-green-600">
                    {(metrics.activePartnerships.totalStudents / 1000).toFixed(0)}K Students
                  </span>
                </div>
              </div>
              <BuildingLibraryIcon className="h-12 w-12 text-green-500" />
            </div>
          </div>

          {/* Revenue */}
          <div className="bg-white rounded-xl shadow-sm p-6 border-l-4 border-purple-500">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Total ARR</p>
                <p className="text-3xl font-bold text-gray-900">
                  {formatIndianCurrency(metrics.revenue.totalARR)}
                </p>
                <div className="flex items-center mt-2">
                  <TrendingUpIcon className="h-4 w-4 text-purple-500 mr-1" />
                  <span className="text-sm text-purple-600">
                    {formatPercentage(metrics.revenue.growthRate)} Growth
                  </span>
                </div>
              </div>
              <CurrencyRupeeIcon className="h-12 w-12 text-purple-500" />
            </div>
          </div>

          {/* Onboarding */}
          <div className="bg-white rounded-xl shadow-sm p-6 border-l-4 border-orange-500">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-600">Onboarding</p>
                <p className="text-3xl font-bold text-gray-900">{metrics.onboardingStatus.inProgress}</p>
                <div className="flex items-center mt-2">
                  <CheckCircleIcon className="h-4 w-4 text-orange-500 mr-1" />
                  <span className="text-sm text-orange-600">
                    {metrics.onboardingStatus.completedThisMonth} This Month
                  </span>
                </div>
              </div>
              <AcademicCapIcon className="h-12 w-12 text-orange-500" />
            </div>
          </div>
        </div>

        {/* Tab Navigation */}
        <div className="bg-white rounded-xl shadow-sm mb-6">
          <div className="border-b border-gray-200">
            <nav className="-mb-px flex space-x-8 px-6">
              {[
                { id: 'overview', name: 'Overview', icon: ChartBarIcon },
                { id: 'applications', name: 'Applications', icon: DocumentTextIcon },
                { id: 'active', name: 'Active Partnerships', icon: BuildingLibraryIcon },
                { id: 'onboarding', name: 'Onboarding', icon: AcademicCapIcon },
                { id: 'revenue', name: 'Revenue', icon: CurrencyRupeeIcon }
              ].map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setSelectedTab(tab.id as any)}
                  className={`${
                    selectedTab === tab.id
                      ? 'border-blue-500 text-blue-600'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  } whitespace-nowrap py-4 px-1 border-b-2 font-medium text-sm flex items-center`}
                >
                  <tab.icon className="h-4 w-4 mr-2" />
                  {tab.name}
                </button>
              ))}
            </nav>
          </div>

          {/* Tab Content */}
          <div className="p-6">
            {selectedTab === 'overview' && (
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* Institution Types Breakdown */}
                <div className="bg-gray-50 rounded-lg p-4">
                  <h3 className="text-lg font-semibold mb-4">Institution Types</h3>
                  <div className="space-y-3">
                    {Object.entries(metrics.activePartnerships.byInstitutionType).map(([type, count]) => (
                      <div key={type} className="flex items-center justify-between">
                        <div className="flex items-center">
                          <span className="text-2xl mr-3">{getInstitutionTypeIcon(type)}</span>
                          <span className="font-medium">{type}</span>
                        </div>
                        <div className="text-right">
                          <div className="font-bold text-lg">{count}</div>
                          <div className="text-sm text-gray-500">partnerships</div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Regional Distribution */}
                <div className="bg-gray-50 rounded-lg p-4">
                  <h3 className="text-lg font-semibold mb-4">Regional Distribution</h3>
                  <div className="space-y-3">
                    {Object.entries(metrics.activePartnerships.byRegion).map(([region, count]) => (
                      <div key={region} className="flex items-center justify-between">
                        <div className="flex items-center">
                          <MapPinIcon className="h-5 w-5 text-gray-500 mr-3" />
                          <span className="font-medium">{getRegionName(region)}</span>
                        </div>
                        <div className="text-right">
                          <div className="font-bold text-lg">{count}</div>
                          <div className="text-sm text-gray-500">institutions</div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Phase 5B Progress */}
                <div className="lg:col-span-2 bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg p-6">
                  <h3 className="text-lg font-semibold mb-4">Phase 5B Progress (Months 21-23)</h3>
                  <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                    <div className="text-center">
                      <div className="text-3xl font-bold text-blue-600">
                        {metrics.activePartnerships.total}/15
                      </div>
                      <div className="text-sm text-gray-600">Institutional Partnerships</div>
                      <div className="mt-2">
                        <div className="w-full bg-gray-200 rounded-full h-2">
                          <div 
                            className="bg-blue-600 h-2 rounded-full" 
                            style={{ width: `${(metrics.activePartnerships.total / 15) * 100}%` }}
                          ></div>
                        </div>
                      </div>
                    </div>
                    
                    <div className="text-center">
                      <div className="text-3xl font-bold text-green-600">
                        {(metrics.activePartnerships.totalStudents / 1000).toFixed(0)}K/25K
                      </div>
                      <div className="text-sm text-gray-600">Active Students</div>
                      <div className="mt-2">
                        <div className="w-full bg-gray-200 rounded-full h-2">
                          <div 
                            className="bg-green-600 h-2 rounded-full" 
                            style={{ width: `${(metrics.activePartnerships.totalStudents / 25000) * 100}%` }}
                          ></div>
                        </div>
                      </div>
                    </div>

                    <div className="text-center">
                      <div className="text-3xl font-bold text-purple-600">
                        {metrics.activePartnerships.totalFaculty}/200
                      </div>
                      <div className="text-sm text-gray-600">Trained Faculty</div>
                      <div className="mt-2">
                        <div className="w-full bg-gray-200 rounded-full h-2">
                          <div 
                            className="bg-purple-600 h-2 rounded-full" 
                            style={{ width: `${(metrics.activePartnerships.totalFaculty / 200) * 100}%` }}
                          ></div>
                        </div>
                      </div>
                    </div>

                    <div className="text-center">
                      <div className="text-3xl font-bold text-orange-600">
                        {formatIndianCurrency(metrics.revenue.totalARR)}
                      </div>
                      <div className="text-sm text-gray-600">ARR (Target: ₹200Cr)</div>
                      <div className="mt-2">
                        <div className="w-full bg-gray-200 rounded-full h-2">
                          <div 
                            className="bg-orange-600 h-2 rounded-full" 
                            style={{ width: `${(metrics.revenue.totalARR / 2000000000) * 100}%` }}
                          ></div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {selectedTab === 'applications' && (
              <div>
                <div className="flex items-center justify-between mb-6">
                  <h3 className="text-lg font-semibold">Partnership Applications</h3>
                  <button className="btn btn-primary">New Application</button>
                </div>
                
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
                  <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                    <div className="text-2xl font-bold text-yellow-800">{metrics.applications.pending}</div>
                    <div className="text-sm text-yellow-600">Pending Review</div>
                  </div>
                  <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                    <div className="text-2xl font-bold text-green-800">{metrics.applications.approved}</div>
                    <div className="text-sm text-green-600">Approved</div>
                  </div>
                  <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                    <div className="text-2xl font-bold text-red-800">{metrics.applications.rejected}</div>
                    <div className="text-sm text-red-600">Rejected</div>
                  </div>
                </div>

                <div className="bg-white border border-gray-200 rounded-lg">
                  <div className="px-4 py-3 border-b border-gray-200">
                    <h4 className="font-medium">Recent Applications</h4>
                  </div>
                  <div className="p-4">
                    <div className="text-gray-500 text-center py-8">
                      Application list would be implemented here with filtering, sorting, and pagination
                    </div>
                  </div>
                </div>
              </div>
            )}

            {selectedTab === 'revenue' && (
              <div className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                  <div className="bg-gradient-to-br from-green-50 to-green-100 rounded-lg p-6">
                    <div className="text-sm text-green-600 font-medium">Total ARR</div>
                    <div className="text-3xl font-bold text-green-800 mt-2">
                      {formatIndianCurrency(metrics.revenue.totalARR)}
                    </div>
                    <div className="text-sm text-green-600 mt-1">
                      Target: ₹200 Cr ({formatPercentage(metrics.revenue.totalARR / 2000000000)})
                    </div>
                  </div>

                  <div className="bg-gradient-to-br from-blue-50 to-blue-100 rounded-lg p-6">
                    <div className="text-sm text-blue-600 font-medium">Avg Contract Value</div>
                    <div className="text-3xl font-bold text-blue-800 mt-2">
                      {formatIndianCurrency(metrics.revenue.averageContractValue)}
                    </div>
                    <div className="text-sm text-blue-600 mt-1">Per partnership</div>
                  </div>

                  <div className="bg-gradient-to-br from-purple-50 to-purple-100 rounded-lg p-6">
                    <div className="text-sm text-purple-600 font-medium">Renewal Rate</div>
                    <div className="text-3xl font-bold text-purple-800 mt-2">
                      {formatPercentage(metrics.revenue.renewalRate)}
                    </div>
                    <div className="text-sm text-purple-600 mt-1">Partnership retention</div>
                  </div>
                </div>

                <div className="bg-white border border-gray-200 rounded-lg p-6">
                  <h4 className="font-medium mb-4">Revenue Breakdown by Institution Type</h4>
                  <div className="space-y-4">
                    {Object.entries(metrics.activePartnerships.byInstitutionType).map(([type, count]) => {
                      const revenue = count * metrics.revenue.averageContractValue;
                      return (
                        <div key={type} className="flex items-center justify-between py-2 border-b border-gray-100">
                          <div className="flex items-center">
                            <span className="text-xl mr-3">{getInstitutionTypeIcon(type)}</span>
                            <div>
                              <div className="font-medium">{type}</div>
                              <div className="text-sm text-gray-500">{count} partnerships</div>
                            </div>
                          </div>
                          <div className="text-right">
                            <div className="font-bold">{formatIndianCurrency(revenue)}</div>
                            <div className="text-sm text-gray-500">
                              {formatPercentage(revenue / metrics.revenue.totalARR)} of total
                            </div>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              </div>
            )}

            {/* Other tabs would be implemented similarly */}
            {selectedTab !== 'overview' && selectedTab !== 'applications' && selectedTab !== 'revenue' && (
              <div className="text-center py-12 text-gray-500">
                {selectedTab.charAt(0).toUpperCase() + selectedTab.slice(1)} content coming soon...
              </div>
            )}
          </div>
        </div>

        {/* Quick Actions */}
        {userRole === 'admin' || userRole === 'partnership_manager' ? (
          <div className="bg-white rounded-xl shadow-sm p-6">
            <h3 className="text-lg font-semibold mb-4">Quick Actions</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <button className="btn btn-primary flex items-center justify-center">
                <DocumentTextIcon className="h-4 w-4 mr-2" />
                New Partnership Application
              </button>
              <button className="btn btn-secondary flex items-center justify-center">
                <BuildingLibraryIcon className="h-4 w-4 mr-2" />
                Institution Directory
              </button>
              <button className="btn btn-secondary flex items-center justify-center">
                <ChartBarIcon className="h-4 w-4 mr-2" />
                Generate Report
              </button>
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
};

export default PartnershipOnboardingDashboard;