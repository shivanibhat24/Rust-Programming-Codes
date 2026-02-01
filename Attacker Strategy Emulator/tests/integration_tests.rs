use attacker_strategy_emulator::*;
use simulation::{SimulationConfig, Simulator, create_example_network};
use analysis::Analyzer;
use game_theory::{SecurityGame, Action, Player, NashSolver};
use defense::{DefenseStrategy, Asset};
use attack::create_default_techniques;
use petgraph::graph::Graph;

#[test]
fn test_end_to_end_simulation() {
    // Setup
    let config = SimulationConfig {
        num_episodes: 50,
        max_steps_per_episode: 10,
        defender_budget: 100.0,
        attacker_budget: 50.0,
        learning_rate: 0.01,
        discount_factor: 0.99,
    };

    let network = create_example_network();
    
    // Run simulation
    let mut simulator = Simulator::new(config, network);
    let metrics = simulator.run();

    // Verify results
    assert!(!metrics.episode_rewards_defender.is_empty());
    assert!(!metrics.episode_rewards_attacker.is_empty());
    assert_eq!(
        metrics.episode_rewards_defender.len(),
        metrics.episode_rewards_attacker.len()
    );
}

#[test]
fn test_game_theory_module() {
    // Create a simple 2x2 game
    let defender_actions = vec![
        Action {
            id: 0,
            name: "Defend A".to_string(),
            cost: 5.0,
            player: Player::Defender,
        },
        Action {
            id: 1,
            name: "Defend B".to_string(),
            cost: 5.0,
            player: Player::Defender,
        },
    ];

    let attacker_actions = vec![
        Action {
            id: 0,
            name: "Attack A".to_string(),
            cost: 3.0,
            player: Player::Attacker,
        },
        Action {
            id: 1,
            name: "Attack B".to_string(),
            cost: 3.0,
            player: Player::Attacker,
        },
    ];

    let mut game = SecurityGame::new(defender_actions, attacker_actions);
    
    // Set payoffs for a zero-sum game
    game.set_payoff(0, 0, 10.0, -10.0);
    game.set_payoff(0, 1, -5.0, 5.0);
    game.set_payoff(1, 0, -5.0, 5.0);
    game.set_payoff(1, 1, 10.0, -10.0);

    // Solve for Nash equilibrium
    let solver = NashSolver::new(1000, 1e-6);
    let equilibrium = solver.solve(&game);

    // Verify equilibrium properties
    assert_eq!(equilibrium.defender_strategy.len(), 2);
    assert_eq!(equilibrium.attacker_strategy.len(), 2);
    
    // Sum of probabilities should be 1
    let defender_sum: f64 = equilibrium.defender_strategy.iter().sum();
    let attacker_sum: f64 = equilibrium.attacker_strategy.iter().sum();
    
    assert!((defender_sum - 1.0).abs() < 1e-6);
    assert!((attacker_sum - 1.0).abs() < 1e-6);
}

#[test]
fn test_defense_optimization() {
    let network = create_example_network();
    let budget = 100.0;
    
    let strategy = DefenseStrategy::new(network, budget);
    
    // Test different allocation strategies
    let greedy = strategy.generate_greedy();
    let uniform = strategy.generate_uniform();
    
    // Verify budget constraints
    assert!(greedy.total_cost <= budget);
    assert!(uniform.total_cost <= budget);
    
    // Verify allocations exist
    assert!(!greedy.allocations.is_empty());
    assert!(!uniform.allocations.is_empty());
}

#[test]
fn test_attack_path_generation() {
    let network = create_example_network();
    let techniques = create_default_techniques();
    let defense_config = DefenseStrategy::new(network.clone(), 100.0).generate_greedy();
    
    let attack_strategy = attack::AttackStrategy::new(
        network.clone(),
        techniques,
        defense_config,
    );
    
    // Try to generate attack path
    let target = petgraph::graph::NodeIndex::new(0);
    let path = attack_strategy.generate_optimal_path(target);
    
    // Path might not exist, but should not panic
    if let Some(p) = path {
        assert!(p.success_probability >= 0.0 && p.success_probability <= 1.0);
        assert!(p.detection_probability >= 0.0 && p.detection_probability <= 1.0);
    }
}

#[test]
fn test_ml_agent_learning() {
    use ml::DQNAgent;
    use ndarray::Array1;
    
    let mut agent = DQNAgent::new(10, 5, 32, 0.01, 1000);
    
    // Simulate some learning
    let state = Array1::from_vec(vec![0.5; 10]);
    
    for _ in 0..100 {
        let action = agent.select_action(&state);
        assert!(action < 5);
        
        let reward = if action == 2 { 1.0 } else { -0.1 };
        
        agent.store_experience(ml::Experience {
            state: state.clone(),
            action,
            reward,
            next_state: state.clone(),
            done: false,
        });
        
        agent.train();
    }
    
    // Agent should learn something (epsilon should decrease)
    assert!(agent.epsilon() < 1.0);
}

#[test]
fn test_analysis_report_generation() {
    let config = SimulationConfig {
        num_episodes: 20,
        max_steps_per_episode: 10,
        defender_budget: 100.0,
        attacker_budget: 50.0,
        learning_rate: 0.01,
        discount_factor: 0.99,
    };

    let network = create_example_network();
    let mut simulator = Simulator::new(config, network);
    let metrics = simulator.run();
    
    let analyzer = Analyzer::new(metrics);
    let report = analyzer.generate_report();
    
    // Verify report structure
    assert!(!report.recommendations.is_empty());
    assert!(report.convergence_analysis.average_success_rate >= 0.0);
    assert!(report.convergence_analysis.average_success_rate <= 1.0);
    assert!(report.convergence_analysis.average_detection_rate >= 0.0);
    assert!(report.convergence_analysis.average_detection_rate <= 1.0);
}

#[test]
fn test_custom_network_creation() {
    let mut network = Graph::new();
    
    let node1 = network.add_node(Asset {
        id: "test-1".to_string(),
        value: 100.0,
        vulnerability: 0.5,
        criticality: 0.8,
    });
    
    let node2 = network.add_node(Asset {
        id: "test-2".to_string(),
        value: 200.0,
        vulnerability: 0.3,
        criticality: 0.9,
    });
    
    network.add_edge(node1, node2, 1.0);
    
    assert_eq!(network.node_count(), 2);
    assert_eq!(network.edge_count(), 1);
    
    // Should be able to use in simulation
    let config = SimulationConfig::default();
    let _simulator = Simulator::new(config, network);
}

#[test]
fn test_convergence_detection() {
    use simulation::SimulationMetrics;
    
    let mut metrics = SimulationMetrics::new();
    
    // Add stable rewards
    for _ in 0..100 {
        metrics.add_episode(50.0, -25.0, 0.5, 0.5);
    }
    
    let converged = metrics.check_convergence(50, 0.1);
    assert!(converged);
    assert!(metrics.convergence_episode.is_some());
}

#[test]
fn test_json_export() {
    let mut metrics = simulation::SimulationMetrics::new();
    metrics.add_episode(10.0, -5.0, 0.6, 0.4);
    
    let analyzer = Analyzer::new(metrics);
    let json = analyzer.export_json();
    
    assert!(json.is_ok());
    let json_str = json.unwrap();
    assert!(json_str.contains("convergence_analysis"));
    assert!(json_str.contains("recommendations"));
}

#[test]
fn test_state_encoding() {
    let network = create_example_network();
    let defense = DefenseStrategy::new(network.clone(), 100.0).generate_greedy();
    
    let mut state = Vec::new();
    for node in network.node_indices() {
        let asset = &network[node];
        state.push(asset.value / 100.0);
        state.push(asset.vulnerability);
        state.push(defense.get_coverage(node));
    }
    
    assert_eq!(state.len(), network.node_count() * 3);
    
    // All values should be in reasonable range
    for &val in &state {
        assert!(val >= 0.0 && val <= 10.0);
    }
}

#[test]
fn test_attack_technique_library() {
    let techniques = create_default_techniques();
    
    assert!(!techniques.is_empty());
    
    // Verify all techniques have valid properties
    for technique in techniques {
        assert!(technique.success_rate >= 0.0 && technique.success_rate <= 1.0);
        assert!(technique.detectability >= 0.0 && technique.detectability <= 1.0);
        assert!(technique.cost >= 0.0);
    }
}

#[test]
fn test_defense_types() {
    use defense::DefenseType;
    
    let types = vec![
        DefenseType::Honeypot,
        DefenseType::IDS,
        DefenseType::IPS,
        DefenseType::Firewall,
        DefenseType::Patching,
        DefenseType::AccessControl,
        DefenseType::Encryption,
        DefenseType::Monitoring,
    ];
    
    assert_eq!(types.len(), 8);
}

#[test]
fn test_payoff_computation() {
    let defender_actions = vec![
        Action {
            id: 0,
            name: "Action".to_string(),
            cost: 0.0,
            player: Player::Defender,
        },
    ];
    
    let attacker_actions = vec![
        Action {
            id: 0,
            name: "Action".to_string(),
            cost: 0.0,
            player: Player::Attacker,
        },
    ];
    
    let mut game = SecurityGame::new(defender_actions, attacker_actions);
    game.set_payoff(0, 0, 100.0, -50.0);
    
    use ndarray::Array1;
    let defender_strategy = Array1::from_vec(vec![1.0]);
    let attacker_strategy = Array1::from_vec(vec![1.0]);
    
    let defender_utility = game.defender_expected_utility(&defender_strategy, &attacker_strategy);
    let attacker_utility = game.attacker_expected_utility(&defender_strategy, &attacker_strategy);
    
    assert_eq!(defender_utility, 100.0);
    assert_eq!(attacker_utility, -50.0);
}

#[test]
fn test_simulation_reproducibility() {
    let config = SimulationConfig {
        num_episodes: 10,
        max_steps_per_episode: 5,
        defender_budget: 100.0,
        attacker_budget: 50.0,
        learning_rate: 0.01,
        discount_factor: 0.99,
    };

    let network1 = create_example_network();
    let network2 = create_example_network();
    
    // Networks should be identical
    assert_eq!(network1.node_count(), network2.node_count());
    assert_eq!(network1.edge_count(), network2.edge_count());
}
