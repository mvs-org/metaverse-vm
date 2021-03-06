const expect = require("chai").expect;
const Web3 = require("web3");
const utils = require("./utils");

const web3 = new Web3("http://localhost:9933");

describe("Test Web3 API", function () {
	before(() => {
		utils.open();
	});

	after(() => {
		utils.close();
	});

	it.skip("should get client version", async function () {
		const version = await web3.eth.getNodeInfo();
		expect(version).to.be.equal("Hyperspace/v2.1/dvm-rpc-1.2.2");
	});

	it("should remote sha3", async function () {
		const data = web3.utils.stringToHex("hello");
		const local_hash = web3.utils.sha3("hello");

		const hash = await utils.customRequest("web3_sha3", [data]);
		expect(hash.result).to.be.equal(local_hash);
	});
});
