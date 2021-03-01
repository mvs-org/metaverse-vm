## Steps Into MMR Proof

- **Target Chain: Ethereum**
- **Sampling Strategy: One By One**
- **Relay Block's Number: 3**
- **Game Id: 3**
- **Last Confirmed Block's Number on Hyperspace When Game(3) Started: 0**
- **The Proof Order is From Foot to Top, then Left to Right**

---

### POC Version

1. **Round 0, Target 4**
	```
	                  R3
	                /   \
	               c     f
	              / \   / \
	             a   b d   e
	Block Number 0   1 2   3

	This Proposal Say: I think the MMR Root is R3, and I prove it contains a(last confirmed block's MMR Hash which is block 0's)
	Gen Proof With: [a]
	```

1. **Round 1, Sample 3**
	```
	                  R3
	                /   \
	               c     f
	              / \   / \
	             a   b d   e
	Block Number 0   1 2   3

	This Extended Prove: R3(proposed root which is block 3's MMR root) contains d(current sample point block's MMR Hash which is block 2's)
	Gen Proof With: [d]
	```

1. **Round 2, Sample 2**
	```
	                  R3
	                /   \
	               c     f
	              / \   / \
	             a   b d   e
	Block Number 0   1 2   3

	This Extended Prove: R3(proposed root which is block 3's MMR root) contains b(current sample point block's MMR Hash which is block 1's)
	Gen Proof With: [b]
	```

1. **Reach Last Confirmed Block, Game Over**

### TODO

1. **Round 0, Target 4**
	```
	             | Global:             | Current:
	             |              R3     |               R3
	             |            /   \    |             /   \
	             |           c     f   |            c     f
	             |          / \   / \  |           / \   / \
	             |         a   b d   e |          a   b d   e
	Block Number |         0   1 2   3 |          0   1 2   3

	This Proposal Say: I think the MMR Root is R3, and I prove it contains a(last confirmed block's MMR Hash which is block 0's)
	Gen Proof With: [a]
	Proof: [b, f]
	             |     R3
	             |    / \
	             |   -   f
	             |  / \
	             | a   b
	Block Number | 0   1
	```

1. **Round 1, Sample 3**
	```
	             | Global:             | Current:
	             |              R3     |              R2
	             |            /   \    |             / \
	             |           c     f   |            c   \
	             |          / \   / \  |           / \   \
	             |         a   b d   e |          a   b   d
	Block Number |         0   1 2   3 |          0   1   2

	This Extended Prove: R3(previous MMR Root) contains d(current sample point block's MMR Hash which is block 2's)
	Gen Proof With: [d]
	Proof: [e, c]
	             |   R3
	             |  / \
	             | c   -
	             |    / \
	             |   d   e
	Block Number |   2   3
	```

1. **Round 2, Sample 2**
	```
	             | Global:             | Current:
	             |              R3     |
	             |            /   \    |
	             |           c     f   |            R1
	             |          / \   / \  |           / \
	             |         a   b d   e |          a   b
	Block Number |         0   1 2   3 |          0   1

	This Extended Prove: R2(previous MMR Root) contains b(current sample point block's MMR Hash which is block 1's)
	Gen Proof With: [b]
	Proof: [a, d]
	             |     R2
	             |    / \
	             |   -   \
	             |  / \   \
	             | a   b   d
	Block Number | 0   1   3
	```

1. **Reach Last Confirmed Block, Game Over**
