/**
 * Indian Market Dashboard
 * Phase 5A: India-focused dashboard integrating all regional features
 */

import React, { useState, useEffect } from 'react';
import { HindiLocalizationService } from '../../localization/hindi-support';
import IndianPaymentService from '../../services/payments-india';
import { INDIAN_MARKET_CONFIG } from '../../config/infrastructure';

interface IndianDashboardProps {
  userId: string;
  userProfile: {
    name: string;
    type: 'student' | 'faculty' | 'professional' | 'enterprise';
    institution?: string;
    location: {
      city: string;
      state: string;
      pincode: string;
    };
    language: 'en-IN' | 'hi-IN';
    subscription: {
      tier: 'free' | 'student' | 'pro' | 'enterprise';
      expiresAt?: Date;
    };
  };
}

interface RegionalStats {
  localUsers: number;
  nearbyInstitutions: number;
  regionalProjects: number;
  cityRanking: number;
}

interface EducationalProgress {
  currentSemester: number;
  completedModules: number;
  totalModules: number;
  upcomingExams: Array<{
    name: string;
    date: Date;
    type: 'internal' | 'external' | 'competitive';
  }>;
  certificationProgress: Array<{
    name: string;
    progress: number;
    deadline: Date;
  }>;
}

const IndianMarketDashboard: React.FC<IndianDashboardProps> = ({
  userProfile
}) => {
  const [localization] = useState(new HindiLocalizationService());
  const [paymentService] = useState(new IndianPaymentService());
  
  const [regionalStats, setRegionalStats] = useState<RegionalStats | null>(null);
  const [educationalProgress, setEducationalProgress] = useState<EducationalProgress | null>(null);
  const [currentDateTime, setCurrentDateTime] = useState(new Date());
  const [gateCountdown, setGateCountdown] = useState<string>('');

  // Set user's language preference
  useEffect(() => {
    localization.setLanguage(userProfile.language);
  }, [userProfile.language, localization]);

  // Update time every minute for IST display
  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentDateTime(new Date());
    }, 60000);

    return () => clearInterval(timer);
  }, []);

  // Load regional statistics
  useEffect(() => {
    loadRegionalData();
  }, [userProfile.location]);

  // Load educational progress for students
  useEffect(() => {
    if (userProfile.type === 'student') {
      loadEducationalProgress();
    }
  }, [userProfile.type]);

  // Calculate GATE countdown
  useEffect(() => {
    calculateGATECountdown();
  }, [currentDateTime]);

  const loadRegionalData = async () => {
    // Mock regional data - in production, fetch from analytics API
    setRegionalStats({
      localUsers: Math.floor(Math.random() * 1000) + 100,
      nearbyInstitutions: Math.floor(Math.random() * 50) + 5,
      regionalProjects: Math.floor(Math.random() * 500) + 50,
      cityRanking: Math.floor(Math.random() * 20) + 1
    });
  };

  const loadEducationalProgress = async () => {
    // Mock educational progress - in production, fetch from education service
    const nextGATE = new Date();
    nextGATE.setMonth(1); // February
    nextGATE.setFullYear(nextGATE.getFullYear() + (nextGATE.getMonth() > 1 ? 1 : 0));

    setEducationalProgress({
      currentSemester: 5,
      completedModules: 8,
      totalModules: 12,
      upcomingExams: [
        {
          name: 'Mid Semester - Digital Electronics',
          date: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000),
          type: 'internal'
        },
        {
          name: 'GATE 2026',
          date: nextGATE,
          type: 'competitive'
        }
      ],
      certificationProgress: [
        {
          name: 'SIS Certified Associate',
          progress: 65,
          deadline: new Date(Date.now() + 45 * 24 * 60 * 60 * 1000)
        }
      ]
    });
  };

  const calculateGATECountdown = () => {
    const nextGATE = new Date();
    nextGATE.setMonth(1, 15); // February 15th
    nextGATE.setFullYear(nextGATE.getFullYear() + (nextGATE.getMonth() > 1 ? 1 : 0));
    
    const diffMs = nextGATE.getTime() - currentDateTime.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    
    if (diffDays > 0) {
      const text = localization.getText('exam.gate');
      setGateCountdown(`${text}: ${diffDays} days remaining`);
    }
  };

  const getRegionalPricing = () => {
    const regionalPricing = paymentService.getRegionalPricing(
      INDIAN_MARKET_CONFIG.pricing.tiers.pro.price,
      userProfile.location
    );
    return regionalPricing;
  };

  const getFestivalGreeting = () => {
    const today = new Date();
    const month = today.getMonth() + 1;
    const day = today.getDate();

    // Check for major Indian festivals
    if (month === 10 && day >= 15 && day <= 25) return 'Happy Diwali! 🪔';
    if (month === 10 && day >= 1 && day <= 10) return 'Happy Dussehra! 🏹';
    if (month === 3 && day >= 10 && day <= 15) return 'Happy Holi! 🎨';
    if (month === 8 && day === 15) return 'Happy Independence Day! 🇮🇳';
    if (month === 1 && day === 26) return 'Happy Republic Day! 🇮🇳';
    
    return '';
  };

  const regionalPricing = getRegionalPricing();

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      {/* Header with IST time and regional info */}
      <div className="mb-8 bg-white rounded-lg shadow-sm p-6">
        <div className="flex justify-between items-start">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">
              {localization.getText('nav.dashboard')}
            </h1>
            <p className="text-gray-600">
              {localization.getText('greeting.welcome')}, {userProfile.name}
            </p>
            {getFestivalGreeting() && (
              <p className="text-orange-600 font-semibold mt-1">
                {getFestivalGreeting()}
              </p>
            )}
          </div>
          
          <div className="text-right">
            <div className="text-2xl font-bold text-blue-600">
              {localization.formatTime(currentDateTime)}
            </div>
            <div className="text-sm text-gray-500">
              {localization.formatDate(currentDateTime, 'long')} IST
            </div>
            <div className="text-sm text-gray-500 mt-1">
              {userProfile.location.city}, {userProfile.location.state}
            </div>
          </div>
        </div>

        {/* Language toggle */}
        <div className="mt-4 flex items-center space-x-4">
          <button
            onClick={() => localization.setLanguage('en-IN')}
            className={`px-3 py-1 rounded ${
              userProfile.language === 'en-IN' 
                ? 'bg-blue-100 text-blue-700' 
                : 'bg-gray-100 text-gray-600'
            }`}
          >
            English
          </button>
          <button
            onClick={() => localization.setLanguage('hi-IN')}
            className={`px-3 py-1 rounded ${
              userProfile.language === 'hi-IN' 
                ? 'bg-blue-100 text-blue-700' 
                : 'bg-gray-100 text-gray-600'
            }`}
          >
            हिन्दी
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Main Content Area */}
        <div className="lg:col-span-2 space-y-6">
          
          {/* Regional Statistics */}
          {regionalStats && (
            <div className="bg-white rounded-lg shadow-sm p-6">
              <h2 className="text-xl font-bold text-gray-900 mb-4">
                Regional Activity ({userProfile.location.city})
              </h2>
              
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="text-center">
                  <div className="text-2xl font-bold text-blue-600">
                    {localization.formatNumber(regionalStats.localUsers)}
                  </div>
                  <div className="text-sm text-gray-600">Local Users</div>
                </div>
                
                <div className="text-center">
                  <div className="text-2xl font-bold text-green-600">
                    {regionalStats.nearbyInstitutions}
                  </div>
                  <div className="text-sm text-gray-600">Nearby Institutions</div>
                </div>
                
                <div className="text-center">
                  <div className="text-2xl font-bold text-purple-600">
                    {localization.formatNumber(regionalStats.regionalProjects)}
                  </div>
                  <div className="text-sm text-gray-600">Regional Projects</div>
                </div>
                
                <div className="text-center">
                  <div className="text-2xl font-bold text-orange-600">
                    #{regionalStats.cityRanking}
                  </div>
                  <div className="text-sm text-gray-600">City Ranking</div>
                </div>
              </div>
            </div>
          )}

          {/* Educational Progress (for students) */}
          {userProfile.type === 'student' && educationalProgress && (
            <div className="bg-white rounded-lg shadow-sm p-6">
              <h2 className="text-xl font-bold text-gray-900 mb-4">
                {localization.getText('edu.progress')}
              </h2>
              
              <div className="space-y-4">
                {/* Current Semester Progress */}
                <div>
                  <div className="flex justify-between items-center mb-2">
                    <span className="text-sm font-medium">
                      Semester {educationalProgress.currentSemester} Progress
                    </span>
                    <span className="text-sm text-gray-600">
                      {educationalProgress.completedModules}/{educationalProgress.totalModules} modules
                    </span>
                  </div>
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div 
                      className="bg-blue-600 h-2 rounded-full" 
                      style={{ 
                        width: `${(educationalProgress.completedModules / educationalProgress.totalModules) * 100}%` 
                      }}
                    />
                  </div>
                </div>

                {/* Upcoming Exams */}
                <div>
                  <h3 className="font-semibold text-gray-800 mb-2">Upcoming Exams</h3>
                  <div className="space-y-2">
                    {educationalProgress.upcomingExams.map((exam, index) => (
                      <div key={index} className="flex justify-between items-center p-3 bg-gray-50 rounded">
                        <div>
                          <div className="font-medium">{exam.name}</div>
                          <div className="text-sm text-gray-600">
                            {localization.formatDate(exam.date)}
                          </div>
                        </div>
                        <span className={`px-2 py-1 rounded text-xs font-medium ${
                          exam.type === 'competitive' ? 'bg-red-100 text-red-700' :
                          exam.type === 'external' ? 'bg-yellow-100 text-yellow-700' :
                          'bg-blue-100 text-blue-700'
                        }`}>
                          {exam.type}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>

                {/* GATE Countdown */}
                {gateCountdown && (
                  <div className="bg-red-50 border border-red-200 rounded p-4">
                    <div className="text-red-800 font-semibold">{gateCountdown}</div>
                    <div className="text-sm text-red-600 mt-1">
                      Start your preparation with our GATE-aligned modules!
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Recent Activity */}
          <div className="bg-white rounded-lg shadow-sm p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-4">Recent Activity</h2>
            
            <div className="space-y-3">
              <div className="flex items-center space-x-3 p-3 bg-gray-50 rounded">
                <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                <div className="flex-1">
                  <div className="font-medium">Completed Logic Gates Lab</div>
                  <div className="text-sm text-gray-600">2 hours ago</div>
                </div>
                <div className="text-green-600 font-semibold">Grade: A</div>
              </div>
              
              <div className="flex items-center space-x-3 p-3 bg-gray-50 rounded">
                <div className="w-2 h-2 bg-blue-500 rounded-full"></div>
                <div className="flex-1">
                  <div className="font-medium">Started Multiplexer Design Module</div>
                  <div className="text-sm text-gray-600">1 day ago</div>
                </div>
                <div className="text-blue-600">In Progress</div>
              </div>
              
              <div className="flex items-center space-x-3 p-3 bg-gray-50 rounded">
                <div className="w-2 h-2 bg-purple-500 rounded-full"></div>
                <div className="flex-1">
                  <div className="font-medium">Joined Study Group: "GATE 2026 Prep"</div>
                  <div className="text-sm text-gray-600">3 days ago</div>
                </div>
                <div className="text-purple-600">Community</div>
              </div>
            </div>
          </div>
        </div>

        {/* Sidebar */}
        <div className="space-y-6">
          
          {/* Subscription Status with Regional Pricing */}
          <div className="bg-white rounded-lg shadow-sm p-6">
            <h3 className="font-bold text-gray-900 mb-4">
              {localization.getText('billing.subscription')}
            </h3>
            
            <div className="space-y-3">
              <div className="flex justify-between items-center">
                <span>Current Plan:</span>
                <span className="font-semibold capitalize">
                  {userProfile.subscription.tier}
                </span>
              </div>
              
              {userProfile.subscription.tier === 'free' && (
                <>
                  <div className="border-t pt-3">
                    <div className="text-sm text-gray-600 mb-2">
                      Upgrade to Pro for your city:
                    </div>
                    <div className="space-y-2">
                      <div className="flex justify-between">
                        <span className="text-sm">Original Price:</span>
                        <span className="text-sm line-through text-gray-500">
                          {localization.formatCurrency(regionalPricing.originalPrice)}
                        </span>
                      </div>
                      {regionalPricing.discount > 0 && (
                        <div className="flex justify-between">
                          <span className="text-sm text-green-600">
                            {regionalPricing.discountReason}:
                          </span>
                          <span className="text-sm text-green-600">
                            -{regionalPricing.discount}%
                          </span>
                        </div>
                      )}
                      <div className="flex justify-between font-bold">
                        <span>Your Price:</span>
                        <span className="text-green-600">
                          {localization.formatCurrency(regionalPricing.finalPrice)}
                        </span>
                      </div>
                    </div>
                    
                    <button className="w-full mt-3 bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700 transition-colors">
                      {localization.getText('action.upgrade')} to Pro
                    </button>
                  </div>
                </>
              )}

              {userProfile.subscription.expiresAt && (
                <div className="text-sm text-gray-600">
                  Expires: {localization.formatDate(userProfile.subscription.expiresAt)}
                </div>
              )}
            </div>
          </div>

          {/* Quick Actions */}
          <div className="bg-white rounded-lg shadow-sm p-6">
            <h3 className="font-bold text-gray-900 mb-4">Quick Actions</h3>
            
            <div className="space-y-2">
              <button className="w-full text-left p-3 bg-blue-50 hover:bg-blue-100 rounded transition-colors">
                <div className="font-medium text-blue-800">
                  {localization.getText('action.create')} New Project
                </div>
                <div className="text-sm text-blue-600">Start designing circuits</div>
              </button>
              
              <button className="w-full text-left p-3 bg-green-50 hover:bg-green-100 rounded transition-colors">
                <div className="font-medium text-green-800">
                  {localization.getText('exam.gate')}
                </div>
                <div className="text-sm text-green-600">Practice questions & mock tests</div>
              </button>
              
              <button className="w-full text-left p-3 bg-purple-50 hover:bg-purple-100 rounded transition-colors">
                <div className="font-medium text-purple-800">
                  {localization.getText('nav.marketplace')}
                </div>
                <div className="text-sm text-purple-600">Browse components & templates</div>
              </button>
            </div>
          </div>

          {/* Regional Compliance Status */}
          <div className="bg-white rounded-lg shadow-sm p-6">
            <h3 className="font-bold text-gray-900 mb-4">Data & Compliance</h3>
            
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm">Data Location:</span>
                <span className="text-sm font-medium text-green-600">🇮🇳 India</span>
              </div>
              
              <div className="flex items-center justify-between">
                <span className="text-sm">PDPB Compliant:</span>
                <span className="text-sm font-medium text-green-600">✅ Yes</span>
              </div>
              
              <div className="flex items-center justify-between">
                <span className="text-sm">GST Registration:</span>
                <span className="text-sm font-medium text-green-600">✅ Active</span>
              </div>
              
              <button className="w-full mt-3 text-blue-600 text-sm hover:underline">
                View Privacy Settings
              </button>
            </div>
          </div>

          {/* Support & Help */}
          <div className="bg-white rounded-lg shadow-sm p-6">
            <h3 className="font-bold text-gray-900 mb-4">Support & Help</h3>
            
            <div className="space-y-2">
              <button className="w-full text-left text-sm p-2 hover:bg-gray-50 rounded">
                📚 Help Center (English/Hindi)
              </button>
              <button className="w-full text-left text-sm p-2 hover:bg-gray-50 rounded">
                💬 Community Forum
              </button>
              <button className="w-full text-left text-sm p-2 hover:bg-gray-50 rounded">
                📞 Support (IST Business Hours)
              </button>
              <button className="w-full text-left text-sm p-2 hover:bg-gray-50 rounded">
                🎓 Faculty Training Program
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default IndianMarketDashboard;