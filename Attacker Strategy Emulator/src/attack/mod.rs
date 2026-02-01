use ndarray::Array1;
use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::defense::{Asset, DefenseConfiguration, NetworkGraph};

/// Attack phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackPhase {
    Reconnaissance,
    InitialAccess,
    Execution,
    Persistence,
    PrivilegeEscalation,
    DefenseEvasion,
    CredentialAccess,
    Discovery,
    LateralMovement,
    Collection,
    Exfiltration,
    Impact,
}

/// Attack technique
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackTechnique {
    pub id: String,
    pub name: String,
    pub phase: AttackPhase,
    pub success_rate: f64,
    pub detectability: f64,
    pub cost: f64,
    pub required_access: AccessLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AccessLevel {
    None,
    User,
    Admin,
    Root,
}

/// Attack path through the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPath {
    pub nodes: Vec<NodeIndex>,
    pub techniques: Vec<AttackTechnique>,
    pub total_cost: f64,
    pub success_probability: f64,
    pub detection_probability: f64,
    pub expected_value: f64,
}

impl AttackPath {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            techniques: Vec::new(),
            total_cost: 0.0,
            success_probability: 1.0,
            detection_probability: 0.0,
            expected_value: 0.0,
        }
    }

    pub fn add_step(
        &mut self,
        node: NodeIndex,
        technique: AttackTechnique,
        defense_effectiveness: f64,
    ) {
        self.nodes.push(node);
        
        // Update probabilities
        let detection = technique.detectability * defense_effectiveness;
        self.success_probability *= technique.success_rate * (1.0 - detection);
        self.detection_probability = 1.0 - (1.0 - self.detection_probability) * (1.0 - detection);
        
        self.total_cost += technique.cost;
        self.techniques.push(technique);
    }

    pub fn calculate_expected_value(&mut self, target_value: f64) {
        let capture_value = target_value * self.success_probability;
        let penalty = self.detection_probability * target_value * 0.5; // Cost of being detected
        self.expected_value = capture_value - penalty - self.total_cost;
    }
}

impl Default for AttackPath {
    fn default() -> Self {
        Self::new()
    }
}

/// Attack strategy generator
pub struct AttackStrategy {
    network: NetworkGraph,
    techniques: Vec<AttackTechnique>,
    defense: DefenseConfiguration,
}

impl AttackStrategy {
    pub fn new(
        network: NetworkGraph,
        techniques: Vec<AttackTechnique>,
        defense: DefenseConfiguration,
    ) -> Self {
        Self {
            network,
            techniques,
            defense,
        }
    }

    /// Generate optimal attack path using dynamic programming
    pub fn generate_optimal_path(&self, target: NodeIndex) -> Option<AttackPath> {
        let target_asset = &self.network[target];
        let mut best_path = None;
        let mut best_value = f64::NEG_INFINITY;

        // Try different entry points
        for entry in self.network.node_indices() {
            if let Some(path) = self.find_path_to_target(entry, target) {
                if path.expected_value > best_value {
                    best_value = path.expected_value;
                    best_path = Some(path);
                }
            }
        }

        best_path
    }

    /// Find attack path from entry to target
    fn find_path_to_target(&self, entry: NodeIndex, target: NodeIndex) -> Option<AttackPath> {
        let mut path = AttackPath::new();
        let mut current = entry;
        let mut visited = HashSet::new();
        let mut access_level = AccessLevel::None;

        visited.insert(current);

        // Simulate attack progression
        while current != target {
            // Try to move laterally
            let neighbors: Vec<NodeIndex> = self
                .network
                .neighbors(current)
                .filter(|n| !visited.contains(n))
                .collect();

            if neighbors.is_empty() {
                return None; // Dead end
            }

            // Choose best neighbor
            let next = *neighbors.first()?;
            visited.insert(next);

            // Select technique for lateral movement
            let technique = self.select_technique(AttackPhase::LateralMovement, access_level)?;
            let defense_effectiveness = self.defense.get_effectiveness(next, crate::defense::DefenseType::IDS);
            
            path.add_step(next, technique, defense_effectiveness);
            current = next;

            // Update access level (simplified)
            if path.success_probability > 0.7 {
                access_level = AccessLevel::User;
            }
            if path.success_probability > 0.5 && path.techniques.len() > 3 {
                access_level = AccessLevel::Admin;
            }
        }

        // Add final exfiltration step
        if let Some(exfil_technique) = self.select_technique(AttackPhase::Exfiltration, access_level) {
            let defense_effectiveness = self.defense.get_effectiveness(target, crate::defense::DefenseType::IDS);
            path.add_step(target, exfil_technique, defense_effectiveness);
        }

        // Calculate expected value
        let target_value = self.network[target].value;
        path.calculate_expected_value(target_value);

        Some(path)
    }

    /// Select best technique for a phase
    fn select_technique(&self, phase: AttackPhase, access: AccessLevel) -> Option<AttackTechnique> {
        self.techniques
            .iter()
            .filter(|t| t.phase == phase && t.required_access <= access)
            .max_by(|a, b| {
                let score_a = a.success_rate / (1.0 + a.cost);
                let score_b = b.success_rate / (1.0 + b.cost);
                score_a.partial_cmp(&score_b).unwrap()
            })
            .cloned()
    }

    /// Generate attack strategy from ML policy
    pub fn from_policy(&self, policy: &Array1<f64>) -> Vec<AttackPath> {
        let mut paths = Vec::new();
        
        // Each policy value represents probability of attacking a node
        for (i, &prob) in policy.iter().enumerate() {
            if prob > 0.2 && i < self.network.node_count() {
                let target = NodeIndex::new(i);
                if let Some(path) = self.generate_optimal_path(target) {
                    paths.push(path);
                }
            }
        }

        // Sort by expected value
        paths.sort_by(|a, b| b.expected_value.partial_cmp(&a.expected_value).unwrap());
        paths
    }

    /// Evaluate attack strategy against defense
    pub fn evaluate(&self, target: NodeIndex) -> f64 {
        if let Some(path) = self.generate_optimal_path(target) {
            path.expected_value
        } else {
            f64::NEG_INFINITY
        }
    }
}

/// Kill chain analyzer
pub struct KillChainAnalyzer {
    attack_graph: HashMap<AttackPhase, Vec<AttackTechnique>>,
}

impl KillChainAnalyzer {
    pub fn new(techniques: Vec<AttackTechnique>) -> Self {
        let mut attack_graph = HashMap::new();
        
        for technique in techniques {
            attack_graph
                .entry(technique.phase)
                .or_insert_with(Vec::new)
                .push(technique);
        }

        Self { attack_graph }
    }

    /// Analyze possible kill chains
    pub fn analyze_kill_chains(&self) -> Vec<Vec<AttackPhase>> {
        vec![
            vec![
                AttackPhase::Reconnaissance,
                AttackPhase::InitialAccess,
                AttackPhase::Execution,
                AttackPhase::Persistence,
                AttackPhase::LateralMovement,
                AttackPhase::Exfiltration,
            ],
            vec![
                AttackPhase::Reconnaissance,
                AttackPhase::InitialAccess,
                AttackPhase::PrivilegeEscalation,
                AttackPhase::CredentialAccess,
                AttackPhase::LateralMovement,
                AttackPhase::Exfiltration,
            ],
            vec![
                AttackPhase::Reconnaissance,
                AttackPhase::InitialAccess,
                AttackPhase::DefenseEvasion,
                AttackPhase::Discovery,
                AttackPhase::Collection,
                AttackPhase::Impact,
            ],
        ]
    }

    /// Calculate success probability for kill chain
    pub fn kill_chain_probability(&self, chain: &[AttackPhase]) -> f64 {
        let mut prob = 1.0;

        for phase in chain {
            if let Some(techniques) = self.attack_graph.get(phase) {
                let max_success = techniques
                    .iter()
                    .map(|t| t.success_rate)
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);
                prob *= max_success;
            } else {
                return 0.0;
            }
        }

        prob
    }
}

/// Utility functions for creating attack techniques
pub fn create_default_techniques() -> Vec<AttackTechnique> {
    vec![
        AttackTechnique {
            id: "T1595".to_string(),
            name: "Active Scanning".to_string(),
            phase: AttackPhase::Reconnaissance,
            success_rate: 0.95,
            detectability: 0.3,
            cost: 1.0,
            required_access: AccessLevel::None,
        },
        AttackTechnique {
            id: "T1190".to_string(),
            name: "Exploit Public-Facing Application".to_string(),
            phase: AttackPhase::InitialAccess,
            success_rate: 0.6,
            detectability: 0.5,
            cost: 5.0,
            required_access: AccessLevel::None,
        },
        AttackTechnique {
            id: "T1566".to_string(),
            name: "Phishing".to_string(),
            phase: AttackPhase::InitialAccess,
            success_rate: 0.7,
            detectability: 0.4,
            cost: 2.0,
            required_access: AccessLevel::None,
        },
        AttackTechnique {
            id: "T1059".to_string(),
            name: "Command and Scripting Interpreter".to_string(),
            phase: AttackPhase::Execution,
            success_rate: 0.8,
            detectability: 0.6,
            cost: 3.0,
            required_access: AccessLevel::User,
        },
        AttackTechnique {
            id: "T1078".to_string(),
            name: "Valid Accounts".to_string(),
            phase: AttackPhase::Persistence,
            success_rate: 0.85,
            detectability: 0.3,
            cost: 4.0,
            required_access: AccessLevel::User,
        },
        AttackTechnique {
            id: "T1548".to_string(),
            name: "Abuse Elevation Control Mechanism".to_string(),
            phase: AttackPhase::PrivilegeEscalation,
            success_rate: 0.5,
            detectability: 0.7,
            cost: 6.0,
            required_access: AccessLevel::User,
        },
        AttackTechnique {
            id: "T1027".to_string(),
            name: "Obfuscated Files or Information".to_string(),
            phase: AttackPhase::DefenseEvasion,
            success_rate: 0.7,
            detectability: 0.4,
            cost: 3.0,
            required_access: AccessLevel::User,
        },
        AttackTechnique {
            id: "T1110".to_string(),
            name: "Brute Force".to_string(),
            phase: AttackPhase::CredentialAccess,
            success_rate: 0.4,
            detectability: 0.8,
            cost: 5.0,
            required_access: AccessLevel::User,
        },
        AttackTechnique {
            id: "T1018".to_string(),
            name: "Remote System Discovery".to_string(),
            phase: AttackPhase::Discovery,
            success_rate: 0.9,
            detectability: 0.5,
            cost: 2.0,
            required_access: AccessLevel::User,
        },
        AttackTechnique {
            id: "T1021".to_string(),
            name: "Remote Services".to_string(),
            phase: AttackPhase::LateralMovement,
            success_rate: 0.7,
            detectability: 0.6,
            cost: 4.0,
            required_access: AccessLevel::User,
        },
        AttackTechnique {
            id: "T1560".to_string(),
            name: "Archive Collected Data".to_string(),
            phase: AttackPhase::Collection,
            success_rate: 0.85,
            detectability: 0.4,
            cost: 2.0,
            required_access: AccessLevel::User,
        },
        AttackTechnique {
            id: "T1041".to_string(),
            name: "Exfiltration Over C2 Channel".to_string(),
            phase: AttackPhase::Exfiltration,
            success_rate: 0.6,
            detectability: 0.7,
            cost: 5.0,
            required_access: AccessLevel::User,
        },
        AttackTechnique {
            id: "T1486".to_string(),
            name: "Data Encrypted for Impact".to_string(),
            phase: AttackPhase::Impact,
            success_rate: 0.8,
            detectability: 0.9,
            cost: 7.0,
            required_access: AccessLevel::Admin,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attack_path() {
        let mut path = AttackPath::new();
        
        let technique = AttackTechnique {
            id: "T1".to_string(),
            name: "Test".to_string(),
            phase: AttackPhase::InitialAccess,
            success_rate: 0.8,
            detectability: 0.5,
            cost: 5.0,
            required_access: AccessLevel::None,
        };

        path.add_step(NodeIndex::new(0), technique, 0.3);
        assert!(path.success_probability < 1.0);
        assert!(path.detection_probability > 0.0);
    }

    #[test]
    fn test_kill_chain_analyzer() {
        let techniques = create_default_techniques();
        let analyzer = KillChainAnalyzer::new(techniques);
        
        let chains = analyzer.analyze_kill_chains();
        assert!(!chains.is_empty());

        for chain in chains {
            let prob = analyzer.kill_chain_probability(&chain);
            assert!(prob >= 0.0 && prob <= 1.0);
        }
    }
}
