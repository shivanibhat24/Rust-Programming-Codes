use crate::attack::{AttackStrategy, AttackTechnique, create_default_techniques};
use crate::defense::{Asset, DefenseConfiguration, DefenseStrategy, NetworkGraph};
use crate::game_theory::{SecurityGame, Action, Player, StackelbergSolver, StrategyProfile};
use crate::ml::{DQNAgent, Experience};
use ndarray::Array1;
use petgraph::graph::{Graph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub num_episodes: usize,
    pub max_steps_per_episode: usize,
    pub defender_budget: f64,
    pub attacker_budget: f64,
    pub learning_rate: f64,
    pub discount_factor: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            num_episodes: 1000,
            max_steps_per_episode: 50,
            defender_budget: 100.0,
            attacker_budget: 50.0,
            learning_rate: 0.001,
            discount_factor: 0.99,
        }
    }
}

/// Simulation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub episode: usize,
    pub step: usize,
    pub defender_total_reward: f64,
    pub attacker_total_reward: f64,
    pub attacks_succeeded: usize,
    pub attacks_detected: usize,
    pub current_defense: DefenseConfiguration,
}

/// Simulation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub episode_rewards_defender: Vec<f64>,
    pub episode_rewards_attacker: Vec<f64>,
    pub success_rates: Vec<f64>,
    pub detection_rates: Vec<f64>,
    pub nash_distances: Vec<f64>,
    pub convergence_episode: Option<usize>,
}

impl SimulationMetrics {
    pub fn new() -> Self {
        Self {
            episode_rewards_defender: Vec::new(),
            episode_rewards_attacker: Vec::new(),
            success_rates: Vec::new(),
            detection_rates: Vec::new(),
            nash_distances: Vec::new(),
            convergence_episode: None,
        }
    }

    pub fn add_episode(
        &mut self,
        defender_reward: f64,
        attacker_reward: f64,
        success_rate: f64,
        detection_rate: f64,
    ) {
        self.episode_rewards_defender.push(defender_reward);
        self.episode_rewards_attacker.push(attacker_reward);
        self.success_rates.push(success_rate);
        self.detection_rates.push(detection_rate);
    }

    pub fn check_convergence(&mut self, window: usize, threshold: f64) -> bool {
        if self.convergence_episode.is_some() {
            return true;
        }

        if self.episode_rewards_defender.len() < window {
            return false;
        }

        let recent = &self.episode_rewards_defender[self.episode_rewards_defender.len() - window..];
        let mean = recent.iter().sum::<f64>() / window as f64;
        let variance = recent.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / window as f64;

        if variance < threshold {
            self.convergence_episode = Some(self.episode_rewards_defender.len());
            true
        } else {
            false
        }
    }
}

impl Default for SimulationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Main simulation engine
pub struct Simulator {
    config: SimulationConfig,
    network: NetworkGraph,
    techniques: Vec<AttackTechnique>,
    attacker_agent: DQNAgent,
    metrics: SimulationMetrics,
}

impl Simulator {
    pub fn new(config: SimulationConfig, network: NetworkGraph) -> Self {
        let techniques = create_default_techniques();
        
        // State size: network size + defense features
        let state_size = network.node_count() * 3; // value, vulnerability, defense_coverage
        // Action size: number of nodes to target
        let action_size = network.node_count();
        
        let attacker_agent = DQNAgent::new(
            state_size,
            action_size,
            128,
            config.learning_rate,
            10000,
        );

        Self {
            config,
            network,
            techniques,
            attacker_agent,
            metrics: SimulationMetrics::new(),
        }
    }

    /// Run full simulation
    pub fn run(&mut self) -> SimulationMetrics {
        tracing::info!("Starting simulation with {} episodes", self.config.num_episodes);

        for episode in 0..self.config.num_episodes {
            let (defender_reward, attacker_reward, stats) = self.run_episode(episode);
            
            self.metrics.add_episode(
                defender_reward,
                attacker_reward,
                stats.success_rate,
                stats.detection_rate,
            );

            if episode % 100 == 0 {
                tracing::info!(
                    "Episode {}: Defender={:.2}, Attacker={:.2}, Success={:.2}%, Detection={:.2}%",
                    episode,
                    defender_reward,
                    attacker_reward,
                    stats.success_rate * 100.0,
                    stats.detection_rate * 100.0
                );
            }

            // Check convergence
            if self.metrics.check_convergence(50, 1.0) {
                tracing::info!("Converged at episode {}", episode);
                break;
            }
        }

        self.metrics.clone()
    }

    /// Run single episode
    fn run_episode(&mut self, episode: usize) -> (f64, f64, EpisodeStats) {
        // Defender moves first (Stackelberg)
        let defense_strategy = DefenseStrategy::new(
            self.network.clone(),
            self.config.defender_budget,
        );
        let defense_config = defense_strategy.generate_greedy();

        let mut total_defender_reward = 0.0;
        let mut total_attacker_reward = 0.0;
        let mut attacks_succeeded = 0;
        let mut attacks_detected = 0;
        let mut total_attacks = 0;

        // Attacker observes and responds
        for step in 0..self.config.max_steps_per_episode {
            let state = self.encode_state(&defense_config);
            let target_idx = self.attacker_agent.select_action(&state);
            
            if target_idx >= self.network.node_count() {
                continue;
            }

            let target = NodeIndex::new(target_idx);
            total_attacks += 1;

            // Simulate attack
            let attack_strategy = AttackStrategy::new(
                self.network.clone(),
                self.techniques.clone(),
                defense_config.clone(),
            );

            if let Some(attack_path) = attack_strategy.generate_optimal_path(target) {
                let success = attack_path.success_probability > 0.5;
                let detected = attack_path.detection_probability > 0.3;

                let reward = if success {
                    attacks_succeeded += 1;
                    attack_path.expected_value
                } else {
                    -attack_path.total_cost
                };

                if detected {
                    attacks_detected += 1;
                }

                // Defender reward is negative of attacker success
                let defender_reward = if success { -reward } else { reward.abs() * 0.5 };
                
                total_attacker_reward += reward;
                total_defender_reward += defender_reward;

                // Store experience for learning
                let next_state = self.encode_state(&defense_config);
                self.attacker_agent.store_experience(Experience {
                    state: state.clone(),
                    action: target_idx,
                    reward,
                    next_state,
                    done: step == self.config.max_steps_per_episode - 1,
                });

                // Train agent
                self.attacker_agent.train();
            }
        }

        let stats = EpisodeStats {
            success_rate: if total_attacks > 0 {
                attacks_succeeded as f64 / total_attacks as f64
            } else {
                0.0
            },
            detection_rate: if total_attacks > 0 {
                attacks_detected as f64 / total_attacks as f64
            } else {
                0.0
            },
        };

        (total_defender_reward, total_attacker_reward, stats)
    }

    /// Encode current state for ML agent
    fn encode_state(&self, defense: &DefenseConfiguration) -> Array1<f64> {
        let mut state = Vec::new();

        for node in self.network.node_indices() {
            let asset = &self.network[node];
            state.push(asset.value / 100.0); // Normalize
            state.push(asset.vulnerability);
            state.push(defense.get_coverage(node));
        }

        Array1::from_vec(state)
    }

    /// Get trained attacker policy
    pub fn get_attacker_policy(&self) -> Array1<f64> {
        let state = self.encode_state(&DefenseConfiguration::new(self.config.defender_budget));
        self.attacker_agent.get_policy(&state)
    }

    /// Get simulation metrics
    pub fn metrics(&self) -> &SimulationMetrics {
        &self.metrics
    }

    /// Optimize defense against learned attacker
    pub fn optimize_defense(&self) -> DefenseConfiguration {
        let defense_strategy = DefenseStrategy::new(
            self.network.clone(),
            self.config.defender_budget,
        );

        // Use attacker policy to inform defense
        let attacker_policy = self.get_attacker_policy();
        defense_strategy.from_strategy_vector(&attacker_policy)
    }
}

#[derive(Debug, Clone)]
struct EpisodeStats {
    success_rate: f64,
    detection_rate: f64,
}

/// Utility function to create example network
pub fn create_example_network() -> NetworkGraph {
    let mut graph = Graph::new();

    // Create nodes representing different assets
    let web_server = graph.add_node(Asset {
        id: "web-server".to_string(),
        value: 50.0,
        vulnerability: 0.7,
        criticality: 0.6,
    });

    let database = graph.add_node(Asset {
        id: "database".to_string(),
        value: 100.0,
        vulnerability: 0.5,
        criticality: 1.0,
    });

    let app_server = graph.add_node(Asset {
        id: "app-server".to_string(),
        value: 70.0,
        vulnerability: 0.6,
        criticality: 0.8,
    });

    let file_server = graph.add_node(Asset {
        id: "file-server".to_string(),
        value: 80.0,
        vulnerability: 0.4,
        criticality: 0.7,
    });

    let workstation = graph.add_node(Asset {
        id: "workstation".to_string(),
        value: 30.0,
        vulnerability: 0.8,
        criticality: 0.4,
    });

    // Add edges representing network connections
    graph.add_edge(web_server, app_server, 1.0);
    graph.add_edge(app_server, database, 1.0);
    graph.add_edge(app_server, file_server, 1.0);
    graph.add_edge(workstation, app_server, 1.0);
    graph.add_edge(workstation, file_server, 1.0);

    graph
}

/// Run quick test simulation
pub fn run_quick_test() -> SimulationMetrics {
    let config = SimulationConfig {
        num_episodes: 100,
        max_steps_per_episode: 20,
        defender_budget: 100.0,
        attacker_budget: 50.0,
        learning_rate: 0.01,
        discount_factor: 0.99,
    };

    let network = create_example_network();
    let mut simulator = Simulator::new(config, network);
    simulator.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_creation() {
        let config = SimulationConfig::default();
        let network = create_example_network();
        let simulator = Simulator::new(config, network);
        
        assert!(simulator.network.node_count() > 0);
    }

    #[test]
    fn test_metrics() {
        let mut metrics = SimulationMetrics::new();
        metrics.add_episode(10.0, -5.0, 0.7, 0.3);
        
        assert_eq!(metrics.episode_rewards_defender.len(), 1);
        assert_eq!(metrics.episode_rewards_attacker.len(), 1);
    }

    #[test]
    fn test_quick_simulation() {
        let metrics = run_quick_test();
        assert!(!metrics.episode_rewards_defender.is_empty());
    }
}
