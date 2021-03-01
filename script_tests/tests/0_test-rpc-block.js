const expect = require("chai").expect;
const Web3 = require("web3");

const web3 = new Web3("http://localhost:9933");

describe("Test Block RPC", function () {
	it("The block number should not be zero", async function () {
		expect(await web3.eth.getBlockNumber()).to.not.equal(0);
	});

	it("Should return the genesis block", async function () {
		const block = await web3.eth.getBlock(0);
		expect(block).to.include({
			author: "0x0000000000000000000000000000000000000000",
			difficulty: "0",
			extraData: "0x",
			gasLimit: 4294967295,
			gasUsed: 0,
			logsBloom:
				"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
			miner: "0x0000000000000000000000000000000000000000",
			number: 0,
			receiptsRoot: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
			size: 505,
			timestamp: 0,
			totalDifficulty: null,
			transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
		});

		expect(block.transactions).to.be.a("array").empty;
		expect(block.uncles).to.be.a("array").empty;
		expect(block.sealFields).to.eql([
			"0x0000000000000000000000000000000000000000000000000000000000000000",
			"0x0000000000000000",
		]);
		expect(block.hash).to.be.a("string").lengthOf(66);
		expect(block.parentHash).to.be.a("string").lengthOf(66);
		expect(block.timestamp).to.be.a("number");
	});

	it("should have empty uncles and correct sha3Uncles", async function () {
		const block = await web3.eth.getBlock(0);
		expect(block.uncles).to.be.a("array").empty;
		expect(block.sha3Uncles).to.equal(
			"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"
		);
	});

	it("should have empty transactions and correct transactionRoot", async function () {
		const block = await web3.eth.getBlock(0);
		expect(block.transactions).to.be.a("array").empty;
		expect(block).to.include({
			transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
		});
	});

	it("get block by hash", async function () {
		const latest_block = await web3.eth.getBlock("latest");
		const block = await web3.eth.getBlock(latest_block.hash);
		expect(block.hash).to.be.eq(latest_block.hash);
	});

	it("get block by number", async function () {
		const block = await web3.eth.getBlock(3);
		expect(block.number).not.null;
	});

	it("should include previous block hash as parent", async function () {
		const block = await web3.eth.getBlock("latest");

		// previous block
		const previous_block_number = block.number - 1;
		const previous_block = await web3.eth.getBlock(previous_block_number);

		expect(block.hash).to.not.equal(previous_block.hash);
		expect(block.parentHash).to.equal(previous_block.hash);
	});

	it("should have valid timestamp after block production", async function () {
		const block = await web3.eth.getBlock("latest");

		// previous block
		const previous_block_number = block.number - 1;
		const previous_block = await web3.eth.getBlock(previous_block_number);

		expect(block.timestamp - previous_block.timestamp).to.be.eq(6);
	});

	it("should get transactions count by block number ", async function () {
		expect(await web3.eth.getBlockTransactionCount(0)).to.equal(0);
	});

	it("should get transactions count by earliest block", async function () {
		expect(await web3.eth.getBlockTransactionCount("earliest")).to.equal(0);
	});

	it("should get transactions count by latest block", async function () {
		expect(await web3.eth.getBlockTransactionCount("latest")).to.equal(0);
	});

	it("should get transactions count by pending block", async function () {
		expect(await web3.eth.getBlockTransactionCount("pending")).to.equal(null);
	});

	it("should return null if the block doesnt exist", async function () {
		expect(
			await web3.eth.getBlockTransactionCount(
				"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
			)
		).to.null;
	});

	it("should return null when no uncle was found", async function () {
		expect(await web3.eth.getUncle(0, 0)).to.be.null;
	});
});
