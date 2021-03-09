const expect = require("chai").expect;
const Web3 = require("web3");

const web3 = new Web3("http://localhost:9933");

describe("Test RPC Constants", function () {
	it("Should have 0 hashrate", async function () {
		expect(await web3.eth.getHashrate()).to.equal(0);
	});

	it("should have chainId 23", async function () {
		expect(await web3.eth.getChainId()).to.equal(43);
	});

	it("should have no account", async function () {
		expect(await web3.eth.getAccounts()).to.eql([]);
	});

	it("block author should be 0x0000000000000000000000000000000000000000", async function () {
		expect(await web3.eth.getCoinbase()).to.equal("0x0000000000000000000000000000000000000000");
	});

	it("should gas price is 0x0", async function () {
		expect(await web3.eth.getGasPrice()).to.equal("1");
	});

	it("should protocal version is 1", async function () {
		expect(await web3.eth.getProtocolVersion()).to.equal(1);
	});

	it("should is syncing is false", async function () {
		expect(await web3.eth.isSyncing()).to.be.false;
	});
});
