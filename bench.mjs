import crypto from 'crypto';
import { MerkleTree } from './index.js';
import { assert } from 'console';

const NUMBER_OF_LEAVES = 500_000;

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

console.time('protobuf serialize');
const buf = tree.toProtobuf();
console.timeEnd('protobuf serialize');

console.time('protobuf deserialize');
const reconstitutedTree = MerkleTree.fromProtobuf(new Uint8Array(buf));
console.timeEnd('protobuf deserialize');
assert(tree.root() === reconstitutedTree.root());

console.log('generating 100,000 sets of 7 random numbers (node)')
console.time('node random number generation');
const nodeNumbers = generateCollection(100_000, 7);
console.timeEnd('node random number generation');
console.log(`generated ${nodeNumbers.length} sets of numbers`);



function byteArrayToHexString(bytes) {
  return '0x' + Array.from(bytes, byte => {
    return ('0' + (byte & 0xFF).toString(16)).slice(-2);
  }).join('')
}

function generateRandomArray(size) {
    const arr = new Array(size);
    for (let i = 0; i < size; i++) {
        arr[i] = crypto.randomInt(1, 80);
    }
    return arr;
}

function generateCollection(totalSize, arraySize) {
    const collection = [];
    for (let i = 0; i < totalSize; i++) {
        collection.push(generateRandomArray(arraySize));
    }
    return collection;
}
