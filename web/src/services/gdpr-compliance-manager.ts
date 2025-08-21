// Phase 6B: GDPR Compliance Manager
// Handles GDPR, CCPA, LGPD, and DPDP compliance across global regions
// @ts-nocheck

import { GLOBAL_INFRASTRUCTURE_CONFIG } from '../config/global-infrastructure';

export interface UserConsent {
  userId: string;
  consentType: 'necessary' | 'functional' | 'analytics' | 'marketing' | 'ai_processing';
  granted: boolean;
  timestamp: Date;
  ipAddress: string;
  userAgent: string;
  region: string;
  expiryDate?: Date;
  withdrawalDate?: Date;
}

export interface DataSubject {
  userId: string;
  email: string;
  region: string;
  dataCategories: string[];
  retentionPeriod: number; // days
  lastActivity: Date;
  consentStatus: UserConsent[];
  dataProcessingHistory: DataProcessingRecord[];
}

export interface DataProcessingRecord {
  id: string;
  userId: string;
  purpose: 'authentication' | 'education' | 'analytics' | 'ai_training' | 'communication';
  dataType: 'personal' | 'behavioral' | 'educational' | 'biometric' | 'special_category';
  processingDate: Date;
  region: string;
  legalBasis: 'consent' | 'contract' | 'legal_obligation' | 'vital_interests' | 'public_task' | 'legitimate_interests';
  retentionDays: number;
  status: 'active' | 'archived' | 'deleted';
}

export interface DataSubjectRequest {
  id: string;
  userId: string;
  type: 'access' | 'rectification' | 'erasure' | 'portability' | 'restriction' | 'objection';
  status: 'received' | 'processing' | 'completed' | 'rejected';
  requestDate: Date;
  responseDate?: Date;
  region: string;
  description: string;
  documents?: string[];
  responseData?: any;
}

export interface ComplianceAudit {
  id: string;
  region: string;
  regulation: 'gdpr' | 'ccpa' | 'lgpd' | 'dpdp';
  auditDate: Date;
  scope: string[];
  findings: AuditFinding[];
  complianceScore: number; // 0-100
  recommendations: string[];
  nextAuditDate: Date;
}

export interface AuditFinding {
  category: 'data_processing' | 'consent_management' | 'security' | 'retention' | 'documentation';
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  remediation: string;
  deadline: Date;
  status: 'open' | 'in_progress' | 'resolved';
}

export class GDPRComplianceManager {
  private dataSubjects: Map<string, DataSubject> = new Map();
  private subjectRequests: Map<string, DataSubjectRequest> = new Map();
  private processingRecords: Map<string, DataProcessingRecord> = new Map();
  private auditHistory: Map<string, ComplianceAudit> = new Map();
  private eventEmitter: any;
  private complianceInterval?: NodeJS.Timeout;

  constructor() {
    // Browser-compatible event emitter
    this.eventEmitter = {
      events: {} as { [event: string]: Function[] },
      on: function(event: string, listener: any): any {
        if (!this.events[event]) this.events[event] = [];
        this.events[event].push(listener);
        return this;
      },
      emit: function(event: string, ...args: any[]): any {
        if (this.events[event]) {
          this.events[event].forEach(listener => listener(...args));
        }
        return this;
      }
    };

    this.initializeComplianceFramework();
    this.startComplianceMonitoring();
  }

  // =============================================================================
  // INITIALIZATION
  // =============================================================================

  private initializeComplianceFramework(): void {
    // Initialize sample data subjects for different regions
    this.createSampleDataSubjects();
    this.createSampleProcessingRecords();
    this.createSampleAudits();
  }

  private createSampleDataSubjects(): void {
    const sampleUsers = [
      { id: 'user_eu_001', email: 'student@university.de', region: 'eu-central-1' },
      { id: 'user_eu_002', email: 'teacher@school.uk', region: 'eu-west-2' },
      { id: 'user_us_001', email: 'student@college.edu', region: 'us-east-1' },
      { id: 'user_br_001', email: 'aluno@universidade.br', region: 'sa-east-1' },
      { id: 'user_in_001', email: 'student@iit.ac.in', region: 'ap-south-1' }
    ];

    sampleUsers.forEach(user => {
      const dataSubject: DataSubject = {
        userId: user.id,
        email: user.email,
        region: user.region,
        dataCategories: ['personal', 'educational', 'behavioral'],
        retentionPeriod: this.getRetentionPeriod(user.region),
        lastActivity: new Date(Date.now() - Math.random() * 30 * 24 * 60 * 60 * 1000), // Last 30 days
        consentStatus: this.generateConsentStatus(user.id, user.region),
        dataProcessingHistory: []
      };

      this.dataSubjects.set(user.id, dataSubject);
    });
  }

  private getRetentionPeriod(region: string): number {
    // Retention periods based on regional regulations
    const retentionPeriods: { [key: string]: number } = {
      'eu-central-1': 1095, // 3 years (GDPR)
      'eu-west-2': 1095,    // 3 years (GDPR)
      'eu-west-1': 1095,    // 3 years (GDPR)
      'us-east-1': 2555,    // 7 years (varies by state)
      'us-west-2': 2555,    // 7 years
      'sa-east-1': 1825,    // 5 years (LGPD)
      'ap-south-1': 1095    // 3 years (DPDP)
    };
    return retentionPeriods[region] || 1095; // Default 3 years
  }

  private generateConsentStatus(userId: string, region: string): UserConsent[] {
    const consentTypes: UserConsent['consentType'][] = [
      'necessary', 'functional', 'analytics', 'marketing', 'ai_processing'
    ];

    return consentTypes.map(type => ({
      userId,
      consentType: type,
      granted: type === 'necessary' || Math.random() > 0.3, // Necessary always granted, others 70% chance
      timestamp: new Date(Date.now() - Math.random() * 90 * 24 * 60 * 60 * 1000), // Last 90 days
      ipAddress: this.generateRandomIP(),
      userAgent: 'Mozilla/5.0 (Educational Platform)',
      region,
      expiryDate: type !== 'necessary' ? new Date(Date.now() + 365 * 24 * 60 * 60 * 1000) : undefined // 1 year
    }));
  }

  private generateRandomIP(): string {
    return `${Math.floor(Math.random() * 256)}.${Math.floor(Math.random() * 256)}.${Math.floor(Math.random() * 256)}.${Math.floor(Math.random() * 256)}`;
  }

  private createSampleProcessingRecords(): void {
    this.dataSubjects.forEach((subject, userId) => {
      // Create processing records for each user
      const recordTypes = [
        { purpose: 'authentication', dataType: 'personal', legalBasis: 'contract' },
        { purpose: 'education', dataType: 'educational', legalBasis: 'consent' },
        { purpose: 'analytics', dataType: 'behavioral', legalBasis: 'legitimate_interests' },
        { purpose: 'ai_training', dataType: 'educational', legalBasis: 'consent' }
      ];

      recordTypes.forEach((record, index) => {
        const processingRecord: DataProcessingRecord = {
          id: `proc_${userId}_${index}`,
          userId,
          purpose: record.purpose as any,
          dataType: record.dataType as any,
          processingDate: new Date(Date.now() - Math.random() * 60 * 24 * 60 * 60 * 1000), // Last 60 days
          region: subject.region,
          legalBasis: record.legalBasis as any,
          retentionDays: subject.retentionPeriod,
          status: 'active'
        };

        this.processingRecords.set(processingRecord.id, processingRecord);
        subject.dataProcessingHistory.push(processingRecord);
      });

      this.dataSubjects.set(userId, subject);
    });
  }

  private createSampleAudits(): void {
    const regions = ['eu-central-1', 'eu-west-2', 'us-east-1', 'sa-east-1', 'ap-south-1'];
    
    regions.forEach(region => {
      const regulation = this.getRegionRegulation(region);
      const auditId = `audit_${region}_${Date.now()}`;
      
      const audit: ComplianceAudit = {
        id: auditId,
        region,
        regulation,
        auditDate: new Date(Date.now() - Math.random() * 90 * 24 * 60 * 60 * 1000), // Last 90 days
        scope: ['data_processing', 'consent_management', 'security', 'retention'],
        findings: this.generateAuditFindings(),
        complianceScore: 85 + Math.floor(Math.random() * 15), // 85-100%
        recommendations: [
          'Implement automated data retention policies',
          'Enhance consent management interface',
          'Regular staff training on data protection'
        ],
        nextAuditDate: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000) // 1 year from now
      };

      this.auditHistory.set(auditId, audit);
    });
  }

  private getRegionRegulation(region: string): 'gdpr' | 'ccpa' | 'lgpd' | 'dpdp' {
    const regionRegulations: { [key: string]: 'gdpr' | 'ccpa' | 'lgpd' | 'dpdp' } = {
      'eu-central-1': 'gdpr',
      'eu-west-2': 'gdpr',
      'eu-west-1': 'gdpr',
      'us-east-1': 'ccpa',
      'us-west-2': 'ccpa',
      'sa-east-1': 'lgpd',
      'ap-south-1': 'dpdp'
    };
    return regionRegulations[region] || 'gdpr';
  }

  private generateAuditFindings(): AuditFinding[] {
    const findings: AuditFinding[] = [];
    
    // Generate 0-3 random findings
    const findingCount = Math.floor(Math.random() * 4);
    
    for (let i = 0; i < findingCount; i++) {
      const categories: AuditFinding['category'][] = ['data_processing', 'consent_management', 'security', 'retention', 'documentation'];
      const severities: AuditFinding['severity'][] = ['low', 'medium', 'high', 'critical'];
      
      const finding: AuditFinding = {
        category: categories[Math.floor(Math.random() * categories.length)],
        severity: severities[Math.floor(Math.random() * severities.length)],
        description: 'Sample audit finding requiring attention',
        remediation: 'Implement recommended controls and procedures',
        deadline: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000), // 30 days
        status: Math.random() > 0.7 ? 'resolved' : 'open'
      };
      
      findings.push(finding);
    }
    
    return findings;
  }

  // =============================================================================
  // CONSENT MANAGEMENT
  // =============================================================================

  public async recordConsent(
    userId: string,
    consentType: UserConsent['consentType'],
    granted: boolean,
    userContext: {
      ipAddress: string;
      userAgent: string;
      region: string;
    }
  ): Promise<boolean> {
    try {
      const subject = this.dataSubjects.get(userId);
      if (!subject) {
        throw new Error(`Data subject ${userId} not found`);
      }

      const consent: UserConsent = {
        userId,
        consentType,
        granted,
        timestamp: new Date(),
        ipAddress: userContext.ipAddress,
        userAgent: userContext.userAgent,
        region: userContext.region,
        expiryDate: consentType !== 'necessary' ? 
          new Date(Date.now() + 365 * 24 * 60 * 60 * 1000) : undefined
      };

      // Update existing consent or add new
      const existingIndex = subject.consentStatus.findIndex(c => c.consentType === consentType);
      if (existingIndex >= 0) {
        subject.consentStatus[existingIndex] = consent;
      } else {
        subject.consentStatus.push(consent);
      }

      this.dataSubjects.set(userId, subject);
      
      this.eventEmitter.emit('consentRecorded', {
        userId,
        consentType,
        granted,
        timestamp: consent.timestamp
      });

      return true;
    } catch (error) {
      this.eventEmitter.emit('consentError', { userId, error });
      return false;
    }
  }

  public async withdrawConsent(userId: string, consentType: UserConsent['consentType']): Promise<boolean> {
    const subject = this.dataSubjects.get(userId);
    if (!subject) return false;

    const consentIndex = subject.consentStatus.findIndex(c => c.consentType === consentType);
    if (consentIndex >= 0) {
      subject.consentStatus[consentIndex].granted = false;
      subject.consentStatus[consentIndex].withdrawalDate = new Date();
      
      this.dataSubjects.set(userId, subject);
      
      this.eventEmitter.emit('consentWithdrawn', { userId, consentType });
      
      // If analytics consent withdrawn, stop analytics processing
      if (consentType === 'analytics') {
        await this.stopDataProcessing(userId, 'analytics');
      }
      
      return true;
    }
    
    return false;
  }

  public getConsentStatus(userId: string): UserConsent[] | null {
    const subject = this.dataSubjects.get(userId);
    return subject ? subject.consentStatus : null;
  }

  // =============================================================================
  // DATA SUBJECT RIGHTS
  // =============================================================================

  public async createDataSubjectRequest(
    userId: string,
    type: DataSubjectRequest['type'],
    description: string,
    region: string
  ): Promise<string> {
    const requestId = `dsr_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    const request: DataSubjectRequest = {
      id: requestId,
      userId,
      type,
      status: 'received',
      requestDate: new Date(),
      region,
      description
    };

    this.subjectRequests.set(requestId, request);
    
    this.eventEmitter.emit('subjectRequestReceived', request);
    
    // Auto-process simple requests
    if (type === 'access') {
      setTimeout(() => this.processAccessRequest(requestId), 1000);
    }
    
    return requestId;
  }

  private async processAccessRequest(requestId: string): Promise<void> {
    const request = this.subjectRequests.get(requestId);
    if (!request) return;

    request.status = 'processing';
    this.subjectRequests.set(requestId, request);

    // Gather all user data
    const subject = this.dataSubjects.get(request.userId);
    if (!subject) return;

    const userData = {
      personalData: {
        userId: subject.userId,
        email: subject.email,
        region: subject.region,
        lastActivity: subject.lastActivity
      },
      consentHistory: subject.consentStatus,
      processingHistory: subject.dataProcessingHistory,
      dataCategories: subject.dataCategories
    };

    request.status = 'completed';
    request.responseDate = new Date();
    request.responseData = userData;
    this.subjectRequests.set(requestId, request);

    this.eventEmitter.emit('subjectRequestCompleted', request);
  }

  public async processErasureRequest(requestId: string): Promise<boolean> {
    const request = this.subjectRequests.get(requestId);
    if (!request || request.type !== 'erasure') return false;

    request.status = 'processing';
    this.subjectRequests.set(requestId, request);

    try {
      // Check if erasure is allowed (no legal obligations, etc.)
      const canErase = await this.validateErasureRequest(request.userId);
      
      if (canErase) {
        await this.eraseUserData(request.userId);
        request.status = 'completed';
        request.responseDate = new Date();
      } else {
        request.status = 'rejected';
        request.responseDate = new Date();
      }

      this.subjectRequests.set(requestId, request);
      this.eventEmitter.emit('subjectRequestCompleted', request);
      
      return canErase;
    } catch (error) {
      request.status = 'rejected';
      this.subjectRequests.set(requestId, request);
      return false;
    }
  }

  private async validateErasureRequest(userId: string): Promise<boolean> {
    // Check for legal obligations that prevent erasure
    const processingRecords = Array.from(this.processingRecords.values())
      .filter(record => record.userId === userId && record.status === 'active');
    
    // Can't erase if there are legal obligations
    const hasLegalObligations = processingRecords.some(record => 
      record.legalBasis === 'legal_obligation' || record.legalBasis === 'vital_interests'
    );
    
    return !hasLegalObligations;
  }

  private async eraseUserData(userId: string): Promise<void> {
    // Mark data subject as deleted
    this.dataSubjects.delete(userId);
    
    // Archive processing records
    this.processingRecords.forEach((record, recordId) => {
      if (record.userId === userId) {
        record.status = 'deleted';
        this.processingRecords.set(recordId, record);
      }
    });

    this.eventEmitter.emit('userDataErased', { userId });
  }

  // =============================================================================
  // DATA PROCESSING TRACKING
  // =============================================================================

  public async recordDataProcessing(
    userId: string,
    purpose: DataProcessingRecord['purpose'],
    dataType: DataProcessingRecord['dataType'],
    legalBasis: DataProcessingRecord['legalBasis'],
    region: string
  ): Promise<string> {
    const recordId = `proc_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    const record: DataProcessingRecord = {
      id: recordId,
      userId,
      purpose,
      dataType,
      processingDate: new Date(),
      region,
      legalBasis,
      retentionDays: this.getRetentionPeriod(region),
      status: 'active'
    };

    this.processingRecords.set(recordId, record);
    
    // Update data subject history
    const subject = this.dataSubjects.get(userId);
    if (subject) {
      subject.dataProcessingHistory.push(record);
      this.dataSubjects.set(userId, subject);
    }

    this.eventEmitter.emit('dataProcessingRecorded', record);
    
    return recordId;
  }

  private async stopDataProcessing(userId: string, purpose: DataProcessingRecord['purpose']): Promise<void> {
    this.processingRecords.forEach((record, recordId) => {
      if (record.userId === userId && record.purpose === purpose && record.status === 'active') {
        record.status = 'archived';
        this.processingRecords.set(recordId, record);
      }
    });

    this.eventEmitter.emit('dataProcessingStopped', { userId, purpose });
  }

  // =============================================================================
  // COMPLIANCE MONITORING
  // =============================================================================

  private startComplianceMonitoring(): void {
    this.complianceInterval = setInterval(() => {
      this.checkDataRetention();
      this.checkConsentExpiry();
      this.generateComplianceMetrics();
    }, 60000); // Every minute
  }

  private checkDataRetention(): void {
    const now = new Date();
    
    this.processingRecords.forEach((record, recordId) => {
      if (record.status === 'active') {
        const retentionEnd = new Date(record.processingDate.getTime() + record.retentionDays * 24 * 60 * 60 * 1000);
        
        if (now > retentionEnd) {
          record.status = 'archived';
          this.processingRecords.set(recordId, record);
          
          this.eventEmitter.emit('dataRetentionExpired', record);
        }
      }
    });
  }

  private checkConsentExpiry(): void {
    const now = new Date();
    
    this.dataSubjects.forEach((subject, userId) => {
      subject.consentStatus.forEach(consent => {
        if (consent.expiryDate && consent.granted && now > consent.expiryDate) {
          consent.granted = false;
          consent.withdrawalDate = now;
          
          this.eventEmitter.emit('consentExpired', { userId, consentType: consent.consentType });
        }
      });
      
      this.dataSubjects.set(userId, subject);
    });
  }

  private generateComplianceMetrics(): void {
    const metrics = {
      totalDataSubjects: this.dataSubjects.size,
      activeProcessingRecords: Array.from(this.processingRecords.values()).filter(r => r.status === 'active').length,
      pendingRequests: Array.from(this.subjectRequests.values()).filter(r => r.status === 'received').length,
      expiredConsents: this.getExpiredConsentsCount(),
      complianceScore: this.calculateComplianceScore()
    };

    this.eventEmitter.emit('complianceMetricsUpdated', metrics);
  }

  private getExpiredConsentsCount(): number {
    let expiredCount = 0;
    const now = new Date();
    
    this.dataSubjects.forEach(subject => {
      subject.consentStatus.forEach(consent => {
        if (consent.expiryDate && now > consent.expiryDate && consent.granted) {
          expiredCount++;
        }
      });
    });
    
    return expiredCount;
  }

  private calculateComplianceScore(): number {
    let score = 100;
    
    // Deduct points for expired consents
    const expiredConsents = this.getExpiredConsentsCount();
    score -= expiredConsents * 2;
    
    // Deduct points for pending requests
    const pendingRequests = Array.from(this.subjectRequests.values()).filter(r => r.status === 'received').length;
    score -= pendingRequests * 5;
    
    // Deduct points for overdue processing records
    const overdueRecords = Array.from(this.processingRecords.values()).filter(r => {
      const retentionEnd = new Date(r.processingDate.getTime() + r.retentionDays * 24 * 60 * 60 * 1000);
      return r.status === 'active' && new Date() > retentionEnd;
    }).length;
    score -= overdueRecords * 10;
    
    return Math.max(0, Math.min(100, score));
  }

  // =============================================================================
  // PUBLIC API
  // =============================================================================

  public getComplianceStatus(): any {
    return {
      dataSubjects: this.dataSubjects.size,
      processingRecords: this.processingRecords.size,
      pendingRequests: Array.from(this.subjectRequests.values()).filter(r => r.status !== 'completed').length,
      complianceScore: this.calculateComplianceScore(),
      regions: Array.from(new Set(Array.from(this.dataSubjects.values()).map(s => s.region))),
      lastAudit: Array.from(this.auditHistory.values()).sort((a, b) => b.auditDate.getTime() - a.auditDate.getTime())[0]
    };
  }

  public getDataSubject(userId: string): DataSubject | undefined {
    return this.dataSubjects.get(userId);
  }

  public getSubjectRequest(requestId: string): DataSubjectRequest | undefined {
    return this.subjectRequests.get(requestId);
  }

  public getAllSubjectRequests(): DataSubjectRequest[] {
    return Array.from(this.subjectRequests.values());
  }

  public getAuditHistory(): ComplianceAudit[] {
    return Array.from(this.auditHistory.values());
  }

  public getRegionCompliance(region: string): any {
    const subjects = Array.from(this.dataSubjects.values()).filter(s => s.region === region);
    const records = Array.from(this.processingRecords.values()).filter(r => r.region === region);
    const requests = Array.from(this.subjectRequests.values()).filter(r => r.region === region);
    
    return {
      region,
      regulation: this.getRegionRegulation(region),
      dataSubjects: subjects.length,
      processingRecords: records.length,
      pendingRequests: requests.filter(r => r.status !== 'completed').length,
      compliance: GLOBAL_INFRASTRUCTURE_CONFIG.COMPLIANCE[this.getRegionRegulation(region.replace('-', '_') as any) as keyof typeof GLOBAL_INFRASTRUCTURE_CONFIG.COMPLIANCE]
    };
  }

  // Event subscription methods
  public onConsentEvent(callback: Function): void {
    this.eventEmitter.on('consentRecorded', callback);
    this.eventEmitter.on('consentWithdrawn', callback);
    this.eventEmitter.on('consentExpired', callback);
  }

  public onSubjectRequest(callback: Function): void {
    this.eventEmitter.on('subjectRequestReceived', callback);
    this.eventEmitter.on('subjectRequestCompleted', callback);
  }

  public onComplianceAlert(callback: Function): void {
    this.eventEmitter.on('dataRetentionExpired', callback);
    this.eventEmitter.on('userDataErased', callback);
  }

  public onMetricsUpdate(callback: Function): void {
    this.eventEmitter.on('complianceMetricsUpdated', callback);
  }

  // Cleanup
  public destroy(): void {
    if (this.complianceInterval) {
      clearInterval(this.complianceInterval);
    }
  }
}

export default GDPRComplianceManager;