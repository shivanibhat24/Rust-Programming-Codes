use ndarray::{Array1, Array2, Axis};
use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Experience replay buffer for DQN
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayBuffer {
    capacity: usize,
    buffer: VecDeque<Experience>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub state: Array1<f64>,
    pub action: usize,
    pub reward: f64,
    pub next_state: Array1<f64>,
    pub done: bool,
}

impl ReplayBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, experience: Experience) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(experience);
    }

    pub fn sample(&self, batch_size: usize) -> Vec<Experience> {
        let mut rng = rand::thread_rng();
        let mut samples = Vec::with_capacity(batch_size);
        
        for _ in 0..batch_size {
            if let Some(exp) = self.buffer.get(rng.gen_range(0..self.buffer.len())) {
                samples.push(exp.clone());
            }
        }
        
        samples
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// Simple neural network for Q-value approximation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QNetwork {
    weights_1: Array2<f64>,
    bias_1: Array1<f64>,
    weights_2: Array2<f64>,
    bias_2: Array1<f64>,
    learning_rate: f64,
}

impl QNetwork {
    pub fn new(
        input_size: usize,
        hidden_size: usize,
        output_size: usize,
        learning_rate: f64,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, 0.1).unwrap();

        let weights_1 = Array2::from_shape_fn((hidden_size, input_size), |_| {
            normal.sample(&mut rng)
        });
        let bias_1 = Array1::zeros(hidden_size);

        let weights_2 = Array2::from_shape_fn((output_size, hidden_size), |_| {
            normal.sample(&mut rng)
        });
        let bias_2 = Array1::zeros(output_size);

        Self {
            weights_1,
            bias_1,
            weights_2,
            bias_2,
            learning_rate,
        }
    }

    /// Forward pass through the network
    pub fn forward(&self, state: &Array1<f64>) -> Array1<f64> {
        // First layer
        let hidden = self.weights_1.dot(state) + &self.bias_1;
        let hidden_activated = hidden.mapv(|x| x.max(0.0)); // ReLU

        // Output layer
        self.weights_2.dot(&hidden_activated) + &self.bias_2
    }

    /// Select action using epsilon-greedy policy
    pub fn select_action(&self, state: &Array1<f64>, epsilon: f64) -> usize {
        let mut rng = rand::thread_rng();
        
        if rng.gen::<f64>() < epsilon {
            // Explore: random action
            let q_values = self.forward(state);
            rng.gen_range(0..q_values.len())
        } else {
            // Exploit: best action
            let q_values = self.forward(state);
            q_values
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(idx, _)| idx)
                .unwrap()
        }
    }

    /// Update network weights using gradient descent
    pub fn update(
        &mut self,
        state: &Array1<f64>,
        action: usize,
        target: f64,
    ) {
        // Forward pass
        let hidden_raw = self.weights_1.dot(state) + &self.bias_1;
        let hidden = hidden_raw.mapv(|x| x.max(0.0));
        let output = self.weights_2.dot(&hidden) + &self.bias_2;

        // Compute loss gradient
        let mut output_grad = Array1::zeros(output.len());
        output_grad[action] = 2.0 * (output[action] - target);

        // Backprop to hidden layer
        let weights_2_grad = output_grad.clone().insert_axis(Axis(1)).dot(
            &hidden.clone().insert_axis(Axis(0))
        );
        let bias_2_grad = output_grad.clone();

        let hidden_grad = self.weights_2.t().dot(&output_grad);
        let hidden_grad_activated = hidden_grad.iter()
            .zip(hidden_raw.iter())
            .map(|(g, h)| if *h > 0.0 { *g } else { 0.0 })
            .collect::<Vec<_>>();
        let hidden_grad_activated = Array1::from_vec(hidden_grad_activated);

        let weights_1_grad = hidden_grad_activated.clone().insert_axis(Axis(1)).dot(
            &state.clone().insert_axis(Axis(0))
        );
        let bias_1_grad = hidden_grad_activated;

        // Update weights
        self.weights_2 = &self.weights_2 - &(weights_2_grad * self.learning_rate);
        self.bias_2 = &self.bias_2 - &(bias_2_grad * self.learning_rate);
        self.weights_1 = &self.weights_1 - &(weights_1_grad * self.learning_rate);
        self.bias_1 = &self.bias_1 - &(bias_1_grad * self.learning_rate);
    }
}

/// Deep Q-Network agent for learning attack strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DQNAgent {
    q_network: QNetwork,
    target_network: QNetwork,
    replay_buffer: ReplayBuffer,
    epsilon: f64,
    epsilon_decay: f64,
    epsilon_min: f64,
    gamma: f64,
    batch_size: usize,
    update_frequency: usize,
    steps: usize,
}

impl DQNAgent {
    pub fn new(
        state_size: usize,
        action_size: usize,
        hidden_size: usize,
        learning_rate: f64,
        buffer_capacity: usize,
    ) -> Self {
        let q_network = QNetwork::new(state_size, hidden_size, action_size, learning_rate);
        let target_network = q_network.clone();

        Self {
            q_network,
            target_network,
            replay_buffer: ReplayBuffer::new(buffer_capacity),
            epsilon: 1.0,
            epsilon_decay: 0.995,
            epsilon_min: 0.01,
            gamma: 0.99,
            batch_size: 32,
            update_frequency: 100,
            steps: 0,
        }
    }

    /// Select action based on current policy
    pub fn select_action(&self, state: &Array1<f64>) -> usize {
        self.q_network.select_action(state, self.epsilon)
    }

    /// Store experience in replay buffer
    pub fn store_experience(&mut self, experience: Experience) {
        self.replay_buffer.push(experience);
    }

    /// Train the agent on a batch of experiences
    pub fn train(&mut self) {
        if self.replay_buffer.len() < self.batch_size {
            return;
        }

        let batch = self.replay_buffer.sample(self.batch_size);

        for exp in batch {
            // Compute target Q-value
            let next_q_values = self.target_network.forward(&exp.next_state);
            let max_next_q = next_q_values
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max);

            let target = if exp.done {
                exp.reward
            } else {
                exp.reward + self.gamma * max_next_q
            };

            // Update Q-network
            self.q_network.update(&exp.state, exp.action, target);
        }

        // Decay epsilon
        self.epsilon = (self.epsilon * self.epsilon_decay).max(self.epsilon_min);

        // Update target network periodically
        self.steps += 1;
        if self.steps % self.update_frequency == 0 {
            self.target_network = self.q_network.clone();
        }
    }

    /// Get current policy as probability distribution
    pub fn get_policy(&self, state: &Array1<f64>) -> Array1<f64> {
        let q_values = self.q_network.forward(state);
        
        // Softmax for probability distribution
        let max_q = q_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_q = q_values.mapv(|x| (x - max_q).exp());
        let sum_exp = exp_q.sum();
        
        exp_q / sum_exp
    }

    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }
}

/// Policy gradient agent for continuous action spaces
#[derive(Debug, Clone)]
pub struct PolicyGradientAgent {
    policy_network: QNetwork,
    baseline: f64,
    learning_rate: f64,
    trajectory: Vec<(Array1<f64>, usize, f64)>,
}

impl PolicyGradientAgent {
    pub fn new(
        state_size: usize,
        action_size: usize,
        hidden_size: usize,
        learning_rate: f64,
    ) -> Self {
        Self {
            policy_network: QNetwork::new(state_size, hidden_size, action_size, learning_rate),
            baseline: 0.0,
            learning_rate,
            trajectory: Vec::new(),
        }
    }

    /// Sample action from policy
    pub fn select_action(&self, state: &Array1<f64>) -> usize {
        let logits = self.policy_network.forward(state);
        
        // Softmax to get probabilities
        let max_logit = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_logits = logits.mapv(|x| (x - max_logit).exp());
        let sum_exp = exp_logits.sum();
        let probs = exp_logits / sum_exp;

        // Sample from categorical distribution
        let mut rng = rand::thread_rng();
        let sample: f64 = rng.gen();
        let mut cumsum = 0.0;
        
        for (i, &p) in probs.iter().enumerate() {
            cumsum += p;
            if sample <= cumsum {
                return i;
            }
        }
        
        probs.len() - 1
    }

    /// Store transition
    pub fn store_transition(&mut self, state: Array1<f64>, action: usize, reward: f64) {
        self.trajectory.push((state, action, reward));
    }

    /// Update policy based on episode trajectory
    pub fn update_policy(&mut self) {
        if self.trajectory.is_empty() {
            return;
        }

        // Compute returns
        let mut returns = Vec::with_capacity(self.trajectory.len());
        let mut g = 0.0;
        for (_, _, reward) in self.trajectory.iter().rev() {
            g = reward + 0.99 * g;
            returns.push(g);
        }
        returns.reverse();

        // Update baseline
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        self.baseline = 0.9 * self.baseline + 0.1 * mean_return;

        // Update policy for each step
        for ((state, action, _), &ret) in self.trajectory.iter().zip(returns.iter()) {
            let advantage = ret - self.baseline;
            self.policy_network.update(state, *action, advantage);
        }

        self.trajectory.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_buffer() {
        let mut buffer = ReplayBuffer::new(10);
        
        for i in 0..15 {
            buffer.push(Experience {
                state: Array1::from_elem(4, i as f64),
                action: i,
                reward: i as f64,
                next_state: Array1::from_elem(4, (i + 1) as f64),
                done: false,
            });
        }

        assert_eq!(buffer.len(), 10);
    }

    #[test]
    fn test_q_network() {
        let network = QNetwork::new(4, 8, 2, 0.01);
        let state = Array1::from_elem(4, 0.5);
        let output = network.forward(&state);
        
        assert_eq!(output.len(), 2);
    }

    #[test]
    fn test_dqn_agent() {
        let mut agent = DQNAgent::new(4, 2, 8, 0.01, 100);
        let state = Array1::from_elem(4, 0.5);
        
        let action = agent.select_action(&state);
        assert!(action < 2);

        agent.store_experience(Experience {
            state: state.clone(),
            action,
            reward: 1.0,
            next_state: state.clone(),
            done: false,
        });

        assert_eq!(agent.replay_buffer.len(), 1);
    }
}
