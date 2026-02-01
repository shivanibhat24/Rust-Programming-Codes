use ndarray::Array1;
use petgraph::graph::{Graph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of defense mechanisms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefenseType {
    Honeypot,
    IDS,
    IPS,
    Firewall,
    Patching,
    AccessControl,
    Encryption,
    Monitoring,
}

/// Defense resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseAllocation {
    pub defense_type: DefenseType,
    pub target: NodeIndex,
    pub coverage: f64,      // 0.0 to 1.0
    pub effectiveness: f64, // Detection/prevention rate
    pub cost: f64,
}

/// Network node representing an asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub value: f64,
    pub vulnerability: f64,
    pub criticality: f64,
}

/// Defense configuration for the entire network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseConfiguration {
    pub allocations: Vec<DefenseAllocation>,
    pub total_budget: f64,
    pub total_cost: f64,
}

impl DefenseConfiguration {
    pub fn new(budget: f64) -> Self {
        Self {
            allocations: Vec::new(),
            total_budget: budget,
            total_cost: 0.0,
        }
    }

    pub fn add_allocation(&mut self, allocation: DefenseAllocation) -> Result<(), String> {
        if self.total_cost + allocation.cost > self.total_budget {
            return Err("Budget exceeded".to_string());
        }

        self.total_cost += allocation.cost;
        self.allocations.push(allocation);
        Ok(())
    }

    pub fn remaining_budget(&self) -> f64 {
        self.total_budget - self.total_cost
    }

    pub fn get_coverage(&self, node: NodeIndex) -> f64 {
        self.allocations
            .iter()
            .filter(|a| a.target == node)
            .map(|a| a.coverage)
            .sum::<f64>()
            .min(1.0)
    }

    pub fn get_effectiveness(&self, node: NodeIndex, defense_type: DefenseType) -> f64 {
        self.allocations
            .iter()
            .filter(|a| a.target == node && a.defense_type == defense_type)
            .map(|a| a.effectiveness)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }
}

/// Network topology for security game
pub type NetworkGraph = Graph<Asset, f64>;

/// Defense strategy generator
pub struct DefenseStrategy {
    network: NetworkGraph,
    budget: f64,
}

impl DefenseStrategy {
    pub fn new(network: NetworkGraph, budget: f64) -> Self {
        Self { network, budget }
    }

    /// Generate optimal defense allocation based on asset values
    pub fn generate_greedy(&self) -> DefenseConfiguration {
        let mut config = DefenseConfiguration::new(self.budget);

        // Sort nodes by value * vulnerability
        let mut node_scores: Vec<(NodeIndex, f64)> = self
            .network
            .node_indices()
            .map(|idx| {
                let asset = &self.network[idx];
                (idx, asset.value * asset.vulnerability * asset.criticality)
            })
            .collect();

        node_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Allocate defenses to highest-value nodes
        let defense_types = [
            DefenseType::IDS,
            DefenseType::Firewall,
            DefenseType::Honeypot,
            DefenseType::Monitoring,
        ];

        for (node, _score) in node_scores {
            for &defense_type in &defense_types {
                let cost = self.estimate_defense_cost(defense_type);
                
                if config.total_cost + cost <= config.total_budget {
                    let allocation = DefenseAllocation {
                        defense_type,
                        target: node,
                        coverage: 0.8,
                        effectiveness: self.estimate_effectiveness(defense_type),
                        cost,
                    };

                    let _ = config.add_allocation(allocation);
                }
            }

            if config.remaining_budget() < 1.0 {
                break;
            }
        }

        config
    }

    /// Generate uniform defense allocation
    pub fn generate_uniform(&self) -> DefenseConfiguration {
        let mut config = DefenseConfiguration::new(self.budget);
        let node_count = self.network.node_count();
        let budget_per_node = self.budget / node_count as f64;

        for node in self.network.node_indices() {
            let allocation = DefenseAllocation {
                defense_type: DefenseType::IDS,
                target: node,
                coverage: 0.5,
                effectiveness: 0.6,
                cost: budget_per_node,
            };

            let _ = config.add_allocation(allocation);
        }

        config
    }

    /// Generate defense from strategy vector
    pub fn from_strategy_vector(&self, strategy: &Array1<f64>) -> DefenseConfiguration {
        let mut config = DefenseConfiguration::new(self.budget);
        let nodes: Vec<NodeIndex> = self.network.node_indices().collect();

        for (i, &prob) in strategy.iter().enumerate() {
            if i >= nodes.len() {
                break;
            }

            if prob > 0.3 {
                // Allocate defense proportional to probability
                let cost = self.budget * prob / strategy.sum();
                let allocation = DefenseAllocation {
                    defense_type: DefenseType::IDS,
                    target: nodes[i],
                    coverage: prob,
                    effectiveness: 0.7 * prob,
                    cost,
                };

                let _ = config.add_allocation(allocation);
            }
        }

        config
    }

    fn estimate_defense_cost(&self, defense_type: DefenseType) -> f64 {
        match defense_type {
            DefenseType::Honeypot => 5.0,
            DefenseType::IDS => 10.0,
            DefenseType::IPS => 15.0,
            DefenseType::Firewall => 8.0,
            DefenseType::Patching => 3.0,
            DefenseType::AccessControl => 7.0,
            DefenseType::Encryption => 12.0,
            DefenseType::Monitoring => 6.0,
        }
    }

    fn estimate_effectiveness(&self, defense_type: DefenseType) -> f64 {
        match defense_type {
            DefenseType::Honeypot => 0.4,
            DefenseType::IDS => 0.7,
            DefenseType::IPS => 0.85,
            DefenseType::Firewall => 0.6,
            DefenseType::Patching => 0.9,
            DefenseType::AccessControl => 0.75,
            DefenseType::Encryption => 0.95,
            DefenseType::Monitoring => 0.5,
        }
    }

    pub fn network(&self) -> &NetworkGraph {
        &self.network
    }
}

/// Defense optimizer using game-theoretic principles
pub struct DefenseOptimizer {
    strategy: DefenseStrategy,
    attacker_profile: AttackerProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackerProfile {
    pub skill_level: f64,
    pub resources: f64,
    pub objectives: Vec<String>,
}

impl DefenseOptimizer {
    pub fn new(strategy: DefenseStrategy, attacker_profile: AttackerProfile) -> Self {
        Self {
            strategy,
            attacker_profile,
        }
    }

    /// Optimize defense against rational attacker
    pub fn optimize(&self) -> DefenseConfiguration {
        // Start with greedy allocation
        let mut best_config = self.strategy.generate_greedy();
        let mut best_score = self.evaluate_defense(&best_config);

        // Try variations
        for _ in 0..10 {
            let candidate = self.generate_variation(&best_config);
            let score = self.evaluate_defense(&candidate);

            if score > best_score {
                best_score = score;
                best_config = candidate;
            }
        }

        best_config
    }

    fn evaluate_defense(&self, config: &DefenseConfiguration) -> f64 {
        let network = self.strategy.network();
        let mut score = 0.0;

        for node in network.node_indices() {
            let asset = &network[node];
            let coverage = config.get_coverage(node);
            let protection = coverage * asset.value;
            score += protection;
        }

        // Penalty for unspent budget
        score -= (config.remaining_budget() / config.total_budget) * 10.0;

        score
    }

    fn generate_variation(&self, base: &DefenseConfiguration) -> DefenseConfiguration {
        // Simple variation: swap two allocations
        let mut new_config = base.clone();
        
        if new_config.allocations.len() >= 2 {
            let mut rng = rand::thread_rng();
            use rand::Rng;
            let idx1 = rng.gen_range(0..new_config.allocations.len());
            let idx2 = rng.gen_range(0..new_config.allocations.len());
            
            if idx1 != idx2 {
                new_config.allocations.swap(idx1, idx2);
            }
        }

        new_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_network() -> NetworkGraph {
        let mut graph = Graph::new();
        
        graph.add_node(Asset {
            id: "server1".to_string(),
            value: 100.0,
            vulnerability: 0.7,
            criticality: 0.9,
        });
        
        graph.add_node(Asset {
            id: "server2".to_string(),
            value: 80.0,
            vulnerability: 0.5,
            criticality: 0.7,
        });

        graph
    }

    #[test]
    fn test_defense_configuration() {
        let mut config = DefenseConfiguration::new(100.0);
        
        let allocation = DefenseAllocation {
            defense_type: DefenseType::IDS,
            target: NodeIndex::new(0),
            coverage: 0.8,
            effectiveness: 0.7,
            cost: 10.0,
        };

        assert!(config.add_allocation(allocation).is_ok());
        assert_eq!(config.total_cost, 10.0);
        assert_eq!(config.remaining_budget(), 90.0);
    }

    #[test]
    fn test_defense_strategy() {
        let network = create_test_network();
        let strategy = DefenseStrategy::new(network, 100.0);
        
        let config = strategy.generate_greedy();
        assert!(config.total_cost <= 100.0);
        assert!(!config.allocations.is_empty());
    }
}
