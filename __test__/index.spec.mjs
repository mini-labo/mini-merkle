import test from 'ava'

import { MerkleTree } from '../index.js'

test('builds expected root', (t) => {
  const expectedRootHash = '0x91a79b79542123017284587e3c5de1a872908f07ea670fd59e6342f5a1f229ba'

  const firstTicket = MerkleTree.numbersToLeaf([1, 2, 3, 4, 5, 6, 7]);
  const secondTicket = MerkleTree.numbersToLeaf([11, 22, 33, 44, 55, 66, 77]);
  const thirdTicket = MerkleTree.numbersToLeaf([70, 69, 40, 31, 19, 80, 80]);
  const leaves = [firstTicket, secondTicket, thirdTicket];

  const root = new MerkleTree(leaves).root();

  t.is(byteArrayToHexString(root), expectedRootHash);
});

test('builds expected proofs', (t) => {
  const expectedProofHashes = [
    '0xe909d30fd8e5d59c14be0814a22f2d8f06ee0ea08f746ac1249a6f4bda3937d4',
    '0xf8e133436e22a5e99566e9f1746022aff37d6010635111bffc4a94c555116833'
  ]

  const firstTicket = MerkleTree.numbersToLeaf([1, 2, 3, 4, 5, 6, 7]);
  const secondTicket = MerkleTree.numbersToLeaf([11, 22, 33, 44, 55, 66, 77]);
  const thirdTicket = MerkleTree.numbersToLeaf([70, 69, 40, 31, 19, 80, 80]);
  const leaves = [firstTicket, secondTicket, thirdTicket];

  const tree = new MerkleTree(leaves);
  // index 1
  const proofs = tree.generateProof(0);
  t.not(proofs, null);
  for (let i = 0; i < proofs.length; i++) {
      t.is(byteArrayToHexString(proofs[i]), expectedProofHashes[i]); 
  }
});

function byteArrayToHexString(bytes) {
  return '0x' + Array.from(bytes, byte => {
    return ('0' + (byte & 0xFF).toString(16)).slice(-2);
  }).join('')
}
