//! # Attacker Strategy Emulator
//! 
//! A game-theoretic security simulation framework that models adaptive adversaries
//! using machine learning. This library provides tools to:
//! 
//! - Model security as a Stackelberg game between defenders and attackers
//! - Train ML agents to discover optimal attack strategies
//! - Analyze defense effectiveness against rational adversaries
//! - Generate actionable security recommendations
//! 
//! ## Key Features
//! 
//! - **Game Theory**: Stackelberg games, Nash equilibrium solving
//! - **Machine Learning**: DQN agents for strategy learning
//! - **Network Modeling**: Graph-based asset and attack path modeling
//! - **Defense Optimization**: Automated defense allocation and optimization
//! - **Attack Simulation**: MITRE ATT&CK-based attack chain modeling
//! 
//! ## Quick Start
//! 
//! ```rust,no_run
//! use attacker_strategy_emulator::simulation::{Simulator, SimulationConfig, create_example_network};
//! use attacker_strategy_emulator::analysis::Analyzer;
//! 
//! let config = SimulationConfig::default();
//! let network = create_example_network();
//! let mut simulator = Simulator::new(config, network);
//! 
//! let metrics = simulator.run();
//! let analyzer = Analyzer::new(metrics);
//! analyzer.print_summary();
//! ```

pub mod game_theory;
pub mod ml;
pub mod defense;
pub mod attack;
pub mod simulation;
pub mod analysis;

// Re-export commonly used types
pub use game_theory::{SecurityGame, Action, Player, StrategyProfile, NashSolver, StackelbergSolver};
pub use ml::{DQNAgent, PolicyGradientAgent, QNetwork, ReplayBuffer};
pub use defense::{DefenseStrategy, DefenseConfiguration, DefenseType, Asset};
pub use attack::{AttackStrategy, AttackTechnique, AttackPhase, AttackPath, create_default_techniques};
pub use simulation::{Simulator, SimulationConfig, SimulationMetrics, create_example_network};
pub use analysis::{Analyzer, AnalysisReport};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
