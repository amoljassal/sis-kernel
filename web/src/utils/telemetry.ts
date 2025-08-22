import { trace, metrics, SpanStatusCode, SpanKind } from '@opentelemetry/api';

export interface EducationalContext {
  institutionId?: string;
  institutionType?: 'K12' | 'University' | 'CommunityCollege' | 'TrainingCenter';
  academicLevel?: 'Elementary' | 'MiddleSchool' | 'HighSchool' | 'Undergraduate' | 'Graduate' | 'Professional';
  region?: 'US' | 'India' | 'Europe' | 'Asia' | 'Global';
  studentCount?: number;
  isExamPeriod?: boolean;
  academicCalendarPhase?: 'Enrollment' | 'Classes' | 'Midterms' | 'Finals' | 'Break' | 'Graduation';
  courseCatalogSize?: number;
  peakHours?: string[];
}

export interface PerformanceMetrics {
  responseTime: number;
  memoryUsage: number;
  cpuUsage: number;
  activeUsers: number;
  concurrentSessions: number;
  errorRate: number;
  throughput: number;
  databaseConnections: number;
}

export interface StudentExperienceMetrics {
  averagePageLoadTime: number;
  searchResponseTime: number;
  collaborationLatency: number;
  videoStreamingQuality: number;
  documentSyncTime: number;
  satisfactionScore: number;
  completionRate: number;
  engagementDuration: number;
}

class SISAILabTelemetry {
  private tracer = trace.getTracer('sis-ai-lab', '1.0.0');
  private meter = metrics.getMeter('sis-ai-lab', '1.0.0');
  
  // Educational metrics
  private readonly studentExperienceGauge = this.meter.createGauge('sis_student_experience', {
    description: 'Student experience quality metrics'
  });
  
  private readonly institutionMetricsGauge = this.meter.createGauge('sis_institution_metrics', {
    description: 'Institution-specific metrics'
  });
  
  private readonly academicCalendarGauge = this.meter.createGauge('sis_academic_calendar', {
    description: 'Academic calendar context metrics'
  });
  
  // Performance metrics
  private readonly performanceCounter = this.meter.createCounter('sis_performance_events', {
    description: 'Performance-related events'
  });
  
  private readonly responseTimeHistogram = this.meter.createHistogram('sis_response_time', {
    description: 'Response time distribution',
    unit: 'ms'
  });
  
  private readonly activeUsersGauge = this.meter.createGauge('sis_active_users', {
    description: 'Currently active users'
  });
  
  // ML and AI metrics
  private readonly mlPredictionCounter = this.meter.createCounter('sis_ml_predictions', {
    description: 'ML model predictions made'
  });
  
  private readonly aiopsIncidentCounter = this.meter.createCounter('sis_aiops_incidents', {
    description: 'AIOps incidents detected and resolved'
  });
  
  private readonly autoScalingCounter = this.meter.createCounter('sis_auto_scaling_events', {
    description: 'Auto-scaling events triggered'
  });

  async initialize(): Promise<void> {
    // Initialize basic telemetry
    console.log('SIS AI-Lab telemetry initialized successfully');
    
    // Note: Full OpenTelemetry SDK initialization would happen in a Node.js environment
    // For browser environments, we use the basic API for manual instrumentation
  }

  // Educational context tracing
  async traceEducationalOperation<T>(
    operationName: string,
    educationalContext: EducationalContext,
    operation: () => Promise<T>
  ): Promise<T> {
    const span = this.tracer.startSpan(operationName, {
      kind: SpanKind.INTERNAL,
      attributes: {
        'operation.type': 'educational',
        'educational.institution_id': educationalContext.institutionId,
        'educational.institution_type': educationalContext.institutionType,
        'educational.academic_level': educationalContext.academicLevel,
        'educational.region': educationalContext.region,
        'educational.student_count': educationalContext.studentCount,
        'educational.is_exam_period': educationalContext.isExamPeriod,
        'educational.academic_phase': educationalContext.academicCalendarPhase,
        'educational.catalog_size': educationalContext.courseCatalogSize
      }
    });

    try {
      const startTime = Date.now();
      const result = await operation();
      const duration = Date.now() - startTime;
      
      span.setAttributes({
        'operation.duration_ms': duration,
        'operation.success': true
      });
      
      span.setStatus({ code: SpanStatusCode.OK });
      
      // Record educational metrics
      this.recordEducationalMetrics(educationalContext, duration);
      
      return result;
    } catch (error) {
      span.setAttributes({
        'operation.error': true,
        'error.message': error instanceof Error ? error.message : 'Unknown error'
      });
      
      span.setStatus({
        code: SpanStatusCode.ERROR,
        message: error instanceof Error ? error.message : 'Unknown error'
      });
      
      throw error;
    } finally {
      span.end();
    }
  }

  // Student experience metrics
  recordStudentExperience(metrics: StudentExperienceMetrics, context: EducationalContext): void {
    this.studentExperienceGauge.record(metrics.averagePageLoadTime, {
      metric_type: 'page_load_time',
      institution_type: context.institutionType || 'Unknown',
      region: context.region || 'Global'
    });
    
    this.studentExperienceGauge.record(metrics.searchResponseTime, {
      metric_type: 'search_response_time',
      institution_type: context.institutionType || 'Unknown',
      region: context.region || 'Global'
    });
    
    this.studentExperienceGauge.record(metrics.collaborationLatency, {
      metric_type: 'collaboration_latency',
      institution_type: context.institutionType || 'Unknown',
      region: context.region || 'Global'
    });
    
    this.studentExperienceGauge.record(metrics.satisfactionScore, {
      metric_type: 'satisfaction_score',
      institution_type: context.institutionType || 'Unknown',
      academic_phase: context.academicCalendarPhase || 'Unknown'
    });
    
    this.studentExperienceGauge.record(metrics.engagementDuration, {
      metric_type: 'engagement_duration',
      institution_type: context.institutionType || 'Unknown',
      is_exam_period: context.isExamPeriod ? 'true' : 'false'
    });
  }

  // Performance monitoring
  recordPerformanceMetrics(metrics: PerformanceMetrics, context: EducationalContext): void {
    this.responseTimeHistogram.record(metrics.responseTime, {
      endpoint_type: 'educational',
      institution_type: context.institutionType || 'Unknown',
      region: context.region || 'Global'
    });
    
    this.activeUsersGauge.record(metrics.activeUsers, {
      institution_type: context.institutionType || 'Unknown',
      academic_phase: context.academicCalendarPhase || 'Unknown'
    });
    
    this.performanceCounter.add(1, {
      metric_type: 'response_time_recorded',
      performance_tier: this.getPerformanceTier(metrics.responseTime),
      institution_type: context.institutionType || 'Unknown'
    });
  }

  // ML operations tracing
  async traceMLPrediction<T>(
    modelName: string,
    predictionType: string,
    educationalContext: EducationalContext,
    prediction: () => Promise<T>
  ): Promise<T> {
    const span = this.tracer.startSpan(`ml_prediction_${modelName}`, {
      kind: SpanKind.INTERNAL,
      attributes: {
        'ml.model_name': modelName,
        'ml.prediction_type': predictionType,
        'ml.educational_context': true,
        'educational.institution_type': educationalContext.institutionType,
        'educational.region': educationalContext.region,
        'educational.student_count': educationalContext.studentCount
      }
    });

    try {
      const startTime = Date.now();
      const result = await prediction();
      const duration = Date.now() - startTime;
      
      span.setAttributes({
        'ml.prediction_duration_ms': duration,
        'ml.prediction_success': true
      });
      
      this.mlPredictionCounter.add(1, {
        model_name: modelName,
        prediction_type: predictionType,
        institution_type: educationalContext.institutionType || 'Unknown',
        success: 'true'
      });
      
      span.setStatus({ code: SpanStatusCode.OK });
      return result;
    } catch (error) {
      span.setAttributes({
        'ml.prediction_error': true,
        'error.message': error instanceof Error ? error.message : 'Unknown error'
      });
      
      this.mlPredictionCounter.add(1, {
        model_name: modelName,
        prediction_type: predictionType,
        institution_type: educationalContext.institutionType || 'Unknown',
        success: 'false'
      });
      
      span.setStatus({
        code: SpanStatusCode.ERROR,
        message: error instanceof Error ? error.message : 'Unknown error'
      });
      
      throw error;
    } finally {
      span.end();
    }
  }

  // AIOps incident tracking
  recordAIOpsIncident(
    incidentType: string,
    severity: 'low' | 'medium' | 'high' | 'critical',
    resolved: boolean,
    resolutionTimeMs?: number,
    educationalImpact?: 'none' | 'low' | 'medium' | 'high'
  ): void {
    this.aiopsIncidentCounter.add(1, {
      incident_type: incidentType,
      severity: severity,
      resolved: resolved ? 'true' : 'false',
      educational_impact: educationalImpact || 'none'
    });
    
    if (resolved && resolutionTimeMs) {
      this.responseTimeHistogram.record(resolutionTimeMs, {
        operation_type: 'aiops_resolution',
        incident_type: incidentType,
        severity: severity
      });
    }
  }

  // Auto-scaling events
  recordAutoScalingEvent(
    componentName: string,
    scalingDirection: 'up' | 'down',
    instancesBefore: number,
    instancesAfter: number,
    educationalContext: EducationalContext,
    reason: string
  ): void {
    this.autoScalingCounter.add(1, {
      component: componentName,
      direction: scalingDirection,
      reason: reason,
      institution_type: educationalContext.institutionType || 'Unknown',
      academic_phase: educationalContext.academicCalendarPhase || 'Unknown'
    });
    
    const span = this.tracer.startSpan('auto_scaling_event', {
      attributes: {
        'scaling.component': componentName,
        'scaling.direction': scalingDirection,
        'scaling.instances_before': instancesBefore,
        'scaling.instances_after': instancesAfter,
        'scaling.reason': reason,
        'educational.institution_type': educationalContext.institutionType,
        'educational.academic_phase': educationalContext.academicCalendarPhase
      }
    });
    
    span.end();
  }

  private recordEducationalMetrics(context: EducationalContext, operationDurationMs: number): void {
    this.institutionMetricsGauge.record(context.studentCount || 0, {
      metric_type: 'student_count',
      institution_type: context.institutionType || 'Unknown',
      region: context.region || 'Global'
    });
    
    this.academicCalendarGauge.record(operationDurationMs, {
      metric_type: 'operation_duration',
      academic_phase: context.academicCalendarPhase || 'Unknown',
      is_exam_period: context.isExamPeriod ? 'true' : 'false'
    });
  }

  private getPerformanceTier(responseTime: number): string {
    if (responseTime < 100) return 'excellent';
    if (responseTime < 300) return 'good';
    if (responseTime < 1000) return 'acceptable';
    return 'poor';
  }

  // Create a span for any generic operation
  createSpan(name: string, attributes?: Record<string, string | number | boolean>) {
    return this.tracer.startSpan(name, { attributes });
  }

  // Get the current tracer for manual instrumentation
  getTracer() {
    return this.tracer;
  }

  // Get the meter for custom metrics
  getMeter() {
    return this.meter;
  }
}

// Export singleton instance
export const telemetry = new SISAILabTelemetry();

// Initialize telemetry on module load
telemetry.initialize().catch(console.error);

export default telemetry;