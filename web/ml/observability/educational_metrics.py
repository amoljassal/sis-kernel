"""
Educational metrics collection and analysis for SIS AI-Lab platform.
Provides specialized metrics for educational institutions and student experience monitoring.
"""

import asyncio
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import json

import pandas as pd
import numpy as np
from prometheus_client import Counter, Histogram, Gauge, CollectorRegistry, generate_latest
from opentelemetry import trace, metrics
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.exporter.otlp.proto.grpc.metric_exporter import OTLPMetricExporter
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
from opentelemetry.sdk.resources import Resource

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class InstitutionType(Enum):
    K12 = "K12"
    UNIVERSITY = "University"
    COMMUNITY_COLLEGE = "CommunityCollege"
    TRAINING_CENTER = "TrainingCenter"

class AcademicLevel(Enum):
    ELEMENTARY = "Elementary"
    MIDDLE_SCHOOL = "MiddleSchool"
    HIGH_SCHOOL = "HighSchool"
    UNDERGRADUATE = "Undergraduate"
    GRADUATE = "Graduate"
    PROFESSIONAL = "Professional"

class AcademicPhase(Enum):
    ENROLLMENT = "Enrollment"
    CLASSES = "Classes"
    MIDTERMS = "Midterms"
    FINALS = "Finals"
    BREAK = "Break"
    GRADUATION = "Graduation"

@dataclass
class EducationalContext:
    institution_id: str
    institution_type: InstitutionType
    academic_level: AcademicLevel
    region: str
    student_count: int
    is_exam_period: bool = False
    academic_phase: AcademicPhase = AcademicPhase.CLASSES
    course_catalog_size: int = 0
    peak_hours: List[int] = None
    timezone: str = "UTC"

@dataclass
class StudentExperienceMetrics:
    average_page_load_time: float
    search_response_time: float
    collaboration_latency: float
    video_streaming_quality: float
    document_sync_time: float
    satisfaction_score: float
    completion_rate: float
    engagement_duration: float
    feature_adoption_rate: float
    help_desk_tickets: int
    error_rate: float
    timestamp: datetime = None

    def __post_init__(self):
        if self.timestamp is None:
            self.timestamp = datetime.utcnow()

@dataclass
class InstitutionalMetrics:
    active_users: int
    concurrent_sessions: int
    total_courses: int
    total_assignments: int
    collaboration_sessions: int
    storage_usage_gb: float
    bandwidth_usage_gbps: float
    cost_per_student: float
    teacher_student_ratio: float
    resource_utilization: float
    timestamp: datetime = None

    def __post_init__(self):
        if self.timestamp is None:
            self.timestamp = datetime.utcnow()

class EducationalMetricsCollector:
    """Specialized metrics collector for educational institutions."""
    
    def __init__(self):
        # Initialize OpenTelemetry
        self._setup_telemetry()
        
        # Create Prometheus metrics registry
        self.registry = CollectorRegistry()
        
        # Student experience metrics
        self.student_page_load_time = Histogram(
            'sis_student_page_load_time_seconds',
            'Page load time for students',
            ['institution_type', 'academic_level', 'region'],
            registry=self.registry
        )
        
        self.student_satisfaction = Gauge(
            'sis_student_satisfaction_score',
            'Student satisfaction score (0-10)',
            ['institution_type', 'academic_phase', 'region'],
            registry=self.registry
        )
        
        self.collaboration_latency = Histogram(
            'sis_collaboration_latency_seconds',
            'Real-time collaboration latency',
            ['institution_type', 'feature_type', 'region'],
            registry=self.registry
        )
        
        self.feature_adoption = Gauge(
            'sis_feature_adoption_rate',
            'Feature adoption rate (0-1)',
            ['institution_type', 'feature_name', 'academic_level'],
            registry=self.registry
        )
        
        # Institutional metrics
        self.active_students = Gauge(
            'sis_active_students_total',
            'Total active students',
            ['institution_type', 'academic_phase', 'region'],
            registry=self.registry
        )
        
        self.concurrent_sessions = Gauge(
            'sis_concurrent_sessions_total',
            'Concurrent user sessions',
            ['institution_type', 'session_type', 'region'],
            registry=self.registry
        )
        
        self.resource_utilization = Gauge(
            'sis_resource_utilization_ratio',
            'Resource utilization ratio (0-1)',
            ['institution_type', 'resource_type', 'region'],
            registry=self.registry
        )
        
        self.cost_per_student = Gauge(
            'sis_cost_per_student_usd',
            'Cost per student in USD',
            ['institution_type', 'cost_category', 'region'],
            registry=self.registry
        )
        
        # Academic calendar metrics
        self.academic_calendar = Gauge(
            'sis_academic_calendar_phase',
            'Current academic calendar phase',
            ['institution_type', 'phase', 'region'],
            registry=self.registry
        )
        
        self.exam_period_intensity = Gauge(
            'sis_exam_period_intensity',
            'Exam period intensity factor',
            ['institution_type', 'academic_level', 'region'],
            registry=self.registry
        )
        
        # Educational outcome metrics
        self.assignment_completion = Gauge(
            'sis_assignment_completion_rate',
            'Assignment completion rate (0-1)',
            ['institution_type', 'academic_level', 'subject'],
            registry=self.registry
        )
        
        self.collaboration_effectiveness = Gauge(
            'sis_collaboration_effectiveness_score',
            'Collaboration effectiveness score (0-10)',
            ['institution_type', 'collaboration_type', 'region'],
            registry=self.registry
        )
        
        # Operational metrics
        self.educational_errors = Counter(
            'sis_educational_errors_total',
            'Educational-specific errors',
            ['institution_type', 'error_type', 'severity'],
            registry=self.registry
        )
        
        self.help_desk_tickets = Counter(
            'sis_help_desk_tickets_total',
            'Help desk tickets created',
            ['institution_type', 'ticket_category', 'priority'],
            registry=self.registry
        )
        
        # Initialize metrics history
        self.metrics_history: List[Dict[str, Any]] = []
        self.context_history: List[EducationalContext] = []
        
    def _setup_telemetry(self):
        """Setup OpenTelemetry for distributed tracing and metrics."""
        # Create resource with educational attributes
        resource = Resource.create({
            "service.name": "sis-educational-metrics",
            "service.version": "1.0.0",
            "service.namespace": "sis-ai-lab",
            "educational.platform": "sis-ai-lab",
            "educational.component": "metrics-collector"
        })
        
        # Setup tracing
        trace.set_tracer_provider(TracerProvider(resource=resource))
        tracer_provider = trace.get_tracer_provider()
        
        otlp_exporter = OTLPSpanExporter(
            endpoint="http://localhost:4317",
            insecure=True
        )
        
        span_processor = BatchSpanProcessor(otlp_exporter)
        tracer_provider.add_span_processor(span_processor)
        
        self.tracer = trace.get_tracer(__name__)
        
        # Setup metrics
        metric_reader = PeriodicExportingMetricReader(
            OTLPMetricExporter(endpoint="http://localhost:4317", insecure=True),
            export_interval_millis=10000
        )
        
        metrics.set_meter_provider(MeterProvider(
            resource=resource,
            metric_readers=[metric_reader]
        ))
        
        self.meter = metrics.get_meter(__name__)
    
    async def collect_student_experience_metrics(
        self, 
        context: EducationalContext, 
        metrics: StudentExperienceMetrics
    ):
        """Collect and record student experience metrics."""
        with self.tracer.start_as_current_span("collect_student_experience_metrics") as span:
            span.set_attributes({
                "institution.type": context.institution_type.value,
                "institution.id": context.institution_id,
                "academic.level": context.academic_level.value,
                "academic.phase": context.academic_phase.value,
                "region": context.region,
                "student.count": context.student_count
            })
            
            # Record page load time
            self.student_page_load_time.labels(
                institution_type=context.institution_type.value,
                academic_level=context.academic_level.value,
                region=context.region
            ).observe(metrics.average_page_load_time)
            
            # Record satisfaction score
            self.student_satisfaction.labels(
                institution_type=context.institution_type.value,
                academic_phase=context.academic_phase.value,
                region=context.region
            ).set(metrics.satisfaction_score)
            
            # Record collaboration latency
            self.collaboration_latency.labels(
                institution_type=context.institution_type.value,
                feature_type="real_time_collaboration",
                region=context.region
            ).observe(metrics.collaboration_latency)
            
            # Record feature adoption
            self.feature_adoption.labels(
                institution_type=context.institution_type.value,
                feature_name="collaboration_tools",
                academic_level=context.academic_level.value
            ).set(metrics.feature_adoption_rate)
            
            # Store in history
            self.metrics_history.append({
                "type": "student_experience",
                "context": asdict(context),
                "metrics": asdict(metrics),
                "timestamp": datetime.utcnow().isoformat()
            })
            
            logger.info(f"Collected student experience metrics for {context.institution_id}")
    
    async def collect_institutional_metrics(
        self, 
        context: EducationalContext, 
        metrics: InstitutionalMetrics
    ):
        """Collect and record institutional-level metrics."""
        with self.tracer.start_as_current_span("collect_institutional_metrics") as span:
            span.set_attributes({
                "institution.type": context.institution_type.value,
                "institution.id": context.institution_id,
                "active.users": metrics.active_users,
                "concurrent.sessions": metrics.concurrent_sessions
            })
            
            # Record active students
            self.active_students.labels(
                institution_type=context.institution_type.value,
                academic_phase=context.academic_phase.value,
                region=context.region
            ).set(metrics.active_users)
            
            # Record concurrent sessions
            self.concurrent_sessions.labels(
                institution_type=context.institution_type.value,
                session_type="educational",
                region=context.region
            ).set(metrics.concurrent_sessions)
            
            # Record resource utilization
            self.resource_utilization.labels(
                institution_type=context.institution_type.value,
                resource_type="compute",
                region=context.region
            ).set(metrics.resource_utilization)
            
            # Record cost per student
            self.cost_per_student.labels(
                institution_type=context.institution_type.value,
                cost_category="total",
                region=context.region
            ).set(metrics.cost_per_student)
            
            # Store in history
            self.metrics_history.append({
                "type": "institutional",
                "context": asdict(context),
                "metrics": asdict(metrics),
                "timestamp": datetime.utcnow().isoformat()
            })
            
            logger.info(f"Collected institutional metrics for {context.institution_id}")
    
    async def record_academic_calendar_event(
        self, 
        context: EducationalContext, 
        event_type: str, 
        intensity: float = 1.0
    ):
        """Record academic calendar-related events."""
        with self.tracer.start_as_current_span("record_academic_calendar_event") as span:
            span.set_attributes({
                "calendar.event_type": event_type,
                "calendar.phase": context.academic_phase.value,
                "calendar.intensity": intensity,
                "is_exam_period": context.is_exam_period
            })
            
            # Record academic phase
            self.academic_calendar.labels(
                institution_type=context.institution_type.value,
                phase=context.academic_phase.value,
                region=context.region
            ).set(1.0)
            
            # Record exam period intensity
            if context.is_exam_period:
                self.exam_period_intensity.labels(
                    institution_type=context.institution_type.value,
                    academic_level=context.academic_level.value,
                    region=context.region
                ).set(intensity)
            
            logger.info(f"Recorded academic calendar event: {event_type} for {context.institution_id}")
    
    async def analyze_educational_trends(
        self, 
        time_range_hours: int = 24
    ) -> Dict[str, Any]:
        """Analyze educational trends over a specified time range."""
        with self.tracer.start_as_current_span("analyze_educational_trends") as span:
            span.set_attributes({
                "analysis.time_range_hours": time_range_hours
            })
            
            cutoff_time = datetime.utcnow() - timedelta(hours=time_range_hours)
            recent_metrics = [
                m for m in self.metrics_history 
                if datetime.fromisoformat(m["timestamp"]) > cutoff_time
            ]
            
            if not recent_metrics:
                return {"status": "no_data", "time_range_hours": time_range_hours}
            
            # Analyze student experience trends
            student_metrics = [m for m in recent_metrics if m["type"] == "student_experience"]
            institutional_metrics = [m for m in recent_metrics if m["type"] == "institutional"]
            
            analysis = {
                "time_range_hours": time_range_hours,
                "total_samples": len(recent_metrics),
                "student_experience": self._analyze_student_trends(student_metrics),
                "institutional": self._analyze_institutional_trends(institutional_metrics),
                "recommendations": self._generate_recommendations(recent_metrics)
            }
            
            logger.info(f"Completed educational trends analysis for {time_range_hours} hours")
            return analysis
    
    def _analyze_student_trends(self, metrics: List[Dict]) -> Dict[str, Any]:
        """Analyze student experience trends."""
        if not metrics:
            return {"status": "no_data"}
        
        satisfaction_scores = [m["metrics"]["satisfaction_score"] for m in metrics]
        page_load_times = [m["metrics"]["average_page_load_time"] for m in metrics]
        engagement_durations = [m["metrics"]["engagement_duration"] for m in metrics]
        
        return {
            "satisfaction": {
                "average": np.mean(satisfaction_scores),
                "trend": "improving" if len(satisfaction_scores) > 1 and satisfaction_scores[-1] > satisfaction_scores[0] else "declining",
                "min": np.min(satisfaction_scores),
                "max": np.max(satisfaction_scores)
            },
            "performance": {
                "avg_page_load": np.mean(page_load_times),
                "performance_trend": "improving" if len(page_load_times) > 1 and page_load_times[-1] < page_load_times[0] else "declining"
            },
            "engagement": {
                "avg_duration": np.mean(engagement_durations),
                "engagement_trend": "increasing" if len(engagement_durations) > 1 and engagement_durations[-1] > engagement_durations[0] else "decreasing"
            }
        }
    
    def _analyze_institutional_trends(self, metrics: List[Dict]) -> Dict[str, Any]:
        """Analyze institutional trends."""
        if not metrics:
            return {"status": "no_data"}
        
        active_users = [m["metrics"]["active_users"] for m in metrics]
        resource_utilization = [m["metrics"]["resource_utilization"] for m in metrics]
        cost_per_student = [m["metrics"]["cost_per_student"] for m in metrics]
        
        return {
            "usage": {
                "avg_active_users": np.mean(active_users),
                "usage_trend": "increasing" if len(active_users) > 1 and active_users[-1] > active_users[0] else "decreasing",
                "peak_usage": np.max(active_users)
            },
            "resources": {
                "avg_utilization": np.mean(resource_utilization),
                "utilization_trend": "increasing" if len(resource_utilization) > 1 and resource_utilization[-1] > resource_utilization[0] else "decreasing"
            },
            "cost": {
                "avg_cost_per_student": np.mean(cost_per_student),
                "cost_trend": "increasing" if len(cost_per_student) > 1 and cost_per_student[-1] > cost_per_student[0] else "decreasing"
            }
        }
    
    def _generate_recommendations(self, metrics: List[Dict]) -> List[str]:
        """Generate recommendations based on metric analysis."""
        recommendations = []
        
        # Extract recent metrics for analysis
        recent_student_metrics = [m for m in metrics if m["type"] == "student_experience"]
        recent_institutional_metrics = [m for m in metrics if m["type"] == "institutional"]
        
        if recent_student_metrics:
            avg_satisfaction = np.mean([m["metrics"]["satisfaction_score"] for m in recent_student_metrics])
            avg_page_load = np.mean([m["metrics"]["average_page_load_time"] for m in recent_student_metrics])
            
            if avg_satisfaction < 7.0:
                recommendations.append("Student satisfaction is below optimal (7.0). Consider investigating user experience issues.")
            
            if avg_page_load > 3.0:
                recommendations.append("Page load times are high (>3s). Consider implementing performance optimizations.")
        
        if recent_institutional_metrics:
            avg_utilization = np.mean([m["metrics"]["resource_utilization"] for m in recent_institutional_metrics])
            avg_cost = np.mean([m["metrics"]["cost_per_student"] for m in recent_institutional_metrics])
            
            if avg_utilization > 0.8:
                recommendations.append("Resource utilization is high (>80%). Consider scaling up infrastructure.")
            
            if avg_cost > 100:
                recommendations.append("Cost per student is high (>$100). Consider cost optimization strategies.")
        
        return recommendations
    
    def get_metrics_export(self) -> str:
        """Export metrics in Prometheus format."""
        return generate_latest(self.registry)
    
    async def start_continuous_collection(self, interval_seconds: int = 60):
        """Start continuous metrics collection."""
        logger.info(f"Starting continuous metrics collection every {interval_seconds} seconds")
        
        while True:
            try:
                # This would typically be called by external systems
                # For demo purposes, we'll simulate some data collection
                await asyncio.sleep(interval_seconds)
                logger.debug("Metrics collection cycle completed")
            except Exception as e:
                logger.error(f"Error in continuous collection: {e}")
                await asyncio.sleep(interval_seconds)

# Global instance
educational_metrics = EducationalMetricsCollector()

if __name__ == "__main__":
    # Example usage
    import asyncio
    
    async def demo():
        # Create sample context
        context = EducationalContext(
            institution_id="demo_university",
            institution_type=InstitutionType.UNIVERSITY,
            academic_level=AcademicLevel.UNDERGRADUATE,
            region="US",
            student_count=5000,
            is_exam_period=True,
            academic_phase=AcademicPhase.FINALS
        )
        
        # Create sample student metrics
        student_metrics = StudentExperienceMetrics(
            average_page_load_time=2.1,
            search_response_time=0.8,
            collaboration_latency=0.3,
            video_streaming_quality=8.5,
            document_sync_time=1.2,
            satisfaction_score=8.2,
            completion_rate=0.85,
            engagement_duration=45.6,
            feature_adoption_rate=0.78,
            help_desk_tickets=3,
            error_rate=0.02
        )
        
        # Create sample institutional metrics
        institutional_metrics = InstitutionalMetrics(
            active_users=1250,
            concurrent_sessions=890,
            total_courses=450,
            total_assignments=2800,
            collaboration_sessions=340,
            storage_usage_gb=850.5,
            bandwidth_usage_gbps=2.3,
            cost_per_student=75.50,
            teacher_student_ratio=0.15,
            resource_utilization=0.72
        )
        
        # Collect metrics
        await educational_metrics.collect_student_experience_metrics(context, student_metrics)
        await educational_metrics.collect_institutional_metrics(context, institutional_metrics)
        await educational_metrics.record_academic_calendar_event(context, "finals_week", intensity=1.5)
        
        # Analyze trends
        analysis = await educational_metrics.analyze_educational_trends(24)
        print("Educational Trends Analysis:")
        print(json.dumps(analysis, indent=2))
        
        # Export metrics
        metrics_export = educational_metrics.get_metrics_export()
        print("\nPrometheus Metrics Export:")
        print(metrics_export[:500] + "..." if len(metrics_export) > 500 else metrics_export)
    
    asyncio.run(demo())