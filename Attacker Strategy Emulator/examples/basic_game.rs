use attacker_strategy_emulator::game_theory::{SecurityGame, Action, Player, NashSolver, StackelbergSolver};
use ndarray::Array1;

fn main() {
    println!("🎮 Security Game Theory - Basic Example\n");
    println!("{}", "=".repeat(80));

    // Define defender actions
    let defender_actions = vec![
        Action {
            id: 0,
            name: "Deploy Honeypot on Server A".to_string(),
            cost: 10.0,
            player: Player::Defender,
        },
        Action {
            id: 1,
            name: "Deploy IDS on Server B".to_string(),
            cost: 15.0,
            player: Player::Defender,
        },
        Action {
            id: 2,
            name: "Patch Both Servers".to_string(),
            cost: 8.0,
            player: Player::Defender,
        },
    ];

    // Define attacker actions
    let attacker_actions = vec![
        Action {
            id: 0,
            name: "Attack Server A".to_string(),
            cost: 5.0,
            player: Player::Attacker,
        },
        Action {
            id: 1,
            name: "Attack Server B".to_string(),
            cost: 7.0,
            player: Player::Attacker,
        },
        Action {
            id: 2,
            name: "Exploit Both Servers".to_string(),
            cost: 12.0,
            player: Player::Attacker,
        },
    ];

    // Create security game
    let mut game = SecurityGame::new(defender_actions.clone(), attacker_actions.clone());

    // Set payoff matrix
    // Defender payoffs (higher is better for defender)
    game.set_payoff(0, 0, 50.0, -30.0);  // Honeypot catches attacker on A
    game.set_payoff(0, 1, -80.0, 60.0);  // B is vulnerable
    game.set_payoff(0, 2, -100.0, 70.0); // Both attacked, only A protected

    game.set_payoff(1, 0, -70.0, 50.0);  // A is vulnerable
    game.set_payoff(1, 1, 40.0, -20.0);  // IDS detects on B
    game.set_payoff(1, 2, -90.0, 60.0);  // Both attacked, only B protected

    game.set_payoff(2, 0, 30.0, -10.0);  // Both patched, A targeted
    game.set_payoff(2, 1, 35.0, -15.0);  // Both patched, B targeted
    game.set_payoff(2, 2, 20.0, -5.0);   // Both patched, both attacked

    println!("\n📋 Game Configuration:");
    println!("{}", game);

    // Solve for Nash equilibrium
    println!("\n🔍 Computing Nash Equilibrium...");
    let nash_solver = NashSolver::new(1000, 1e-6);
    let nash_equilibrium = nash_solver.solve(&game);

    println!("\n✅ Nash Equilibrium Found:");
    println!("\nDefender Strategy:");
    for (i, action) in defender_actions.iter().enumerate() {
        println!("  • {}: {:.1}%", action.name, nash_equilibrium.defender_strategy[i] * 100.0);
    }

    println!("\nAttacker Strategy:");
    for (i, action) in attacker_actions.iter().enumerate() {
        println!("  • {}: {:.1}%", action.name, nash_equilibrium.attacker_strategy[i] * 100.0);
    }

    // Compute expected utilities
    let defender_utility = game.defender_expected_utility(
        &nash_equilibrium.defender_strategy,
        &nash_equilibrium.attacker_strategy,
    );
    let attacker_utility = game.attacker_expected_utility(
        &nash_equilibrium.defender_strategy,
        &nash_equilibrium.attacker_strategy,
    );

    println!("\n💰 Expected Utilities:");
    println!("  Defender: {:.2}", defender_utility);
    println!("  Attacker: {:.2}", attacker_utility);

    // Solve for Stackelberg equilibrium
    println!("\n\n🎯 Computing Stackelberg Equilibrium (Defender Commits First)...");
    let stackelberg_solver = StackelbergSolver::new(10);
    let stackelberg_equilibrium = stackelberg_solver.solve(&game);

    println!("\n✅ Stackelberg Equilibrium Found:");
    println!("\nDefender Strategy (Commitment):");
    for (i, action) in defender_actions.iter().enumerate() {
        println!("  • {}: {:.1}%", action.name, stackelberg_equilibrium.defender_strategy[i] * 100.0);
    }

    println!("\nAttacker Best Response:");
    for (i, action) in attacker_actions.iter().enumerate() {
        println!("  • {}: {:.1}%", action.name, stackelberg_equilibrium.attacker_strategy[i] * 100.0);
    }

    let stackelberg_defender_utility = game.defender_expected_utility(
        &stackelberg_equilibrium.defender_strategy,
        &stackelberg_equilibrium.attacker_strategy,
    );

    println!("\n💰 Stackelberg Defender Utility: {:.2}", stackelberg_defender_utility);

    // Compare equilibria
    println!("\n\n📊 Equilibrium Comparison:");
    println!("{}", "-".repeat(80));
    println!("Nash Equilibrium:");
    println!("  Defender Utility: {:.2}", defender_utility);
    println!("  Attacker Utility: {:.2}", attacker_utility);
    println!("\nStackelberg Equilibrium:");
    println!("  Defender Utility: {:.2}", stackelberg_defender_utility);
    println!("  Improvement: {:.2}", stackelberg_defender_utility - defender_utility);

    println!("\n💡 Insight:");
    if stackelberg_defender_utility > defender_utility {
        println!("  The defender benefits from committing to a strategy first!");
        println!("  This is the essence of security-by-design: commit early to defenses.");
    } else {
        println!("  In this scenario, commitment doesn't provide additional advantage.");
    }

    println!("\n{}", "=".repeat(80));
}
