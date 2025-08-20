/**
 * Phase 5A Integration Tests
 * Validates all India-specific components work together correctly
 */

import { describe, test, expect, beforeEach } from 'vitest';
import IndianPaymentService from '../services/payments-india';
import IndianComplianceService from '../services/compliance-india';
import IndianEducationService from '../services/education-india';
import HindiLocalizationService from '../localization/hindi-support';
import { INDIAN_MARKET_CONFIG, GLOBAL_INFRASTRUCTURE_CONFIG, PERFORMANCE_TARGETS, AUTO_SCALING_CONFIG } from '../config/infrastructure';

describe('Phase 5A: India Infrastructure Integration', () => {
  let paymentService: IndianPaymentService;
  let complianceService: IndianComplianceService;
  let educationService: IndianEducationService;
  let localizationService: HindiLocalizationService;

  beforeEach(() => {
    paymentService = new IndianPaymentService();
    complianceService = new IndianComplianceService();
    educationService = new IndianEducationService();
    localizationService = new HindiLocalizationService();
  });

  describe('Infrastructure Configuration', () => {
    test('should have AP-SOUTH-1 as primary region', () => {
      const primaryRegion = GLOBAL_INFRASTRUCTURE_CONFIG.primary;
      
      expect(primaryRegion.code).toBe('ap-south-1');
      expect(primaryRegion.isPrimary).toBe(true);
      expect(primaryRegion.maxConcurrentUsers).toBe(6000);
      expect(primaryRegion.targetLatency).toBe(50); // 50ms for Indian users
    });

    test('should have proper ISP partnerships configured', () => {
      const primaryRegion = GLOBAL_INFRASTRUCTURE_CONFIG.primary;
      
      expect(primaryRegion.ispPartners).toContain('bharti-airtel');
      expect(primaryRegion.ispPartners).toContain('reliance-jio');
      expect(primaryRegion.ispPartners).toContain('bsnl');
      expect(primaryRegion.ispPartners).toContain('tata-communications');
    });

    test('should have data residency enabled for India', () => {
      const primaryRegion = GLOBAL_INFRASTRUCTURE_CONFIG.primary;
      
      expect(primaryRegion.dataResidency).toBe(true);
      expect(primaryRegion.complianceRequirements).toContain('PDPB');
      expect(primaryRegion.complianceRequirements).toContain('IT_ACT_2000');
    });
  });

  describe('Indian Payment System Integration', () => {
    test('should calculate GST correctly', () => {
      const baseAmount = 2999; // Pro plan price
      const taxes = paymentService.calculateIndianTaxes(baseAmount);

      expect(taxes.baseAmount).toBe(baseAmount);
      expect(taxes.gstAmount).toBe(Math.round(baseAmount * 0.18)); // 18% GST
      expect(taxes.finalAmount).toBe(baseAmount + taxes.gstAmount);
    });

    test('should apply regional discounts correctly', () => {
      const basePrice = INDIAN_MARKET_CONFIG.pricing.tiers.pro.price;
      
      // Test Tier 1 city (no discount)
      const mumbaiPricing = paymentService.getRegionalPricing(basePrice, {
        city: 'mumbai',
        state: 'maharashtra',
        pincode: '400001'
      });
      
      expect(mumbaiPricing.discount).toBe(0);
      expect(mumbaiPricing.finalPrice).toBe(basePrice);

      // Test rural area (50% discount)
      const ruralPricing = paymentService.getRegionalPricing(basePrice, {
        city: 'rural_village',
        state: 'uttar pradesh',
        pincode: '123456'
      });
      
      expect(ruralPricing.discount).toBe(50);
      expect(ruralPricing.finalPrice).toBe(Math.round(basePrice * 0.5));
    });

    test('should create UPI payment correctly', async () => {
      const amount = 2999;
      const vpa = 'test@paytm';
      const description = 'SIS Pro Subscription';

      const result = await paymentService.processUPIPayment(vpa, amount, description);

      expect(result).toHaveProperty('transactionId');
      expect(result).toHaveProperty('status');
      expect(result).toHaveProperty('upiRef');
      expect(typeof result.transactionId).toBe('string');
    });

    test('should calculate EMI correctly', () => {
      const amount = 25000; // Enterprise plan
      const duration = 12; // 12 months
      const interestRate = 12; // 12% annual

      const emi = paymentService.calculateEMI(amount, duration, interestRate);

      expect(emi.monthlyEMI).toBeGreaterThan(0);
      expect(emi.totalAmount).toBeGreaterThan(amount);
      expect(emi.totalInterest).toBeGreaterThan(0);
      expect(emi.emiBreakdown).toHaveLength(duration);
    });
  });

  describe('Indian Compliance System', () => {
    test('should implement PDPB compliance for Indian users', async () => {
      const userId = 'test-user-123';
      const userProfile = {
        nationality: 'Indian',
        residency: 'India',
        isMinor: false
      };

      const compliance = await complianceService.implementPDPBCompliance(userId, userProfile);

      expect(compliance.consentRequired).toBe(true);
      expect(compliance.crossBorderTransfer).toBe(false); // Data stays in India
      expect(compliance.dataCategories).toContain('personal_identifiers');
      expect(compliance.userRights).toContain('right_to_access');
      expect(compliance.retentionPeriod).toBe(2555); // 7 years for adults
    });

    test('should configure data residency for Indian users', async () => {
      const userId = 'test-user-123';
      const userDetails = {
        nationality: 'Indian',
        residency: 'India',
        userType: 'student' as const
      };

      const dataResidency = await complianceService.configureDataResidency(userId, userDetails);

      expect(dataResidency.userType).toBe('indian_citizen');
      expect(dataResidency.dataLocation).toBe('india_only');
      expect(dataResidency.backupLocations).toEqual(['ap-south-1a', 'ap-south-1b']);
    });

    test('should handle GST compliance', async () => {
      const transactionData = {
        amount: 2999,
        userGST: '27AABCS1234C1ZM', // Maharashtra GST
        transactionType: 'B2C' as const,
        serviceCategory: 'software'
      };

      const gstCompliance = await complianceService.manageGSTCompliance(transactionData);

      expect(gstCompliance.gstAmount).toBe(Math.round(2999 * 0.18));
      expect(gstCompliance.hsnCode).toBe('998314'); // Software services HSN
      expect(gstCompliance.reverseCharge).toBe(false); // B2C transaction
    });

    test('should record user consent properly', async () => {
      const userId = 'test-user-123';
      const consentData = {
        dataProcessing: true,
        marketing: false,
        analytics: true,
        thirdPartySharing: false,
        timestamp: new Date(),
        ipAddress: '203.0.113.1',
        userAgent: 'Mozilla/5.0 (test)'
      };

      const consent = await complianceService.recordUserConsent(userId, consentData);

      expect(consent.status).toBe('valid');
      expect(consent.consentId).toContain('consent_');
      expect(consent.expiryDate).toBeInstanceOf(Date);
    });
  });

  describe('Indian Education System Integration', () => {
    test('should create IIT partnership correctly', async () => {
      const institutionId = 'iit_bombay';
      const partnershipDetails = {
        type: 'curriculum_adoption' as const,
        duration: 12,
        studentsCount: 500,
        facultyCount: 25,
        customRequirements: ['Hindi support', 'GATE preparation'],
        pricing: {
          model: 'site_license' as const,
          amount: 500000,
          currency: 'INR' as const
        }
      };

      const partnership = await educationService.createInstitutionPartnership(
        institutionId,
        partnershipDetails
      );

      expect(partnership.partnershipId).toContain('partnership_');
      expect(partnership.contractDetails.terms.jurisdiction).toBe('Mumbai, Maharashtra');
      expect(partnership.implementationPlan).toHaveProperty('phase1');
      expect(partnership.successMetrics).toHaveProperty('studentEngagement');
    });

    test('should align with AICTE curriculum', async () => {
      const branch = 'ECE';
      const semester = 5;

      const alignment = await educationService.alignWithAICTECurriculum(branch, semester);

      expect(alignment.alignedModules).toBeInstanceOf(Array);
      expect(alignment.learningOutcomes).toBeInstanceOf(Array);
      expect(alignment.assessmentStrategy).toHaveProperty('continuous');
      expect(alignment.labRequirements).toHaveProperty('duration');
    });

    test('should generate GATE preparation plan', async () => {
      const studentProfile = {
        id: 'student-123',
        name: 'Test Student',
        rollNumber: '2021ECE001',
        institution: {} as any,
        branch: 'ECE',
        semester: 5,
        yearOfStudy: 3,
        academicPerformance: {
          cgpa: 8.5,
          sgpaHistory: [8.2, 8.5, 8.7, 8.6],
          backlogs: 0
        },
        placementPreferences: ['Core Electronics'],
        learningPath: {
          completedModules: ['sis_101'],
          currentModule: 'sis_201',
          recommendedNext: ['sis_301'],
          weakAreas: ['Timing Analysis'],
          strongAreas: ['Logic Design']
        },
        certificationGoals: ['SIS Certified Professional']
      };

      const gatePrep = await educationService.generateGATEPreparationPlan(
        studentProfile,
        2026,
        'EC'
      );

      expect(gatePrep.studyPlan).toBeDefined();
      expect(gatePrep.keyTopics).toContain('Digital Circuits');
      expect(gatePrep.keyTopics).toContain('Analog Circuits');
      expect(gatePrep.mockTestSchedule).toBeDefined();
    });

    test('should assess student progress', async () => {
      const studentId = 'student-123';
      const assessmentData = {
        moduleId: 'sis_101',
        assessmentType: 'quiz' as const,
        score: 85,
        maxScore: 100,
        timeSpent: 45,
        attemptsCount: 1,
        helpSought: false
      };

      const assessment = await educationService.assessStudentProgress(studentId, assessmentData);

      expect(assessment.currentGrade).toContain('A'); // Should be A+ for 85%
      expect(assessment.progressPercentage).toBeGreaterThan(0);
      expect(assessment.recommendations).toBeInstanceOf(Array);
      expect(assessment.nextSteps).toBeInstanceOf(Array);
    });
  });

  describe('Hindi Localization Integration', () => {
    test('should provide Hindi translations', () => {
      localizationService.setLanguage('hi-IN');

      const dashboard = localizationService.getText('nav.dashboard');
      const projects = localizationService.getText('nav.projects');
      const settings = localizationService.getText('nav.settings');

      expect(dashboard).toBe('डैशबोर्ड');
      expect(projects).toBe('प्रोजेक्ट्स');
      expect(settings).toBe('सेटिंग्स');
    });

    test('should format Indian numbers correctly', () => {
      const number = 1234567;
      
      const indianFormat = localizationService.formatNumber(number, 'indian');
      const intlFormat = localizationService.formatNumber(number, 'international');

      expect(indianFormat).toBe('12,34,567');
      expect(intlFormat).toBe('1,234,567');
    });

    test('should format Indian currency correctly', () => {
      const amount = 2999;
      
      const formatted = localizationService.formatCurrency(amount, {
        showSymbol: true,
        showDecimals: true
      });

      expect(formatted).toBe('₹2,999.00');
    });

    test('should format dates in Indian format', () => {
      const date = new Date('2025-01-20T14:30:00+05:30');
      
      const shortFormat = localizationService.formatDate(date, 'short');
      const longFormat = localizationService.formatDate(date, 'long');

      expect(shortFormat).toMatch(/20\/01\/2025/); // DD/MM/YYYY
      expect(longFormat).toContain('January');
      expect(longFormat).toContain('2025');
    });

    test('should provide educational terminology in both languages', () => {
      const terms = localizationService.getEducationalTerms();
      
      expect(terms.logic_design.english).toBe('Logic Design');
      expect(terms.logic_design.hindi).toBe('तर्क डिज़ाइन');
      expect(terms.logic_design.definition).toBeDefined();
    });

    test('should provide Indian grading system translations', () => {
      const grading = localizationService.getGradingSystem();
      
      expect(grading.outstanding.english).toBe('Outstanding (O)');
      expect(grading.outstanding.hindi).toBe('उत्कृष्ट (O)');
      expect(grading.excellent.hindi).toBe('उत्तम (A+)');
    });
  });

  describe('Cross-Service Integration', () => {
    test('should integrate payment and compliance services', async () => {
      // Create a transaction that requires both payment processing and compliance
      const userId = 'test-user-123';
      const amount = 2999;
      
      // Calculate taxes (payment service)
      const taxes = paymentService.calculateIndianTaxes(amount);
      
      // Check compliance requirements (compliance service)
      const userProfile = {
        nationality: 'Indian',
        residency: 'India',
        isMinor: false
      };
      const compliance = await complianceService.implementPDPBCompliance(userId, userProfile);

      // Verify integration
      expect(taxes.finalAmount).toBeGreaterThan(amount); // GST added
      expect(compliance.consentRequired).toBe(true); // Consent needed for Indian users
      expect(compliance.crossBorderTransfer).toBe(false); // Data stays in India
    });

    test('should integrate education and localization services', async () => {
      // Test Hindi support in educational content
      localizationService.setLanguage('hi-IN');
      
      // Get educational terms in Hindi
      const terms = localizationService.getEducationalTerms();
      const gradingSystem = localizationService.getGradingSystem();
      
      // Verify educational service works with localization
      expect(terms.logic_design.hindi).toBe('तर्क डिज़ाइन');
      expect(gradingSystem.excellent.hindi).toBe('उत्तम (A+)');
      
      // Test AICTE alignment with Hindi support
      const alignment = await educationService.alignWithAICTECurriculum('ECE', 5);
      expect(alignment.alignedModules).toBeInstanceOf(Array);
    });

    test('should integrate all services for complete user journey', async () => {
      const userId = 'test-user-123';
      const userDetails = {
        nationality: 'Indian',
        residency: 'India',
        userType: 'student' as const,
        location: { city: 'mumbai', state: 'maharashtra', pincode: '400001' },
        subscriptionTier: 'pro'
      };

      // 1. Set up compliance
      const compliance = await complianceService.implementPDPBCompliance(userId, {
        nationality: userDetails.nationality,
        residency: userDetails.residency,
        isMinor: false
      });

      // 2. Calculate regional pricing
      const basePrice = INDIAN_MARKET_CONFIG.pricing.tiers.pro.price;
      const regionalPricing = paymentService.getRegionalPricing(basePrice, userDetails.location);

      // 3. Set up educational profile
      const partnership = await educationService.createInstitutionPartnership('iit_bombay', {
        type: 'curriculum_adoption',
        duration: 12,
        studentsCount: 1,
        facultyCount: 1,
        customRequirements: [],
        pricing: { model: 'per_student', amount: regionalPricing.finalPrice, currency: 'INR' }
      });

      // 4. Configure localization
      localizationService.setLanguage('hi-IN');
      const welcomeText = localizationService.getText('greeting.welcome');

      // Verify complete integration
      expect(compliance.consentRequired).toBe(true);
      expect(regionalPricing.finalPrice).toBeLessThanOrEqual(basePrice);
      expect(partnership.partnershipId).toBeDefined();
      expect(typeof welcomeText).toBe('string');
    });
  });

  describe('Performance and Scalability', () => {
    test('should meet Indian performance targets', () => {
      expect(PERFORMANCE_TARGETS.india.latency.p95).toBe(50); // 50ms target
      expect(PERFORMANCE_TARGETS.india.availability.target).toBe(99.95); // 99.95% uptime
      expect(PERFORMANCE_TARGETS.india.throughput.concurrentUsers).toBe(6000); // 6k concurrent users
    });

    test('should have proper auto-scaling configuration', () => {
      expect(AUTO_SCALING_CONFIG.india.normalHours.minInstances).toBe(8);
      expect(AUTO_SCALING_CONFIG.india.peakHours.maxInstances).toBe(40);
      expect(AUTO_SCALING_CONFIG.india.examSeason.maxInstances).toBe(60);
    });
  });
});

// Export test utilities for other test files
export {
  IndianPaymentService,
  IndianComplianceService,
  IndianEducationService,
  HindiLocalizationService
};