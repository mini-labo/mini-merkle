import crypto from 'crypto';
import { MerkleTreeWrapper as MerkleTree } from './index.js';
import { assert } from 'console';

const NUMBER_OF_LEAVES = 100_000;

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
console.log(`generated tree with root: ${byteArrayToHexString(tree.getRoot())}`);
console.timeEnd('native (.rs) tree generation');

console.time('protobuf serialize');
const buf = tree.toProtobuf();
console.timeEnd('protobuf serialize');

console.time('naive json (raw obj)');
const naiveJson = JSON.stringify(tree);
console.log(naiveJson.length);
console.timeEnd('naive json (raw obj)');

console.time('fetch data for js');
const javascriptTree = {
  root: tree.getRoot(),
  nodes: tree.getNodes(),
  leaves: tree.getLeaves()
}
console.timeEnd('fetch data for js');

console.time('fetch only leaves for js');
const leaves = tree.getLeaves();
console.timeEnd('fetch only leaves for js');
console.time('serialize leaves');
const jleaves = JSON.stringify(leaves);
console.log(jleaves.length);
console.timeEnd('serialize leaves');

console.time('json serialize');
const json = JSON.stringify(javascriptTree);
console.log(json.length);
console.timeEnd('json serialize');
//

console.time('protobuf deserialize');
const reconstitutedTree = MerkleTree.fromProtobuf(new Uint8Array(buf));
console.timeEnd('protobuf deserialize');
console.log(`original root: ${byteArrayToHexString(tree.getRoot())}`)
console.log(`reconstituted root: ${byteArrayToHexString(reconstitutedTree.getRoot())}`)

console.time('rebuilding from leaf data only TEST');
// javascript leaf data retrieved from getter
const newTree = new MerkleTree(leaves);
console.log('new tree root: ', byteArrayToHexString(newTree.getRoot()));
console.timeEnd('rebuilding from leaf data only TEST');

assert(byteArrayToHexString(tree.getRoot()) === byteArrayToHexString(reconstitutedTree.getRoot()));

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
