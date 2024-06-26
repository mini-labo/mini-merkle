/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export class MerkleTree {
  constructor(leaves: Array<Array<number>>)
  root(): Array<number>
  nodes(): Array<Array<Array<number>>>
  leaves(): Array<Array<number>>
  generateProof(leafIndex: number): Array<Array<number>> | null
  /** build a "bytes32" equivalent padded leaf containing seven numbers */
  static numbersToLeaf(numbers: Array<number>): Array<number>
  /** convert a "bytes32" equivalent padded leaf back to original number array */
  static leafToNumbers(encodedLeaf: Array<number>): Array<number>
}
