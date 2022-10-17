import { SigningCosmWasmClient, Secp256k1HdWallet, GasPrice } from "cosmwasm";

import * as fs from 'fs';

const nft_wasm = fs.readFileSync("../artifacts/nft.wasm");
// Update nft_contract_address with the address of the deployed contract
const nft_contract_address = "juno159mpaj2are86udh5s59wgjdg963xul8assrvr9js8m29xnc0t7jsmxrrrw"
const rpcEndpoint = "https://rpc.uni.junonetwork.io/";
const gas = GasPrice.fromString("0.025ujunox");

const mnemonic = "test peanut elevator motor proud globe obtain gasp sad balance nature ladder"
const wallet_address = "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4"

describe("Soulband + Mutable NFT Tests", () => {

    xit("Upload code to testnet", async () => {
        const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: 'juno' })
        const client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet, { 
            gasPrice: gas
        })
        const resNftContractUpload = await client.upload(wallet_address, nft_wasm, "auto");
        console.log("NFT Contract: " + JSON.stringify(resNftContractUpload.logs[0].events));
    }).timeout(50000);

    xit("Instantiate code on testnet", async() => {
        const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: 'juno' })
        const client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet, {
            gasPrice: gas
        })
        const resNftContractInstantiation = await client.instantiate(wallet_address, 805, {minter:wallet_address, name: "TestNfts", symbol: "TEST" }, "nft-nt-mutable", "auto", {admin: wallet_address});
        
        // NFT Contract Address: juno10788h33szv07c7vt9na7jfd8c8r87gz3peu8ycdcdnwa442p8tesqy0n2c
        console.log("NFT Contract: " + JSON.stringify(resNftContractInstantiation));

    }).timeout(40000);

    it("Execute a mint on testnet", async() => {
        const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: 'juno' })
        const client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet, {
            gasPrice: gas
        })
        const res = await client.execute(wallet_address, nft_contract_address,
            { 
                mint: { 
                    token_id: "Cadet #2",
                    owner: wallet_address,
                    token_uri: "https://starships.example.com/Starship/Enterprise.json",
                    extension: {
                        name: "test1",
                        image: "path/to/test/image.png",
                        cohort: "fall-2022",
                        description: "Sample description",
                        attributes: null,
                        badges: ["cosmwasm", "move"],
                        skills: ["javascript", "rust"],
                        github_url: "github.com/test",
                        is_for_hire: true,
                    }
                } 
            }, "auto");
        console.log("Mint response: ", JSON.stringify(res))
        const query = await client.queryContractSmart(nft_contract_address, { nft_info: { token_id: "Cadet #1" } })
        console.log("NFT Info: ", JSON.stringify(query))
    }).timeout(40000);

    xit("Update NFT metadata on testnet", async() => {
        const wallet = await Secp256k1HdWallet.fromMnemonic(mnemonic, { prefix: 'juno' })
        const client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, wallet, {
            gasPrice: gas
        })
        const resUpdateMetadata = await client.execute(wallet_address, nft_contract_address,
        {
            update_metadata: { token_id: "1", token_uri: "https://starships.example.com/Starship/Enterprise.json", 
            metadata: {
                name: "test1",
                image: "path/to/test/image.png",
                cohort: "2021",
                description: "Sample description",
                attributes: null,
                badges: null,
                skills: ["javascript", "rust"],
                github_url: "github.com/test",
                is_for_hire: true,
            }}
        }, "auto");
        console.log("Update Metadata response: ", JSON.stringify(resUpdateMetadata))
        const query = await client.queryContractSmart(nft_contract_address, { nft_info: { token_id: "1" } })
        console.log("NFT Info: ", JSON.stringify(query))
    }).timeout(40000);
});