import * as anchor from "@project-serum/anchor";
import { utils, BN } from "@project-serum/anchor";
import {PublicKey} from "@solana/web3.js";
import { Program } from "@project-serum/anchor";
import * as token from "@solana/spl-token"
import { NftStakeAuth } from "../target/types/nft_stake_auth";

//constants
const collectionAddress = new PublicKey("AyRhD1Yh8MAdZAhQL8eK1FZcogg1GW4Y87HqsWJytTzo"); // Mint Address of the Collection NFT for which the staking to be activated
const tokenMint = new PublicKey("5Nyxoz6SUavnfu4vTYzvP9hVndTez1YEBW9zvbBV3zuS"); // Mint of the Token to be given as reward
const tokenAccount = new PublicKey("HMAA7vvmESq6qzE6iuhAfPrr6AefHHBJs7KvfsoMSkye"); // Token account for the reward token

// NFT of the collection - must be owned by the Signer
const nftMint = new PublicKey("DskQgewLBTmPBZwWAZ5U7swcPeggpZ5eRbb6gurY1oZd");
const nftToken = new PublicKey("H8nooeBKDQTcp75zZoyBuMHiWjsWPWH42q3dWJn8CA3a");
const nftMetadata = new PublicKey("9naYoZ4uZxCPPvQPnQwzmFFnFKwrLiUNLf7sQnqWr4RN")
const nftEdition = new PublicKey("FkHtoHUk6kWehv6VrH1mdbgfBM7xxhmcpEubZ8cz3quq");

// NFT from a different collection
const nftMint2 = new PublicKey("N7aCLcbFrFi17J2DfW9G9FoBHjJXZr9hSy8HSP9wzPL");
const nftToken2 = new PublicKey("HyUXydfnfu4kJABGE63cy9EeyfDqqqgKs6wuo6FkY1PL");
const nftMetadata2 = new PublicKey("C5TEagbhLjyhhVeWcCGJBFHwZt6TSWZaoXxK3q2Tpwg1");
const nftEdition2 = new PublicKey("D9XovNCb3Jgt5t4akUDdWnRPovRaZr2uxwJdzhNhDxU8");

// Configure the client to use the local cluster.
anchor.setProvider(anchor.AnchorProvider.env());

const program = anchor.workspace.NftStakeAuth as Program<NftStakeAuth>;
const programId = program.idl.metadata.address;

// PDAs
const [stakeDetails] = PublicKey.findProgramAddressSync([
    utils.bytes.utf8.encode("stake"),
    collectionAddress.toBytes(),
    program.provider.publicKey.toBytes()
], programId);

const [tokenAuthority] = PublicKey.findProgramAddressSync([
    utils.bytes.utf8.encode("token-authority"),
    stakeDetails.toBytes()
], programId);

const [nftAuthority] = PublicKey.findProgramAddressSync([
    utils.bytes.utf8.encode("nft-authority"),
    stakeDetails.toBytes()
], programId);

const [nftRecord] = PublicKey.findProgramAddressSync([
    utils.bytes.utf8.encode("nft-record"),
    stakeDetails.toBytes(),
    nftMint.toBytes()
], programId);

const [nftRecord2] = PublicKey.findProgramAddressSync([
    utils.bytes.utf8.encode("nft-record"),
    stakeDetails.toBytes(),
    nftMint2.toBytes()
], programId);

const nftCustody = token.getAssociatedTokenAddressSync(nftMint, nftAuthority, true);

describe("nft-stake-auth", () => {
  it("initializes staking", async() => {
    const minimumPeriod = new BN(0);
    const reward = new BN(100);

    const tx = await program.methods.initStaking(
      reward,
      minimumPeriod
    )
    .accounts({
      stakeDetails,
      tokenMint,
      tokenAuthority,
      collectionAddress,
      nftAuthority
    })
    .rpc();

    console.log("TX: ", tx);

    let stakeAccount = await program.account.details.fetch(stakeDetails);
    console.log(stakeAccount);
  });

  it("stakes NFT", async() => {
    const tx = await program.methods.stake()
    .accounts({
      stakeDetails,
      nftRecord,
      nftMint,
      nftToken,
      nftMetadata,
      nftAuthority,
      nftEdition,
      nftCustody,
    })
    .rpc()

    console.log("TX: ", tx);

    let stakeAccount = await program.account.details.fetch(stakeDetails);
    let nftRecordAccount = await program.account.nftRecord.fetch(nftRecord);

    console.log("Stake Details: ", stakeAccount);
    console.log("NFT Record: ", nftRecordAccount);
  });

  it("stakes NFT from different collection and fails", async() => {
    try {
      const tx = await program.methods.stake()
      .accounts({
        stakeDetails,
        nftRecord: nftRecord2,
        nftMint: nftMint2,
        nftToken: nftToken2,
        nftMetadata: nftMetadata2,
        nftEdition: nftEdition2,
        nftAuthority,
        nftCustody,
      })
      .rpc()
    } catch(e) {
        console.log(e)
    }
  });

  it("claims rewards without unstaking", async() => {
    let nftRecordAccount = await program.account.nftRecord.fetch(nftRecord);
    console.log("NFT Staked at: ", nftRecordAccount.stakedAt.toNumber());

    const tx = await program.methods.withdrawReward()
    .accounts({
      stakeDetails,
      nftRecord,
      rewardMint: tokenMint,
      rewardReceiveAccount: tokenAccount,
      tokenAuthority            
    })
    .rpc()

    console.log("TX: ", tx);


    nftRecordAccount = await program.account.nftRecord.fetch(nftRecord);
    console.log("NFT Staked at: ", nftRecordAccount.stakedAt.toNumber());
  });

  it("claims rewards and unstakes", async() => {
    let nftRecordAccount = await program.account.nftRecord.fetch(nftRecord);
    console.log("NFT Staked at: ", nftRecordAccount.stakedAt.toNumber());

    const tx = await program.methods.unstake()
    .accounts({
      stakeDetails,
      nftRecord,
      rewardMint: tokenMint,
      rewardReceiveAccount: tokenAccount,
      tokenAuthority,
      nftAuthority,
      nftCustody,
      nftMint,
      nftReceiveAccount: nftToken         
    })
    .rpc()

    console.log("TX: ", tx);
  });

  it("closes staking", async() => {
    const tx = await program.methods.closeStaking()
    .accounts({
      stakeDetails,
      tokenMint,
      tokenAuthority       
    })
    .rpc()

    console.log("TX: ", tx);
  });

});
