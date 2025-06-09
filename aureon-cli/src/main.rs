use aureon_node::Blockchain;
use aureon_core::types::Transaction;

fn main() {
    let mut chain = Blockchain::new();
    println!("Genesis Block Hash: {}", chain.blocks[0].hash());

    let tx = Transaction {
        from: "Alice".to_string(),
        to: "Bob".to_string(),
        amount: 100,
        signature: "0xSIGNATURE".to_string(),
    };

    let new_block = chain.add_block(vec![tx]);
    println!("New Block Hash: {}", new_block.hash());
}