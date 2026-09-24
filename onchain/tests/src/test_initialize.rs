use anchor_client::{
    CommitmentConfig,
    Client, Cluster,
};
use solana_keypair::read_keypair_file;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

#[test]
fn test_initialize() {
    let program_id = "EDzVvstsabPPHYx6QLgw1J8ZRFhJrGv2fiz2a6o9Kym9";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::try_from(program_id).unwrap();
    let program = client.program(program_id).unwrap();
    let counter = Pubkey::find_program_address(
        &[zk_pool::constants::COUNTER_SEED],
        &program_id,
    )
    .0;

    let initialize_tx = program
        .request()
        .accounts(zk_pool::accounts::Initialize {
            payer: payer.pubkey(),
            counter,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(zk_pool::instruction::Initialize {})
        .send()
        .expect("");

    println!("Initialize transaction signature {}", initialize_tx);

    let increment_tx = program
        .request()
        .accounts(zk_pool::accounts::Increment {
            counter,
            authority: payer.pubkey(),
        })
        .args(zk_pool::instruction::Increment {})
        .send()
        .expect("");

    println!("Increment transaction signature {}", increment_tx);
}
