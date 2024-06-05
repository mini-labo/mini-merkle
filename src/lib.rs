use napi_derive::napi;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use num_traits::Zero;
use prost::Message;
use rayon::prelude::*;
use tiny_keccak::{Hasher, Keccak};

pub mod mini {
  pub mod merkle {
    include!(concat!(env!("OUT_DIR"), "/mini.merkle.rs"));
  }
}

// Wrapper for a fixed-size array to allow conversion
#[derive(Debug)]
struct U8Array32([u8; 32]);

impl From<Vec<u8>> for U8Array32 {
  fn from(vec: Vec<u8>) -> Self {
    let mut array = [0u8; 32];
    array.copy_from_slice(&vec[..32]);
    U8Array32(array)
  }
}

impl From<U8Array32> for Vec<u8> {
  fn from(array: U8Array32) -> Self {
    array.0.to_vec()
  }
}

#[napi]
pub struct MerkleTreeWrapper {
    inner: MerkleTree
}

#[derive(Debug)]
struct MerkleTree {
  pub leaves: Vec<[u8; 32]>,
  pub nodes: Vec<Vec<[u8; 32]>>,
  pub root: [u8; 32],
}

fn keccak256(input: &[u8]) -> [u8; 32] {
  let mut hasher = Keccak::v256();
  hasher.update(input);
  let mut result = [0u8; 32];
  hasher.finalize(&mut result);
  result
}

/// build a "bytes32" equivalent from an array of numbers (u8)
fn numbers_to_encoded_bytes_leaf(numbers: &[u8]) -> [u8; 32] {
  if numbers.len() != 7 {
    panic!("The array must contain exactly 7 numbers");
  }

  let mut encoded = BigUint::zero();

  for &num in numbers {
    if num < 1 || num > 80 {
      panic!("Numbers must be between 1 and 80");
    }
    encoded = (encoded << 7) | BigUint::from(num);
  }

  let mut encoded_bytes = [0u8; 32];
  let bytes = encoded.to_bytes_be();
  encoded_bytes[(32 - bytes.len())..].copy_from_slice(&bytes);

  encoded_bytes
}

fn encoded_bytes_leaf_to_numbers(encoded_leaf: &[u8; 32]) -> Vec<u8> {
  let mut encoded = BigUint::from_bytes_be(encoded_leaf);
  let mut numbers = Vec::new();

  for _ in 0..7 {
    let number = (&encoded & BigUint::from(0x7f_u32)).to_u8().unwrap();
    numbers.insert(0, number);
    encoded >>= 7;
  }

  numbers
}

#[napi]
impl MerkleTreeWrapper {
  #[napi(constructor)]
  pub fn new(leaves: Vec<Vec<u8>>) -> Self {
    let leaves: Vec<[u8; 32]> = leaves
      .into_iter()
      .map(|v| {
        let mut array = [0u8; 32];
        array.copy_from_slice(&v[..32]);
        array
      })
      .collect();

    let mut nodes = Vec::new();

    let leaf_hashes: Vec<[u8; 32]> = leaves.iter().map(|leaf| keccak256(leaf)).collect();
    nodes.push(leaf_hashes.clone());

    let mut current_level = leaf_hashes.clone();

    while current_level.len() > 1 {
      if current_level.len() % 2 != 0 {
        current_level.push(*current_level.last().unwrap());
      }

      let next_level: Vec<[u8; 32]> = current_level
        .par_chunks(2)
        .map(|pair| {
          let mut combined = Vec::new();
          if pair[0] < pair[1] {
            combined.extend_from_slice(&pair[0]);
            combined.extend_from_slice(&pair[1]);
          } else {
            combined.extend_from_slice(&pair[1]);
            combined.extend_from_slice(&pair[0]);
          }
          keccak256(&combined)
        })
        .collect();

      nodes.push(next_level.clone());
      current_level = next_level;
    }

    let root = current_level.first().copied().unwrap_or([0; 32]);

    MerkleTreeWrapper {
        inner: MerkleTree {
                root,
                nodes,
                leaves
            }
        }
  }

  #[napi]
  pub fn get_root(&self) -> Vec<u8> {
    self.inner.root.to_vec()
  }

  #[napi]
  pub fn get_nodes(&self) -> Vec<Vec<Vec<u8>>> {
    self
      .inner
      .nodes
      .par_iter()
      .map(|level| level.iter().map(|node| node.to_vec()).collect())
      .collect::<Vec<Vec<Vec<u8>>>>()
  }

  #[napi]
  pub fn get_leaves(&self) -> Vec<Vec<u8>> {
    self
      .inner
      .leaves
      .par_iter()
      .map(|leaf| leaf.to_vec())
      .collect::<Vec<Vec<u8>>>()
  }

  #[napi]
  pub fn generate_proof(&self, leaf_index: u32) -> Option<Vec<Vec<u8>>> {
    let leaf_index = leaf_index as usize;

    if leaf_index >= self.inner.leaves.len() {
      return None;
    }

    let mut proof = Vec::new();
    let mut index = leaf_index;

    for level in &self.inner.nodes {
      if level.len() == 1 {
        break;
      }

      let pair_index = if index % 2 == 0 { index + 1 } else { index - 1 };

      if pair_index < level.len() {
        proof.push(level[pair_index].to_vec());
      }

      index /= 2;
    }

    Some(proof)
  }

  #[napi]
  pub fn to_protobuf(&self) -> Vec<u8> {
    let mut tree = mini::merkle::MerkleTree::default();
    tree.leaves = self.inner.leaves.iter().map(|leaf| leaf.to_vec()).collect();
    tree.levels = self
      .inner
      .nodes
      .iter()
      .map(|level| {
        let mut level_msg = mini::merkle::Level::default();
        level_msg.nodes = level.iter().map(|node| node.to_vec()).collect();
        level_msg
      })
      .collect();
    tree.root = self.inner.root.to_vec();

    let mut buf = Vec::new();
    buf.reserve(tree.encoded_len());
    tree.encode(&mut buf).unwrap();
    buf
  }

  #[napi]
  pub fn from_protobuf(bytes: &[u8]) -> Self {
    let tree = mini::merkle::MerkleTree::decode(bytes).unwrap();

    let leaves = tree
      .leaves
      .iter()
      .map(|leaf| {
        let mut leaf_array = [0u8; 32];
        leaf_array.copy_from_slice(leaf);
        leaf_array
      })
      .collect();

    let nodes = tree
      .levels
      .iter()
      .map(|level| {
        level
          .nodes
          .iter()
          .map(|node| {
            let mut node_array = [0u8; 32];
            node_array.copy_from_slice(node);
            node_array
          })
          .collect()
      })
      .collect();

    let mut root = [0u8; 32];
    root.copy_from_slice(&tree.root);

    MerkleTreeWrapper {
            inner: MerkleTree {
                root,
                nodes,
                leaves
            }
        }
  }

  #[napi]
  /// build a "bytes32" equivalent padded leaf containing seven numbers
  pub fn numbers_to_leaf(numbers: Vec<u8>) -> Vec<u8> {
    numbers_to_encoded_bytes_leaf(&numbers).to_vec()
  }

  #[napi]
  /// convert a "bytes32" equivalent padded leaf back to original number array
  pub fn leaf_to_numbers(encoded_leaf: Vec<u8>) -> Vec<u8> {
    let mut array = [0u8; 32];
    array.copy_from_slice(&encoded_leaf[..32]);
    encoded_bytes_leaf_to_numbers(&array)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use ethers::utils::hex;

  #[test]
  fn generates_expected_proofs_and_root() {
    let first_ticket = numbers_to_encoded_bytes_leaf(&[1, 2, 3, 4, 5, 6, 7]);
    let second_ticket = numbers_to_encoded_bytes_leaf(&[11, 22, 33, 44, 55, 66, 77]);
    let third_ticket = numbers_to_encoded_bytes_leaf(&[70, 69, 40, 31, 19, 80, 80]);

    let deterministic_leaves = vec![first_ticket, second_ticket, third_ticket];

    let merkle_tree = MerkleTreeWrapper::new(
      deterministic_leaves
        .into_iter()
        .map(|a| a.to_vec())
        .collect(),
    );

    let expected_root = "0x91a79b79542123017284587e3c5de1a872908f07ea670fd59e6342f5a1f229ba";
    let root = merkle_tree.inner.root;
    assert_eq!(hex::encode(&root), &expected_root[2..]);

    let expected_proofs = vec![
      vec![
        "e909d30fd8e5d59c14be0814a22f2d8f06ee0ea08f746ac1249a6f4bda3937d4",
        "f8e133436e22a5e99566e9f1746022aff37d6010635111bffc4a94c555116833",
      ],
      vec![
        "51aa51f13aff8cefcb99341df4dce6c099d189a8e4b1dfc4d9b610e29e623fb8",
        "f8e133436e22a5e99566e9f1746022aff37d6010635111bffc4a94c555116833",
      ],
      vec!["b12598b450b90e998749bdf1631801103ad7566c0844ca874ef079b3963548de"],
    ];

    for (index, expected_proof) in expected_proofs.iter().enumerate() {
      if let Some(proof) = merkle_tree.generate_proof(index as u32) {
        let proof_hex: Vec<String> = proof.iter().map(|p| hex::encode(p)).collect();
        assert_eq!(proof_hex, *expected_proof);
      } else {
        panic!("No proof generated for leaf {}", index);
      }
    }
  }

  #[test]
  fn converts_numbers() {
    let first_ticket = numbers_to_encoded_bytes_leaf(&[1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(
      encoded_bytes_leaf_to_numbers(&first_ticket),
      vec![1, 2, 3, 4, 5, 6, 7]
    );
  }

  #[test]
  fn converts_protobuf() {
    let first_ticket = numbers_to_encoded_bytes_leaf(&[1, 2, 3, 4, 5, 6, 7]);
    let second_ticket = numbers_to_encoded_bytes_leaf(&[11, 22, 33, 44, 55, 66, 77]);
    let third_ticket = numbers_to_encoded_bytes_leaf(&[70, 69, 40, 31, 19, 80, 80]);

    let deterministic_leaves = vec![first_ticket, second_ticket, third_ticket];

    let merkle_tree = MerkleTreeWrapper::new(
      deterministic_leaves
        .into_iter()
        .map(|a| a.to_vec())
        .collect(),
    );

    let proto = merkle_tree.to_protobuf();
    let reconstituted_tree = MerkleTreeWrapper::from_protobuf(&proto);

    assert_eq!(merkle_tree.inner.root, reconstituted_tree.inner.root);
    assert_eq!(merkle_tree.inner.root, reconstituted_tree.inner.root);
  }
}
