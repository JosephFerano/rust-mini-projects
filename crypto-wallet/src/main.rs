use secp256k1::{PublicKey, SecretKey, Secp256k1};
use tiny_keccak::keccak256;
use anyhow::{Result};
use serde::{Serialize, Deserialize};
use std::{str::FromStr, fs::OpenOptions, io::BufReader, io::BufWriter};
use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};
use secp256k1::rand::rngs;
use web3::{types::{U256, H256, TransactionParameters, Address}, Web3};
use web3::transports::WebSocket;

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    pub secret_key: String,
    pub pub_key: String,
    pub pub_address: String,
}

impl Display for Wallet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wallet:\n\tSecret Key: {}\n\tPublic Key: {}\n\tPublic Address: {}", self.secret_key, self.pub_key, self.pub_address)
    }
}

impl Wallet {
    pub fn new(secret_key: &SecretKey, public_key: &PublicKey) -> Self {
        Wallet {
            secret_key: secret_key.to_string(),
            pub_key: public_key.to_string(),
            pub_address: format!("{:?}", public_key_address(public_key)),
        }
    }

    pub fn save_to_file(&self, file_path: &str) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)?;
        let buf_writer = BufWriter::new(file);
        serde_json::to_writer_pretty(buf_writer, self)?;

        Ok(())
    }

    pub fn from_file(file_path: &str) -> Result<Wallet> {
        let file = OpenOptions::new()
            .read(true)
            .open(file_path)?;
        let buf_reader = BufReader::new(file);

        let wallet: Wallet = serde_json::from_reader(buf_reader)?;
        Ok(wallet)
    }

    pub fn get_secret_key(&self) -> Result<SecretKey> {
        let sk = SecretKey::from_str(&self.secret_key)?;
        Ok(sk)
    }

    pub fn get_public_key(&self) -> Result<PublicKey> {
        let pk = PublicKey::from_str(&self.secret_key)?;
        Ok(pk)
    }

    pub async fn get_balance(&self, conn: &Web3<WebSocket>) -> Result<f64> {
        let addr = Address::from_str(&self.pub_address)?;
        let balance = conn.eth().balance(addr, None).await?;

        Ok(wei_to_eth(balance))
    }
}

pub fn get_timer() -> u64 {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    duration.as_secs() << 30 | duration.subsec_nanos() as u64
}

pub fn generate_keypair() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    let mut rng = rngs::JitterRng::new_with_timer(get_timer);
    secp.generate_keypair(&mut rng)
}

pub fn public_key_address(pub_key: &PublicKey) -> Address {
    let pub_key = pub_key.serialize_uncompressed();

    debug_assert_eq!(pub_key[0], 0x04);
    let hash = keccak256(&pub_key[1..]);

    Address::from_slice(&hash[12..])
}

pub fn wei_to_eth(amount: U256) -> f64 {
    amount.as_u128() as f64 / 1_000_000_000_000_000_000.0
}

pub fn eth_to_wei(amount: f64) -> U256 {
    U256::from((amount * 1_000_000_000_000_000_000.0) as u128)
}

pub async fn establish_web3_connection(url: &str) -> Result<Web3<WebSocket>> {
    let transport = WebSocket::new(url).await?;
    Ok(Web3::new(transport))
}

pub fn create_transaction(to: Address, amount: U256) -> TransactionParameters {
    TransactionParameters {
        to: Some(to),
        value: amount,
        ..Default::default()
    }
}

pub async fn sign_and_send(conn: &Web3<WebSocket>, tx: TransactionParameters, secret_key: &SecretKey) -> Result<H256> {
    let signed = conn.accounts()
        .sign_transaction(tx, secret_key)
        .await?;
    let results =
        conn.eth()
            .send_raw_transaction(signed.raw_transaction)
            .await?;
    Ok(results)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    // let (secret, pub_key) = generate_keypair();
    // let wallet = Wallet::new(&secret, &pub_key);

    // wallet.save_to_file("wallet.json")
    let wallet = Wallet::from_file("wallet.json")?;
    println!("{wallet}");
    let endpoint = std::env::var("ALCHEMY_GOERLI_WS")?;
    let conn = establish_web3_connection(&endpoint).await?;

    let block_num = conn.eth().block_number().await?;
    println!("Block Number: {block_num}");

    let balance = wallet.get_balance(&conn).await?;
    println!("Balance: {balance}");

    let test_wallet = std::env::var("TEST_WALLET")?;
    let tx = create_transaction(
        Address::from_str(&test_wallet)?,
        eth_to_wei(0.001)
    );

    let tx_hash = sign_and_send(&conn, tx, &wallet.get_secret_key()?).await?;
    println!("{tx_hash:?}");

    Ok(())
}
