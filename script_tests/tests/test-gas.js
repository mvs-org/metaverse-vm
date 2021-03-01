const expect = require("chai").expect;
const Web3 = require("web3");
const web3 = new Web3("http://localhost:9933");

const addressFrom = "0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b";
// substrate: '5ELRpquT7C3mWtjeqFMYqgNbcNgWKSr3mYtVi1Uvtc2R7YEx';
const privKey = "99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

describe("Test gas", function () {
	// Solidity: contract test { function multiply(uint a) public pure returns(uint d) {return a * 7;}}
	const TEST_CONTRACT_BYTECODE =
		"0x6080604052348015600f57600080fd5b5060ae8061001e6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063c6888fa114602d575b600080fd5b605660048036036020811015604157600080fd5b8101908080359060200190929190505050606c565b6040518082815260200191505060405180910390f35b600060078202905091905056fea265627a7a72315820f06085b229f27f9ad48b2ff3dd9714350c1698a37853a30136fa6c5a7762af7364736f6c63430005110032";
	const FIRST_CONTRACT_ADDRESS = "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a";

	const TEST_CONTRACT_ABI = {
		constant: true,
		inputs: [{ internalType: "uint256", name: "a", type: "uint256" }],
		name: "multiply",
		outputs: [{ internalType: "uint256", name: "d", type: "uint256" }],
		payable: false,
		stateMutability: "pure",
		type: "function",
	};

	it("eth_estimateGas for contract creation", async function () {
		expect(
			await web3.eth.estimateGas({
				from: addressFrom,
				data: TEST_CONTRACT_BYTECODE,
			})
		).to.equal(91019);
	});

	it("eth_estimateGas for contract call", async function () {
		const contract = new web3.eth.Contract([TEST_CONTRACT_ABI], FIRST_CONTRACT_ADDRESS, {
			from: addressFrom,
			gasPrice: "0x01",
		});

		expect(await contract.methods.multiply(3).estimateGas()).to.equal(21204);
	});
});
