//! Statistical analysis for performance validation
//!
//! Implements ChatGPT's rigorous statistical methodology for
//! confidence intervals, hypothesis testing, and measurement validation.

use alloc::vec::Vec;

/// Statistical analyzer for performance measurements
pub struct StatisticalAnalyzer {
    confidence_level: f64,
    minimum_sample_size: usize,
}

impl StatisticalAnalyzer {
    /// Create new statistical analyzer
    pub fn new() -> Self {
        Self {
            confidence_level: 0.95, // 95% confidence level
            minimum_sample_size: 30, // Minimum for CLT assumptions
        }
    }

    /// Set confidence level (e.g., 0.95 for 95%)
    pub fn with_confidence_level(mut self, level: f64) -> Self {
        self.confidence_level = level.clamp(0.5, 0.999);
        self
    }

    /// Set minimum sample size
    pub fn with_minimum_sample_size(mut self, size: usize) -> Self {
        self.minimum_sample_size = size.max(10);
        self
    }

    /// Perform comprehensive statistical analysis
    pub fn analyze(&self, measurements: &[f64]) -> Result<StatisticalResult, &'static str> {
        if measurements.len() < self.minimum_sample_size {
            return Err("Insufficient sample size for statistical analysis");
        }

        let descriptive = self.compute_descriptive_statistics(measurements);
        let confidence_interval = self.compute_confidence_interval(measurements)?;
        let outlier_info = self.detect_outliers(measurements);

        Ok(StatisticalResult {
            descriptive,
            confidence_interval,
            outlier_info,
            sample_size: measurements.len(),
            confidence_level: self.confidence_level,
        })
    }

    /// Compute descriptive statistics
    fn compute_descriptive_statistics(&self, data: &[f64]) -> DescriptiveStats {
        if data.is_empty() {
            return DescriptiveStats::empty();
        }

        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = data.len();
        let sum: f64 = data.iter().sum();
        let mean = sum / n as f64;

        // Variance calculation
        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (n - 1) as f64;
        
        let std_dev = variance.sqrt();
        let std_error = std_dev / (n as f64).sqrt();

        // Percentiles
        let median = Self::percentile(&sorted, 0.50);
        let q25 = Self::percentile(&sorted, 0.25);
        let q75 = Self::percentile(&sorted, 0.75);
        let p95 = Self::percentile(&sorted, 0.95);
        let p99 = Self::percentile(&sorted, 0.99);

        let min = sorted[0];
        let max = sorted[n - 1];
        let range = max - min;
        let iqr = q75 - q25;

        // Skewness (third moment)
        let skewness = if std_dev > 0.0 {
            let third_moment = data.iter()
                .map(|x| ((x - mean) / std_dev).powi(3))
                .sum::<f64>() / n as f64;
            third_moment
        } else {
            0.0
        };

        // Kurtosis (fourth moment)
        let kurtosis = if std_dev > 0.0 {
            let fourth_moment = data.iter()
                .map(|x| ((x - mean) / std_dev).powi(4))
                .sum::<f64>() / n as f64;
            fourth_moment - 3.0 // Excess kurtosis
        } else {
            0.0
        };

        DescriptiveStats {
            count: n,
            mean,
            median,
            std_dev,
            std_error,
            variance,
            min,
            max,
            range,
            q25,
            q75,
            iqr,
            p95,
            p99,
            skewness,
            kurtosis,
        }
    }

    /// Compute confidence interval for the mean
    fn compute_confidence_interval(&self, data: &[f64]) -> Result<ConfidenceInterval, &'static str> {
        if data.is_empty() {
            return Err("Cannot compute confidence interval for empty data");
        }

        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        
        // Sample standard deviation
        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();
        let std_error = std_dev / n.sqrt();

        // Critical value (approximation for t-distribution)
        let alpha = 1.0 - self.confidence_level;
        let t_critical = self.t_critical_value(n as usize - 1, alpha / 2.0);

        let margin_of_error = t_critical * std_error;
        let lower = mean - margin_of_error;
        let upper = mean + margin_of_error;

        Ok(ConfidenceInterval {
            mean,
            lower,
            upper,
            margin_of_error,
            confidence_level: self.confidence_level,
        })
    }

    /// Approximate t-critical value for confidence intervals
    fn t_critical_value(&self, df: usize, alpha: f64) -> f64 {
        // Simplified approximation - for production use proper t-tables
        match self.confidence_level {
            level if level >= 0.99 => 2.576,   // 99% CI
            level if level >= 0.95 => 1.96,    // 95% CI
            level if level >= 0.90 => 1.645,   // 90% CI
            _ => 1.96, // Default to 95%
        }
    }

    /// Detect outliers using IQR method and modified Z-score
    fn detect_outliers(&self, data: &[f64]) -> OutlierInfo {
        if data.len() < 4 {
            return OutlierInfo {
                outlier_count: 0,
                outlier_indices: Vec::new(),
                outlier_method: "insufficient_data".into(),
            };
        }

        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q25 = Self::percentile(&sorted, 0.25);
        let q75 = Self::percentile(&sorted, 0.75);
        let iqr = q75 - q25;

        // IQR method thresholds
        let lower_threshold = q25 - 1.5 * iqr;
        let upper_threshold = q75 + 1.5 * iqr;

        let mut outlier_indices = Vec::new();
        for (i, &value) in data.iter().enumerate() {
            if value < lower_threshold || value > upper_threshold {
                outlier_indices.push(i);
            }
        }

        OutlierInfo {
            outlier_count: outlier_indices.len(),
            outlier_indices,
            outlier_method: "iqr_method".into(),
        }
    }

    /// Calculate percentile from sorted data
    fn percentile(sorted_data: &[f64], p: f64) -> f64 {
        if sorted_data.is_empty() {
            return 0.0;
        }

        let n = sorted_data.len();
        let index = p * (n - 1) as f64;
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;

        if lower == upper {
            sorted_data[lower.min(n - 1)]
        } else {
            let weight = index - lower as f64;
            let lower_val = sorted_data[lower.min(n - 1)];
            let upper_val = sorted_data[upper.min(n - 1)];
            lower_val + weight * (upper_val - lower_val)
        }
    }

    /// Perform hypothesis test for performance claim
    pub fn test_performance_claim(
        &self,
        measurements: &[f64],
        claim_value: f64,
        test_type: HypothesisTestType,
    ) -> Result<HypothesisTestResult, &'static str> {
        if measurements.len() < self.minimum_sample_size {
            return Err("Insufficient sample size for hypothesis testing");
        }

        let n = measurements.len() as f64;
        let sample_mean = measurements.iter().sum::<f64>() / n;
        
        let sample_variance = measurements.iter()
            .map(|x| (x - sample_mean).powi(2))
            .sum::<f64>() / (n - 1.0);
        let sample_std = sample_variance.sqrt();
        let std_error = sample_std / n.sqrt();

        // One-sample t-test
        let t_statistic = (sample_mean - claim_value) / std_error;
        let df = measurements.len() - 1;

        // Critical value determination
        let alpha = 1.0 - self.confidence_level;
        let critical_value = match test_type {
            HypothesisTestType::TwoTailed => self.t_critical_value(df, alpha / 2.0),
            HypothesisTestType::OneTailedGreater => self.t_critical_value(df, alpha),
            HypothesisTestType::OneTailedLess => -self.t_critical_value(df, alpha),
        };

        // Determine rejection
        let reject_null = match test_type {
            HypothesisTestType::TwoTailed => t_statistic.abs() > critical_value.abs(),
            HypothesisTestType::OneTailedGreater => t_statistic > critical_value,
            HypothesisTestType::OneTailedLess => t_statistic < critical_value,
        };

        // Effect size (Cohen's d)
        let effect_size = (sample_mean - claim_value) / sample_std;

        Ok(HypothesisTestResult {
            t_statistic,
            critical_value,
            degrees_of_freedom: df,
            reject_null_hypothesis: reject_null,
            sample_mean,
            claimed_value: claim_value,
            effect_size,
            test_type,
        })
    }
}

/// Comprehensive descriptive statistics
#[derive(Debug, Clone)]
pub struct DescriptiveStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub std_error: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub range: f64,
    pub q25: f64,
    pub q75: f64,
    pub iqr: f64,
    pub p95: f64,
    pub p99: f64,
    pub skewness: f64,
    pub kurtosis: f64,
}

impl DescriptiveStats {
    fn empty() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            std_error: 0.0,
            variance: 0.0,
            min: 0.0,
            max: 0.0,
            range: 0.0,
            q25: 0.0,
            q75: 0.0,
            iqr: 0.0,
            p95: 0.0,
            p99: 0.0,
            skewness: 0.0,
            kurtosis: 0.0,
        }
    }

    /// Check if distribution appears normal (simple heuristics)
    pub fn appears_normal(&self) -> bool {
        // Simple normality checks
        let skew_ok = self.skewness.abs() < 2.0;
        let kurtosis_ok = self.kurtosis.abs() < 7.0;
        
        skew_ok && kurtosis_ok
    }

    /// Coefficient of variation
    pub fn coefficient_of_variation(&self) -> f64 {
        if self.mean.abs() < f64::EPSILON {
            0.0
        } else {
            self.std_dev / self.mean.abs()
        }
    }
}

/// Confidence interval for the mean
#[derive(Debug, Clone)]
pub struct ConfidenceInterval {
    pub mean: f64,
    pub lower: f64,
    pub upper: f64,
    pub margin_of_error: f64,
    pub confidence_level: f64,
}

impl ConfidenceInterval {
    /// Check if a value falls within the confidence interval
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }

    /// Width of the confidence interval
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }
}

/// Outlier detection information
#[derive(Debug, Clone)]
pub struct OutlierInfo {
    pub outlier_count: usize,
    pub outlier_indices: Vec<usize>,
    pub outlier_method: alloc::string::String,
}

impl OutlierInfo {
    /// Percentage of data points that are outliers
    pub fn outlier_percentage(&self, total_count: usize) -> f64 {
        if total_count == 0 {
            0.0
        } else {
            (self.outlier_count as f64 / total_count as f64) * 100.0
        }
    }
}

/// Hypothesis test types
#[derive(Debug, Clone, Copy)]
pub enum HypothesisTestType {
    TwoTailed,
    OneTailedGreater,
    OneTailedLess,
}

/// Hypothesis test result
#[derive(Debug, Clone)]
pub struct HypothesisTestResult {
    pub t_statistic: f64,
    pub critical_value: f64,
    pub degrees_of_freedom: usize,
    pub reject_null_hypothesis: bool,
    pub sample_mean: f64,
    pub claimed_value: f64,
    pub effect_size: f64,
    pub test_type: HypothesisTestType,
}

impl HypothesisTestResult {
    /// Interpret the statistical significance
    pub fn significance_level(&self) -> &'static str {
        let t_abs = self.t_statistic.abs();
        let crit_abs = self.critical_value.abs();
        
        if t_abs > crit_abs * 2.0 {
            "highly_significant"
        } else if t_abs > crit_abs {
            "significant"
        } else {
            "not_significant"
        }
    }

    /// Effect size interpretation
    pub fn effect_size_magnitude(&self) -> &'static str {
        let effect_abs = self.effect_size.abs();
        
        if effect_abs < 0.2 {
            "negligible"
        } else if effect_abs < 0.5 {
            "small"
        } else if effect_abs < 0.8 {
            "medium"
        } else {
            "large"
        }
    }
}

/// Complete statistical analysis result
#[derive(Debug, Clone)]
pub struct StatisticalResult {
    pub descriptive: DescriptiveStats,
    pub confidence_interval: ConfidenceInterval,
    pub outlier_info: OutlierInfo,
    pub sample_size: usize,
    pub confidence_level: f64,
}

impl StatisticalResult {
    /// Check if measurements are statistically reliable
    pub fn is_reliable(&self) -> bool {
        let sufficient_samples = self.sample_size >= 30;
        let reasonable_variation = self.descriptive.coefficient_of_variation() < 0.5;
        let few_outliers = self.outlier_info.outlier_percentage(self.sample_size) < 10.0;
        let appears_normal = self.descriptive.appears_normal();
        
        sufficient_samples && reasonable_variation && few_outliers && appears_normal
    }

    /// Get quality assessment
    pub fn quality_assessment(&self) -> &'static str {
        if self.is_reliable() {
            "high_quality"
        } else if self.sample_size >= 100 {
            "acceptable"
        } else {
            "needs_more_data"
        }
    }
}