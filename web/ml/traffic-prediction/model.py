#!/usr/bin/env python3
"""
SIS AI-Lab Traffic Prediction Model
Educational Platform Traffic Pattern Analysis and Prediction

Multi-AI Synthesis Features:
- Educational time-based patterns (Grok's academic focus)
- Real-time adaptation (Claude's learning engines)
- Global scaling predictions (ChatGPT's operational excellence)
- Multi-tenant resource optimization (Gemini's hybrid architecture)
"""

import numpy as np
import pandas as pd
from sklearn.ensemble import RandomForestRegressor, GradientBoostingRegressor
from sklearn.preprocessing import StandardScaler, LabelEncoder
from sklearn.model_selection import train_test_split, cross_val_score
from sklearn.metrics import mean_absolute_error, mean_squared_error, r2_score
import joblib
import logging
from datetime import datetime, timedelta
import warnings
warnings.filterwarnings('ignore')

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class EducationalTrafficPredictor:
    """
    Educational traffic prediction model with academic schedule awareness
    """
    
    def __init__(self):
        self.rf_model = RandomForestRegressor(
            n_estimators=200,
            max_depth=15,
            random_state=42,
            n_jobs=-1
        )
        self.gb_model = GradientBoostingRegressor(
            n_estimators=150,
            max_depth=8,
            learning_rate=0.1,
            random_state=42
        )
        self.scaler = StandardScaler()
        self.label_encoders = {}
        self.is_trained = False
        
    def create_features(self, df):
        """
        Create educational-specific features for traffic prediction
        """
        features = df.copy()
        
        # Time-based features
        features['hour'] = pd.to_datetime(features['timestamp']).dt.hour
        features['day_of_week'] = pd.to_datetime(features['timestamp']).dt.dayofweek
        features['month'] = pd.to_datetime(features['timestamp']).dt.month
        features['quarter'] = pd.to_datetime(features['timestamp']).dt.quarter
        
        # Educational schedule features
        features['is_school_hours'] = ((features['hour'] >= 8) & (features['hour'] <= 18)).astype(int)
        features['is_weekend'] = (features['day_of_week'] >= 5).astype(int)
        features['is_exam_period'] = 0  # Will be enhanced with calendar integration
        features['is_holiday'] = 0      # Will be enhanced with calendar integration
        
        # Cyclical encoding for time features
        features['hour_sin'] = np.sin(2 * np.pi * features['hour'] / 24)
        features['hour_cos'] = np.cos(2 * np.pi * features['hour'] / 24)
        features['day_sin'] = np.sin(2 * np.pi * features['day_of_week'] / 7)
        features['day_cos'] = np.cos(2 * np.pi * features['day_of_week'] / 7)
        
        # Rolling statistics
        features['traffic_ma_6h'] = features['traffic_load'].rolling(window=6, min_periods=1).mean()
        features['traffic_ma_24h'] = features['traffic_load'].rolling(window=24, min_periods=1).mean()
        features['traffic_std_6h'] = features['traffic_load'].rolling(window=6, min_periods=1).std().fillna(0)
        
        # AI model usage patterns
        features['ai_requests_ratio'] = features['ai_requests'] / (features['total_requests'] + 1)
        features['collaboration_ratio'] = features['collaboration_sessions'] / (features['total_users'] + 1)
        
        # Regional features
        if 'region' in features.columns:
            if 'region' not in self.label_encoders:
                self.label_encoders['region'] = LabelEncoder()
                features['region_encoded'] = self.label_encoders['region'].fit_transform(features['region'])
            else:
                features['region_encoded'] = self.label_encoders['region'].transform(features['region'])
        
        return features
    
    def prepare_data(self, df):
        """
        Prepare training data with proper feature engineering
        """
        logger.info("Preparing training data...")
        
        # Create features
        features_df = self.create_features(df)
        
        # Select feature columns
        feature_columns = [
            'hour', 'day_of_week', 'month', 'quarter',
            'is_school_hours', 'is_weekend', 'is_exam_period', 'is_holiday',
            'hour_sin', 'hour_cos', 'day_sin', 'day_cos',
            'traffic_ma_6h', 'traffic_ma_24h', 'traffic_std_6h',
            'ai_requests_ratio', 'collaboration_ratio',
            'cpu_usage', 'memory_usage', 'active_users'
        ]
        
        if 'region_encoded' in features_df.columns:
            feature_columns.append('region_encoded')
        
        # Handle missing columns
        for col in feature_columns:
            if col not in features_df.columns:
                features_df[col] = 0
                
        X = features_df[feature_columns].fillna(0)
        y = features_df['traffic_load']
        
        return X, y
    
    def train(self, training_data_path):
        """
        Train the ensemble model on educational traffic data
        """
        logger.info(f"Loading training data from {training_data_path}")
        
        try:
            df = pd.read_csv(training_data_path)
        except FileNotFoundError:
            logger.warning("Training data not found, generating synthetic data...")
            df = self.generate_synthetic_data()
        
        X, y = self.prepare_data(df)
        
        # Split data
        X_train, X_test, y_train, y_test = train_test_split(
            X, y, test_size=0.2, random_state=42, shuffle=False
        )
        
        # Scale features
        X_train_scaled = self.scaler.fit_transform(X_train)
        X_test_scaled = self.scaler.transform(X_test)
        
        # Train models
        logger.info("Training Random Forest model...")
        self.rf_model.fit(X_train_scaled, y_train)
        
        logger.info("Training Gradient Boosting model...")
        self.gb_model.fit(X_train_scaled, y_train)
        
        # Evaluate models
        rf_pred = self.rf_model.predict(X_test_scaled)
        gb_pred = self.gb_model.predict(X_test_scaled)
        
        # Ensemble prediction
        ensemble_pred = 0.6 * rf_pred + 0.4 * gb_pred
        
        # Calculate metrics
        mae = mean_absolute_error(y_test, ensemble_pred)
        mse = mean_squared_error(y_test, ensemble_pred)
        r2 = r2_score(y_test, ensemble_pred)
        
        logger.info(f"Model Performance:")
        logger.info(f"MAE: {mae:.2f}")
        logger.info(f"MSE: {mse:.2f}")
        logger.info(f"R²: {r2:.3f}")
        
        self.is_trained = True
        return {
            'mae': mae,
            'mse': mse,
            'r2': r2,
            'feature_importance': dict(zip(X.columns, self.rf_model.feature_importances_))
        }
    
    def predict(self, input_data):
        """
        Predict traffic load for given input features
        """
        if not self.is_trained:
            raise ValueError("Model must be trained before making predictions")
        
        if isinstance(input_data, dict):
            input_data = pd.DataFrame([input_data])
        
        X = self.prepare_data(input_data)[0]
        X_scaled = self.scaler.transform(X)
        
        # Ensemble prediction
        rf_pred = self.rf_model.predict(X_scaled)
        gb_pred = self.gb_model.predict(X_scaled)
        ensemble_pred = 0.6 * rf_pred + 0.4 * gb_pred
        
        return ensemble_pred
    
    def predict_next_hours(self, current_data, hours_ahead=24):
        """
        Predict traffic for the next N hours
        """
        predictions = []
        current_time = datetime.now()
        
        for i in range(hours_ahead):
            future_time = current_time + timedelta(hours=i)
            
            prediction_data = {
                'timestamp': future_time,
                'traffic_load': current_data.get('traffic_load', 50),
                'ai_requests': current_data.get('ai_requests', 100),
                'total_requests': current_data.get('total_requests', 500),
                'collaboration_sessions': current_data.get('collaboration_sessions', 50),
                'total_users': current_data.get('total_users', 200),
                'cpu_usage': current_data.get('cpu_usage', 60),
                'memory_usage': current_data.get('memory_usage', 70),
                'active_users': current_data.get('active_users', 150),
                'region': current_data.get('region', 'global')
            }
            
            pred = self.predict(prediction_data)
            predictions.append({
                'timestamp': future_time,
                'predicted_traffic': pred[0],
                'confidence': self.calculate_confidence(prediction_data)
            })
        
        return predictions
    
    def calculate_confidence(self, data):
        """
        Calculate prediction confidence based on feature patterns
        """
        # Simple confidence calculation based on time patterns
        hour = pd.to_datetime(data['timestamp']).hour
        day_of_week = pd.to_datetime(data['timestamp']).dayofweek
        
        # Higher confidence during regular school hours
        if 8 <= hour <= 18 and day_of_week < 5:
            return 0.85
        elif day_of_week >= 5:  # Weekend
            return 0.70
        else:  # Night hours
            return 0.60
    
    def generate_synthetic_data(self, days=30):
        """
        Generate synthetic educational traffic data
        """
        logger.info("Generating synthetic educational traffic data...")
        
        dates = pd.date_range(
            start=datetime.now() - timedelta(days=days),
            end=datetime.now(),
            freq='H'
        )
        
        data = []
        for timestamp in dates:
            hour = timestamp.hour
            day_of_week = timestamp.dayofweek
            
            # Base traffic with educational patterns
            base_traffic = 30
            
            # School hours increase
            if 8 <= hour <= 18 and day_of_week < 5:
                base_traffic += 40
            
            # Peak hours (9-11 AM, 2-4 PM)
            if (9 <= hour <= 11) or (14 <= hour <= 16):
                base_traffic += 20
            
            # Weekend reduction
            if day_of_week >= 5:
                base_traffic *= 0.6
            
            # Add some randomness
            traffic_load = max(0, base_traffic + np.random.normal(0, 10))
            
            data.append({
                'timestamp': timestamp,
                'traffic_load': traffic_load,
                'ai_requests': int(traffic_load * 2),
                'total_requests': int(traffic_load * 10),
                'collaboration_sessions': int(traffic_load * 0.5),
                'total_users': int(traffic_load * 3),
                'cpu_usage': min(100, traffic_load + np.random.normal(0, 5)),
                'memory_usage': min(100, traffic_load * 0.8 + np.random.normal(0, 5)),
                'active_users': int(traffic_load * 2.5),
                'region': np.random.choice(['us', 'eu', 'asia', 'global'])
            })
        
        return pd.DataFrame(data)
    
    def save_model(self, model_path):
        """
        Save trained model to disk
        """
        model_data = {
            'rf_model': self.rf_model,
            'gb_model': self.gb_model,
            'scaler': self.scaler,
            'label_encoders': self.label_encoders,
            'is_trained': self.is_trained
        }
        joblib.dump(model_data, model_path)
        logger.info(f"Model saved to {model_path}")
    
    def load_model(self, model_path):
        """
        Load trained model from disk
        """
        model_data = joblib.load(model_path)
        self.rf_model = model_data['rf_model']
        self.gb_model = model_data['gb_model']
        self.scaler = model_data['scaler']
        self.label_encoders = model_data['label_encoders']
        self.is_trained = model_data['is_trained']
        logger.info(f"Model loaded from {model_path}")

def main():
    """
    Train and evaluate the educational traffic prediction model
    """
    logger.info("Starting SIS AI-Lab Traffic Prediction Model Training")
    
    predictor = EducationalTrafficPredictor()
    
    # Train model
    metrics = predictor.train('/data/traffic_history.csv')
    
    logger.info("Training completed successfully")
    logger.info(f"Model performance: {metrics}")
    
    # Save model
    predictor.save_model('/models/traffic_predictor.pkl')
    
    # Example prediction
    current_data = {
        'traffic_load': 45,
        'ai_requests': 90,
        'total_requests': 450,
        'collaboration_sessions': 25,
        'total_users': 180,
        'cpu_usage': 55,
        'memory_usage': 65,
        'active_users': 140,
        'region': 'global'
    }
    
    predictions = predictor.predict_next_hours(current_data, 6)
    logger.info("Next 6 hours predictions:")
    for pred in predictions:
        logger.info(f"{pred['timestamp'].strftime('%H:%M')}: {pred['predicted_traffic']:.1f} (confidence: {pred['confidence']:.2f})")

if __name__ == "__main__":
    main()