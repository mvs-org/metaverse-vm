const expect = require("chai").expect;
const assert = require("chai").assert;
const Web3 = require("web3");
const conf = require("./config.js");

const web3 = new Web3(conf.host);
const account = web3.eth.accounts.wallet.add(conf.privKey);
const jsontest = new web3.eth.Contract(conf.abi);
jsontest.options.from = conf.address;
jsontest.options.gas = conf.gas;

describe("Test Contract Log", function () {
	after(() => {
		web3.currentProvider.disconnect();
	});

	it("Deploy json test contract", async function () {
		const instance = await jsontest
			.deploy({
				data: conf.bytecode,
				arguments: [],
			})
			.send();
		jsontest.options.address = instance.options.address;
		conf.jsontestAddress = jsontest.options.address;
	}).timeout(10000);

	it("Get default bool value", async function () {
		const data = await jsontest.methods.getBool().call();
		expect(data).to.be.false;
	});

	it("Get storage at index 0 before change", async function () {
		const data = await web3.eth.getStorageAt(jsontest.options.address, 0);
		expect(data).to.be.equal(
			"0x0000000000000000000000000000000000000000000000000000000000000000"
		);
	});

	it("Set bool to true", async function () {
		const value = true;
		await jsontest.methods.setBool(value).send();
		const data = await jsontest.methods.getBool().call();
		expect(data).to.be.equal(value);
	}).timeout(80000);

	it("Get storage at index 0 after change", async function () {
		const data = await web3.eth.getStorageAt(jsontest.options.address, 0);
		expect(data).to.be.equal(
			"0x0000000000000000000000000000000000000000000000000000000000000001"
		);
	});

	it("Set Int8", async function () {
		const value = -11;
		await jsontest.methods.setInt8(value).send();
		const data = await jsontest.methods.getInt8().call();
		expect(+data).to.be.eq(value);
	}).timeout(80000);

	it("Set Uint8", async function () {
		const value = 11;
		await jsontest.methods.setUint8(value).send();
		const data = await jsontest.methods.getUint8().call();
		expect(+data).to.be.eq(value);
	}).timeout(80000);

	it("Set Int256", async function () {
		const value = -12;
		await jsontest.methods.setInt256(value).send();
		const data = await jsontest.methods.getInt256().call();
		expect(+data).to.be.eq(value);
	}).timeout(80000);

	it("Set Uint256", async function () {
		const value = 12;
		await jsontest.methods.setUint256(value).send();
		const data = await jsontest.methods.getUint256().call();
		expect(+data).to.be.eq(value);
	}).timeout(80000);

	it("Set Address", async function () {
		const value = "0xFFfFfFffFFfffFFfFFfFFFFFffFFFffffFfFFFfF";
		await jsontest.methods.setAddress(value).send();
		const data = await jsontest.methods.getAddress().call();
		expect(data).to.be.equal(value);
	}).timeout(80000);

	it("Set Bytes32", async function () {
		const value = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
		await jsontest.methods.setBytes32(value).send();
		const data = await jsontest.methods.getBytes32().call();
		expect(data).to.be.equal(value);
	}).timeout(80000);

	it("Fire event log0", function (done) {
		jsontest.methods.fireEventLog0().send();
		jsontest.once(
			"Log0",
			{
				fromBlock: 0,
			},
			function (error, event) {
				expect(event.raw.data).to.be.equal(
					"0x000000000000000000000000000000000000000000000000000000000000002a"
				);
				expect(event.signature).to.be.equal(
					"0x65c9ac8011e286e89d02a269890f41d67ca2cc597b2c76c7c69321ff492be580"
				);
				done();
			}
		);
	}).timeout(80000);

	it("Fire event Log0Anonym", function (done) {
		jsontest.methods.fireEventLog0Anonym().send();
		jsontest.once(
			"Log0Anonym",
			{
				fromBlock: 0,
			},
			function (error, event) {
				expect(event.raw.data).to.be.equal(
					"0x000000000000000000000000000000000000000000000000000000000000002a"
				);
				expect(event.signature).to.be.null;
				done();
			}
		);
	}).timeout(80000);

	it("Fire event Log1", function (done) {
		jsontest.methods.fireEventLog1().send();
		jsontest.once(
			"Log1",
			{
				fromBlock: 0,
			},
			function (error, event) {
				expect(event.raw.data).to.be.equal(
					"0x000000000000000000000000000000000000000000000000000000000000002a"
				);
				assert.isArray(event.raw.topics);
				expect(event.raw.topics.length).to.be.equal(2);
				expect(event.raw.topics[1]).to.be.equal(
					"0x0000000000000000000000000000000000000000000000000000000000000001"
				);
				expect(event.signature).to.be.equal(
					"0x81933b308056e7e85668661dcd102b1f22795b4431f9cf4625794f381c271c6b"
				);
				done();
			}
		);
	}).timeout(80000);

	it("Fire event Log1Anonym", function (done) {
		jsontest.methods.fireEventLog1Anonym().send();
		jsontest.once(
			"Log1Anonym",
			{
				fromBlock: 0,
			},
			function (error, event) {
				expect(event.raw.data).to.be.equal(
					"0x000000000000000000000000000000000000000000000000000000000000002a"
				);
				assert.isArray(event.raw.topics);
				expect(event.raw.topics.length).to.be.equal(1);
				expect(event.raw.topics[0]).to.be.equal(
					"0x0000000000000000000000000000000000000000000000000000000000000001"
				);
				expect(event.signature).to.be.null;
				done();
			}
		);
	}).timeout(80000);

	it("Fire event Log2", function (done) {
		jsontest.methods.fireEventLog2().send();
		jsontest.once(
			"Log2",
			{
				fromBlock: 0,
			},
			function (error, event) {
				expect(event.raw.data).to.be.equal(
					"0x000000000000000000000000000000000000000000000000000000000000002a"
				);
				assert.isArray(event.raw.topics);
				expect(event.raw.topics.length).to.be.equal(3);
				expect(event.raw.topics[1]).to.be.equal(
					"0x0000000000000000000000000000000000000000000000000000000000000001"
				);
				expect(event.raw.topics[2]).to.be.equal(
					"0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b"
				);
				expect(event.signature).to.be.equal(
					"0x0e216b62efbb97e751a2ce09f607048751720397ecfb9eef1e48a6644948985b"
				);
				done();
			}
		);
	}).timeout(80000);

	it("Fire event Log2Anonym", function (done) {
		jsontest.methods.fireEventLog2Anonym().send();
		jsontest.once(
			"Log2Anonym",
			{
				fromBlock: 0,
			},
			function (error, event) {
				expect(event.raw.data).to.be.equal(
					"0x000000000000000000000000000000000000000000000000000000000000002a"
				);
				assert.isArray(event.raw.topics);
				expect(event.raw.topics.length).to.be.equal(2);
				expect(event.raw.topics[0]).to.be.equal(
					"0x0000000000000000000000000000000000000000000000000000000000000001"
				);
				expect(event.raw.topics[1]).to.be.equal(
					"0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b"
				);
				expect(event.signature).to.be.null;
				done();
			}
		);
	}).timeout(80000);

	it("Fire event Log3", function (done) {
		jsontest.methods.fireEventLog3().send();
		jsontest.once(
			"Log3",
			{
				fromBlock: 0,
			},
			function (error, event) {
				expect(event.raw.data).to.be.equal(
					"0x000000000000000000000000000000000000000000000000000000000000002a"
				);
				assert.isArray(event.raw.topics);
				expect(event.raw.topics.length).to.be.equal(4);
				expect(event.raw.topics[1]).to.be.equal(
					"0x0000000000000000000000000000000000000000000000000000000000000001"
				);
				expect(event.raw.topics[2]).to.be.equal(
					"0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b"
				);
				expect(event.raw.topics[3]).to.be.equal(
					"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
				);
				expect(event.signature).to.be.equal(
					"0x317b31292193c2a4f561cc40a95ea0d97a2733f14af6d6d59522473e1f3ae65f"
				);
				done();
			}
		);
	}).timeout(80000);

	it("Fire event Log3Anonym", function (done) {
		jsontest.methods.fireEventLog3Anonym().send();
		jsontest.once(
			"Log3Anonym",
			{
				fromBlock: 0,
			},
			function (error, event) {
				expect(event.raw.data).to.be.equal(
					"0x000000000000000000000000000000000000000000000000000000000000002a"
				);
				assert.isArray(event.raw.topics);
				expect(event.raw.topics.length).to.be.equal(3);
				expect(event.raw.topics[0]).to.be.equal(
					"0x0000000000000000000000000000000000000000000000000000000000000001"
				);
				expect(event.raw.topics[1]).to.be.equal(
					"0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b"
				);
				expect(event.raw.topics[2]).to.be.equal(
					"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
				);
				expect(event.signature).to.be.null;
				done();
			}
		);
	}).timeout(80000);

	it("Fire event Log4", function (done) {
		jsontest.methods.fireEventLog4().send();
		jsontest.once(
			"Log4",
			{
				fromBlock: 0,
			},
			function (error, event) {
				expect(event.raw.data).to.be.equal(
					"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe9000000000000000000000000000000000000000000000000000000000000002a"
				);
				assert.isArray(event.raw.topics);
				expect(event.raw.topics.length).to.be.equal(4);
				expect(event.raw.topics[1]).to.be.equal(
					"0x0000000000000000000000000000000000000000000000000000000000000001"
				);
				expect(event.raw.topics[2]).to.be.equal(
					"0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b"
				);
				expect(event.raw.topics[3]).to.be.equal(
					"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
				);
				expect(event.signature).to.be.equal(
					"0xd5f0a30e4be0c6be577a71eceb7464245a796a7e6a55c0d971837b250de05f4e"
				);
				done();
			}
		);
	}).timeout(80000);

	it("Fire event Log3Anonym", function (done) {
		jsontest.methods.fireEventLog4Anonym().send();
		jsontest.once(
			"Log4Anonym",
			{
				fromBlock: 0,
			},
			function (error, event) {
				expect(event.raw.data).to.be.equal(
					"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe9000000000000000000000000000000000000000000000000000000000000002a"
				);
				assert.isArray(event.raw.topics);
				expect(event.raw.topics.length).to.be.equal(3);
				expect(event.raw.topics[0]).to.be.equal(
					"0x0000000000000000000000000000000000000000000000000000000000000001"
				);
				expect(event.raw.topics[1]).to.be.equal(
					"0x0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b"
				);
				expect(event.raw.topics[2]).to.be.equal(
					"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
				);
				expect(event.signature).to.be.null;
				done();
			}
		);
	}).timeout(80000);
});
