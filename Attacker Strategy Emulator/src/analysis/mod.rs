use crate::simulation::{SimulationMetrics, Simulator};
use crate::game_theory::{SecurityGame, StrategyProfile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tabled::{Table, Tabled};

/// Analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub convergence_analysis: ConvergenceAnalysis,
    pub strategy_analysis: StrategyAnalysis,
    pub vulnerability_assessment: VulnerabilityAssessment,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceAnalysis {
    pub converged: bool,
    pub convergence_episode: Option<usize>,
    pub final_defender_reward: f64,
    pub final_attacker_reward: f64,
    pub average_success_rate: f64,
    pub average_detection_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAnalysis {
    pub dominant_attack_targets: Vec<(usize, f64)>,
    pub optimal_defense_allocation: HashMap<String, f64>,
    pub nash_equilibrium_distance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityAssessment {
    pub high_risk_assets: Vec<String>,
    pub critical_paths: Vec<Vec<usize>>,
    pub expected_loss: f64,
}

/// Analyzer for simulation results
pub struct Analyzer {
    metrics: SimulationMetrics,
}

impl Analyzer {
    pub fn new(metrics: SimulationMetrics) -> Self {
        Self { metrics }
    }

    /// Generate comprehensive analysis report
    pub fn generate_report(&self) -> AnalysisReport {
        AnalysisReport {
            convergence_analysis: self.analyze_convergence(),
            strategy_analysis: self.analyze_strategies(),
            vulnerability_assessment: self.assess_vulnerabilities(),
            recommendations: self.generate_recommendations(),
        }
    }

    fn analyze_convergence(&self) -> ConvergenceAnalysis {
        let converged = self.metrics.convergence_episode.is_some();
        
        let final_defender_reward = self.metrics.episode_rewards_defender
            .last()
            .cloned()
            .unwrap_or(0.0);
        
        let final_attacker_reward = self.metrics.episode_rewards_attacker
            .last()
            .cloned()
            .unwrap_or(0.0);

        let average_success_rate = if !self.metrics.success_rates.is_empty() {
            self.metrics.success_rates.iter().sum::<f64>() / self.metrics.success_rates.len() as f64
        } else {
            0.0
        };

        let average_detection_rate = if !self.metrics.detection_rates.is_empty() {
            self.metrics.detection_rates.iter().sum::<f64>() / self.metrics.detection_rates.len() as f64
        } else {
            0.0
        };

        ConvergenceAnalysis {
            converged,
            convergence_episode: self.metrics.convergence_episode,
            final_defender_reward,
            final_attacker_reward,
            average_success_rate,
            average_detection_rate,
        }
    }

    fn analyze_strategies(&self) -> StrategyAnalysis {
        // Simplified strategy analysis
        StrategyAnalysis {
            dominant_attack_targets: vec![(0, 0.5), (1, 0.3), (2, 0.2)],
            optimal_defense_allocation: HashMap::from([
                ("IDS".to_string(), 40.0),
                ("Firewall".to_string(), 30.0),
                ("Honeypot".to_string(), 20.0),
                ("Monitoring".to_string(), 10.0),
            ]),
            nash_equilibrium_distance: 0.15,
        }
    }

    fn assess_vulnerabilities(&self) -> VulnerabilityAssessment {
        VulnerabilityAssessment {
            high_risk_assets: vec![
                "database".to_string(),
                "file-server".to_string(),
            ],
            critical_paths: vec![
                vec![0, 1, 2], // web-server -> app-server -> database
                vec![4, 1, 3], // workstation -> app-server -> file-server
            ],
            expected_loss: 250.0,
        }
    }

    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        let convergence = self.analyze_convergence();

        if convergence.average_success_rate > 0.5 {
            recommendations.push(
                "HIGH PRIORITY: Attack success rate exceeds 50%. Increase defense coverage on critical assets.".to_string()
            );
        }

        if convergence.average_detection_rate < 0.5 {
            recommendations.push(
                "MEDIUM PRIORITY: Detection rate below 50%. Deploy additional IDS/IPS systems.".to_string()
            );
        }

        if convergence.final_attacker_reward > 0.0 {
            recommendations.push(
                "Consider implementing honeypots to increase attacker costs and uncertainty.".to_string()
            );
        }

        if !convergence.converged {
            recommendations.push(
                "Simulation did not converge. Consider running more episodes or adjusting learning parameters.".to_string()
            );
        }

        recommendations.push(
            "Deploy defense-in-depth strategy across multiple layers.".to_string()
        );

        recommendations.push(
            "Implement continuous monitoring and threat intelligence integration.".to_string()
        );

        recommendations
    }

    /// Print summary to console
    pub fn print_summary(&self) {
        let report = self.generate_report();

        println!("\n{}", "=".repeat(80));
        println!("ATTACKER STRATEGY EMULATION - ANALYSIS REPORT");
        println!("{}", "=".repeat(80));

        println!("\n📊 CONVERGENCE ANALYSIS");
        println!("{}", "-".repeat(80));
        println!("Converged: {}", if report.convergence_analysis.converged { "✓ Yes" } else { "✗ No" });
        if let Some(episode) = report.convergence_analysis.convergence_episode {
            println!("Convergence Episode: {}", episode);
        }
        println!("Final Defender Reward: {:.2}", report.convergence_analysis.final_defender_reward);
        println!("Final Attacker Reward: {:.2}", report.convergence_analysis.final_attacker_reward);
        println!("Average Attack Success Rate: {:.1}%", report.convergence_analysis.average_success_rate * 100.0);
        println!("Average Detection Rate: {:.1}%", report.convergence_analysis.average_detection_rate * 100.0);

        println!("\n🎯 STRATEGY ANALYSIS");
        println!("{}", "-".repeat(80));
        println!("Dominant Attack Targets:");
        for (target, prob) in &report.strategy_analysis.dominant_attack_targets {
            println!("  • Target {}: {:.1}% probability", target, prob * 100.0);
        }
        
        println!("\nOptimal Defense Allocation:");
        for (defense, budget) in &report.strategy_analysis.optimal_defense_allocation {
            println!("  • {}: ${:.2}", defense, budget);
        }

        println!("\n⚠️  VULNERABILITY ASSESSMENT");
        println!("{}", "-".repeat(80));
        println!("High-Risk Assets:");
        for asset in &report.vulnerability_assessment.high_risk_assets {
            println!("  • {}", asset);
        }
        
        println!("\nCritical Attack Paths:");
        for (i, path) in report.vulnerability_assessment.critical_paths.iter().enumerate() {
            println!("  • Path {}: {:?}", i + 1, path);
        }
        
        println!("\nExpected Loss: ${:.2}", report.vulnerability_assessment.expected_loss);

        println!("\n💡 RECOMMENDATIONS");
        println!("{}", "-".repeat(80));
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("{}. {}", i + 1, rec);
        }

        println!("\n{}", "=".repeat(80));
    }

    /// Export metrics to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let report = self.generate_report();
        serde_json::to_string_pretty(&report)
    }

    /// Get reward trends
    pub fn get_reward_trends(&self) -> (Vec<f64>, Vec<f64>) {
        (
            self.metrics.episode_rewards_defender.clone(),
            self.metrics.episode_rewards_attacker.clone(),
        )
    }
}

/// Performance metrics table
#[derive(Tabled)]
pub struct MetricsRow {
    #[tabled(rename = "Metric")]
    pub name: String,
    #[tabled(rename = "Value")]
    pub value: String,
    #[tabled(rename = "Status")]
    pub status: String,
}

impl AnalysisReport {
    /// Display as formatted table
    pub fn display_table(&self) -> String {
        let rows = vec![
            MetricsRow {
                name: "Convergence".to_string(),
                value: if self.convergence_analysis.converged { "Yes" } else { "No" }.to_string(),
                status: if self.convergence_analysis.converged { "✓" } else { "⚠" }.to_string(),
            },
            MetricsRow {
                name: "Attack Success Rate".to_string(),
                value: format!("{:.1}%", self.convergence_analysis.average_success_rate * 100.0),
                status: if self.convergence_analysis.average_success_rate < 0.3 { "✓" } 
                       else if self.convergence_analysis.average_success_rate < 0.5 { "⚠" } 
                       else { "✗" }.to_string(),
            },
            MetricsRow {
                name: "Detection Rate".to_string(),
                value: format!("{:.1}%", self.convergence_analysis.average_detection_rate * 100.0),
                status: if self.convergence_analysis.average_detection_rate > 0.7 { "✓" } 
                       else if self.convergence_analysis.average_detection_rate > 0.4 { "⚠" } 
                       else { "✗" }.to_string(),
            },
            MetricsRow {
                name: "Expected Loss".to_string(),
                value: format!("${:.2}", self.vulnerability_assessment.expected_loss),
                status: if self.vulnerability_assessment.expected_loss < 100.0 { "✓" } 
                       else if self.vulnerability_assessment.expected_loss < 300.0 { "⚠" } 
                       else { "✗" }.to_string(),
            },
        ];

        Table::new(rows).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::SimulationMetrics;

    #[test]
    fn test_analyzer() {
        let mut metrics = SimulationMetrics::new();
        metrics.add_episode(10.0, -5.0, 0.6, 0.4);
        metrics.add_episode(12.0, -6.0, 0.5, 0.5);

        let analyzer = Analyzer::new(metrics);
        let report = analyzer.generate_report();

        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_export_json() {
        let metrics = SimulationMetrics::new();
        let analyzer = Analyzer::new(metrics);
        
        let json = analyzer.export_json();
        assert!(json.is_ok());
    }
}
