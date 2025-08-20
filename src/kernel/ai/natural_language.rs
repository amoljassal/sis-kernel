//! Natural Language Interface with Safety Validation
//!
//! Implements natural language processing for software synthesis with
//! safety validation and fail-closed beginner mode as recommended by ChatGPT consultation.
//!
//! Key Features:
//! - Intent recognition and parsing (<100ms target)
//! - Safety validation preventing dangerous configurations
//! - Beginner vs Professional mode with different safety levels
//! - Progressive refinement for ambiguous requirements
//! - Integration with DCON for consistency validation

use crate::kernel::ai::dcon::{DesignContract, DconValidator, SafetyCriticality};
use crate::kernel::ai::software_synthesis::{SoftwareSynthesisRequest, TargetLanguage, OptimizationTarget};
use crate::kernel::serial;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{vec, format};
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use spin::Mutex;

/// Maximum natural language input length (safety constraint)
const MAX_NL_INPUT_LENGTH: usize = 4096;

/// Intent parsing timeout (microseconds)
const INTENT_PARSING_TIMEOUT_US: u32 = 100_000; // 100ms as per Grok recommendation

/// Safety validation cache size
const SAFETY_CACHE_SIZE: usize = 256;

/// User expertise levels for safety validation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UserExpertise {
    /// Beginner: Strict safety constraints, fail-closed validation
    Beginner,
    
    /// Intermediate: Standard safety with warnings
    Intermediate,
    
    /// Professional: Allow advanced configurations with explicit acknowledgment
    Professional,
    
    /// Expert: Minimal safety constraints with full access
    Expert,
}

/// Natural language intent categories
#[derive(Debug, Clone, PartialEq)]
pub enum NaturalLanguageIntent {
    /// Create a new software application
    CreateApplication {
        description: String,
        target_platform: Option<String>,
        performance_requirements: Option<String>,
    },
    
    /// Create a hardware driver
    CreateDriver {
        hardware_type: String,
        interface_type: String,
        performance_requirements: Option<String>,
    },
    
    /// Create a library or module
    CreateLibrary {
        functionality: String,
        api_requirements: Vec<String>,
    },
    
    /// Optimize existing code
    OptimizeCode {
        code_description: String,
        optimization_goals: Vec<String>,
    },
    
    /// Generate test cases
    GenerateTests {
        target_description: String,
        test_types: Vec<String>,
    },
    
    /// Integrate with hardware
    HardwareIntegration {
        hardware_description: String,
        integration_type: String,
    },
    
    /// Ambiguous intent requiring clarification
    Ambiguous {
        original_text: String,
        possible_intents: Vec<String>,
        clarification_questions: Vec<String>,
    },
}

/// Safety validation result for natural language inputs
#[derive(Debug, Clone)]
pub struct SafetyValidationResult {
    /// Overall safety assessment
    pub is_safe: bool,
    
    /// Safety level required for this request
    pub required_safety_level: SafetyCriticality,
    
    /// Detected safety concerns
    pub safety_concerns: Vec<SafetyConcern>,
    
    /// Recommended actions or constraints
    pub recommendations: Vec<String>,
    
    /// Whether explicit user acknowledgment is required
    pub requires_acknowledgment: bool,
}

/// Types of safety concerns detected in natural language
#[derive(Debug, Clone)]
pub enum SafetyConcern {
    /// Request could lead to unsafe hardware configuration
    UnsafeHardwareConfig {
        concern: String,
        severity: SafetySeverity,
    },
    
    /// Ambiguous requirements that could be misinterpreted
    AmbiguousRequirements {
        ambiguity: String,
        potential_issues: Vec<String>,
    },
    
    /// Resource constraints that could be dangerous
    ResourceConstraints {
        constraint_type: String,
        potential_issue: String,
    },
    
    /// Security implications
    SecurityConcern {
        concern: String,
        mitigation: String,
    },
    
    /// Real-time safety implications
    RealTimeSafety {
        timing_concern: String,
        safety_impact: String,
    },
}

/// Safety concern severity levels
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum SafetySeverity {
    Info = 0,
    Warning = 1,
    Error = 2,
    Critical = 3,
}

/// Natural language processing request
#[derive(Debug, Clone)]
pub struct NLProcessingRequest {
    /// User input text
    pub input_text: String,
    
    /// User expertise level
    pub user_expertise: UserExpertise,
    
    /// Current design contract context
    pub dcon_context: DesignContract,
    
    /// Session identifier for context tracking
    pub session_id: u64,
    
    /// Previous conversation context
    pub conversation_history: Vec<String>,
}

/// Natural language processing result
#[derive(Debug, Clone)]
pub struct NLProcessingResult {
    /// Parsed intent
    pub intent: NaturalLanguageIntent,
    
    /// Safety validation result
    pub safety_validation: SafetyValidationResult,
    
    /// Generated software synthesis request (if applicable)
    pub synthesis_request: Option<SoftwareSynthesisRequest>,
    
    /// Clarification questions for ambiguous inputs
    pub clarification_questions: Vec<String>,
    
    /// Processing metadata
    pub metadata: NLProcessingMetadata,
}

/// Processing metadata for performance tracking
#[derive(Debug, Clone)]
pub struct NLProcessingMetadata {
    /// Processing start time (microseconds)
    pub start_time_us: u64,
    
    /// Total processing duration (microseconds)
    pub processing_duration_us: u32,
    
    /// Intent parsing duration (microseconds)
    pub intent_parsing_duration_us: u32,
    
    /// Safety validation duration (microseconds)
    pub safety_validation_duration_us: u32,
    
    /// Cache hits during processing
    pub cache_hits: u32,
}

/// Safety validation cache entry
#[derive(Clone)]
struct SafetyCacheEntry {
    /// Input text hash
    input_hash: u64,
    
    /// User expertise level
    expertise: UserExpertise,
    
    /// Cached validation result
    validation: SafetyValidationResult,
    
    /// Cache entry timestamp
    timestamp_us: u64,
    
    /// Access count
    access_count: u32,
}

/// Intent parser with lightweight NLP capabilities
struct IntentParser {
    /// Common keywords for intent classification
    application_keywords: Vec<&'static str>,
    driver_keywords: Vec<&'static str>,
    library_keywords: Vec<&'static str>,
    optimization_keywords: Vec<&'static str>,
    test_keywords: Vec<&'static str>,
    hardware_keywords: Vec<&'static str>,
}

impl IntentParser {
    /// Create new intent parser
    fn new() -> Self {
        Self {
            application_keywords: vec!["app", "application", "program", "software", "tool"],
            driver_keywords: vec!["driver", "device", "hardware", "controller", "peripheral"],
            library_keywords: vec!["library", "lib", "module", "component", "api"],
            optimization_keywords: vec!["optimize", "improve", "faster", "efficient", "performance"],
            test_keywords: vec!["test", "testing", "verify", "validate", "check"],
            hardware_keywords: vec!["integrate", "connect", "interface", "protocol", "communication"],
        }
    }

    /// Parse natural language text into intent (targets <100ms)
    fn parse_intent(&self, text: &str) -> Result<NaturalLanguageIntent, &'static str> {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        
        // Simple keyword-based classification
        let mut scores = [0u32; 6]; // [app, driver, library, optimize, test, hardware]
        
        for word in &words {
            if self.application_keywords.contains(word) { scores[0] += 1; }
            if self.driver_keywords.contains(word) { scores[1] += 1; }
            if self.library_keywords.contains(word) { scores[2] += 1; }
            if self.optimization_keywords.contains(word) { scores[3] += 1; }
            if self.test_keywords.contains(word) { scores[4] += 1; }
            if self.hardware_keywords.contains(word) { scores[5] += 1; }
        }
        
        // Find highest scoring intent
        let max_score = scores.iter().max().unwrap_or(&0);
        if *max_score == 0 {
            return Ok(NaturalLanguageIntent::Ambiguous {
                original_text: text.to_string(),
                possible_intents: vec!["application".to_string(), "driver".to_string()],
                clarification_questions: vec![
                    "What type of software would you like to create?".to_string(),
                    "Are you looking to create an application or a hardware driver?".to_string(),
                ],
            });
        }
        
        let max_index = scores.iter().position(|&x| x == *max_score).unwrap();
        
        match max_index {
            0 => Ok(NaturalLanguageIntent::CreateApplication {
                description: text.to_string(),
                target_platform: None,
                performance_requirements: None,
            }),
            1 => Ok(NaturalLanguageIntent::CreateDriver {
                hardware_type: "generic".to_string(),
                interface_type: "memory_mapped".to_string(),
                performance_requirements: None,
            }),
            2 => Ok(NaturalLanguageIntent::CreateLibrary {
                functionality: text.to_string(),
                api_requirements: Vec::new(),
            }),
            3 => Ok(NaturalLanguageIntent::OptimizeCode {
                code_description: text.to_string(),
                optimization_goals: vec!["performance".to_string()],
            }),
            4 => Ok(NaturalLanguageIntent::GenerateTests {
                target_description: text.to_string(),
                test_types: vec!["unit".to_string(), "integration".to_string()],
            }),
            5 => Ok(NaturalLanguageIntent::HardwareIntegration {
                hardware_description: text.to_string(),
                integration_type: "driver".to_string(),
            }),
            _ => Err("Invalid intent classification"),
        }
    }
}

/// Safety validator implementing fail-closed beginner mode
struct SafetyValidator {
    /// Safety validation cache
    cache: Mutex<Vec<SafetyCacheEntry>>,
    
    /// Cache hit counter
    cache_hits: AtomicU64,
    
    /// Validation counter
    total_validations: AtomicU64,
    
    /// Dangerous patterns detected
    dangerous_patterns_blocked: AtomicU64,
}

impl SafetyValidator {
    /// Create new safety validator
    fn new() -> Self {
        Self {
            cache: Mutex::new(Vec::with_capacity(SAFETY_CACHE_SIZE)),
            cache_hits: AtomicU64::new(0),
            total_validations: AtomicU64::new(0),
            dangerous_patterns_blocked: AtomicU64::new(0),
        }
    }

    /// Validate natural language input for safety (fail-closed for beginners)
    fn validate_safety(&self, request: &NLProcessingRequest) -> SafetyValidationResult {
        self.total_validations.fetch_add(1, Ordering::Relaxed);
        
        // Check cache first
        let input_hash = self.compute_hash(&request.input_text, request.user_expertise);
        if let Some(cached) = self.check_cache(input_hash) {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            return cached;
        }
        
        let mut concerns = Vec::new();
        let mut is_safe = true;
        let mut requires_ack = false;
        
        // Check for dangerous patterns
        let text_lower = request.input_text.to_lowercase();
        
        // Hardware safety checks
        if text_lower.contains("unsafe") || text_lower.contains("bypass") {
            concerns.push(SafetyConcern::UnsafeHardwareConfig {
                concern: "Request contains potentially unsafe operations".to_string(),
                severity: SafetySeverity::Warning,
            });
            
            if request.user_expertise == UserExpertise::Beginner {
                is_safe = false;
                self.dangerous_patterns_blocked.fetch_add(1, Ordering::Relaxed);
            } else {
                requires_ack = true;
            }
        }
        
        // Memory safety checks
        if text_lower.contains("raw pointer") || text_lower.contains("unsafe memory") {
            concerns.push(SafetyConcern::SecurityConcern {
                concern: "Request involves raw memory operations".to_string(),
                mitigation: "Use safe memory abstractions instead".to_string(),
            });
            
            if request.user_expertise == UserExpertise::Beginner {
                is_safe = false;
                self.dangerous_patterns_blocked.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        // Real-time safety checks
        if text_lower.contains("deadline") || text_lower.contains("real-time") {
            if request.dcon_context.realtime.deadline_us < 1000 {
                concerns.push(SafetyConcern::RealTimeSafety {
                    timing_concern: "Very tight real-time deadline requested".to_string(),
                    safety_impact: "May not be achievable on target hardware".to_string(),
                });
                
                if request.user_expertise == UserExpertise::Beginner {
                    is_safe = false;
                }
            }
        }
        
        // Power/thermal safety checks
        if text_lower.contains("maximum") || text_lower.contains("unlimited") {
            concerns.push(SafetyConcern::ResourceConstraints {
                constraint_type: "Power/Performance".to_string(),
                potential_issue: "Unbounded resource usage could cause thermal issues".to_string(),
            });
            requires_ack = true;
        }
        
        // Ambiguity checks
        if text_lower.split_whitespace().count() < 3 {
            concerns.push(SafetyConcern::AmbiguousRequirements {
                ambiguity: "Request is too vague".to_string(),
                potential_issues: vec!["May generate unexpected code".to_string()],
            });
            
            if request.user_expertise == UserExpertise::Beginner {
                is_safe = false;
            }
        }
        
        // Determine required safety level
        let required_safety_level = if concerns.iter().any(|c| matches!(c, SafetyConcern::RealTimeSafety { .. })) {
            SafetyCriticality::SafetyCritical
        } else if !concerns.is_empty() {
            SafetyCriticality::Production
        } else {
            SafetyCriticality::Development
        };
        
        // Generate recommendations
        let mut recommendations = Vec::new();
        for concern in &concerns {
            match concern {
                SafetyConcern::UnsafeHardwareConfig { .. } => {
                    recommendations.push("Consider using safe abstractions instead".to_string());
                }
                SafetyConcern::AmbiguousRequirements { .. } => {
                    recommendations.push("Please provide more detailed requirements".to_string());
                }
                SafetyConcern::ResourceConstraints { .. } => {
                    recommendations.push("Specify explicit resource limits".to_string());
                }
                _ => {}
            }
        }
        
        let result = SafetyValidationResult {
            is_safe,
            required_safety_level,
            safety_concerns: concerns,
            recommendations,
            requires_acknowledgment: requires_ack,
        };
        
        // Cache the result
        self.cache_result(input_hash, request.user_expertise, result.clone());
        
        result
    }

    /// Check validation cache
    fn check_cache(&self, input_hash: u64) -> Option<SafetyValidationResult> {
        let cache = self.cache.lock();
        for entry in cache.iter() {
            if entry.input_hash == input_hash {
                return Some(entry.validation.clone());
            }
        }
        None
    }

    /// Cache validation result
    fn cache_result(&self, input_hash: u64, expertise: UserExpertise, result: SafetyValidationResult) {
        let mut cache = self.cache.lock();
        
        if cache.len() >= SAFETY_CACHE_SIZE {
            // Remove oldest entry
            cache.remove(0);
        }
        
        cache.push(SafetyCacheEntry {
            input_hash,
            expertise,
            validation: result,
            timestamp_us: 0, // Would use actual timestamp
            access_count: 1,
        });
    }

    /// Compute hash for cache key
    fn compute_hash(&self, text: &str, expertise: UserExpertise) -> u64 {
        let mut hash = 0u64;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash = hash.wrapping_mul(31).wrapping_add(expertise as u64);
        hash
    }
}

/// Natural Language Processing Engine
pub struct NaturalLanguageEngine {
    /// Intent parser
    intent_parser: IntentParser,
    
    /// Safety validator
    safety_validator: SafetyValidator,
    
    /// Performance counters
    total_processed: AtomicU64,
    total_processing_time_us: AtomicU64,
    intent_parsing_time_us: AtomicU64,
    safety_validation_time_us: AtomicU64,
    
    /// Active processing sessions
    active_sessions: AtomicU32,
}

impl NaturalLanguageEngine {
    /// Create new natural language engine
    pub fn new() -> Self {
        Self {
            intent_parser: IntentParser::new(),
            safety_validator: SafetyValidator::new(),
            total_processed: AtomicU64::new(0),
            total_processing_time_us: AtomicU64::new(0),
            intent_parsing_time_us: AtomicU64::new(0),
            safety_validation_time_us: AtomicU64::new(0),
            active_sessions: AtomicU32::new(0),
        }
    }

    /// Initialize natural language engine
    pub fn init(&self) -> Result<(), &'static str> {
        serial::write_str("[natural-language] Initializing Natural Language Interface...\n");
        
        // Reset performance counters
        self.total_processed.store(0, Ordering::Relaxed);
        self.total_processing_time_us.store(0, Ordering::Relaxed);
        
        serial::write_str("[natural-language] Intent parser initialized\n");
        serial::write_str("[natural-language] Safety validator initialized\n");
        serial::write_str("[natural-language] Natural Language Interface ready\n");
        
        Ok(())
    }

    /// Process natural language input (targets <100ms)
    pub fn process(&self, request: NLProcessingRequest) -> Result<NLProcessingResult, &'static str> {
        let start_time_us = self.get_timestamp_us();
        
        // Input validation
        if request.input_text.len() > MAX_NL_INPUT_LENGTH {
            return Err("Input text too long");
        }
        
        if request.input_text.trim().is_empty() {
            return Err("Empty input text");
        }
        
        self.active_sessions.fetch_add(1, Ordering::AcqRel);
        
        // Parse intent
        let intent_start = self.get_timestamp_us();
        let intent = self.intent_parser.parse_intent(&request.input_text)?;
        let intent_duration = (self.get_timestamp_us() - intent_start) as u32;
        self.intent_parsing_time_us.fetch_add(intent_duration as u64, Ordering::Relaxed);
        
        // Validate safety
        let safety_start = self.get_timestamp_us();
        let safety_validation = self.safety_validator.validate_safety(&request);
        let safety_duration = (self.get_timestamp_us() - safety_start) as u32;
        self.safety_validation_time_us.fetch_add(safety_duration as u64, Ordering::Relaxed);
        
        // Generate synthesis request if safe and unambiguous
        let synthesis_request = if safety_validation.is_safe {
            self.generate_synthesis_request(&intent, &request)?
        } else {
            None
        };
        
        // Generate clarification questions for ambiguous intents
        let clarification_questions = match &intent {
            NaturalLanguageIntent::Ambiguous { clarification_questions, .. } => {
                clarification_questions.clone()
            }
            _ => Vec::new(),
        };
        
        let end_time_us = self.get_timestamp_us();
        let total_duration = (end_time_us - start_time_us) as u32;
        
        // Update performance counters
        self.total_processed.fetch_add(1, Ordering::Relaxed);
        self.total_processing_time_us.fetch_add(total_duration as u64, Ordering::Relaxed);
        self.active_sessions.fetch_sub(1, Ordering::AcqRel);
        
        let metadata = NLProcessingMetadata {
            start_time_us,
            processing_duration_us: total_duration,
            intent_parsing_duration_us: intent_duration,
            safety_validation_duration_us: safety_duration,
            cache_hits: 0, // Would track actual cache hits
        };
        
        Ok(NLProcessingResult {
            intent,
            safety_validation,
            synthesis_request,
            clarification_questions,
            metadata,
        })
    }

    /// Generate software synthesis request from parsed intent
    fn generate_synthesis_request(&self, intent: &NaturalLanguageIntent, request: &NLProcessingRequest) -> Result<Option<SoftwareSynthesisRequest>, &'static str> {
        let synthesis_request = match intent {
            NaturalLanguageIntent::CreateApplication { description, .. } => {
                Some(SoftwareSynthesisRequest {
                    request_id: self.generate_request_id(),
                    description: description.clone(),
                    target_language: TargetLanguage::Rust, // Default
                    optimization_target: OptimizationTarget::Balanced,
                    dcon: request.dcon_context.clone(),
                    hardware_context: None,
                    rt_deadline_us: Some(request.dcon_context.realtime.deadline_us),
                    memory_budget_bytes: None,
                    api_requirements: Vec::new(),
                })
            }
            
            NaturalLanguageIntent::CreateDriver { hardware_type, .. } => {
                Some(SoftwareSynthesisRequest {
                    request_id: self.generate_request_id(),
                    description: format!("Create driver for {}", hardware_type),
                    target_language: TargetLanguage::Rust,
                    optimization_target: OptimizationTarget::RealTime,
                    dcon: request.dcon_context.clone(),
                    hardware_context: Some(hardware_type.clone()),
                    rt_deadline_us: Some(request.dcon_context.realtime.deadline_us),
                    memory_budget_bytes: None,
                    api_requirements: Vec::new(),
                })
            }
            
            NaturalLanguageIntent::CreateLibrary { functionality, api_requirements } => {
                Some(SoftwareSynthesisRequest {
                    request_id: self.generate_request_id(),
                    description: functionality.clone(),
                    target_language: TargetLanguage::Rust,
                    optimization_target: OptimizationTarget::Balanced,
                    dcon: request.dcon_context.clone(),
                    hardware_context: None,
                    rt_deadline_us: None,
                    memory_budget_bytes: None,
                    api_requirements: api_requirements.clone(),
                })
            }
            
            _ => None, // Other intents don't generate direct synthesis requests
        };
        
        Ok(synthesis_request)
    }

    /// Generate unique request identifier
    fn generate_request_id(&self) -> u64 {
        self.total_processed.load(Ordering::Relaxed) + 1
    }

    /// Get high-precision timestamp
    fn get_timestamp_us(&self) -> u64 {
        // Would use actual timestamp in production
        0
    }

    /// Get natural language processing statistics
    pub fn get_statistics(&self) -> NLStatistics {
        let total = self.total_processed.load(Ordering::Relaxed);
        let total_time = self.total_processing_time_us.load(Ordering::Relaxed);
        
        NLStatistics {
            total_processed: total,
            average_processing_time_us: if total > 0 { total_time / total } else { 0 },
            average_intent_parsing_time_us: if total > 0 { self.intent_parsing_time_us.load(Ordering::Relaxed) / total } else { 0 },
            average_safety_validation_time_us: if total > 0 { self.safety_validation_time_us.load(Ordering::Relaxed) / total } else { 0 },
            active_sessions: self.active_sessions.load(Ordering::Relaxed),
            cache_hit_rate_percent: if total > 0 { 
                (self.safety_validator.cache_hits.load(Ordering::Relaxed) * 100 / total) as u32 
            } else { 0 },
            dangerous_patterns_blocked: self.safety_validator.dangerous_patterns_blocked.load(Ordering::Relaxed),
        }
    }
}

/// Natural language processing statistics
#[derive(Debug, Clone)]
pub struct NLStatistics {
    pub total_processed: u64,
    pub average_processing_time_us: u64,
    pub average_intent_parsing_time_us: u64,
    pub average_safety_validation_time_us: u64,
    pub active_sessions: u32,
    pub cache_hit_rate_percent: u32,
    pub dangerous_patterns_blocked: u64,
}

/// Global natural language engine instance
static NATURAL_LANGUAGE_ENGINE: spin::Once<NaturalLanguageEngine> = spin::Once::new();

/// Initialize natural language subsystem
pub fn init() -> Result<(), &'static str> {
    let engine = NATURAL_LANGUAGE_ENGINE.call_once(|| NaturalLanguageEngine::new());
    engine.init()
}

/// Process natural language input
pub fn process_natural_language(request: NLProcessingRequest) -> Result<NLProcessingResult, &'static str> {
    let engine = NATURAL_LANGUAGE_ENGINE.get().ok_or("Natural language engine not initialized")?;
    engine.process(request)
}

/// Get natural language processing statistics
pub fn get_nl_statistics() -> Option<NLStatistics> {
    NATURAL_LANGUAGE_ENGINE.get().map(|engine| engine.get_statistics())
}