use attacker_strategy_emulator::simulation::{Simulator, SimulationConfig, create_example_network};
use attacker_strategy_emulator::analysis::Analyzer;
use colored::*;
use std::io::{self, Write};

fn main() {
    println!("{}", "=" .repeat(80).bright_blue());
    println!("{}", "    🎮 ATTACKER STRATEGY EMULATOR - FULL SIMULATION".bright_white().bold());
    println!("{}", "    Game-Theoretic Security Analysis with ML-Based Adversaries".bright_cyan());
    println!("{}", "=".repeat(80).bright_blue());

    // Configuration
    let config = SimulationConfig {
        num_episodes: 1000,
        max_steps_per_episode: 30,
        defender_budget: 120.0,
        attacker_budget: 60.0,
        learning_rate: 0.005,
        discount_factor: 0.95,
    };

    println!("\n⚙️  {} {}", "Simulation Configuration:".bright_white().bold(), "");
    println!("   Episodes: {}", config.num_episodes);
    println!("   Steps per Episode: {}", config.max_steps_per_episode);
    println!("   Defender Budget: ${:.2}", config.defender_budget);
    println!("   Attacker Budget: ${:.2}", config.attacker_budget);
    println!("   Learning Rate: {}", config.learning_rate);
    println!("   Discount Factor: {}", config.discount_factor);

    // Create network
    let network = create_example_network();
    
    println!("\n🌐 {} {}", "Network Topology:".bright_white().bold(), "");
    println!("   Nodes: {}", network.node_count());
    println!("   Edges: {}", network.edge_count());
    
    println!("\n   Assets:");
    for (i, node) in network.node_indices().enumerate() {
        let asset = &network[node];
        let vuln_color = if asset.vulnerability > 0.7 {
            "red"
        } else if asset.vulnerability > 0.4 {
            "yellow"
        } else {
            "green"
        };
        
        println!("   {}. {} - Value: ${:.0}, Vuln: {}, Criticality: {:.1}",
            i + 1,
            asset.id.bright_white().bold(),
            asset.value,
            match vuln_color {
                "red" => format!("{:.0}%", asset.vulnerability * 100.0).red(),
                "yellow" => format!("{:.0}%", asset.vulnerability * 100.0).yellow(),
                _ => format!("{:.0}%", asset.vulnerability * 100.0).green(),
            },
            asset.criticality
        );
    }

    // Prompt to start
    println!("\n{}", "-".repeat(80));
    print!("\n{} ", "Press Enter to start simulation...".bright_yellow());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    // Run simulation
    println!("\n{}", "🚀 Starting Simulation...".bright_green().bold());
    println!("{}", "-".repeat(80));

    let mut simulator = Simulator::new(config, network);
    let metrics = simulator.run();

    // Analysis
    println!("\n{}", "=".repeat(80).bright_blue());
    println!("{}", "📊 SIMULATION COMPLETE - ANALYZING RESULTS".bright_white().bold());
    println!("{}", "=".repeat(80).bright_blue());

    let analyzer = Analyzer::new(metrics.clone());
    analyzer.print_summary();

    // Visualize trends
    println!("\n{}", "📈 Reward Trends (Last 20 Episodes)".bright_white().bold());
    println!("{}", "-".repeat(80));
    
    let (defender_rewards, attacker_rewards) = analyzer.get_reward_trends();
    let start = defender_rewards.len().saturating_sub(20);
    
    for i in start..defender_rewards.len() {
        let episode = i + 1;
        let d_reward = defender_rewards[i];
        let a_reward = attacker_rewards[i];
        
        let d_bar_len = ((d_reward.abs() / 50.0) * 30.0).min(30.0) as usize;
        let a_bar_len = ((a_reward.abs() / 50.0) * 30.0).min(30.0) as usize;
        
        let d_bar = if d_reward >= 0.0 {
            "█".repeat(d_bar_len).green()
        } else {
            "█".repeat(d_bar_len).red()
        };
        
        let a_bar = if a_reward >= 0.0 {
            "█".repeat(a_bar_len).green()
        } else {
            "█".repeat(a_bar_len).red()
        };
        
        println!("Ep {:>4} | D: {:>7.1} {} | A: {:>7.1} {}",
            episode,
            d_reward,
            d_bar,
            a_reward,
            a_bar
        );
    }

    // Export results
    println!("\n{}", "-".repeat(80));
    print!("\n{} ", "Export results to JSON? (y/n):".bright_yellow());
    io::stdout().flush().unwrap();
    let mut export_input = String::new();
    io::stdin().read_line(&mut export_input).unwrap();
    
    if export_input.trim().to_lowercase() == "y" {
        match analyzer.export_json() {
            Ok(json) => {
                std::fs::write("/home/claude/simulation_results.json", json)
                    .expect("Failed to write file");
                println!("{}", "✅ Results exported to simulation_results.json".green());
            }
            Err(e) => {
                println!("{} {}", "❌ Export failed:".red(), e);
            }
        }
    }

    // Optimize defense
    println!("\n{}", "=".repeat(80).bright_blue());
    println!("{}", "🛡️  DEFENSE OPTIMIZATION".bright_white().bold());
    println!("{}", "=".repeat(80).bright_blue());

    let optimized_defense = simulator.optimize_defense();
    
    println!("\n✅ Optimized Defense Configuration:");
    println!("   Total Allocations: {}", optimized_defense.allocations.len());
    println!("   Total Cost: ${:.2}", optimized_defense.total_cost);
    println!("   Budget Utilization: {:.1}%", 
        (optimized_defense.total_cost / optimized_defense.total_budget) * 100.0
    );

    println!("\n   Defense Allocations:");
    for alloc in &optimized_defense.allocations {
        println!("   • {:?} - Coverage: {:.0}%, Effectiveness: {:.0}%, Cost: ${:.2}",
            alloc.defense_type,
            alloc.coverage * 100.0,
            alloc.effectiveness * 100.0,
            alloc.cost
        );
    }

    // Final summary
    println!("\n{}", "=".repeat(80).bright_blue());
    println!("{}", "🎯 KEY TAKEAWAYS".bright_white().bold());
    println!("{}", "=".repeat(80).bright_blue());
    
    let report = analyzer.generate_report();
    
    let threat_level = if report.convergence_analysis.average_success_rate > 0.6 {
        "CRITICAL".bright_red().bold()
    } else if report.convergence_analysis.average_success_rate > 0.3 {
        "ELEVATED".bright_yellow().bold()
    } else {
        "MANAGED".bright_green().bold()
    };

    println!("\n🔒 Security Posture: {}", threat_level);
    println!("   Attack Success Rate: {:.1}%", 
        report.convergence_analysis.average_success_rate * 100.0);
    println!("   Detection Rate: {:.1}%", 
        report.convergence_analysis.average_detection_rate * 100.0);
    println!("   Expected Annual Loss: ${:.2}", 
        report.vulnerability_assessment.expected_loss);

    println!("\n💰 ROI Analysis:");
    let prevention_value = report.vulnerability_assessment.expected_loss 
        * (1.0 - report.convergence_analysis.average_success_rate);
    let roi = (prevention_value - optimized_defense.total_cost) / optimized_defense.total_cost * 100.0;
    
    println!("   Defense Investment: ${:.2}", optimized_defense.total_cost);
    println!("   Expected Prevention Value: ${:.2}", prevention_value);
    println!("   ROI: {:.1}%", roi);

    if roi > 100.0 {
        println!("   {}", "✅ Defense investment is cost-effective".green());
    } else if roi > 0.0 {
        println!("   {}", "⚠️  Marginal ROI - consider optimization".yellow());
    } else {
        println!("   {}", "❌ Negative ROI - strategy needs revision".red());
    }

    println!("\n{}", "=".repeat(80).bright_blue());
    println!("{}", "🏁 Simulation Complete!".bright_white().bold());
    println!("{}", "=".repeat(80).bright_blue());
}
