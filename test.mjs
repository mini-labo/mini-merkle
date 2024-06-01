import { MerkleTree } from './index.js';

const NUMBER_OF_LEAVES = 10_000;

console.log(`generating merkle tree with ${NUMBER_OF_LEAVES} leaves of 7 random numbers`);

console.time('native (.rs) leaf generation')
const nativeLeaves = Array.from({ length: NUMBER_OF_LEAVES })
    .fill(0)
    .map(() => MerkleTree.numbersToLeaf(
        Array.from({ length: 7 }, () => { return Math.floor(Math.random() * 80) + 1})
    ))
console.timeEnd('native (.rs) leaf generation');

console.time('native (.rs) tree generation');
const tree =  new MerkleTree(nativeLeaves);
console.log(`generated tree with root: ${byteArrayToHexString(tree.root())}`);
console.timeEnd('native (.rs) tree generation');

function byteArrayToHexString(bytes) {
  return '0x' + Array.from(bytes, byte => {
    return ('0' + (byte & 0xFF).toString(16)).slice(-2);
  }).join('')
}
