# 🚀 Quick Start Guide

Get up and running with the Attacker Strategy Emulator in 5 minutes!

## Prerequisites

- Rust 1.70+ ([Install Rust](https://www.rust-lang.org/tools/install))
- Git

## Installation (2 minutes)

```bash
# Clone the repository
git clone https://github.com/yourusername/attacker-strategy-emulator.git
cd attacker-strategy-emulator

# Build the project (this may take a few minutes on first run)
cargo build --release

# Run tests to verify installation
cargo test
```

## Your First Simulation (3 minutes)

### Option 1: Run Pre-built Example

```bash
# Run the full simulation example
cargo run --release --example full_simulation

# This will:
# 1. Create a 5-node network
# 2. Train an ML attacker for 1000 episodes
# 3. Generate comprehensive analysis
# 4. Provide defense recommendations
```

### Option 2: Quick Code Example

Create a file `my_first_sim.rs` in the `examples/` directory:

```rust
use attacker_strategy_emulator::simulation::{
    Simulator, SimulationConfig, create_example_network
};
use attacker_strategy_emulator::analysis::Analyzer;

fn main() {
    println!("🎮 My First Security Simulation!");
    
    // 1. Configure the simulation
    let config = SimulationConfig {
        num_episodes: 200,           // Train for 200 episodes
        max_steps_per_episode: 20,   // 20 attacks per episode
        defender_budget: 100.0,      // $100 security budget
        attacker_budget: 50.0,       // Attacker has $50
        learning_rate: 0.005,        // How fast attacker learns
        discount_factor: 0.95,       // Future reward weight
    };

    // 2. Create a network (or build your own!)
    let network = create_example_network();
    println!("Network: {} assets, {} connections", 
        network.node_count(), 
        network.edge_count()
    );

    // 3. Run the simulation
    println!("\n🚀 Training attacker agent...\n");
    let mut simulator = Simulator::new(config, network);
    let metrics = simulator.run();

    // 4. Analyze results
    println!("\n📊 Analyzing results...\n");
    let analyzer = Analyzer::new(metrics);
    analyzer.print_summary();

    // 5. Get optimized defense
    let optimized_defense = simulator.optimize_defense();
    println!("\n✅ Optimized defense uses ${:.2} of budget", 
        optimized_defense.total_cost
    );
}
```

Run it:
```bash
cargo run --release --example my_first_sim
```

## What Just Happened?

The simulation:

1. **Created a network** with 5 assets (web server, database, etc.)
2. **Trained an attacker** using Deep Q-Learning to find optimal targets
3. **Simulated attacks** where the attacker learned from successes/failures
4. **Analyzed results** to find vulnerabilities and recommend defenses
5. **Optimized defenses** based on learned attacker behavior

## Understanding the Output

### Episode Progress
```
Episode   50 | Reward:   45.23 | Avg:   42.10 | Success:  60.0% | Epsilon: 0.606 | Threat: 🟡 MEDIUM
```

- **Reward**: Attacker's total reward this episode
- **Avg**: Rolling average (indicates learning trend)
- **Success**: Attack success rate
- **Epsilon**: Exploration rate (decreases over time)
- **Threat**: Overall threat level

### Convergence
```
Converged: ✓ Yes
Convergence Episode: 487
```

Means strategies stabilized - attacker found optimal approach!

### Recommendations
```
1. HIGH PRIORITY: Attack success rate exceeds 50%. Increase defense coverage on critical assets.
2. Consider implementing honeypots to increase attacker costs and uncertainty.
```

Actionable next steps based on simulation results.

## Next Steps

### Customize Your Network

```rust
use attacker_strategy_emulator::defense::Asset;
use petgraph::graph::Graph;

let mut network = Graph::new();

// Add your assets
let db = network.add_node(Asset {
    id: "production-db".to_string(),
    value: 1000.0,      // How valuable is this?
    vulnerability: 0.7,  // How vulnerable? (0-1)
    criticality: 1.0,    // How critical? (0-1)
});

let web = network.add_node(Asset {
    id: "web-frontend".to_string(),
    value: 500.0,
    vulnerability: 0.8,
    criticality: 0.6,
});

// Connect them (attacker can move between)
network.add_edge(web, db, 1.0);

// Use your network
let mut sim = Simulator::new(config, network);
```

### Run Different Scenarios

```bash
# Quick test (50 episodes)
cargo run --example basic_game

# ML training focused
cargo run --example ml_training

# Full analysis with all features
cargo run --example full_simulation
```

### Analyze Results

```rust
// Export to JSON
let json = analyzer.export_json()?;
std::fs::write("results.json", json)?;

// Get specific metrics
let (defender_rewards, attacker_rewards) = analyzer.get_reward_trends();
```

## Common Use Cases

### 1. Budget Justification

"Should we spend $X on security?"

```rust
for budget in [50.0, 100.0, 150.0, 200.0] {
    let config = SimulationConfig {
        defender_budget: budget,
        ..Default::default()
    };
    let mut sim = Simulator::new(config, network.clone());
    let metrics = sim.run();
    
    println!("Budget ${}: Expected loss ${:.2}", 
        budget, 
        metrics.expected_loss
    );
}
```

### 2. Vulnerability Prioritization

"Which assets should we patch first?"

```rust
let analyzer = Analyzer::new(metrics);
let report = analyzer.generate_report();

for asset in report.vulnerability_assessment.high_risk_assets {
    println!("🔴 CRITICAL: {}", asset);
}
```

### 3. Defense Evaluation

"Is our current defense effective?"

```rust
let attack_success_rate = report.convergence_analysis.average_success_rate;

if attack_success_rate > 0.5 {
    println!("⚠️ Defense needs improvement!");
} else {
    println!("✅ Defense is effective!");
}
```

## Troubleshooting

### Build Fails

**Issue**: "linker errors" or "can't find -lopenblas"

**Solution**: 
```bash
# Ubuntu/Debian
sudo apt-get install libopenblas-dev

# macOS
brew install openblas

# Or disable the feature
cargo build --release --no-default-features
```

### Slow Performance

**Issue**: Training takes too long

**Solutions**:
```rust
// Reduce episodes
let config = SimulationConfig {
    num_episodes: 100,  // Down from 1000
    ..Default::default()
};

// Use release mode
cargo run --release --example full_simulation
```

### No Convergence

**Issue**: Simulation doesn't converge

**Solutions**:
```rust
// Increase episodes
num_episodes: 2000,

// Adjust learning rate
learning_rate: 0.001,  // Lower = more stable

// Check epsilon decay
// (in DQNAgent, epsilon_decay should be 0.995-0.999)
```

## Resources

- **Full Documentation**: See `USAGE.md` for detailed guide
- **Architecture**: See `ARCHITECTURE.md` for design details
- **Examples**: Browse `examples/` directory
- **Tests**: Check `tests/` for usage patterns

## Getting Help

1. Check the documentation files
2. Look at example code
3. Run the tests: `cargo test -- --nocapture`
4. Open an issue on GitHub

## What's Next?

Now that you have the basics, explore:

- 📖 **USAGE.md** - Comprehensive usage guide
- 🏗️ **ARCHITECTURE.md** - System design and philosophy
- 🎯 **Advanced examples** - Complex scenarios
- 🧪 **Tests** - See test files for more examples

---

**Congratulations!** You're now ready to model adaptive adversaries and optimize your security posture! 🛡️

Remember: The goal is to understand how rational attackers think and make their strategies unprofitable.
