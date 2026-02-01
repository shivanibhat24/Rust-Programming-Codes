# 🎮 Attacker Strategy Emulator

> **Game-Theoretic Security Analysis with ML-Based Adaptive Adversaries**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

A cutting-edge security simulation framework that models adversarial behavior using **game theory** and **machine learning**. Unlike traditional security tools that assume static or dumb attackers, this emulator models **rational, adaptive adversaries** that learn optimal strategies through reinforcement learning.

## 🚀 Why This Matters

Most security systems are designed against *known* attack patterns. But sophisticated attackers:
- **Adapt** to your defenses
- **Optimize** their strategies
- **Learn** from failed attempts
- **Find** the weakest links systematically

This emulator trains attacker agents to discover optimal exploitation strategies, then helps you build defenses that collapse the attacker's reward space.

## ✨ Key Features

### 🎯 Game-Theoretic Modeling
- **Stackelberg Games**: Model defender-first commitment strategies
- **Nash Equilibrium**: Find stable security configurations
- **Multi-Stage Games**: Model complex attack scenarios
- **Payoff Analysis**: Understand strategic incentives

### 🤖 Machine Learning
- **Deep Q-Networks (DQN)**: Learn optimal attack strategies
- **Policy Gradients**: Continuous strategy adaptation
- **Experience Replay**: Efficient learning from past attacks
- **Adaptive Adversaries**: Attackers that respond to your defenses

### 🛡️ Defense Optimization
- **Resource Allocation**: Optimal budget distribution
- **Coverage Analysis**: Identify security gaps
- **Cost-Benefit Analysis**: Maximize defense ROI
- **Multi-Layer Defense**: Honeypots, IDS/IPS, firewalls, monitoring

### ⚔️ Attack Simulation
- **MITRE ATT&CK Integration**: Real-world attack techniques
- **Kill Chain Analysis**: Multi-stage attack paths
- **Network Traversal**: Lateral movement simulation
- **Success Probability**: Realistic outcome modeling

### 📊 Analysis & Insights
- **Convergence Detection**: Identify strategic equilibria
- **Vulnerability Assessment**: Find high-risk assets
- **Threat Prioritization**: Focus defenses where it matters
- **Actionable Recommendations**: Concrete next steps

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Attacker Strategy Emulator             │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────┐ │
│  │ Game Theory  │───▶│      ML      │───▶│ Analysis │ │
│  │   Engine     │    │   Agents     │    │  Engine  │ │
│  └──────────────┘    └──────────────┘    └──────────┘ │
│         │                    │                   │     │
│         ▼                    ▼                   ▼     │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────┐ │
│  │   Defense    │    │    Attack    │    │Simulation│ │
│  │  Simulator   │    │  Simulator   │    │  Engine  │ │
│  └──────────────┘    └──────────────┘    └──────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## 📦 Installation

### Prerequisites
- Rust 1.70 or higher
- Cargo package manager

### Build from Source

```bash
git clone https://github.com/yourusername/attacker-strategy-emulator.git
cd attacker-strategy-emulator
cargo build --release
```

### Run Examples

```bash
# Basic game theory demonstration
cargo run --example basic_game

# ML attacker training
cargo run --example ml_training

# Full simulation with analysis
cargo run --example full_simulation
```

## 🎓 Quick Start

### Basic Usage

```rust
use attacker_strategy_emulator::simulation::{
    Simulator, SimulationConfig, create_example_network
};
use attacker_strategy_emulator::analysis::Analyzer;

fn main() {
    // Configure simulation
    let config = SimulationConfig::default();
    
    // Create network topology
    let network = create_example_network();
    
    // Run simulation
    let mut simulator = Simulator::new(config, network);
    let metrics = simulator.run();
    
    // Analyze results
    let analyzer = Analyzer::new(metrics);
    analyzer.print_summary();
}
```

### Custom Network

```rust
use attacker_strategy_emulator::defense::Asset;
use petgraph::graph::Graph;

let mut network = Graph::new();

// Add assets
let db = network.add_node(Asset {
    id: "database".to_string(),
    value: 1000.0,
    vulnerability: 0.6,
    criticality: 1.0,
});

let web = network.add_node(Asset {
    id: "web-server".to_string(),
    value: 500.0,
    vulnerability: 0.7,
    criticality: 0.8,
});

// Connect assets
network.add_edge(web, db, 1.0);
```

### Game Theory Analysis

```rust
use attacker_strategy_emulator::game_theory::{
    SecurityGame, Action, Player, NashSolver
};

// Define actions
let defender_actions = vec![
    Action {
        id: 0,
        name: "Deploy IDS".to_string(),
        cost: 10.0,
        player: Player::Defender,
    },
];

let attacker_actions = vec![
    Action {
        id: 0,
        name: "Exploit Vulnerability".to_string(),
        cost: 5.0,
        player: Player::Attacker,
    },
];

// Create and solve game
let mut game = SecurityGame::new(defender_actions, attacker_actions);
game.set_payoff(0, 0, 50.0, -30.0);

let solver = NashSolver::new(1000, 1e-6);
let equilibrium = solver.solve(&game);
```

## 📚 Core Concepts

### Stackelberg Security Games

In security, the defender typically *commits* to a strategy (deploys defenses), and the attacker *observes* before attacking. This is modeled as a **Stackelberg game**:

1. **Defender** chooses strategy σ_d
2. **Attacker** observes σ_d
3. **Attacker** best-responds with σ_a
4. **Outcome** determined

The defender's goal: choose σ_d that minimizes damage when the attacker best-responds.

### Deep Q-Learning for Attack Strategy

The attacker agent uses DQN to learn:
- **Q(s,a)**: Expected reward for attacking target *a* given state *s*
- **Policy π**: Probability distribution over targets
- **Value**: Total expected reward over time

Through simulation, the agent discovers which assets to prioritize based on:
- Asset value
- Vulnerability
- Defense coverage
- Access difficulty

### Defense Optimization

Given learned attacker strategies, we optimize defense allocation by:

1. Computing **attack probability** for each asset
2. Allocating **defense budget** proportionally to risk
3. **Balancing** coverage vs. effectiveness
4. **Iterating** as attacker adapts

## 🔬 Example Scenarios

### Scenario 1: Honeypot Placement

**Question**: Where should we place honeypots to maximize attacker uncertainty?

**Approach**:
1. Train attacker on network without honeypots
2. Identify primary targets
3. Place honeypots on likely paths
4. Retrain attacker, measure strategy change
5. Analyze impact on attacker reward

**Result**: Honeypots near high-value assets create maximum uncertainty.

### Scenario 2: Budget Allocation

**Question**: How should we allocate a $100k security budget?

**Approach**:
1. Model various defense types (IDS, firewall, patching)
2. Simulate attacker learning for each allocation
3. Measure expected loss for each scenario
4. Choose allocation minimizing expected loss

**Result**: Data-driven budget allocation based on attacker behavior.

### Scenario 3: Zero-Day Impact

**Question**: What if an attacker discovers a zero-day in Asset X?

**Approach**:
1. Update vulnerability scores
2. Retrain attacker with new information
3. Measure strategy shift
4. Identify cascading risks

**Result**: Prioritized response plan for zero-day scenarios.

## 📊 Metrics & Analysis

The emulator provides comprehensive metrics:

- **Convergence Analysis**: Did strategies stabilize?
- **Success Rates**: How often do attacks succeed?
- **Detection Rates**: How often are attacks caught?
- **Expected Loss**: Projected annual damage
- **ROI Analysis**: Return on defense investment
- **Vulnerability Rankings**: Prioritized asset list
- **Critical Paths**: Most dangerous attack routes

## 🛠️ Advanced Features

### Custom Attack Techniques

```rust
use attacker_strategy_emulator::attack::{
    AttackTechnique, AttackPhase, AccessLevel
};

let custom_technique = AttackTechnique {
    id: "CUSTOM-001".to_string(),
    name: "Custom Exploit".to_string(),
    phase: AttackPhase::InitialAccess,
    success_rate: 0.7,
    detectability: 0.4,
    cost: 15.0,
    required_access: AccessLevel::None,
};
```

### Multi-Objective Optimization

```rust
// Optimize for multiple objectives simultaneously
let objectives = vec![
    Objective::MinimizeLoss,
    Objective::MaximizeDetection,
    Objective::MinimizeCost,
];

let pareto_frontier = optimizer.multi_objective_optimize(objectives);
```

### Adversarial Training

```rust
// Co-evolve attacker and defender strategies
for generation in 0..100 {
    // Defender adapts to current attacker
    defender.optimize_against(&attacker);
    
    // Attacker adapts to new defenses
    attacker.train_against(&defender);
    
    // Measure arms race dynamics
    metrics.record_generation(generation, defender, attacker);
}
```

## 🎯 Real-World Applications

- **Penetration Testing**: Discover attack paths before adversaries do
- **Security Architecture**: Design networks resilient to adaptive attacks
- **Budget Justification**: Prove ROI of security investments
- **Incident Response**: Pre-plan responses to likely attack scenarios
- **Red Team Training**: Generate realistic attack scenarios
- **Compliance**: Demonstrate due diligence in security planning

## 🤝 Contributing

Contributions welcome! Areas of interest:

- Additional ML algorithms (PPO, A3C, etc.)
- More sophisticated game-theoretic solvers
- Integration with real network scanners
- Visualization dashboards
- Real-world attack datasets

## 📖 References

- Tambe, M. (2011). *Security and Game Theory*
- Sutton & Barto (2018). *Reinforcement Learning: An Introduction*
- MITRE ATT&CK Framework
- Stackelberg Security Games Research

## ⚖️ License

MIT License - see LICENSE file for details

## 🙏 Acknowledgments

Built with:
- `ndarray` for numerical computing
- `petgraph` for network modeling
- `serde` for serialization
- `colored` for beautiful terminal output

## 📬 Contact

For questions, issues, or collaboration opportunities, please open an issue on GitHub.

---

**Remember**: The goal isn't just to model attacks—it's to *understand* adversarial thinking and build defenses that make attacks *unprofitable*.

🛡️ **Stay secure. Stay ahead.**
