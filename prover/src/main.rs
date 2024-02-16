use sha2::{Digest, Sha256};
use hex::encode;
use std::fs::File;
use std::io::Write;

// Define structs
struct Database {
    data: Vec<String>,
}

struct MerkleNode {
    hash: String,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
}

struct MerkleTree {
    root: Option<Box<MerkleNode>>,
}

// Hashing function
fn hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    encode(result)
}

// Database implementation
impl Database {
    fn new(data: Vec<String>) -> Database {
        Database { data }
    }
}

fn build_tree(data: &[String]) -> Option<Box<MerkleNode>> {
    if data.is_empty() {
        return None;
    }

    if data.len() == 1 {
        let hash = hash(&data[0]);
        return Some(Box::new(MerkleNode {
            hash,
            left: None,
            right: None,
        }));
    }

    let mid = data.len() / 2;
    let left_child = build_tree(&data[..mid].to_vec());
    let right_child = build_tree(&data[mid..].to_vec());

    let combined_hash = match (&left_child, &right_child) {
        (Some(left), Some(right)) => hash(&format!("{}{}", left.hash, right.hash)),
        _ => String::new(), // In case either left_child or right_child is None
    };

    Some(Box::new(MerkleNode {
        hash: encode(combined_hash), // Encode the combined hash
        left: left_child,
        right: right_child,
    }))
}


fn build_merkle_tree(data: &Vec<String>) -> MerkleTree {
    let root = build_tree(data);
    MerkleTree { root }
}

fn generate_merkle_proof(tree: &MerkleTree, item: &str) -> Option<Vec<String>> {
    fn generate_proof(node: &MerkleNode, item: &str, path: &mut Vec<String>) -> bool {
        if node.hash == hash(item) {
            return true;
        }
    
        let mut found = false;
        if let Some(ref left_child) = node.left {
            if generate_proof(left_child.as_ref(), item, path) {
                found = true;
                path.push(node.right.as_ref().unwrap().hash.clone());
            }
        }
    
        if !found {
            if let Some(ref right_child) = node.right {
                if generate_proof(right_child.as_ref(), item, path) {
                    path.push(node.left.as_ref().unwrap().hash.clone());
                }
            }
        }
    
        found
    }
    
    let mut proof = Vec::new();
    if let Some(ref root) = tree.root {
        if generate_proof(root.as_ref(), item, &mut proof) {
            return Some(proof);
        }
    }
    
    None
}


// Output proof to file
fn output_proof_to_file(proof: &[String], file_path: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    for hash in proof {
        writeln!(file, "{}", hash)?;
    }
    Ok(())
}

fn main() {
    // Sample data
    let data = vec!["item1".to_string(), "item2".to_string(), "item3".to_string()];
    let database = Database::new(data.clone());

    // Build Merkle tree
    let merkle_tree = build_merkle_tree(&data);

    // Generate proof for item
    let item_to_prove = "item1";
    let proof = match generate_merkle_proof(&merkle_tree, item_to_prove) {
        Some(proof) => proof,
        None => {
            println!("Item not found in the database.");
            return;
        }
    };

    // Output proof to file
    let file_path = "merkle_proof.txt";
    match output_proof_to_file(&proof, file_path) {
        Ok(()) => println!("Merkle proof has been output to file: {}", file_path),
        Err(err) => eprintln!("Error writing to file: {}", err),
    }
}
