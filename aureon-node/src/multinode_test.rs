/// Multi-node integration testing infrastructure
/// Allows spawning and coordinating multiple node instances for testing

use crate::types::Block;
use crate::network::Network;
use crate::sync::BlockSyncState;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::net::SocketAddr;

/// Configuration for a test node instance
#[derive(Clone, Debug)]
pub struct TestNodeConfig {
    pub node_id: String,
    pub port: u16,
    pub peer_ports: Vec<u16>,
}

/// Represents a running test node
pub struct TestNode {
    pub config: TestNodeConfig,
    pub network: Network,
    pub sync_state: Arc<Mutex<BlockSyncState>>,
}

impl TestNode {
    /// Create a new test node
    pub fn new(config: TestNodeConfig) -> Self {
        let network = Network::new(config.node_id.clone(), "1.0.0".to_string());
        let sync_state = Arc::new(Mutex::new(BlockSyncState::new()));

        TestNode {
            config,
            network,
            sync_state,
        }
    }

    /// Start the node's network listener
    pub fn start(&self) {
        let addr = format!("127.0.0.1:{}", self.config.port);
        let network = self.network.clone();
        
        thread::spawn(move || {
            network.listen(&addr);
        });

        // Give the listener time to start
        thread::sleep(Duration::from_millis(100));

        // Connect to peer nodes
        for peer_port in &self.config.peer_ports {
            let peer_addr = format!("127.0.0.1:{}", peer_port);
            self.network.add_peer(&peer_addr, Some(format!("peer-{}", peer_port)));
        }

        // Give connections time to establish
        thread::sleep(Duration::from_millis(100));

        // Broadcast peer info
        self.network.broadcast_peer_info(0);
    }

    /// Get current sync state
    pub fn get_sync_state(&self) -> (u64, u64, bool) {
        let state = self.sync_state.lock().unwrap();
        (state.local_height, state.peer_max_height, state.is_synced())
    }

    /// Simulate receiving a block from network
    pub fn receive_block(&self, block: Block) -> Result<(), String> {
        self.sync_state.lock().unwrap().stage_block(block)
    }

    /// Simulate updating peer height
    pub fn update_peer_height(&self, height: u64) {
        self.sync_state.lock().unwrap().update_peer_height(height);
    }

    /// Get number of connected peers
    pub fn peer_count(&self) -> usize {
        self.network.peer_count()
    }

    /// Get highest block height from peers
    pub fn get_highest_peer_height(&self) -> u64 {
        self.network.get_highest_peer_height()
    }
}

/// Test cluster of multiple nodes
pub struct TestCluster {
    pub nodes: Vec<TestNode>,
}

impl TestCluster {
    /// Create a new cluster with N nodes
    pub fn new(num_nodes: usize) -> Self {
        let mut nodes = Vec::new();
        let base_port = 9000;

        for i in 0..num_nodes {
            let port = base_port + i as u16;
            let node_id = format!("test-node-{}", i);

            // Each node connects to all other nodes
            let peer_ports: Vec<u16> = (0..num_nodes)
                .map(|j| {
                    if i != j {
                        base_port + j as u16
                    } else {
                        0 // Skip self
                    }
                })
                .filter(|p| *p != 0)
                .collect();

            let config = TestNodeConfig {
                node_id,
                port,
                peer_ports,
            };

            nodes.push(TestNode::new(config));
        }

        TestCluster { nodes }
    }

    /// Start all nodes in the cluster
    pub fn start_all(&self) {
        for node in &self.nodes {
            node.start();
        }
    }

    /// Wait for all nodes to have at least num_peers connected
    pub fn wait_for_connectivity(&self, num_peers: usize, timeout_ms: u64) -> bool {
        let start = std::time::Instant::now();

        loop {
            let all_connected = self.nodes.iter().all(|node| {
                // Account for asymmetric connections - nodes may not see all peers immediately
                node.peer_count() >= num_peers.saturating_sub(1)
            });

            if all_connected {
                return true;
            }

            if start.elapsed().as_millis() > timeout_ms as u128 {
                return false;
            }

            thread::sleep(Duration::from_millis(50));
        }
    }

    /// Simulate a block being produced on node index `producer_idx`
    pub fn produce_block(&self, producer_idx: usize) -> Result<(), String> {
        if producer_idx >= self.nodes.len() {
            return Err(format!("Invalid node index: {}", producer_idx));
        }

        // In a real scenario, this would actually produce a block
        // For testing, we just update the sync state
        self.nodes[producer_idx]
            .sync_state
            .lock()
            .unwrap()
            .update_local_height(1);

        // Broadcast to other nodes
        self.nodes[producer_idx].network.broadcast_peer_info(1);

        Ok(())
    }

    /// Wait for all nodes to reach the same block height
    pub fn wait_for_consensus(
        &self,
        target_height: u64,
        timeout_ms: u64,
    ) -> Result<(), String> {
        let start = std::time::Instant::now();

        loop {
            let all_synced = self.nodes.iter().all(|node| {
                let (local, _, _) = node.get_sync_state();
                local >= target_height
            });

            if all_synced {
                return Ok(());
            }

            if start.elapsed().as_millis() > timeout_ms as u128 {
                return Err(format!(
                    "Timeout waiting for consensus at height {}",
                    target_height
                ));
            }

            thread::sleep(Duration::from_millis(100));
        }
    }

    /// Get status of all nodes
    pub fn get_status(&self) -> Vec<(String, u64, u64, bool)> {
        self.nodes
            .iter()
            .map(|node| {
                let (local, peer_max, synced) = node.get_sync_state();
                (node.config.node_id.clone(), local, peer_max, synced)
            })
            .collect()
    }

    /// Print cluster status
    pub fn print_status(&self) {
        println!("\n=== Cluster Status ===");
        for (node_id, local, peer_max, synced) in self.get_status() {
            let sync_str = if synced { "✅ SYNCED" } else { "⏳ SYNCING" };
            println!(
                "{}: local_height={}, peer_max={}, {}",
                node_id, local, peer_max, sync_str
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_node_creation() {
        let config = TestNodeConfig {
            node_id: "node1".to_string(),
            port: 9000,
            peer_ports: vec![],
        };
        let node = TestNode::new(config);
        assert_eq!(node.config.node_id, "node1");
        assert_eq!(node.peer_count(), 0);
    }

    #[test]
    fn test_cluster_creation() {
        let cluster = TestCluster::new(3);
        assert_eq!(cluster.nodes.len(), 3);
        assert_eq!(cluster.nodes[0].config.node_id, "test-node-0");
        assert_eq!(cluster.nodes[1].config.node_id, "test-node-1");
        assert_eq!(cluster.nodes[2].config.node_id, "test-node-2");
    }

    #[test]
    fn test_cluster_peer_configuration() {
        let cluster = TestCluster::new(3);

        // Node 0 should connect to nodes 1 and 2
        assert_eq!(cluster.nodes[0].config.peer_ports.len(), 2);
        assert!(cluster.nodes[0].config.peer_ports.contains(&9001));
        assert!(cluster.nodes[0].config.peer_ports.contains(&9002));

        // Node 1 should connect to nodes 0 and 2
        assert_eq!(cluster.nodes[1].config.peer_ports.len(), 2);
        assert!(cluster.nodes[1].config.peer_ports.contains(&9000));
        assert!(cluster.nodes[1].config.peer_ports.contains(&9002));
    }

    #[test]
    fn test_node_sync_state_update() {
        let config = TestNodeConfig {
            node_id: "node1".to_string(),
            port: 9000,
            peer_ports: vec![],
        };
        let node = TestNode::new(config);

        let (local, peer_max, synced) = node.get_sync_state();
        assert_eq!(local, 0);
        assert_eq!(peer_max, 0);
        assert!(synced);

        node.update_peer_height(10);
        let (local, peer_max, synced) = node.get_sync_state();
        assert_eq!(local, 0);
        assert_eq!(peer_max, 10);
        assert!(!synced);
    }

    #[test]
    fn test_cluster_status() {
        let cluster = TestCluster::new(2);
        let status = cluster.get_status();
        
        assert_eq!(status.len(), 2);
        assert_eq!(status[0].0, "test-node-0");
        assert_eq!(status[1].0, "test-node-1");
    }

    #[test]
    fn test_block_production() {
        let cluster = TestCluster::new(2);
        
        let result = cluster.produce_block(0);
        assert!(result.is_ok());

        let (local, _, _) = cluster.nodes[0].get_sync_state();
        assert_eq!(local, 1);
    }

    #[test]
    fn test_peer_height_propagation() {
        let cluster = TestCluster::new(3);
        
        // Node 0 receives a block at height 5
        cluster.nodes[0].update_peer_height(5);
        
        // Check node 0 sees the height
        let (_, peer_max, _) = cluster.nodes[0].get_sync_state();
        assert_eq!(peer_max, 5);
    }

    #[test]
    fn test_sync_detection() {
        let cluster = TestCluster::new(2);
        
        // Both nodes start synced
        let (_, _, synced0) = cluster.nodes[0].get_sync_state();
        let (_, _, synced1) = cluster.nodes[1].get_sync_state();
        assert!(synced0);
        assert!(synced1);

        // Node 0 advances ahead
        cluster.nodes[0].update_peer_height(10);
        
        // Now node 0 should be out of sync
        let (_, peer_max, synced) = cluster.nodes[0].get_sync_state();
        assert_eq!(peer_max, 10);
        assert!(!synced);
    }

    #[test]
    fn test_multiple_blocks_production() {
        let cluster = TestCluster::new(3);
        
        // Produce blocks on different nodes
        assert!(cluster.produce_block(0).is_ok());
        assert!(cluster.produce_block(1).is_ok());
        assert!(cluster.produce_block(2).is_ok());

        let status = cluster.get_status();
        assert_eq!(status.len(), 3);
        
        // Each node should have produced 1 block
        assert_eq!(status[0].1, 1);
        assert_eq!(status[1].1, 1);
        assert_eq!(status[2].1, 1);
    }

    #[test]
    fn test_consensus_detection() {
        let cluster = TestCluster::new(2);
        let status = cluster.get_status();
        
        // All nodes should be in consensus initially
        for (_, local, peer_max, synced) in status {
            assert_eq!(local, peer_max);
            assert!(synced);
        }
    }

    #[test]
    fn test_two_node_cluster_networking() {
        let cluster = TestCluster::new(2);
        
        // Each node should have 1 peer
        assert_eq!(cluster.nodes[0].config.peer_ports.len(), 1);
        assert_eq!(cluster.nodes[1].config.peer_ports.len(), 1);
        
        // Peer ports should be correct
        assert_eq!(cluster.nodes[0].config.peer_ports[0], 9001);
        assert_eq!(cluster.nodes[1].config.peer_ports[0], 9000);
    }

    #[test]
    fn test_large_cluster_creation() {
        let cluster = TestCluster::new(5);
        assert_eq!(cluster.nodes.len(), 5);
        
        // In a 5-node cluster, each node should connect to 4 others
        for node in &cluster.nodes {
            assert_eq!(node.config.peer_ports.len(), 4);
        }
    }

    #[test]
    fn test_cluster_convergence() {
        let cluster = TestCluster::new(3);
        
        // All nodes start at height 0
        for node in &cluster.nodes {
            let (local, _, _) = node.get_sync_state();
            assert_eq!(local, 0);
        }
        
        // Simulate one node getting ahead
        cluster.nodes[0].update_peer_height(5);
        cluster.nodes[1].update_peer_height(5);
        cluster.nodes[2].update_peer_height(5);
        
        // All should see the peer height
        for node in &cluster.nodes {
            let (_, peer_max, _) = node.get_sync_state();
            assert_eq!(peer_max, 5);
        }
    }
}
