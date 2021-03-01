const expect = require("chai").expect;
const Web3 = require("web3");
const conf = require("./config.js");
const web3 = new Web3(conf.host);

describe("Test Net API", function () {
	after(() => {
		web3.currentProvider.disconnect();
	});

	it("should get current network ID", async function () {
		expect(await web3.eth.net.getId()).to.be.equal(43);
	});

	it("should check if the node is listening for peer", async function () {
		expect(await web3.eth.net.isListening()).to.be.equal(true);
	});

	it("should get the number of peers connected to", async function () {
		expect(await web3.eth.net.getPeerCount()).to.be.equal(0);
	});
});
