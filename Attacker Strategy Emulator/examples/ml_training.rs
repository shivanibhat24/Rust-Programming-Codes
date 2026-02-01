use attacker_strategy_emulator::ml::{DQNAgent, Experience};
use attacker_strategy_emulator::simulation::{create_example_network, SimulationConfig};
use attacker_strategy_emulator::defense::{DefenseStrategy, DefenseConfiguration};
use attacker_strategy_emulator::attack::{AttackStrategy, create_default_techniques};
use ndarray::Array1;
use colored::*;

fn main() {
    println!("{}", "🤖 ML-Based Attacker Strategy Learning".bright_cyan().bold());
    println!("{}", "=".repeat(80));

    // Setup environment
    let network = create_example_network();
    let techniques = create_default_techniques();
    let defender_budget = 100.0;

    println!("\n📊 Environment Setup:");
    println!("  Network Nodes: {}", network.node_count());
    println!("  Attack Techniques: {}", techniques.len());
    println!("  Defender Budget: ${:.2}", defender_budget);

    // Create defense
    let defense_strategy = DefenseStrategy::new(network.clone(), defender_budget);
    let defense_config = defense_strategy.generate_greedy();

    println!("\n🛡️  Defense Configuration:");
    println!("  Total Allocations: {}", defense_config.allocations.len());
    println!("  Total Cost: ${:.2}", defense_config.total_cost);
    println!("  Remaining Budget: ${:.2}", defense_config.remaining_budget());

    // Initialize DQN agent
    let state_size = network.node_count() * 3;
    let action_size = network.node_count();
    let mut agent = DQNAgent::new(state_size, action_size, 128, 0.001, 10000);

    println!("\n🧠 DQN Agent Configuration:");
    println!("  State Size: {}", state_size);
    println!("  Action Size: {}", action_size);
    println!("  Hidden Layer: 128 neurons");
    println!("  Learning Rate: 0.001");

    // Training loop
    println!("\n🏋️  Training Agent...\n");
    let episodes = 500;
    let steps_per_episode = 20;

    let mut episode_rewards = Vec::new();
    let mut success_rates = Vec::new();

    for episode in 0..episodes {
        let mut total_reward = 0.0;
        let mut successes = 0;

        for step in 0..steps_per_episode {
            // Encode state
            let state = encode_state(&network, &defense_config);
            
            // Select action
            let action = agent.select_action(&state);
            
            if action >= network.node_count() {
                continue;
            }

            // Simulate attack
            let attack_strategy = AttackStrategy::new(
                network.clone(),
                techniques.clone(),
                defense_config.clone(),
            );

            let target = petgraph::graph::NodeIndex::new(action);
            let reward = if let Some(path) = attack_strategy.generate_optimal_path(target) {
                if path.success_probability > 0.5 {
                    successes += 1;
                }
                path.expected_value
            } else {
                -10.0
            };

            total_reward += reward;

            // Store experience
            let next_state = encode_state(&network, &defense_config);
            agent.store_experience(Experience {
                state: state.clone(),
                action,
                reward,
                next_state,
                done: step == steps_per_episode - 1,
            });

            // Train
            agent.train();
        }

        episode_rewards.push(total_reward);
        let success_rate = successes as f64 / steps_per_episode as f64;
        success_rates.push(success_rate);

        // Print progress
        if episode % 50 == 0 {
            let avg_reward = if episode >= 10 {
                episode_rewards[episode.saturating_sub(10)..=episode]
                    .iter()
                    .sum::<f64>() / 10.0
            } else {
                total_reward
            };

            let status = if success_rate > 0.6 {
                "🔴 HIGH".red()
            } else if success_rate > 0.3 {
                "🟡 MEDIUM".yellow()
            } else {
                "🟢 LOW".green()
            };

            println!(
                "Episode {:>4} | Reward: {:>7.2} | Avg: {:>7.2} | Success: {:>5.1}% | Epsilon: {:.3} | Threat: {}",
                episode,
                total_reward,
                avg_reward,
                success_rate * 100.0,
                agent.epsilon(),
                status
            );
        }
    }

    // Analyze learned strategy
    println!("\n{}", "=".repeat(80));
    println!("{}", "📈 Training Results".bright_green().bold());
    println!("{}", "=".repeat(80));

    let final_reward = episode_rewards.last().unwrap_or(&0.0);
    let avg_final_rewards = episode_rewards[episode_rewards.len().saturating_sub(100)..]
        .iter()
        .sum::<f64>() / 100.0;
    let avg_success = success_rates[success_rates.len().saturating_sub(100)..]
        .iter()
        .sum::<f64>() / 100.0;

    println!("\n🎯 Performance Metrics:");
    println!("  Final Episode Reward: {:.2}", final_reward);
    println!("  Avg Last 100 Episodes: {:.2}", avg_final_rewards);
    println!("  Final Success Rate: {:.1}%", success_rates.last().unwrap_or(&0.0) * 100.0);
    println!("  Avg Success Rate (last 100): {:.1}%", avg_success * 100.0);
    println!("  Final Epsilon: {:.3}", agent.epsilon());

    // Get learned policy
    println!("\n🎲 Learned Attack Strategy:");
    let state = encode_state(&network, &defense_config);
    let policy = agent.get_policy(&state);

    for (i, &prob) in policy.iter().enumerate() {
        if i < network.node_count() {
            let asset = &network[petgraph::graph::NodeIndex::new(i)];
            let bar_length = (prob * 50.0) as usize;
            let bar = "█".repeat(bar_length);
            println!("  {} [{:20}] {:.1}%", 
                asset.id.bright_white().bold(),
                bar.bright_cyan(),
                prob * 100.0
            );
        }
    }

    // Recommendations
    println!("\n{}", "=".repeat(80));
    println!("{}", "💡 Defense Recommendations".bright_yellow().bold());
    println!("{}", "=".repeat(80));

    let max_target_idx = policy.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap();

    if max_target_idx < network.node_count() {
        let target_asset = &network[petgraph::graph::NodeIndex::new(max_target_idx)];
        println!("\n🎯 Primary Target: {}", target_asset.id.bright_red().bold());
        println!("   Value: ${:.2}", target_asset.value);
        println!("   Vulnerability: {:.1}%", target_asset.vulnerability * 100.0);
        println!("   Attack Probability: {:.1}%", policy[max_target_idx] * 100.0);
        
        println!("\n✅ Recommended Actions:");
        println!("  1. Increase monitoring on '{}'", target_asset.id);
        println!("  2. Deploy additional IDS/IPS");
        println!("  3. Implement network segmentation");
        println!("  4. Conduct penetration testing on this asset");
        println!("  5. Review and patch known vulnerabilities");
    }

    if avg_success > 0.5 {
        println!("\n⚠️  WARNING: High attack success rate detected!");
        println!("   Consider increasing defense budget or reallocating resources.");
    }

    println!("\n{}", "=".repeat(80));
}

fn encode_state(
    network: &petgraph::Graph<attacker_strategy_emulator::defense::Asset, f64>,
    defense: &DefenseConfiguration,
) -> Array1<f64> {
    let mut state = Vec::new();

    for node in network.node_indices() {
        let asset = &network[node];
        state.push(asset.value / 100.0);
        state.push(asset.vulnerability);
        state.push(defense.get_coverage(node));
    }

    Array1::from_vec(state)
}
