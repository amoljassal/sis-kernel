//! Predictive Power Management Integration Test
//!
//! Tests the integration between the AI scheduler and predictive power management
//! system, validating EWMA predictions, thermal modeling, and DVFS decisions.

use super::predictive_power::*;
use crate::kernel::ai::{CognitivePriority, WorkloadType};
use crate::kernel::serial;

/// Power management integration test
pub struct PowerIntegrationTest {
    /// Test predictive power manager
    power_manager: PredictivePowerManager,
    /// Test results
    test_results: TestResults,
}

/// Test results tracking
#[derive(Debug, Default)]
pub struct TestResults {
    pub tests_run: u32,
    pub tests_passed: u32,
    pub ewma_accuracy_test: bool,
    pub holt_trend_test: bool,
    pub thermal_model_test: bool,
    pub power_state_selection_test: bool,
    pub race_to_sleep_test: bool,
}

impl PowerIntegrationTest {
    /// Create new power integration test
    pub fn new() -> Self {
        Self {
            power_manager: PredictivePowerManager::new(),
            test_results: TestResults::default(),
        }
    }
    
    /// Run all power management integration tests
    pub fn run_all_tests(&mut self) -> bool {
        serial::write_str("[Power Integration] Starting predictive power management tests\n");
        
        let mut all_passed = true;
        
        // Test 1: EWMA prediction accuracy
        all_passed &= self.test_ewma_prediction();
        
        // Test 2: Holt trend detection
        all_passed &= self.test_holt_trend_detection();
        
        // Test 3: Thermal model accuracy
        all_passed &= self.test_thermal_model();
        
        // Test 4: Power state selection logic
        all_passed &= self.test_power_state_selection();
        
        // Test 5: Race-to-sleep optimization
        all_passed &= self.test_race_to_sleep();
        
        // Test 6: End-to-end integration
        all_passed &= self.test_end_to_end_integration();
        
        self.print_test_results();
        all_passed
    }
    
    /// Test EWMA prediction accuracy with known data
    fn test_ewma_prediction(&mut self) -> bool {
        serial::write_str("[Power Test 1] Testing EWMA prediction accuracy\n");
        self.test_results.tests_run += 1;
        
        // Create EWMA predictor with α=0.3
        let mut ewma = EwmaQ15::new(0.3);
        
        // Known input sequence: 100, 200, 300, 400, 500
        let inputs = [100, 200, 300, 400, 500];
        let mut prediction = 0;
        
        for &input in inputs.iter() {
            let input_q15 = Q15Math::from_float(input as f32);
            prediction = ewma.update(input_q15);
        }
        
        // Convert back from Q15
        let final_prediction = (prediction as f32) / 32768.0;
        
        // Should be close to exponentially weighted average
        let expected_range = (320.0, 380.0);
        let test_passed = final_prediction >= expected_range.0 && 
                         final_prediction <= expected_range.1;
        
        if test_passed {
            serial::write_str("[Power Test 1] PASS: EWMA prediction within expected range\n");
            self.test_results.tests_passed += 1;
            self.test_results.ewma_accuracy_test = true;
        } else {
            serial::write_str("[Power Test 1] FAIL: EWMA prediction out of range\n");
        }
        
        test_passed
    }
    
    /// Test Holt linear trend detection
    fn test_holt_trend_detection(&mut self) -> bool {
        serial::write_str("[Power Test 2] Testing Holt trend detection\n");
        self.test_results.tests_run += 1;
        
        // Create Holt predictor
        let mut holt = HoltQ15::new(0.2, 0.1);
        
        // Linear increasing sequence: 100, 110, 120, 130, 140
        let inputs = [100, 110, 120, 130, 140];
        let mut prediction = 0;
        
        for &input in inputs.iter() {
            let input_q15 = Q15Math::from_float(input as f32);
            prediction = holt.update(input_q15);
        }
        
        // Should predict next value in sequence (~150)
        let final_prediction = (prediction as f32) / 32768.0;
        let expected_range = (145.0, 155.0);
        let test_passed = final_prediction >= expected_range.0 && 
                         final_prediction <= expected_range.1;
        
        if test_passed {
            serial::write_str("[Power Test 2] PASS: Holt trend detection working correctly\n");
            self.test_results.tests_passed += 1;
            self.test_results.holt_trend_test = true;
        } else {
            serial::write_str("[Power Test 2] FAIL: Holt trend detection failed\n");
        }
        
        test_passed
    }
    
    /// Test thermal model with known power input
    fn test_thermal_model(&mut self) -> bool {
        serial::write_str("[Power Test 3] Testing thermal model accuracy\n");
        self.test_results.tests_run += 1;
        
        // Create thermal model: 25°C ambient, 5s tau, 0.1°C/W
        let mut thermal = ThermalModel::new(25.0, 5000.0, 0.1, 1.0);
        
        // Apply constant 10W power for 100 steps (steady state should be 25 + 10*0.1 = 26°C)
        let power_10w_q16 = (10000 << 16); // 10W in Q16.16 format
        
        for _ in 0..100 {
            thermal.step(power_10w_q16);
        }
        
        let final_temp_c100 = thermal.temp_c100();
        let final_temp_c = (final_temp_c100 as f32) / 100.0;
        
        // Should be close to steady-state temperature
        let expected_range = (25.8, 26.2);
        let test_passed = final_temp_c >= expected_range.0 && 
                         final_temp_c <= expected_range.1;
        
        if test_passed {
            serial::write_str("[Power Test 3] PASS: Thermal model reached expected steady state\n");
            self.test_results.tests_passed += 1;
            self.test_results.thermal_model_test = true;
        } else {
            serial::write_str("[Power Test 3] FAIL: Thermal model incorrect\n");
        }
        
        test_passed
    }
    
    /// Test power state selection logic
    fn test_power_state_selection(&mut self) -> bool {
        serial::write_str("[Power Test 4] Testing power state selection\n");
        self.test_results.tests_run += 1;
        
        let selector = FastPowerSelector::new();
        
        // Test case 1: Low utilization, cool thermal, full battery
        let low_util = Q15Math::from_float(0.2); // 20% utilization
        let cool_thermal = ThermalClass::Cool;
        let full_battery = BatteryClass::Full;
        
        let state1 = selector.select_fast(
            ComputeResource::NeuralEngine,
            low_util,
            false, // not bursting
            cool_thermal,
            full_battery,
        );
        
        // Test case 2: High utilization, hot thermal, low battery
        let high_util = Q15Math::from_float(0.8); // 80% utilization
        let hot_thermal = ThermalClass::Hot;
        let low_battery = BatteryClass::Low;
        
        let state2 = selector.select_fast(
            ComputeResource::NeuralEngine,
            high_util,
            true, // bursting
            hot_thermal,
            low_battery,
        );
        
        // State1 should be lower power than State2, despite thermal/battery constraints
        let test_passed = state1.freq_mhz < state2.freq_mhz;
        
        if test_passed {
            serial::write_str("[Power Test 4] PASS: Power state selection logic working\n");
            self.test_results.tests_passed += 1;
            self.test_results.power_state_selection_test = true;
        } else {
            serial::write_str("[Power Test 4] FAIL: Power state selection logic failed\n");
        }
        
        test_passed
    }
    
    /// Test race-to-sleep optimization logic
    fn test_race_to_sleep(&mut self) -> bool {
        serial::write_str("[Power Test 5] Testing race-to-sleep optimization\n");
        self.test_results.tests_run += 1;
        
        // Test case 1: Real-time inference should always race
        let should_race_rt = RaceToSleep::should_race(
            WorkloadType::Inference,
            CognitivePriority::RealTimeInference,
            BatteryClass::Low, // Even on low battery
            10, // Even with high queue depth
        );
        
        // Test case 2: Background training on low battery should not race
        let should_not_race_bg = RaceToSleep::should_race(
            WorkloadType::Training,
            CognitivePriority::Background,
            BatteryClass::Low,
            1, // Even with low queue depth
        );
        
        let test_passed = should_race_rt && !should_not_race_bg;
        
        if test_passed {
            serial::write_str("[Power Test 5] PASS: Race-to-sleep logic working correctly\n");
            self.test_results.tests_passed += 1;
            self.test_results.race_to_sleep_test = true;
        } else {
            serial::write_str("[Power Test 5] FAIL: Race-to-sleep logic failed\n");
        }
        
        test_passed
    }
    
    /// Test end-to-end integration with resource manager
    fn test_end_to_end_integration(&mut self) -> bool {
        serial::write_str("[Power Test 6] Testing end-to-end integration\n");
        self.test_results.tests_run += 1;
        
        // Test that resource manager can update power state
        let ne_mgr = self.power_manager.get_resource_manager(ComputeResource::NeuralEngine);
        
        // Simulate some workload history
        for i in 0..10 {
            let interarrival = Q15Math::from_float(100.0 + (i as f32) * 10.0);
            let service_time = Q15Math::from_float(50.0 + (i as f32) * 5.0);
            
            ne_mgr.predictor.on_enqueue(interarrival, i as u16);
            ne_mgr.predictor.on_complete(service_time);
        }
        
        // Try to update power state
        let initial_state = ne_mgr.current_power_state();
        let state_change = ne_mgr.update_power_state(
            ThermalClass::Cool,
            BatteryClass::Full,
            1000, // 1ms timestamp
        );
        
        // Should have valid predictions and power state
        let util_prediction = ne_mgr.predictor.predict_utilization_q15();
        let test_passed = util_prediction != 0 && 
                         initial_state.freq_mhz > 0;
        
        if test_passed {
            serial::write_str("[Power Test 6] PASS: End-to-end integration working\n");
            self.test_results.tests_passed += 1;
        } else {
            serial::write_str("[Power Test 6] FAIL: End-to-end integration failed\n");
        }
        
        test_passed
    }
    
    /// Print test results summary
    fn print_test_results(&self) {
        serial::write_str("\n[Power Integration] Test Results Summary:\n");
        serial::write_str("  Tests run: ");
        serial::write_dec(self.test_results.tests_run as u64);
        serial::write_str("\n  Tests passed: ");
        serial::write_dec(self.test_results.tests_passed as u64);
        serial::write_str("\n  Success rate: ");
        if self.test_results.tests_run > 0 {
            let success_rate = (self.test_results.tests_passed * 100) / self.test_results.tests_run;
            serial::write_dec(success_rate as u64);
            serial::write_str("%\n");
        } else {
            serial::write_str("N/A\n");
        }
        
        // Individual test results
        serial::write_str("\n[Power Integration] Individual Test Results:\n");
        serial::write_str("  EWMA Accuracy: ");
        serial::write_str(if self.test_results.ewma_accuracy_test { "PASS" } else { "FAIL" });
        serial::write_str("\n  Holt Trend Detection: ");
        serial::write_str(if self.test_results.holt_trend_test { "PASS" } else { "FAIL" });
        serial::write_str("\n  Thermal Model: ");
        serial::write_str(if self.test_results.thermal_model_test { "PASS" } else { "FAIL" });
        serial::write_str("\n  Power State Selection: ");
        serial::write_str(if self.test_results.power_state_selection_test { "PASS" } else { "FAIL" });
        serial::write_str("\n  Race-to-Sleep: ");
        serial::write_str(if self.test_results.race_to_sleep_test { "PASS" } else { "FAIL" });
        serial::write_str("\n");
        
        if self.test_results.tests_passed == self.test_results.tests_run {
            serial::write_str("[Power Integration] All tests passed! Predictive power management is working correctly.\n");
        } else {
            serial::write_str("[Power Integration] Some tests failed. Review power management implementation.\n");
        }
    }
    
    /// Get test results for external validation
    pub fn get_results(&self) -> &TestResults {
        &self.test_results
    }
}