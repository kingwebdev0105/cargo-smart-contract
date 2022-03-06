use clap::{crate_description, crate_name, crate_version, value_t_or_exit, App, Arg, SubCommand};
use insurance_contract::state::{InsuranceContractData, INSURANCE_CONTRACT_DATA_LEN};
use solana_clap_utils::{
    fee_payer::fee_payer_arg,
    input_validators::{is_url_or_moniker, is_valid_pubkey, normalize_to_url_if_moniker},
};
use solana_client::rpc_client::RpcClient;
use solana_program::borsh::try_from_slice_unchecked;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

// Helper functions
fn get_clap_app<'a, 'b>(name: &'a str, desc: &'a str, version: &'a str) -> App<'a, 'b> {
    App::new(name)
        .about(desc)
        .version(version)
        .arg(fee_payer_arg().global(true))
        .arg(
            Arg::with_name("json_rpc_url")
                .short("u")
                .long("url")
                .value_name("URL_OR_MONIKER")
                .takes_value(true)
                .global(true)
                .validator(is_url_or_moniker)
                .help(
                    "URL for Solana's JSON RPC or moniker (or their first letter): \
                       [mainnet-beta, testnet, devnet, localhost] \
                    Default is devnet",
                ),
        )
        .subcommand(
            SubCommand::with_name("save")
                .about("Creates on-chain account stored the InsuranceContract identifier")
                .arg(
                    Arg::with_name("insurance_contract_id")
                        .validator(is_valid_id)
                        .value_name("u32")
                        .takes_value(true)
                        .help("Insurance contract ID"),
                ),
        )
        .subcommand(
            SubCommand::with_name("close")
                .about("Set up is_closed status on InsuranceContract account")
                .arg(
                    Arg::with_name("address")
                        .value_name("PUBKEY")
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .help("Insurance contract data account"),
                ),
        )
        .subcommand(
            SubCommand::with_name("show")
                .about("Show InsuranceContract account data")
                .arg(
                    Arg::with_name("address")
                        .value_name("PUBKEY")
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .help("Insurance contract data account"),
                ),
        )
}

fn is_valid_id(string: String) -> Result<(), String> {
    match string.parse::<u32>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Invalid id {}", string)),
    }
}

// CLI commands handlers
fn save(client: &RpcClient, payer: &Keypair, id: u32, data_address: &Keypair) {
    let mut instructions = Vec::<Instruction>::with_capacity(4);
    instructions.append(&mut vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &data_address.pubkey(),
            client
                .get_minimum_balance_for_rent_exemption(INSURANCE_CONTRACT_DATA_LEN)
                .unwrap(),
            INSURANCE_CONTRACT_DATA_LEN as u64,
            &insurance_contract::id(),
        ),
        insurance_contract::instruction::save_insurance_contract(
            &insurance_contract::id(),
            &payer.pubkey(),
            &data_address.pubkey(),
            id,
        )
        .unwrap(),
    ]);

    let recent_blockhash = client.get_recent_blockhash().unwrap().0;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer, data_address],
        recent_blockhash,
    );
    client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .unwrap();
}

fn close(client: &RpcClient, payer: &Keypair, data_address: &Pubkey) {
    let recent_blockhash = client.get_recent_blockhash().unwrap().0;
    let transaction = Transaction::new_signed_with_payer(
        &[insurance_contract::instruction::close_insurance_contract(
            &insurance_contract::id(),
            &payer.pubkey(),
            &data_address,
        )
        .unwrap()],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );
    client
        .send_and_confirm_transaction_with_spinner(&transaction)
        .unwrap();
}

fn show(client: &RpcClient, data_address: &Pubkey) {
    let insurance_account = client.get_account(data_address).unwrap();
    let insurance_data: InsuranceContractData =
        try_from_slice_unchecked(&insurance_account.data).unwrap();
    println!("{:?}", insurance_data);
}

fn main() {
    let app_matches =
        get_clap_app(crate_name!(), crate_description!(), crate_version!()).get_matches();

    let config = solana_cli_config::Config::default();
    let json_rpc_url = normalize_to_url_if_moniker(
        app_matches
            .value_of("json_rpc_url")
            .unwrap_or(&"https://api.devnet.solana.com".to_owned()),
    );
    println!("RPC Client URL: {}", json_rpc_url);
    let client = RpcClient::new(json_rpc_url);

    let payer = read_keypair_file(
        app_matches
            .value_of("fee_payer")
            .unwrap_or(&config.keypair_path),
    )
    .unwrap();

    println!("Payer pubkey: {}", payer.pubkey());

    let (sub_command, sub_matches) = app_matches.subcommand();
    match (sub_command, sub_matches) {
        ("save", Some(arg_matches)) => {
            let contract_id = value_t_or_exit!(arg_matches, "insurance_contract_id", u32);
            let address = Keypair::new();
            println!(
                "Generated new keypair for InsuranceContract Account: {}",
                address.pubkey()
            );
            println!("Saving new InsuranceContract with id: {}", contract_id);

            save(&client, &payer, contract_id, &address);
        }

        ("close", Some(arg_matches)) => {
            let address = value_t_or_exit!(arg_matches, "address", Pubkey);
            println!("Close InsuranceContract: {}", address);

            close(&client, &payer, &address);
        }

        ("show", Some(arg_matches)) => {
            let address = value_t_or_exit!(arg_matches, "address", Pubkey);
            println!("Information of InsuranceContract: {}", address);
            show(&client, &address);
        }

        _ => {
            println!("{}", app_matches.usage());
        }
    }

    println!("Completed!");
}
