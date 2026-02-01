use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a player in the security game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    Defender,
    Attacker,
}

/// Action available to a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: usize,
    pub name: String,
    pub cost: f64,
    pub player: Player,
}

/// Outcome of a game interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub defender_action: usize,
    pub attacker_action: usize,
    pub defender_payoff: f64,
    pub attacker_payoff: f64,
}

/// Security game model (Stackelberg game)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGame {
    pub defender_actions: Vec<Action>,
    pub attacker_actions: Vec<Action>,
    pub payoff_matrix_defender: Array2<f64>,
    pub payoff_matrix_attacker: Array2<f64>,
}

impl SecurityGame {
    /// Create a new security game
    pub fn new(
        defender_actions: Vec<Action>,
        attacker_actions: Vec<Action>,
    ) -> Self {
        let n_defender = defender_actions.len();
        let n_attacker = attacker_actions.len();
        
        Self {
            defender_actions,
            attacker_actions,
            payoff_matrix_defender: Array2::zeros((n_defender, n_attacker)),
            payoff_matrix_attacker: Array2::zeros((n_defender, n_attacker)),
        }
    }

    /// Set payoff for a specific outcome
    pub fn set_payoff(
        &mut self,
        defender_action: usize,
        attacker_action: usize,
        defender_payoff: f64,
        attacker_payoff: f64,
    ) {
        self.payoff_matrix_defender[[defender_action, attacker_action]] = defender_payoff;
        self.payoff_matrix_attacker[[defender_action, attacker_action]] = attacker_payoff;
    }

    /// Compute best response for attacker given defender's mixed strategy
    pub fn attacker_best_response(&self, defender_strategy: &Array1<f64>) -> Array1<f64> {
        let n_attacker = self.attacker_actions.len();
        let expected_payoffs = defender_strategy.dot(&self.payoff_matrix_attacker);
        
        // Pure strategy best response
        let max_payoff = expected_payoffs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mut best_response = Array1::zeros(n_attacker);
        
        for (i, &payoff) in expected_payoffs.iter().enumerate() {
            if (payoff - max_payoff).abs() < 1e-10 {
                best_response[i] = 1.0;
            }
        }
        
        // Normalize
        let sum = best_response.sum();
        if sum > 0.0 {
            best_response / sum
        } else {
            Array1::from_elem(n_attacker, 1.0 / n_attacker as f64)
        }
    }

    /// Compute expected utility for defender given strategies
    pub fn defender_expected_utility(
        &self,
        defender_strategy: &Array1<f64>,
        attacker_strategy: &Array1<f64>,
    ) -> f64 {
        defender_strategy.dot(&self.payoff_matrix_defender.dot(attacker_strategy))
    }

    /// Compute expected utility for attacker given strategies
    pub fn attacker_expected_utility(
        &self,
        defender_strategy: &Array1<f64>,
        attacker_strategy: &Array1<f64>,
    ) -> f64 {
        defender_strategy.dot(&self.payoff_matrix_attacker.dot(attacker_strategy))
    }
}

/// Strategy profile (mixed strategies for both players)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyProfile {
    pub defender_strategy: Array1<f64>,
    pub attacker_strategy: Array1<f64>,
}

impl StrategyProfile {
    pub fn new(defender_strategy: Array1<f64>, attacker_strategy: Array1<f64>) -> Self {
        Self {
            defender_strategy,
            attacker_strategy,
        }
    }

    /// Create uniform random strategies
    pub fn uniform(n_defender: usize, n_attacker: usize) -> Self {
        Self {
            defender_strategy: Array1::from_elem(n_defender, 1.0 / n_defender as f64),
            attacker_strategy: Array1::from_elem(n_attacker, 1.0 / n_attacker as f64),
        }
    }
}

/// Nash equilibrium solver using iterative best response
pub struct NashSolver {
    max_iterations: usize,
    tolerance: f64,
}

impl NashSolver {
    pub fn new(max_iterations: usize, tolerance: f64) -> Self {
        Self {
            max_iterations,
            tolerance,
        }
    }

    /// Find Nash equilibrium using fictitious play
    pub fn solve(&self, game: &SecurityGame) -> StrategyProfile {
        let n_defender = game.defender_actions.len();
        let n_attacker = game.attacker_actions.len();

        let mut defender_strategy = Array1::from_elem(n_defender, 1.0 / n_defender as f64);
        let mut attacker_strategy = Array1::from_elem(n_attacker, 1.0 / n_attacker as f64);

        for iteration in 0..self.max_iterations {
            let old_defender = defender_strategy.clone();
            let old_attacker = attacker_strategy.clone();

            // Attacker best responds to defender
            attacker_strategy = game.attacker_best_response(&defender_strategy);

            // Defender best responds to attacker (simplified)
            let expected_payoffs = game.payoff_matrix_defender.dot(&attacker_strategy);
            let max_payoff = expected_payoffs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            
            defender_strategy = Array1::zeros(n_defender);
            for (i, &payoff) in expected_payoffs.iter().enumerate() {
                if (payoff - max_payoff).abs() < 1e-10 {
                    defender_strategy[i] = 1.0;
                }
            }
            let sum = defender_strategy.sum();
            if sum > 0.0 {
                defender_strategy /= sum;
            }

            // Check convergence
            let diff = (&defender_strategy - &old_defender).mapv(|x| x.abs()).sum()
                + (&attacker_strategy - &old_attacker).mapv(|x| x.abs()).sum();

            if diff < self.tolerance {
                tracing::info!(
                    "Nash equilibrium converged in {} iterations",
                    iteration + 1
                );
                break;
            }
        }

        StrategyProfile::new(defender_strategy, attacker_strategy)
    }
}

/// Stackelberg equilibrium solver (defender commits first)
pub struct StackelbergSolver {
    resolution: usize, // Grid resolution for mixed strategy search
}

impl StackelbergSolver {
    pub fn new(resolution: usize) -> Self {
        Self { resolution }
    }

    /// Find Strong Stackelberg Equilibrium
    pub fn solve(&self, game: &SecurityGame) -> StrategyProfile {
        let n_defender = game.defender_actions.len();
        
        // For simplicity, we'll search over pure strategies and some mixed ones
        // In production, this would use linear programming
        
        let mut best_defender_utility = f64::NEG_INFINITY;
        let mut best_strategy = StrategyProfile::uniform(n_defender, game.attacker_actions.len());

        // Evaluate pure defender strategies
        for i in 0..n_defender {
            let mut defender_strategy = Array1::zeros(n_defender);
            defender_strategy[i] = 1.0;

            let attacker_response = game.attacker_best_response(&defender_strategy);
            let utility = game.defender_expected_utility(&defender_strategy, &attacker_response);

            if utility > best_defender_utility {
                best_defender_utility = utility;
                best_strategy = StrategyProfile::new(defender_strategy, attacker_response);
            }
        }

        best_strategy
    }
}

impl fmt::Display for SecurityGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Security Game")?;
        writeln!(f, "Defender Actions: {}", self.defender_actions.len())?;
        writeln!(f, "Attacker Actions: {}", self.attacker_actions.len())?;
        writeln!(f, "\nPayoff Matrix (Defender):")?;
        writeln!(f, "{}", self.payoff_matrix_defender)?;
        writeln!(f, "\nPayoff Matrix (Attacker):")?;
        writeln!(f, "{}", self.payoff_matrix_attacker)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let defender_actions = vec![
            Action {
                id: 0,
                name: "Honeypot".to_string(),
                cost: 10.0,
                player: Player::Defender,
            },
        ];
        let attacker_actions = vec![
            Action {
                id: 0,
                name: "Scan".to_string(),
                cost: 5.0,
                player: Player::Attacker,
            },
        ];

        let game = SecurityGame::new(defender_actions, attacker_actions);
        assert_eq!(game.defender_actions.len(), 1);
        assert_eq!(game.attacker_actions.len(), 1);
    }

    #[test]
    fn test_nash_solver() {
        let mut game = SecurityGame::new(
            vec![
                Action {
                    id: 0,
                    name: "Defend A".to_string(),
                    cost: 0.0,
                    player: Player::Defender,
                },
                Action {
                    id: 1,
                    name: "Defend B".to_string(),
                    cost: 0.0,
                    player: Player::Defender,
                },
            ],
            vec![
                Action {
                    id: 0,
                    name: "Attack A".to_string(),
                    cost: 0.0,
                    player: Player::Attacker,
                },
                Action {
                    id: 1,
                    name: "Attack B".to_string(),
                    cost: 0.0,
                    player: Player::Attacker,
                },
            ],
        );

        // Set payoffs for matching pennies style game
        game.set_payoff(0, 0, 1.0, -1.0);
        game.set_payoff(0, 1, -1.0, 1.0);
        game.set_payoff(1, 0, -1.0, 1.0);
        game.set_payoff(1, 1, 1.0, -1.0);

        let solver = NashSolver::new(1000, 1e-6);
        let equilibrium = solver.solve(&game);

        // In matching pennies, Nash equilibrium is (0.5, 0.5) for both players
        assert!(equilibrium.defender_strategy.iter().all(|&x| (x - 0.5).abs() < 0.1));
    }
}
