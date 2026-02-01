# Architecture & Design Philosophy

## Overview

The Attacker Strategy Emulator is built on three core principles:

1. **Adversarial Realism**: Model attackers as rational, adaptive agents
2. **Strategic Depth**: Use game theory for rigorous analysis
3. **Practical Utility**: Generate actionable security insights

## System Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                     Application Layer                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │  CLI     │  │Examples  │  │  API     │  │  Tests   │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
└───────────────────────────────────────────────────────────────┘
                            ▼
┌───────────────────────────────────────────────────────────────┐
│                     Core Library Layer                        │
│                                                               │
│  ┌─────────────────┐        ┌─────────────────┐             │
│  │  Simulation     │◄──────►│   Analysis      │             │
│  │    Engine       │        │    Engine       │             │
│  └────────┬────────┘        └─────────────────┘             │
│           │                                                   │
│           ▼                                                   │
│  ┌─────────────────┐        ┌─────────────────┐             │
│  │  Game Theory    │◄──────►│   ML Agents     │             │
│  │    Module       │        │    Module       │             │
│  └────────┬────────┘        └────────┬────────┘             │
│           │                          │                       │
│           └──────────┬───────────────┘                       │
│                      ▼                                       │
│           ┌─────────────────┐                                │
│           │   Data Models   │                                │
│           │  (Defense/Attack)│                               │
│           └─────────────────┘                                │
│                                                               │
└───────────────────────────────────────────────────────────────┘
                            ▼
┌───────────────────────────────────────────────────────────────┐
│                  Infrastructure Layer                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ ndarray  │  │petgraph  │  │  serde   │  │  tokio   │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
└───────────────────────────────────────────────────────────────┘
```

## Module Design

### Game Theory Module

**Purpose**: Provide theoretical foundation for strategic analysis

**Design Decisions**:
- **Matrix representation**: Efficient for payoff computations
- **Solver abstraction**: Different equilibrium concepts via trait
- **Pure strategy support**: Foundation for learning algorithms

**Key Components**:
```rust
SecurityGame
├── Payoff matrices (defender/attacker)
├── Action spaces
└── Utility functions

Solvers
├── NashSolver (fictitious play)
├── StackelbergSolver (commitment)
└── Future: Bayesian, Correlated equilibria
```

**Why this design?**
- Separation of game definition from solution methods
- Extensible to new equilibrium concepts
- Efficient matrix operations via ndarray

### ML Module

**Purpose**: Learn optimal strategies through interaction

**Design Decisions**:
- **DQN over vanilla Q-learning**: Handle large state spaces
- **Experience replay**: Break correlation in training data
- **Target networks**: Stabilize learning
- **Epsilon-greedy**: Balance exploration/exploitation

**Key Components**:
```rust
DQNAgent
├── Q-Network (policy network)
├── Target Network (stable target)
├── Replay Buffer (experience storage)
└── Training loop

QNetwork
├── Input layer (state encoding)
├── Hidden layer (feature extraction)
└── Output layer (Q-values)
```

**Why neural networks?**
- State space too large for tabular Q-learning
- Function approximation generalizes across similar states
- Gradient-based optimization scales to complex problems

### Defense Module

**Purpose**: Model defensive strategies and resource allocation

**Design Decisions**:
- **Graph-based network**: Natural representation of infrastructure
- **Modular defenses**: Different types with different properties
- **Budget constraints**: Realistic resource limitations
- **Coverage model**: Capture overlap and gaps

**Key Components**:
```rust
DefenseStrategy
├── Network topology (petgraph)
├── Asset values & vulnerabilities
├── Budget allocation
└── Coverage computation

DefenseTypes
├── Honeypot (deception)
├── IDS/IPS (detection/prevention)
├── Firewall (access control)
└── Monitoring (visibility)
```

**Design patterns**:
- **Strategy pattern**: Different allocation algorithms
- **Builder pattern**: Complex configuration construction
- **Observer pattern**: Defense effectiveness updates

### Attack Module

**Purpose**: Simulate realistic attack behavior

**Design Decisions**:
- **Kill chain model**: Multi-stage attack progression
- **Probabilistic success**: Capture uncertainty
- **Path planning**: Find optimal routes through network
- **MITRE ATT&CK alignment**: Real-world techniques

**Key Components**:
```rust
AttackStrategy
├── Technique library
├── Path finder
├── Success probability
└── Expected value

AttackPath
├── Node sequence
├── Technique sequence
├── Cumulative probabilities
└── Expected payoff
```

**Why path-based?**
- Captures realistic attack progression
- Enables lateral movement modeling
- Supports kill chain analysis

### Simulation Module

**Purpose**: Orchestrate learning and evolution

**Design Decisions**:
- **Episodic training**: Clear learning boundaries
- **Defender-first**: Stackelberg game structure
- **Adaptive attacker**: Learns from experience
- **Convergence detection**: Identify equilibria

**Key Components**:
```rust
Simulator
├── Episode loop
│   ├── Defense deployment
│   ├── Attack simulation
│   ├── Reward computation
│   └── Learning update
├── Convergence tracking
└── Metrics collection
```

**Control flow**:
```
for episode in episodes:
    defender_deploys(budget)
    for step in steps:
        state = observe()
        action = agent.select(state)
        outcome = simulate(action)
        reward = compute(outcome)
        agent.learn(state, action, reward)
    check_convergence()
```

### Analysis Module

**Purpose**: Extract insights from simulation results

**Design Decisions**:
- **Multi-dimensional metrics**: Capture complexity
- **Actionable recommendations**: Practical guidance
- **Visualization support**: Human understanding
- **Export capabilities**: Integration with other tools

**Key Components**:
```rust
Analyzer
├── Convergence analysis
├── Strategy analysis
├── Vulnerability assessment
└── Recommendation engine
```

## Data Flow

### Training Flow

```
Network Topology → State Encoding → ML Agent → Action Selection
                                         ↓
                                    Attack Sim
                                         ↓
Defense Config ← ← ← ← ← ← ← ← ← Reward & Update
```

### Analysis Flow

```
Simulation Metrics → Analyzer → Report Generation
                                      ↓
                            ┌─────────┴─────────┐
                            ▼                   ▼
                        Console             JSON/CSV
```

## Design Patterns

### Strategy Pattern
Used in: Defense allocation, Attack planning

```rust
trait AllocationStrategy {
    fn allocate(&self, budget: f64) -> DefenseConfiguration;
}

struct GreedyStrategy;
struct UniformStrategy;
struct MLBasedStrategy;
```

### Observer Pattern
Used in: Metrics collection, Convergence detection

```rust
trait MetricsObserver {
    fn on_episode_end(&mut self, metrics: &EpisodeMetrics);
}
```

### Builder Pattern
Used in: Complex configuration objects

```rust
SimulationConfig::builder()
    .episodes(1000)
    .budget(100.0)
    .learning_rate(0.001)
    .build()
```

### Factory Pattern
Used in: Network and technique creation

```rust
NetworkFactory::create_enterprise_network()
TechniqueFactory::create_default_techniques()
```

## Performance Considerations

### Memory Management

**Strategy**: Minimize allocations in hot paths

```rust
// ✓ Reuse buffers
let mut state = Array1::zeros(state_size);
for episode in episodes {
    encode_state_into(&mut state, ...);
}

// ✗ Allocate per iteration
for episode in episodes {
    let state = encode_state(...); // New allocation
}
```

### Computational Efficiency

**Strategy**: Parallelize independent computations

```rust
use rayon::prelude::*;

let results: Vec<_> = scenarios.par_iter()
    .map(|scenario| run_simulation(scenario))
    .collect();
```

### Numerical Stability

**Strategy**: Normalize values, use log-space for probabilities

```rust
// Softmax in log-space to avoid overflow
let log_probs = logits - logits.max();
let probs = log_probs.mapv(|x| x.exp());
let normalized = probs / probs.sum();
```

## Error Handling

### Philosophy

**Fail fast for programmer errors, graceful degradation for runtime issues**

```rust
// Programmer error: panic
assert!(budget > 0.0, "Budget must be positive");

// Runtime issue: return Result
fn load_network(path: &str) -> Result<NetworkGraph, Error> {
    // File might not exist, invalid format, etc.
}
```

### Error Types

```rust
#[derive(Error, Debug)]
pub enum SimulationError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Convergence failed after {0} iterations")]
    ConvergenceFailed(usize),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}
```

## Testing Strategy

### Unit Tests
- Each module has comprehensive unit tests
- Edge cases and boundary conditions
- Property-based testing for numerical stability

### Integration Tests
- End-to-end simulation flows
- Multi-module interaction
- Convergence verification

### Performance Tests
- Benchmark critical paths
- Memory profiling
- Scalability testing

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_nash_equilibrium_simple_game() {
        // Unit test
    }
    
    #[test]
    fn test_full_simulation_convergence() {
        // Integration test
    }
}

#[cfg(bench)]
mod benches {
    #[bench]
    fn bench_q_network_forward_pass(b: &mut Bencher) {
        // Performance test
    }
}
```

## Future Architecture Evolution

### Planned Extensions

1. **Multi-Agent Systems**
   - Multiple attackers competing
   - Coalition formation
   - Distributed defense

2. **Advanced ML**
   - PPO for policy optimization
   - A3C for parallel training
   - Transfer learning across networks

3. **Real-Time Integration**
   - SIEM data ingestion
   - Live defense updates
   - Continuous learning

4. **Visualization**
   - Web dashboard
   - Real-time strategy visualization
   - Interactive scenario builder

### Architectural Changes

```rust
// Future: Plugin architecture
trait SecurityPlugin {
    fn on_attack(&mut self, attack: &Attack);
    fn on_defense(&mut self, defense: &Defense);
}

struct Simulator {
    plugins: Vec<Box<dyn SecurityPlugin>>,
}
```

## References

- Tambe, M. (2011). *Security and Game Theory: Algorithms, Deployed Systems, Lessons Learned*
- Sutton & Barto (2018). *Reinforcement Learning: An Introduction*
- Mnih et al. (2015). "Human-level control through deep reinforcement learning"
- MITRE ATT&CK Framework

## Conclusion

This architecture balances:
- **Theoretical rigor**: Game theory foundation
- **Practical implementation**: Clean, maintainable Rust
- **Extensibility**: Plugin architecture, trait abstractions
- **Performance**: Optimized for simulation workloads

The result is a system that advances the state of security analysis while remaining accessible and practical.
