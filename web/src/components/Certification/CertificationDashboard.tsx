/**
 * Certification Dashboard
 * Phase 5B: Industry certification management and progress tracking
 */

import React, { useState, useEffect } from 'react';
import {
  AcademicCapIcon,
  ShieldCheckIcon,
  TrophyIcon,
  ChartBarIcon,
  ClockIcon,
  CurrencyRupeeIcon,
  DocumentTextIcon,
  UserGroupIcon,
  StarIcon,
  PlayIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon
} from '@heroicons/react/24/outline';

interface CertificationDashboardProps {
  userId: string;
  userRole: 'student' | 'professional' | 'admin';
}

interface CertificationTrack {
  id: string;
  name: string;
  nameHindi: string;
  level: 'Associate' | 'Professional' | 'Expert' | 'Master';
  industryPartners: string[];
  recognizedBy: string[];
  duration: number;
  pricing: {
    student: number;
    professional: number;
    currency: 'INR';
  };
  placementAssurance: {
    enabled: boolean;
    partnersCount: number;
    averageCTC: number;
  };
  enrolled?: boolean;
  progress?: number;
  status?: 'Not_Started' | 'In_Progress' | 'Assessment_Ready' | 'Completed' | 'Certified';
}

interface UserProgress {
  completedCertifications: number;
  inProgressCertifications: number;
  totalScore: number;
  averageScore: number;
  careerReadiness: number;
  placementPotential: {
    estimatedCTC: number;
    topCompanies: string[];
    readinessScore: number;
  };
}

const CertificationDashboard: React.FC<CertificationDashboardProps> = ({ userId }) => {
  const [tracks, setTracks] = useState<CertificationTrack[]>([]);
  const [userProgress, setUserProgress] = useState<UserProgress | null>(null);
  const [selectedTab, setSelectedTab] = useState<'explore' | 'enrolled' | 'completed' | 'analytics'>('explore');
  const [loading, setLoading] = useState(true);
  const [filters, setFilters] = useState({
    level: '',
    recognizedBy: '',
    maxPrice: 10000,
    placementAssurance: false
  });

  useEffect(() => {
    loadCertificationData();
  }, [userId]);

  const loadCertificationData = async () => {
    setLoading(true);
    try {
      // Mock data - in production, this would call actual APIs
      const mockTracks: CertificationTrack[] = [
        {
          id: 'sis_ai_native_systems_associate',
          name: 'SIS AI-Native Systems Associate',
          nameHindi: 'SIS एआई-देशी सिस्टम सहयोगी',
          level: 'Associate',
          industryPartners: ['TCS', 'Infosys', 'Wipro', 'L&T', 'DRDO'],
          recognizedBy: ['NASSCOM', 'IEEE'],
          duration: 8,
          pricing: {
            student: 2999,
            professional: 4999,
            currency: 'INR'
          },
          placementAssurance: {
            enabled: true,
            partnersCount: 15,
            averageCTC: 800000
          },
          enrolled: true,
          progress: 65,
          status: 'In_Progress'
        },
        {
          id: 'sis_embedded_ai_professional',
          name: 'SIS Embedded AI Professional',
          nameHindi: 'SIS एम्बेडेड एआई प्रोफेशनल',
          level: 'Professional',
          industryPartners: ['Qualcomm', 'Intel', 'ARM', 'Bosch', 'Continental'],
          recognizedBy: ['IEEE', 'IETE'],
          duration: 12,
          pricing: {
            student: 4999,
            professional: 7999,
            currency: 'INR'
          },
          placementAssurance: {
            enabled: true,
            partnersCount: 20,
            averageCTC: 1200000
          },
          enrolled: false,
          progress: 0,
          status: 'Not_Started'
        },
        {
          id: 'sis_vlsi_design_expert',
          name: 'SIS VLSI Design Expert',
          nameHindi: 'SIS वीएलएसआई डिज़ाइन विशेषज्ञ',
          level: 'Expert',
          industryPartners: ['TSMC', 'GlobalFoundries', 'Cadence', 'Synopsys', 'Mentor Graphics'],
          recognizedBy: ['IEEE', 'IETE', 'VLSI Society of India'],
          duration: 16,
          pricing: {
            student: 7999,
            professional: 12999,
            currency: 'INR'
          },
          placementAssurance: {
            enabled: true,
            partnersCount: 12,
            averageCTC: 1800000
          },
          enrolled: false,
          progress: 0,
          status: 'Not_Started'
        }
      ];

      const mockProgress: UserProgress = {
        completedCertifications: 2,
        inProgressCertifications: 1,
        totalScore: 175,
        averageScore: 87.5,
        careerReadiness: 78,
        placementPotential: {
          estimatedCTC: 900000,
          topCompanies: ['TCS', 'Infosys', 'L&T Technology Services', 'Wipro'],
          readinessScore: 85
        }
      };

      setTimeout(() => {
        setTracks(mockTracks);
        setUserProgress(mockProgress);
        setLoading(false);
      }, 1000);
    } catch (error) {
      console.error('Failed to load certification data:', error);
      setLoading(false);
    }
  };

  const formatIndianCurrency = (amount: number): string => {
    if (amount >= 10000000) {
      return `₹${(amount / 10000000).toFixed(1)} Cr`;
    } else if (amount >= 100000) {
      return `₹${(amount / 100000).toFixed(1)} L`;
    } else {
      return `₹${amount.toLocaleString('en-IN')}`;
    }
  };

  const getStatusColor = (status: string): string => {
    switch (status) {
      case 'Completed': return 'text-green-600 bg-green-100';
      case 'In_Progress': return 'text-blue-600 bg-blue-100';
      case 'Assessment_Ready': return 'text-orange-600 bg-orange-100';
      case 'Not_Started': return 'text-gray-600 bg-gray-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  const getLevelIcon = (level: string) => {
    switch (level) {
      case 'Associate': return '🥉';
      case 'Professional': return '🥈';
      case 'Expert': return '🥇';
      case 'Master': return '💎';
      default: return '📜';
    }
  };

  const filteredTracks = tracks.filter(track => {
    if (filters.level && track.level !== filters.level) return false;
    if (filters.recognizedBy && !track.recognizedBy.includes(filters.recognizedBy)) return false;
    if (track.pricing.student > filters.maxPrice) return false;
    if (filters.placementAssurance && !track.placementAssurance.enabled) return false;
    return true;
  });

  const enrolledTracks = tracks.filter(track => track.enrolled);
  const completedTracks = tracks.filter(track => track.status === 'Completed');

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

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">Industry Certifications</h1>
              <p className="text-gray-600 mt-1">
                NASSCOM, IEEE & IETE Recognized Professional Certifications
              </p>
            </div>
            <div className="flex items-center space-x-4">
              <div className="text-right">
                <div className="text-sm text-gray-500">Career Readiness</div>
                <div className="text-2xl font-bold text-green-600">
                  {userProgress?.careerReadiness || 0}%
                </div>
              </div>
              <div className="text-right">
                <div className="text-sm text-gray-500">Est. CTC</div>
                <div className="text-2xl font-bold text-blue-600">
                  {userProgress ? formatIndianCurrency(userProgress.placementPotential.estimatedCTC) : '₹0'}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Progress Overview Cards */}
        {userProgress && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <div className="bg-white rounded-xl shadow-sm p-6 border-l-4 border-green-500">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-600">Completed</p>
                  <p className="text-3xl font-bold text-gray-900">{userProgress.completedCertifications}</p>
                  <div className="flex items-center mt-2">
                    <CheckCircleIcon className="h-4 w-4 text-green-500 mr-1" />
                    <span className="text-sm text-green-600">Certified</span>
                  </div>
                </div>
                <TrophyIcon className="h-12 w-12 text-green-500" />
              </div>
            </div>

            <div className="bg-white rounded-xl shadow-sm p-6 border-l-4 border-blue-500">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-600">In Progress</p>
                  <p className="text-3xl font-bold text-gray-900">{userProgress.inProgressCertifications}</p>
                  <div className="flex items-center mt-2">
                    <ClockIcon className="h-4 w-4 text-blue-500 mr-1" />
                    <span className="text-sm text-blue-600">Active</span>
                  </div>
                </div>
                <AcademicCapIcon className="h-12 w-12 text-blue-500" />
              </div>
            </div>

            <div className="bg-white rounded-xl shadow-sm p-6 border-l-4 border-purple-500">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-600">Average Score</p>
                  <p className="text-3xl font-bold text-gray-900">{userProgress.averageScore.toFixed(1)}</p>
                  <div className="flex items-center mt-2">
                    <StarIcon className="h-4 w-4 text-purple-500 mr-1" />
                    <span className="text-sm text-purple-600">Performance</span>
                  </div>
                </div>
                <ChartBarIcon className="h-12 w-12 text-purple-500" />
              </div>
            </div>

            <div className="bg-white rounded-xl shadow-sm p-6 border-l-4 border-orange-500">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-gray-600">Placement Score</p>
                  <p className="text-3xl font-bold text-gray-900">{userProgress.placementPotential.readinessScore}</p>
                  <div className="flex items-center mt-2">
                    <ShieldCheckIcon className="h-4 w-4 text-orange-500 mr-1" />
                    <span className="text-sm text-orange-600">Industry Ready</span>
                  </div>
                </div>
                <UserGroupIcon className="h-12 w-12 text-orange-500" />
              </div>
            </div>
          </div>
        )}

        {/* Tab Navigation */}
        <div className="bg-white rounded-xl shadow-sm mb-6">
          <div className="border-b border-gray-200">
            <nav className="-mb-px flex space-x-8 px-6">
              {[
                { id: 'explore', name: 'Explore Tracks', count: filteredTracks.length },
                { id: 'enrolled', name: 'My Certifications', count: enrolledTracks.length },
                { id: 'completed', name: 'Completed', count: completedTracks.length },
                { id: 'analytics', name: 'Analytics', count: null }
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
                  {tab.name}
                  {tab.count !== null && (
                    <span className={`ml-2 px-2 py-1 text-xs rounded-full ${
                      selectedTab === tab.id ? 'bg-blue-100 text-blue-600' : 'bg-gray-100 text-gray-600'
                    }`}>
                      {tab.count}
                    </span>
                  )}
                </button>
              ))}
            </nav>
          </div>

          {/* Tab Content */}
          <div className="p-6">
            {selectedTab === 'explore' && (
              <div>
                {/* Filters */}
                <div className="mb-6 grid grid-cols-1 md:grid-cols-4 gap-4">
                  <select
                    value={filters.level}
                    onChange={(e) => setFilters({...filters, level: e.target.value})}
                    className="input"
                  >
                    <option value="">All Levels</option>
                    <option value="Associate">Associate</option>
                    <option value="Professional">Professional</option>
                    <option value="Expert">Expert</option>
                    <option value="Master">Master</option>
                  </select>

                  <select
                    value={filters.recognizedBy}
                    onChange={(e) => setFilters({...filters, recognizedBy: e.target.value})}
                    className="input"
                  >
                    <option value="">All Recognition</option>
                    <option value="NASSCOM">NASSCOM</option>
                    <option value="IEEE">IEEE</option>
                    <option value="IETE">IETE</option>
                    <option value="CSI">CSI</option>
                  </select>

                  <div>
                    <label className="label">Max Price: ₹{filters.maxPrice.toLocaleString('en-IN')}</label>
                    <input
                      type="range"
                      min="1000"
                      max="15000"
                      step="1000"
                      value={filters.maxPrice}
                      onChange={(e) => setFilters({...filters, maxPrice: parseInt(e.target.value)})}
                      className="w-full"
                    />
                  </div>

                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      checked={filters.placementAssurance}
                      onChange={(e) => setFilters({...filters, placementAssurance: e.target.checked})}
                      className="mr-2"
                    />
                    <span className="text-sm">Placement Assurance</span>
                  </label>
                </div>

                {/* Certification Tracks Grid */}
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                  {filteredTracks.map(track => (
                    <div key={track.id} className="bg-white border border-gray-200 rounded-lg p-6 hover:shadow-lg transition-shadow">
                      {/* Header */}
                      <div className="flex items-start justify-between mb-4">
                        <div className="flex items-center">
                          <span className="text-2xl mr-3">{getLevelIcon(track.level)}</span>
                          <div>
                            <h3 className="font-semibold text-lg text-gray-900">{track.name}</h3>
                            <p className="text-sm text-gray-500">{track.nameHindi}</p>
                          </div>
                        </div>
                        {track.enrolled && (
                          <span className={`px-2 py-1 text-xs rounded-full ${getStatusColor(track.status || 'Not_Started')}`}>
                            {track.status?.replace('_', ' ')}
                          </span>
                        )}
                      </div>

                      {/* Details */}
                      <div className="space-y-3 mb-4">
                        <div className="flex items-center text-sm text-gray-600">
                          <ClockIcon className="h-4 w-4 mr-2" />
                          <span>{track.duration} weeks duration</span>
                        </div>

                        <div className="flex items-center text-sm text-gray-600">
                          <CurrencyRupeeIcon className="h-4 w-4 mr-2" />
                          <span>₹{track.pricing.student.toLocaleString('en-IN')} (Students)</span>
                        </div>

                        {track.placementAssurance.enabled && (
                          <div className="flex items-center text-sm text-green-600">
                            <ShieldCheckIcon className="h-4 w-4 mr-2" />
                            <span>
                              Avg CTC: {formatIndianCurrency(track.placementAssurance.averageCTC)}
                            </span>
                          </div>
                        )}

                        <div className="flex flex-wrap gap-1">
                          {track.recognizedBy.map(org => (
                            <span key={org} className="px-2 py-1 text-xs bg-blue-100 text-blue-600 rounded">
                              {org}
                            </span>
                          ))}
                        </div>

                        <div className="text-sm text-gray-600">
                          <strong>Industry Partners:</strong> {track.industryPartners.slice(0, 3).join(', ')}
                          {track.industryPartners.length > 3 && ` +${track.industryPartners.length - 3} more`}
                        </div>
                      </div>

                      {/* Progress Bar for Enrolled Tracks */}
                      {track.enrolled && track.progress && (
                        <div className="mb-4">
                          <div className="flex justify-between text-sm text-gray-600 mb-1">
                            <span>Progress</span>
                            <span>{track.progress}%</span>
                          </div>
                          <div className="w-full bg-gray-200 rounded-full h-2">
                            <div 
                              className="bg-blue-600 h-2 rounded-full" 
                              style={{ width: `${track.progress}%` }}
                            ></div>
                          </div>
                        </div>
                      )}

                      {/* Action Buttons */}
                      <div className="flex gap-2">
                        {track.enrolled ? (
                          <>
                            <button className="btn btn-primary flex-1 flex items-center justify-center">
                              <PlayIcon className="h-4 w-4 mr-2" />
                              Continue
                            </button>
                            {track.status === 'Assessment_Ready' && (
                              <button className="btn btn-secondary flex items-center justify-center">
                                <DocumentTextIcon className="h-4 w-4 mr-2" />
                                Take Exam
                              </button>
                            )}
                          </>
                        ) : (
                          <>
                            <button className="btn btn-primary flex-1">
                              Enroll Now
                            </button>
                            <button className="btn btn-secondary">
                              Learn More
                            </button>
                          </>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {selectedTab === 'enrolled' && (
              <div className="space-y-6">
                {enrolledTracks.length > 0 ? (
                  enrolledTracks.map(track => (
                    <div key={track.id} className="bg-white border border-gray-200 rounded-lg p-6">
                      <div className="flex items-center justify-between mb-4">
                        <div className="flex items-center">
                          <span className="text-2xl mr-3">{getLevelIcon(track.level)}</span>
                          <div>
                            <h3 className="font-semibold text-lg">{track.name}</h3>
                            <p className="text-sm text-gray-500">{track.level} Level</p>
                          </div>
                        </div>
                        <span className={`px-3 py-1 text-sm rounded-full ${getStatusColor(track.status || 'Not_Started')}`}>
                          {track.status?.replace('_', ' ')}
                        </span>
                      </div>

                      {track.progress && (
                        <div className="mb-4">
                          <div className="flex justify-between text-sm text-gray-600 mb-2">
                            <span>Overall Progress</span>
                            <span>{track.progress}%</span>
                          </div>
                          <div className="w-full bg-gray-200 rounded-full h-3">
                            <div 
                              className="bg-green-600 h-3 rounded-full transition-all duration-300" 
                              style={{ width: `${track.progress}%` }}
                            ></div>
                          </div>
                        </div>
                      )}

                      <div className="flex gap-3">
                        <button className="btn btn-primary">Continue Learning</button>
                        <button className="btn btn-secondary">View Progress</button>
                        {track.status === 'Assessment_Ready' && (
                          <button className="btn bg-green-600 text-white hover:bg-green-700">
                            Take Certification Exam
                          </button>
                        )}
                      </div>
                    </div>
                  ))
                ) : (
                  <div className="text-center py-12">
                    <AcademicCapIcon className="h-16 w-16 text-gray-400 mx-auto mb-4" />
                    <h3 className="text-lg font-medium text-gray-900 mb-2">No Enrolled Certifications</h3>
                    <p className="text-gray-600 mb-4">Start your professional certification journey today!</p>
                    <button 
                      onClick={() => setSelectedTab('explore')}
                      className="btn btn-primary"
                    >
                      Explore Certifications
                    </button>
                  </div>
                )}
              </div>
            )}

            {selectedTab === 'completed' && (
              <div className="space-y-6">
                {completedTracks.length > 0 ? (
                  completedTracks.map(track => (
                    <div key={track.id} className="bg-gradient-to-r from-green-50 to-blue-50 border border-green-200 rounded-lg p-6">
                      <div className="flex items-center justify-between mb-4">
                        <div className="flex items-center">
                          <TrophyIcon className="h-8 w-8 text-yellow-500 mr-3" />
                          <div>
                            <h3 className="font-semibold text-lg">{track.name}</h3>
                            <p className="text-sm text-gray-600">Completed & Certified</p>
                          </div>
                        </div>
                        <div className="text-right">
                          <div className="text-2xl font-bold text-green-600">92%</div>
                          <div className="text-sm text-gray-500">Score</div>
                        </div>
                      </div>

                      <div className="flex gap-3">
                        <button className="btn btn-primary">View Certificate</button>
                        <button className="btn btn-secondary">Download PDF</button>
                        <button className="btn btn-secondary">Share on LinkedIn</button>
                      </div>
                    </div>
                  ))
                ) : (
                  <div className="text-center py-12">
                    <TrophyIcon className="h-16 w-16 text-gray-400 mx-auto mb-4" />
                    <h3 className="text-lg font-medium text-gray-900 mb-2">No Completed Certifications</h3>
                    <p className="text-gray-600">Complete your enrolled certifications to see them here.</p>
                  </div>
                )}
              </div>
            )}

            {selectedTab === 'analytics' && userProgress && (
              <div className="space-y-6">
                {/* Career Readiness Chart */}
                <div className="bg-white border border-gray-200 rounded-lg p-6">
                  <h3 className="text-lg font-semibold mb-4">Career Readiness Analysis</h3>
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <div className="text-center">
                      <div className="text-4xl font-bold text-blue-600 mb-2">
                        {userProgress.careerReadiness}%
                      </div>
                      <div className="text-sm text-gray-600">Overall Readiness</div>
                    </div>
                    <div className="text-center">
                      <div className="text-4xl font-bold text-green-600 mb-2">
                        {formatIndianCurrency(userProgress.placementPotential.estimatedCTC)}
                      </div>
                      <div className="text-sm text-gray-600">Estimated CTC</div>
                    </div>
                    <div className="text-center">
                      <div className="text-4xl font-bold text-purple-600 mb-2">
                        {userProgress.placementPotential.topCompanies.length}
                      </div>
                      <div className="text-sm text-gray-600">Target Companies</div>
                    </div>
                  </div>
                </div>

                {/* Top Companies */}
                <div className="bg-white border border-gray-200 rounded-lg p-6">
                  <h3 className="text-lg font-semibold mb-4">Target Companies</h3>
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                    {userProgress.placementPotential.topCompanies.map(company => (
                      <div key={company} className="text-center p-4 bg-gray-50 rounded-lg">
                        <div className="font-medium">{company}</div>
                        <div className="text-sm text-gray-500">High Match</div>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Recommendations */}
                <div className="bg-white border border-gray-200 rounded-lg p-6">
                  <h3 className="text-lg font-semibold mb-4">Recommendations</h3>
                  <div className="space-y-3">
                    <div className="flex items-start p-3 bg-blue-50 rounded-lg">
                      <ExclamationTriangleIcon className="h-5 w-5 text-blue-600 mr-3 mt-0.5" />
                      <div>
                        <div className="font-medium text-blue-900">Complete AI-Native Systems certification</div>
                        <div className="text-sm text-blue-700">35% complete - finish to boost placement score by 15%</div>
                      </div>
                    </div>
                    <div className="flex items-start p-3 bg-green-50 rounded-lg">
                      <CheckCircleIcon className="h-5 w-5 text-green-600 mr-3 mt-0.5" />
                      <div>
                        <div className="font-medium text-green-900">Consider VLSI Design Expert track</div>
                        <div className="text-sm text-green-700">Based on your profile, you're eligible for ₹18L+ roles</div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default CertificationDashboard;