# Attacker Strategy Emulator - Usage Guide

## Table of Contents

1. [Getting Started](#getting-started)
2. [Core Concepts](#core-concepts)
3. [Module Guide](#module-guide)
4. [Tutorials](#tutorials)
5. [Best Practices](#best-practices)
6. [Troubleshooting](#troubleshooting)

## Getting Started

### Installation

```bash
# Clone repository
git clone https://github.com/yourusername/attacker-strategy-emulator.git
cd attacker-strategy-emulator

# Build project
cargo build --release

# Run tests
cargo test

# Run basic example
cargo run --example basic_game
```

### Your First Simulation

```rust
use attacker_strategy_emulator::prelude::*;

fn main() {
    // 1. Create a network
    let network = create_example_network();
    
    // 2. Configure simulation
    let config = SimulationConfig {
        num_episodes: 500,
        max_steps_per_episode: 20,
        defender_budget: 100.0,
        attacker_budget: 50.0,
        learning_rate: 0.001,
        discount_factor: 0.99,
    };
    
    // 3. Run simulation
    let mut simulator = Simulator::new(config, network);
    let metrics = simulator.run();
    
    // 4. Analyze results
    let analyzer = Analyzer::new(metrics);
    analyzer.print_summary();
}
```

## Core Concepts

### Game Theory

#### Nash Equilibrium

A Nash equilibrium is a strategy profile where neither player can improve their payoff by unilaterally changing their strategy.

**When to use**: 
- Analyzing simultaneous-move scenarios
- Finding stable strategy pairs
- Theoretical analysis

**Code**:
```rust
let solver = NashSolver::new(1000, 1e-6);
let equilibrium = solver.solve(&game);
```

#### Stackelberg Equilibrium

A Stackelberg equilibrium models sequential decision-making where the leader (defender) commits first, and the follower (attacker) best-responds.

**When to use**:
- Real security scenarios (defenders commit to infrastructure)
- Optimization of defense deployment
- Commitment strategies

**Code**:
```rust
let solver = StackelbergSolver::new(10);
let equilibrium = solver.solve(&game);
```

### Machine Learning

#### Deep Q-Network (DQN)

DQN learns to approximate Q-values (expected future reward) for state-action pairs.

**Hyperparameters**:
- `learning_rate`: How fast the network adapts (0.0001 - 0.01)
- `epsilon`: Exploration rate (starts high, decays)
- `gamma`: Discount factor for future rewards (0.9 - 0.99)
- `batch_size`: Number of experiences per update (32 - 256)

**Tuning guide**:
- High learning rate → Fast learning, unstable
- High epsilon → More exploration, slower convergence
- High gamma → Long-term planning, slower learning
- Large batch → Stable updates, more computation

**Code**:
```rust
let mut agent = DQNAgent::new(
    state_size,      // 15 for 5-node network
    action_size,     // 5 (one per target)
    hidden_size,     // 64, 128, or 256
    learning_rate,   // 0.001
    buffer_capacity, // 10000
);

// Training loop
for episode in 0..1000 {
    let state = encode_state(&network, &defense);
    let action = agent.select_action(&state);
    let reward = simulate_attack(action);
    
    agent.store_experience(Experience {
        state,
        action,
        reward,
        next_state,
        done: false,
    });
    
    agent.train();
}
```

#### Policy Gradient

Policy gradient methods directly optimize the policy (action probabilities) using gradient ascent.

**When to use**:
- Continuous action spaces
- Stochastic policies
- When Q-values are hard to estimate

**Code**:
```rust
let mut agent = PolicyGradientAgent::new(
    state_size,
    action_size,
    hidden_size,
    learning_rate,
);

// Episode loop
for step in episode {
    let action = agent.select_action(&state);
    let reward = simulate(action);
    agent.store_transition(state, action, reward);
}

// Update at episode end
agent.update_policy();
```

## Module Guide

### `game_theory` Module

**Purpose**: Model security as a game between rational players

**Key Types**:
- `SecurityGame`: Payoff matrices for defender-attacker interaction
- `Action`: Available choices for players
- `StrategyProfile`: Mixed strategies for both players
- `NashSolver`: Find Nash equilibria
- `StackelbergSolver`: Find Stackelberg equilibria

**Example**:
```rust
use attacker_strategy_emulator::game_theory::*;

let mut game = SecurityGame::new(defender_actions, attacker_actions);
game.set_payoff(0, 0, 100.0, -50.0); // defender_action=0, attacker_action=0

let solver = NashSolver::new(1000, 1e-6);
let eq = solver.solve(&game);

println!("Defender strategy: {:?}", eq.defender_strategy);
println!("Attacker strategy: {:?}", eq.attacker_strategy);
```

### `ml` Module

**Purpose**: Learn optimal strategies through reinforcement learning

**Key Types**:
- `DQNAgent`: Deep Q-Network agent
- `PolicyGradientAgent`: Policy gradient agent
- `QNetwork`: Neural network for value approximation
- `ReplayBuffer`: Experience replay for training
- `Experience`: Single (s, a, r, s') tuple

**Example**:
```rust
use attacker_strategy_emulator::ml::*;

let mut agent = DQNAgent::new(state_size, action_size, 128, 0.001, 10000);

// Training
agent.store_experience(exp);
agent.train();

// Inference
let action = agent.select_action(&state);
let policy = agent.get_policy(&state); // Probability distribution
```

### `defense` Module

**Purpose**: Model and optimize defensive strategies

**Key Types**:
- `Asset`: Network node with value and vulnerability
- `DefenseConfiguration`: Allocation of defensive resources
- `DefenseStrategy`: Strategy generator
- `DefenseType`: IDS, Firewall, Honeypot, etc.
- `DefenseAllocation`: Single defense placement

**Example**:
```rust
use attacker_strategy_emulator::defense::*;

let strategy = DefenseStrategy::new(network, budget);

// Generate strategies
let greedy = strategy.generate_greedy();
let uniform = strategy.generate_uniform();
let from_vector = strategy.from_strategy_vector(&policy);

// Optimize
let optimizer = DefenseOptimizer::new(strategy, attacker_profile);
let optimal = optimizer.optimize();
```

### `attack` Module

**Purpose**: Simulate attack behavior and paths

**Key Types**:
- `AttackTechnique`: Single attack method (based on MITRE ATT&CK)
- `AttackPhase`: Stage in kill chain
- `AttackPath`: Sequence of techniques through network
- `AttackStrategy`: Strategy generator
- `KillChainAnalyzer`: Analyze attack chains

**Example**:
```rust
use attacker_strategy_emulator::attack::*;

let techniques = create_default_techniques();
let strategy = AttackStrategy::new(network, techniques, defense);

// Generate attack
let path = strategy.generate_optimal_path(target)?;

println!("Success probability: {}", path.success_probability);
println!("Detection probability: {}", path.detection_probability);
println!("Expected value: {}", path.expected_value);
```

### `simulation` Module

**Purpose**: Run complete simulations with learning agents

**Key Types**:
- `Simulator`: Main simulation engine
- `SimulationConfig`: Hyperparameters
- `SimulationMetrics`: Results and statistics
- `SimulationState`: Current state

**Example**:
```rust
use attacker_strategy_emulator::simulation::*;

let config = SimulationConfig {
    num_episodes: 1000,
    max_steps_per_episode: 30,
    defender_budget: 100.0,
    attacker_budget: 50.0,
    learning_rate: 0.001,
    discount_factor: 0.99,
};

let mut sim = Simulator::new(config, network);
let metrics = sim.run();

// Get optimized defense
let defense = sim.optimize_defense();
```

### `analysis` Module

**Purpose**: Analyze simulation results and generate insights

**Key Types**:
- `Analyzer`: Analysis engine
- `AnalysisReport`: Comprehensive results
- `ConvergenceAnalysis`: Convergence metrics
- `StrategyAnalysis`: Strategy insights
- `VulnerabilityAssessment`: Risk assessment

**Example**:
```rust
use attacker_strategy_emulator::analysis::*;

let analyzer = Analyzer::new(metrics);

// Generate report
let report = analyzer.generate_report();

// Display
analyzer.print_summary();

// Export
let json = analyzer.export_json()?;
std::fs::write("results.json", json)?;
```

## Tutorials

### Tutorial 1: Building a Custom Network

```rust
use attacker_strategy_emulator::defense::Asset;
use petgraph::graph::Graph;

let mut network = Graph::new();

// Add nodes
let web = network.add_node(Asset {
    id: "web-server".to_string(),
    value: 500.0,
    vulnerability: 0.7,
    criticality: 0.8,
});

let app = network.add_node(Asset {
    id: "app-server".to_string(),
    value: 800.0,
    vulnerability: 0.5,
    criticality: 0.9,
});

let db = network.add_node(Asset {
    id: "database".to_string(),
    value: 1000.0,
    vulnerability: 0.4,
    criticality: 1.0,
});

// Add edges (connections)
network.add_edge(web, app, 1.0);
network.add_edge(app, db, 1.0);

// Use in simulation
let sim = Simulator::new(config, network);
```

### Tutorial 2: Custom Attack Techniques

```rust
use attacker_strategy_emulator::attack::*;

let techniques = vec![
    AttackTechnique {
        id: "CUSTOM-001".to_string(),
        name: "SQL Injection".to_string(),
        phase: AttackPhase::InitialAccess,
        success_rate: 0.65,
        detectability: 0.55,
        cost: 8.0,
        required_access: AccessLevel::None,
    },
    AttackTechnique {
        id: "CUSTOM-002".to_string(),
        name: "Privilege Escalation via CVE-2024-XXXX".to_string(),
        phase: AttackPhase::PrivilegeEscalation,
        success_rate: 0.75,
        detectability: 0.40,
        cost: 12.0,
        required_access: AccessLevel::User,
    },
];

// Use in attack strategy
let strategy = AttackStrategy::new(network, techniques, defense);
```

### Tutorial 3: Multi-Run Analysis

```rust
fn compare_strategies() -> Vec<(String, f64)> {
    let network = create_example_network();
    let budgets = vec![50.0, 100.0, 150.0, 200.0];
    let mut results = Vec::new();
    
    for budget in budgets {
        let config = SimulationConfig {
            defender_budget: budget,
            ..Default::default()
        };
        
        let mut sim = Simulator::new(config, network.clone());
        let metrics = sim.run();
        let analyzer = Analyzer::new(metrics);
        let report = analyzer.generate_report();
        
        results.push((
            format!("Budget: ${}", budget),
            report.vulnerability_assessment.expected_loss,
        ));
    }
    
    results
}
```

## Best Practices

### Network Design

1. **Start simple**: Begin with 5-10 nodes
2. **Realistic values**: Base asset values on real business impact
3. **Vulnerability scores**: Use CVE scores or pen-test results
4. **Criticality**: Factor in business dependencies

### Simulation Configuration

1. **Episodes**: 
   - Quick test: 100-200
   - Full analysis: 1000+
   - Production: 5000+

2. **Learning rate**:
   - Start: 0.01
   - Stable: 0.001
   - Fine-tune: 0.0001

3. **Budget allocation**:
   - Defender: 2-3x attacker budget (realistic)
   - Equal budgets: Stress test
   - Low defender budget: Worst-case analysis

### Performance Optimization

```rust
// Use release mode for production runs
cargo build --release
cargo run --release --example full_simulation

// Parallel execution for parameter sweeps
use rayon::prelude::*;

let results: Vec<_> = budgets.par_iter()
    .map(|&budget| run_simulation(budget))
    .collect();
```

### Interpreting Results

#### Attack Success Rate
- < 30%: Good security posture
- 30-50%: Moderate risk
- 50-70%: High risk
- \> 70%: Critical - immediate action needed

#### Detection Rate
- \> 70%: Excellent monitoring
- 50-70%: Good coverage
- 30-50%: Needs improvement
- < 30%: Blind spots exist

#### Convergence
- Converged early (< 200 episodes): Simple problem or dominant strategy
- Converged late (> 500 episodes): Complex strategic interaction
- No convergence: Hyperparameter tuning needed or cyclical strategies

## Troubleshooting

### Problem: Agent not learning

**Symptoms**:
- Rewards don't improve
- Success rate stays random
- No convergence

**Solutions**:
1. Increase learning rate
2. Reduce epsilon decay
3. Increase episodes
4. Check reward function
5. Verify state encoding

### Problem: Unstable training

**Symptoms**:
- Rewards oscillate wildly
- Strategy keeps changing
- No equilibrium

**Solutions**:
1. Decrease learning rate
2. Increase batch size
3. Use target network updates
4. Reduce epsilon decay

### Problem: Poor defense allocation

**Symptoms**:
- Budget concentrated on few assets
- High-value assets undefended
- Low ROI

**Solutions**:
1. Adjust asset values
2. Use different allocation strategy
3. Increase defender budget
4. Modify payoff structure

### Problem: Simulation too slow

**Solutions**:
```rust
// Reduce episodes or steps
let config = SimulationConfig {
    num_episodes: 200,  // Down from 1000
    max_steps_per_episode: 10,  // Down from 30
    ..Default::default()
};

// Use release build
cargo build --release

// Reduce network size (for testing)
let network = small_test_network();
```

## Advanced Topics

### Custom State Encoding

```rust
fn encode_state_advanced(
    network: &NetworkGraph,
    defense: &DefenseConfiguration,
    history: &[Attack],
) -> Array1<f64> {
    let mut state = Vec::new();
    
    // Asset features
    for node in network.node_indices() {
        let asset = &network[node];
        state.push(asset.value / 100.0);
        state.push(asset.vulnerability);
        state.push(defense.get_coverage(node));
        state.push(asset.criticality);
    }
    
    // Historical features
    let recent_attacks = history.iter().rev().take(5);
    for attack in recent_attacks {
        state.push(if attack.succeeded { 1.0 } else { 0.0 });
        state.push(attack.target as f64 / network.node_count() as f64);
    }
    
    Array1::from_vec(state)
}
```

### Multi-Agent Simulation

```rust
struct MultiAgentSimulator {
    agents: Vec<DQNAgent>,
    network: NetworkGraph,
}

impl MultiAgentSimulator {
    fn run_competitive(&mut self) {
        // Multiple attackers compete
        for episode in 0..episodes {
            for agent in &mut self.agents {
                let action = agent.select_action(&state);
                let reward = self.compete(action);
                agent.store_experience(exp);
                agent.train();
            }
        }
    }
}
```

## API Reference

For complete API documentation, run:

```bash
cargo doc --open
```

This will generate and open the full API documentation in your browser.

---

For more examples and updates, visit the [GitHub repository](https://github.com/yourusername/attacker-strategy-emulator).
