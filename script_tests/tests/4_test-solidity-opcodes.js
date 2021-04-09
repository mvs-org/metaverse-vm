const expect = require("chai").expect;
const assert = require("chai").assert;
const Web3 = require("web3");
const conf = require("./config.js");
const opcode = require("./bytecode.js");
const web3 = new Web3(conf.host);
const account = web3.eth.accounts.wallet.add(conf.privKey);
const opcodes = new web3.eth.Contract(opcode.abi);
opcodes.options.from = conf.address;
opcodes.options.gas = conf.gas;

describe("Test Solidity OpCodes", function () {
	after(() => {
		web3.currentProvider.disconnect();
	});

	it("Should run without errors the majorit of opcodes", async () => {
		const instance = await opcodes
			.deploy({
				data: opcode.bytecode,
				arguments: [],
			})
			.send();
		opcodes.options.address = instance.options.address;
		await opcodes.methods.test().send();
		await opcodes.methods.test_stop().send();
	}).timeout(120000);

	it("Should throw invalid op code", async () => {
		try {
			await opcodes.methods.test_invalid().send();
		} catch (error) {
			expect(error.receipt.status).to.be.false;
		}
	}).timeout(120000);

	it("Should revert", async () => {
		try {
			await opcodes.methods.test_revert().send();
		} catch (error) {
			expect(error.receipt.status).to.be.false;
		}
	}).timeout(120000);
});
