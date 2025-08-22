#!/usr/bin/env python3
"""
SIS AI-Lab Predictive Auto-Scaling System
Academic-Aware Resource Management for Educational Platform

Multi-AI Synthesis Features:
- Academic calendar integration (Grok's educational awareness)
- Cost-optimized scaling decisions (ChatGPT's operational focus)
- Predictive resource allocation (Claude's intelligent planning)
- Multi-region coordination (Gemini's global architecture)
"""

import asyncio
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Tuple
from dataclasses import dataclass, asdict
from enum import Enum
import json
import numpy as np
import pandas as pd
from sklearn.preprocessing import StandardScaler
import holidays

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class ScalingDirection(Enum):
    UP = "up"
    DOWN = "down"
    STABLE = "stable"

class ResourceType(Enum):
    CPU = "cpu"
    MEMORY = "memory"
    STORAGE = "storage"
    NETWORK = "network"

class ServiceComponent(Enum):
    AI_GATEWAY = "ai-gateway"
    DATABASE = "database"
    WEBSOCKET = "websocket-gateway"
    CDN = "cdn"
    LOAD_BALANCER = "load-balancer"
    COLLABORATION = "collaboration-service"

@dataclass
class ScalingDecision:
    timestamp: datetime
    component: ServiceComponent
    current_instances: int
    target_instances: int
    direction: ScalingDirection
    confidence: float
    reasoning: str
    educational_context: str
    cost_impact: float
    execution_time: Optional[datetime] = None

@dataclass
class ResourcePrediction:
    timestamp: datetime
    component: ServiceComponent
    predicted_load: float
    confidence: float
    recommended_instances: int
    educational_context: Dict[str, Any]

@dataclass
class AcademicCalendar:
    semester_start: datetime
    semester_end: datetime
    exam_periods: List[Tuple[datetime, datetime]]
    holidays: List[Tuple[datetime, datetime]]
    peak_assignment_dates: List[datetime]
    graduation_dates: List[datetime]

class EducationalContextAnalyzer:
    """
    Analyze educational context to inform scaling decisions
    """
    
    def __init__(self):
        self.academic_calendars = {}
        self.regional_holidays = {}
        self._initialize_calendars()
    
    def _initialize_calendars(self):
        """
        Initialize academic calendars for different regions
        """
        current_year = datetime.now().year
        
        # US Academic Calendar (Fall semester)
        self.academic_calendars['us'] = AcademicCalendar(
            semester_start=datetime(current_year, 8, 25),
            semester_end=datetime(current_year, 12, 15),
            exam_periods=[
                (datetime(current_year, 10, 15), datetime(current_year, 10, 22)),  # Midterms
                (datetime(current_year, 12, 8), datetime(current_year, 12, 15))    # Finals
            ],
            holidays=[
                (datetime(current_year, 11, 23), datetime(current_year, 11, 29)),  # Thanksgiving
                (datetime(current_year, 12, 20), datetime(current_year + 1, 1, 15)) # Winter break
            ],
            peak_assignment_dates=[
                datetime(current_year, 9, 30),   # End of September
                datetime(current_year, 11, 15),  # Mid November
                datetime(current_year, 12, 1)    # Early December
            ],
            graduation_dates=[datetime(current_year, 12, 16)]
        )
        
        # India Academic Calendar
        self.academic_calendars['india'] = AcademicCalendar(
            semester_start=datetime(current_year, 7, 1),
            semester_end=datetime(current_year, 11, 30),
            exam_periods=[
                (datetime(current_year, 9, 15), datetime(current_year, 9, 30)),   # Mid-semester
                (datetime(current_year, 11, 15), datetime(current_year, 11, 30))  # End-semester
            ],
            holidays=[
                (datetime(current_year, 8, 15), datetime(current_year, 8, 15)),   # Independence Day
                (datetime(current_year, 10, 2), datetime(current_year, 10, 2)),   # Gandhi Jayanti
                (datetime(current_year, 10, 20), datetime(current_year, 10, 30))  # Diwali break
            ],
            peak_assignment_dates=[
                datetime(current_year, 8, 31),
                datetime(current_year, 10, 15),
                datetime(current_year, 11, 10)
            ],
            graduation_dates=[datetime(current_year, 12, 1)]
        )
        
        # Initialize regional holidays
        self.regional_holidays['us'] = holidays.US(years=current_year)
        self.regional_holidays['india'] = holidays.India(years=current_year)
    
    def get_educational_context(self, timestamp: datetime, region: str = 'us') -> Dict[str, Any]:
        """
        Get educational context for a given timestamp and region
        """
        calendar = self.academic_calendars.get(region)
        if not calendar:
            return {"context": "unknown", "intensity": 0.5}
        
        context = {
            "is_semester_active": calendar.semester_start <= timestamp <= calendar.semester_end,
            "is_exam_period": False,
            "is_holiday": False,
            "is_peak_assignment": False,
            "is_graduation": False,
            "days_to_exam": None,
            "academic_intensity": 0.5
        }
        
        # Check exam periods
        for exam_start, exam_end in calendar.exam_periods:
            if exam_start <= timestamp <= exam_end:
                context["is_exam_period"] = True
                context["academic_intensity"] = 1.0
                break
            elif timestamp < exam_start:
                days_to_exam = (exam_start - timestamp).days
                if days_to_exam <= 7:  # Week before exam
                    context["days_to_exam"] = days_to_exam
                    context["academic_intensity"] = 0.8 + (7 - days_to_exam) * 0.03
        
        # Check holidays
        for holiday_start, holiday_end in calendar.holidays:
            if holiday_start <= timestamp <= holiday_end:
                context["is_holiday"] = True
                context["academic_intensity"] = 0.2
                break
        
        # Check peak assignment dates
        for assignment_date in calendar.peak_assignment_dates:
            if abs((timestamp - assignment_date).days) <= 3:
                context["is_peak_assignment"] = True
                context["academic_intensity"] = max(context["academic_intensity"], 0.8)
        
        # Check graduation
        for grad_date in calendar.graduation_dates:
            if abs((timestamp - grad_date).days) <= 1:
                context["is_graduation"] = True
                context["academic_intensity"] = 0.9
        
        # Regional holidays
        regional_holiday = self.regional_holidays.get(region)
        if regional_holiday and timestamp.date() in regional_holiday:
            context["is_holiday"] = True
            context["academic_intensity"] = min(context["academic_intensity"], 0.3)
        
        return context

class CostOptimizer:
    """
    Optimize scaling decisions for cost efficiency
    """
    
    def __init__(self):
        # Instance costs per hour (example values)
        self.instance_costs = {
            ServiceComponent.AI_GATEWAY: 0.50,      # GPU instances
            ServiceComponent.DATABASE: 0.30,        # High-memory instances
            ServiceComponent.WEBSOCKET: 0.20,       # Standard instances
            ServiceComponent.CDN: 0.10,             # Edge locations
            ServiceComponent.LOAD_BALANCER: 0.15,   # Network appliances
            ServiceComponent.COLLABORATION: 0.25    # Real-time processing
        }
        
        # Regional cost multipliers
        self.regional_multipliers = {
            'us-east-1': 1.0,
            'us-west-2': 1.1,
            'eu-west-1': 1.2,
            'ap-south-1': 0.8,  # Mumbai - cost optimized
            'ap-southeast-1': 1.0
        }
    
    def calculate_cost_impact(self, decision: ScalingDecision, region: str = 'us-east-1') -> float:
        """
        Calculate hourly cost impact of scaling decision
        """
        base_cost = self.instance_costs.get(decision.component, 0.20)
        regional_multiplier = self.regional_multipliers.get(region, 1.0)
        
        current_cost = decision.current_instances * base_cost * regional_multiplier
        target_cost = decision.target_instances * base_cost * regional_multiplier
        
        return target_cost - current_cost
    
    def optimize_scaling_timing(self, predictions: List[ResourcePrediction], 
                              current_time: datetime) -> List[ScalingDecision]:
        """
        Optimize timing of scaling decisions for cost efficiency
        """
        decisions = []
        
        for prediction in predictions:
            # Calculate if we should scale now or wait
            cost_now = self.instance_costs.get(prediction.component, 0.20) * prediction.recommended_instances
            
            # Look ahead to see if demand will drop soon
            future_savings = 0
            if prediction.timestamp > current_time + timedelta(hours=2):
                # If prediction is for more than 2 hours away, consider delayed scaling
                delay_benefit = cost_now * 0.1  # 10% benefit for delayed scaling
                future_savings = delay_benefit
            
            # Educational context consideration
            educational_context = prediction.educational_context
            if educational_context.get('is_exam_period') or educational_context.get('is_peak_assignment'):
                # Scale proactively during critical educational periods
                execution_time = current_time + timedelta(minutes=5)
            elif educational_context.get('is_holiday'):
                # Delay scaling during holidays unless critical
                execution_time = current_time + timedelta(hours=1)
            else:
                # Normal scaling timing
                execution_time = current_time + timedelta(minutes=15)
            
            decision = ScalingDecision(
                timestamp=current_time,
                component=prediction.component,
                current_instances=0,  # Would be fetched from actual infrastructure
                target_instances=prediction.recommended_instances,
                direction=ScalingDirection.UP if prediction.recommended_instances > 0 else ScalingDirection.DOWN,
                confidence=prediction.confidence,
                reasoning=f"Predicted load: {prediction.predicted_load:.2f}",
                educational_context=str(educational_context),
                cost_impact=cost_now,
                execution_time=execution_time
            )
            
            decisions.append(decision)
        
        return decisions

class PredictiveAutoScaler:
    """
    Main predictive auto-scaling engine
    """
    
    def __init__(self, traffic_predictor=None):
        self.traffic_predictor = traffic_predictor
        self.context_analyzer = EducationalContextAnalyzer()
        self.cost_optimizer = CostOptimizer()
        self.scaling_history = []
        self.active_decisions = []
        
        # Scaling thresholds
        self.scaling_thresholds = {
            ServiceComponent.AI_GATEWAY: {
                'scale_up_cpu': 70,
                'scale_down_cpu': 30,
                'scale_up_memory': 80,
                'scale_down_memory': 40,
                'min_instances': 2,
                'max_instances': 20
            },
            ServiceComponent.DATABASE: {
                'scale_up_cpu': 75,
                'scale_down_cpu': 25,
                'scale_up_memory': 85,
                'scale_down_memory': 35,
                'min_instances': 2,
                'max_instances': 10
            },
            ServiceComponent.WEBSOCKET: {
                'scale_up_cpu': 65,
                'scale_down_cpu': 25,
                'scale_up_memory': 75,
                'scale_down_memory': 35,
                'min_instances': 3,
                'max_instances': 15
            },
            ServiceComponent.CDN: {
                'scale_up_cpu': 80,
                'scale_down_cpu': 20,
                'scale_up_memory': 70,
                'scale_down_memory': 30,
                'min_instances': 5,
                'max_instances': 50
            },
            ServiceComponent.LOAD_BALANCER: {
                'scale_up_cpu': 70,
                'scale_down_cpu': 30,
                'scale_up_memory': 75,
                'scale_down_memory': 40,
                'min_instances': 2,
                'max_instances': 8
            },
            ServiceComponent.COLLABORATION: {
                'scale_up_cpu': 65,
                'scale_down_cpu': 30,
                'scale_up_memory': 80,
                'scale_down_memory': 40,
                'min_instances': 2,
                'max_instances': 12
            }
        }
    
    def calculate_required_instances(self, component: ServiceComponent, 
                                   predicted_load: float, 
                                   educational_context: Dict[str, Any]) -> int:
        """
        Calculate required instances based on predicted load and educational context
        """
        thresholds = self.scaling_thresholds.get(component)
        if not thresholds:
            return 2  # Default minimum
        
        # Base calculation
        base_instances = max(2, int(predicted_load / 50) + 1)  # Rough heuristic
        
        # Educational context adjustments
        academic_intensity = educational_context.get('academic_intensity', 0.5)
        
        if educational_context.get('is_exam_period'):
            # 50% more capacity during exams
            base_instances = int(base_instances * 1.5)
        elif educational_context.get('is_peak_assignment'):
            # 30% more capacity during assignment peaks
            base_instances = int(base_instances * 1.3)
        elif educational_context.get('is_holiday'):
            # 60% capacity during holidays
            base_instances = int(base_instances * 0.6)
        else:
            # Scale based on academic intensity
            base_instances = int(base_instances * (0.8 + academic_intensity * 0.4))
        
        # Apply component-specific limits
        base_instances = max(thresholds['min_instances'], 
                           min(thresholds['max_instances'], base_instances))
        
        return base_instances
    
    async def generate_predictions(self, hours_ahead: int = 6, 
                                 region: str = 'us') -> List[ResourcePrediction]:
        """
        Generate resource predictions for the next N hours
        """
        predictions = []
        current_time = datetime.now()
        
        for i in range(hours_ahead):
            prediction_time = current_time + timedelta(hours=i)
            
            # Get educational context
            educational_context = self.context_analyzer.get_educational_context(
                prediction_time, region
            )
            
            # Get traffic predictions if available
            if self.traffic_predictor and self.traffic_predictor.is_trained:
                # Use ML model for traffic prediction
                current_data = {
                    'traffic_load': 45,  # Would be fetched from monitoring
                    'ai_requests': 90,
                    'total_requests': 450,
                    'collaboration_sessions': 25,
                    'total_users': 180,
                    'cpu_usage': 55,
                    'memory_usage': 65,
                    'active_users': 140,
                    'region': region
                }
                
                traffic_predictions = self.traffic_predictor.predict_next_hours(
                    current_data, 1
                )
                predicted_load = traffic_predictions[0]['predicted_traffic'] if traffic_predictions else 50
                confidence = traffic_predictions[0]['confidence'] if traffic_predictions else 0.7
            else:
                # Fallback to simple heuristic
                predicted_load = self._predict_load_heuristic(prediction_time, educational_context)
                confidence = 0.6
            
            # Generate predictions for each component
            for component in ServiceComponent:
                # Adjust load based on component characteristics
                component_load = self._adjust_load_for_component(component, predicted_load, educational_context)
                
                # Calculate recommended instances
                recommended_instances = self.calculate_required_instances(
                    component, component_load, educational_context
                )
                
                prediction = ResourcePrediction(
                    timestamp=prediction_time,
                    component=component,
                    predicted_load=component_load,
                    confidence=confidence,
                    recommended_instances=recommended_instances,
                    educational_context=educational_context
                )
                
                predictions.append(prediction)
        
        return predictions
    
    def _predict_load_heuristic(self, timestamp: datetime, 
                              educational_context: Dict[str, Any]) -> float:
        """
        Simple heuristic for load prediction when ML model is unavailable
        """
        hour = timestamp.hour
        day_of_week = timestamp.weekday()
        
        # Base load
        base_load = 30
        
        # Time-based patterns
        if 8 <= hour <= 18 and day_of_week < 5:  # School hours on weekdays
            base_load = 60
        
        # Peak hours
        if (9 <= hour <= 11) or (14 <= hour <= 16):
            base_load = 80
        
        # Educational context adjustments
        academic_intensity = educational_context.get('academic_intensity', 0.5)
        base_load *= (0.5 + academic_intensity)
        
        # Weekend reduction
        if day_of_week >= 5:
            base_load *= 0.4
        
        return max(10, base_load)
    
    def _adjust_load_for_component(self, component: ServiceComponent, 
                                 base_load: float, 
                                 educational_context: Dict[str, Any]) -> float:
        """
        Adjust load prediction for specific component characteristics
        """
        # Component-specific load patterns
        multipliers = {
            ServiceComponent.AI_GATEWAY: 1.2,      # High during AI-heavy activities
            ServiceComponent.DATABASE: 0.8,        # More stable load
            ServiceComponent.WEBSOCKET: 1.0,       # Proportional to users
            ServiceComponent.CDN: 0.6,             # Varies with content requests
            ServiceComponent.LOAD_BALANCER: 0.9,   # Slightly lower variance
            ServiceComponent.COLLABORATION: 1.3    # High during collaborative work
        }
        
        adjusted_load = base_load * multipliers.get(component, 1.0)
        
        # Special adjustments for educational context
        if educational_context.get('is_exam_period'):
            if component == ServiceComponent.AI_GATEWAY:
                adjusted_load *= 1.5  # Students use AI help more during exams
            elif component == ServiceComponent.COLLABORATION:
                adjusted_load *= 0.8  # Less collaboration during individual exams
        
        return adjusted_load
    
    async def make_scaling_decisions(self, region: str = 'us') -> List[ScalingDecision]:
        """
        Make intelligent scaling decisions based on predictions
        """
        logger.info("Generating scaling decisions...")
        
        # Get predictions
        predictions = await self.generate_predictions(hours_ahead=6, region=region)
        
        # Group predictions by component
        component_predictions = {}
        for pred in predictions:
            if pred.component not in component_predictions:
                component_predictions[pred.component] = []
            component_predictions[pred.component].append(pred)
        
        decisions = []
        current_time = datetime.now()
        
        for component, preds in component_predictions.items():
            # Find the prediction with highest load in next 2 hours
            near_future_preds = [p for p in preds if p.timestamp <= current_time + timedelta(hours=2)]
            
            if near_future_preds:
                max_load_pred = max(near_future_preds, key=lambda p: p.predicted_load)
                
                decision = ScalingDecision(
                    timestamp=current_time,
                    component=component,
                    current_instances=self.scaling_thresholds[component]['min_instances'],  # Would fetch actual
                    target_instances=max_load_pred.recommended_instances,
                    direction=ScalingDirection.UP if max_load_pred.recommended_instances > 2 else ScalingDirection.STABLE,
                    confidence=max_load_pred.confidence,
                    reasoning=f"Peak load predicted: {max_load_pred.predicted_load:.1f}",
                    educational_context=str(max_load_pred.educational_context),
                    cost_impact=0.0  # Will be calculated by cost optimizer
                )
                
                # Calculate cost impact
                decision.cost_impact = self.cost_optimizer.calculate_cost_impact(decision, region)
                
                decisions.append(decision)
        
        # Optimize timing and costs
        optimized_decisions = self.cost_optimizer.optimize_scaling_timing(
            [ResourcePrediction(
                timestamp=d.timestamp,
                component=d.component,
                predicted_load=0,  # Not used in optimization
                confidence=d.confidence,
                recommended_instances=d.target_instances,
                educational_context=eval(d.educational_context) if isinstance(d.educational_context, str) else {}
            ) for d in decisions],
            current_time
        )
        
        self.active_decisions = optimized_decisions
        return optimized_decisions
    
    async def execute_scaling_decisions(self, decisions: List[ScalingDecision]) -> Dict[str, Any]:
        """
        Execute scaling decisions (simulation)
        """
        results = {
            'executed': [],
            'failed': [],
            'total_cost_impact': 0.0
        }
        
        for decision in decisions:
            if decision.execution_time and decision.execution_time <= datetime.now():
                try:
                    # Simulate scaling action
                    logger.info(f"Scaling {decision.component.value}: {decision.current_instances} -> {decision.target_instances}")
                    
                    # In real implementation, call Kubernetes API or cloud provider APIs
                    await asyncio.sleep(0.1)  # Simulate API call
                    
                    results['executed'].append({
                        'component': decision.component.value,
                        'instances': decision.target_instances,
                        'cost_impact': decision.cost_impact
                    })
                    
                    results['total_cost_impact'] += decision.cost_impact
                    
                    # Record in history
                    self.scaling_history.append(decision)
                    
                except Exception as e:
                    logger.error(f"Failed to scale {decision.component.value}: {e}")
                    results['failed'].append({
                        'component': decision.component.value,
                        'error': str(e)
                    })
        
        return results
    
    def get_scaling_metrics(self) -> Dict[str, Any]:
        """
        Get metrics about scaling performance
        """
        if not self.scaling_history:
            return {"no_data": True}
        
        recent_decisions = [d for d in self.scaling_history 
                          if d.timestamp > datetime.now() - timedelta(hours=24)]
        
        total_cost_impact = sum(d.cost_impact for d in recent_decisions)
        avg_confidence = np.mean([d.confidence for d in recent_decisions]) if recent_decisions else 0
        
        component_scaling = {}
        for decision in recent_decisions:
            component = decision.component.value
            if component not in component_scaling:
                component_scaling[component] = {'up': 0, 'down': 0, 'stable': 0}
            component_scaling[component][decision.direction.value] += 1
        
        return {
            'total_decisions_24h': len(recent_decisions),
            'total_cost_impact_24h': total_cost_impact,
            'average_confidence': avg_confidence,
            'component_scaling': component_scaling,
            'active_decisions': len(self.active_decisions)
        }

async def main():
    """
    Example usage of Predictive Auto-Scaler
    """
    logger.info("Starting SIS AI-Lab Predictive Auto-Scaler")
    
    # Initialize the scaler
    scaler = PredictiveAutoScaler()
    
    # Generate scaling decisions
    decisions = await scaler.make_scaling_decisions(region='us')
    
    logger.info(f"Generated {len(decisions)} scaling decisions:")
    for decision in decisions:
        logger.info(f"  {decision.component.value}: {decision.target_instances} instances "
                   f"(confidence: {decision.confidence:.2f}, cost: ${decision.cost_impact:.2f}/hr)")
    
    # Simulate execution
    results = await scaler.execute_scaling_decisions(decisions)
    
    logger.info("Execution results:")
    logger.info(f"  Executed: {len(results['executed'])}")
    logger.info(f"  Failed: {len(results['failed'])}")
    logger.info(f"  Total cost impact: ${results['total_cost_impact']:.2f}/hr")
    
    # Display metrics
    metrics = scaler.get_scaling_metrics()
    logger.info(f"Scaling metrics: {metrics}")

if __name__ == "__main__":
    asyncio.run(main())