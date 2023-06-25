import { Keypair, Connection, Commitment, PublicKey, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { createCreateMetadataAccountV3Instruction } from "@metaplex-foundation/mpl-token-metadata";
import {Metaplex} from "@metaplex-foundation/js";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from '@solana/spl-token';
import wallet from "./wallet.json";

// Import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

// Create devnet connection instance
const commitment: Commitment = "processed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

// Create Metaplex instance
const metaplex = Metaplex.make(connection);

(async () => {

    try {
        // Create mint account for the reward token
        const mintAccount = await createMint(
            connection,
            keypair, // rent payer for this account
            keypair.publicKey, // mint authority
            keypair.publicKey, // freeze authority
            9, // decimals
        );

        console.log("Mint Account Created: ", mintAccount.toBase58());

        // Get Metadata PDA for the reward mint
        const metadata = metaplex.nfts().pdas().metadata({mint: mintAccount});
        
        // Build Transaction to create Metadata account
        const tx = new Transaction().add(
            createCreateMetadataAccountV3Instruction({
                metadata: metadata,
                mint: mintAccount, 
                mintAuthority: keypair.publicKey,
                payer: keypair.publicKey,
                updateAuthority: keypair.publicKey
            }, {
                createMetadataAccountArgsV3: {
                    data: {
                        name: "Reward Token",
                        symbol: "RWT",
                        uri: "",
                        sellerFeeBasisPoints: 0,
                        collection: null,
                        creators: null,
                        uses: null
                    },
                    isMutable: true,
                    collectionDetails: null
                }
            })
        );
        
        // Send transactio on-chain
        await sendAndConfirmTransaction(connection, tx, [keypair]);
        console.log("Metadata Created: ", metadata.toBase58());

        // Create Token Account for the owner
        const tokenAccount = await getOrCreateAssociatedTokenAccount(
            connection,
            keypair, // rent payer for this account
            mintAccount, // mint of the token
            keypair.publicKey, // owner of this token account
        );
        
        console.log("Token Account Created: ", tokenAccount.address.toBase58());

        // Mint 100 Tokens
        let decimals = 1_000_000_000;
        const sig = await mintTo(connection, keypair, mintAccount, tokenAccount.address, keypair, 100 * decimals)

        console.log("Token Minted", sig);
    } catch(error) {
        console.log(`Oops, something went wrong: ${error}`)
    }
})()