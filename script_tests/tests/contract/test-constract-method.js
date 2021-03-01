// Run the transfer balance firstly.
// require("../test-transfer.js");

const expect = require("chai").expect;
const Web3 = require("web3");
const contractFile = require("./compile");

const web3 = new Web3("http://localhost:9933");
const address = "0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b";
const privKey = "99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342"; // Genesis private key
const bytecode = contractFile.evm.bytecode.object;
const abi = contractFile.abi;
const incrementer = new web3.eth.Contract(abi);

describe("Test Contract", function () {
	let create_contract;
	let reset_tx_hash;

	it("Deploy contract", async function () {
		const incrementerTx = incrementer.deploy({
			data: bytecode,
			arguments: [5],
		});

		const createTransaction = await web3.eth.accounts.signTransaction(
			{
				from: address,
				data: incrementerTx.encodeABI(),
				gas: "4294967295",
			},
			privKey
		);

		const createReceipt = await web3.eth.sendSignedTransaction(
			createTransaction.rawTransaction
		);
		create_contract = createReceipt.contractAddress;
	}).timeout(10000);

	it("Get Default Number", function () {
		const get = async () => {
			const data = await incrementer.methods.number().call();

			expect(data).to.be.equal(0);
		};
	});

	it("Get Code", async function () {
		const code = await web3.eth.getCode(create_contract);
		expect(code).equal(
			"0x6080604052348015600f57600080fd5b5060043610603c5760003560e01c80637cf5dab01460415780638381f58a14606c578063d826f88f146088575b600080fd5b606a60048036036020811015605557600080fd5b81019080803590602001909291905050506090565b005b6072609e565b6040518082815260200191505060405180910390f35b608e60a4565b005b806000540160008190555050565b60005481565b6000808190555056fea26469706673582212208b50e17311516ec7038035cfdf617e058505b8946b015558a8d79eca7339310f64736f6c634300060a0033"
		);
	});

	it("Increase Number", async function () {
		const value = 3;
		const encoded = incrementer.methods.increment(value).encodeABI();

		const createTransaction = await web3.eth.accounts.signTransaction(
			{
				from: address,
				to: create_contract,
				data: encoded,
				gas: "4294967295",
			},
			privKey
		);

		const createReceipt = await web3.eth.sendSignedTransaction(
			createTransaction.rawTransaction
		);
		const get = async () => {
			const data = await incrementer.methods.number().call();

			expect(data).to.be.equal(value);
		};
	}).timeout(80000);

	it("Reset Number", async function () {
		const encoded = incrementer.methods.reset().encodeABI();

		const createTransaction = await web3.eth.accounts.signTransaction(
			{
				from: address,
				to: create_contract,
				data: encoded,
				gas: "4294967295",
			},
			privKey
		);

		const createReceipt = await web3.eth.sendSignedTransaction(
			createTransaction.rawTransaction
		);

		// Save tx hash
		reset_tx_hash = createReceipt.transactionHash;

		const get = async () => {
			const data = await incrementer.methods.number().call();

			expect(data).to.be.equal(0);
		};
	}).timeout(80000);

	it("Get Transaction By Hash", async function () {
		const tx = await web3.eth.getTransaction(reset_tx_hash);
		expect(tx.hash).to.equal(reset_tx_hash);
	});
});
