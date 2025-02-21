use bitcoin::Amount;
use nwc::prelude::*;
use publish_nostr::publish_on_nostr;
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str::FromStr;
use std::time::Duration;
use tokio::time;
use unleashed::UnleashedClient;

mod unleashed;

#[derive(Debug, Serialize, Deserialize)]
struct WalletState {
    onchain_balance: Amount,
    lightning_balance: Amount,
}

#[derive(Debug, Serialize, Deserialize)]
struct FundraisingAttempt {
    timestamp: chrono::DateTime<chrono::Utc>,
    post_content: SocialMediaPost,
    donations_received: Amount,
    donor_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SocialMediaPost {
    content: String,
    platform: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

struct Sloppy {
    wallet: WalletState,
    fundraising_history: Vec<FundraisingAttempt>,
}

impl Sloppy {
    async fn new() -> Result<Self, Box<dyn Error>> {
        // Initialize the AI agent
        Ok(Self {
            wallet: WalletState {
                onchain_balance: Amount::from_sat(0),
                lightning_balance: Amount::from_sat(0),
            },
            fundraising_history: Vec::new(),
        })
    }

    // Fetch latest balances and update the wallet state
    async fn refresh_wallet(&mut self, nwc: &NWC) -> Result<(), Box<dyn Error>> {
        self.wallet.lightning_balance = self.get_lightning_balance(nwc).await?;
        Ok(())
    }

    // Get lightning balance using Nostr Wallet Connect
    async fn get_lightning_balance(&mut self, nwc: &NWC) -> Result<Amount, Box<dyn Error>> {
        let balance_msats = nwc.get_balance().await?;
        Ok(Amount::from_sat(balance_msats / 1000))
    }

    async fn generate_fundraising_post(&self) -> Result<String, Box<dyn Error>> {
        // Implement LLM API call with context
        //todo!()
        Ok("Implement me".into())
    }

    async fn publish_post(&self, content: String) -> Result<(), Box<dyn Error>> {
        // Implement social media API integration
        publish_on_nostr().await?;
        Ok(())
    }

    async fn monitor_donations(
        &mut self,
        window: Duration,
    ) -> Result<FundraisingAttempt, Box<dyn Error>> {
        // Implement donation tracking
        todo!()
    }

    async fn update_fundraising_history(
        &mut self,
        attempt: FundraisingAttempt,
    ) -> Result<(), Box<dyn Error>> {
        self.fundraising_history.push(attempt);
        // Save to storage
        Ok(())
    }

    async fn run_survival_loop(&mut self) -> Result<(), Box<dyn Error>> {
        let nwc_uri = std::env::var("NWC_URI")?;
        let unleashed_api_key = std::env::var("UNLEASHED_API")?;

        let ai_client = UnleashedClient::new(&unleashed_api_key)?;

        // Parse NWC uri
        let uri = NostrWalletConnectURI::parse(nwc_uri)?;

        // Initialize NWC client
        let nwc = NWC::new(uri);

        println!("{:?}", nwc.get_info().await?);

        loop {
            // Check current funds
            self.refresh_wallet(&nwc).await?;
            println!("Wallet balance: {:?}", self.wallet);
            println!("{:?}", ai_client.get_balance().await);

            // Generate fundraising post
            let post_content = self.generate_fundraising_post().await?;

            // Publish post
            self.publish_post(post_content).await?;

            // Monitor results
            let metrics = self.monitor_donations(Duration::from_secs(3600)).await?;

            // Update history
            self.update_fundraising_history(metrics).await?;

            // Wait before next iteration
            time::sleep(Duration::from_secs(86400)).await; // 24 hours
        }
    }
}

mod publish_nostr {
    use bitcoin::key::Secp256k1;
    use nostr_sdk::prelude::*;
    use std::error::Error;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    pub async fn publish_on_nostr() -> Result<(), Box<dyn Error>> {
        // let keys = Keys::generate();
        // let secp = Secp256k1::new();
        // println!("publickey: {}", keys.public_key());
        // println!(
        //     "xonly_pubkey: {} \nParity: {:?}",
        //     keys.secret_key().x_only_public_key(&secp).0,
        //     keys.secret_key().x_only_public_key(&secp).1
        // );
        // println!("secretkey_hex: {}", keys.secret_key().to_secret_hex());
        // println!("secretkey: {}", keys.secret_key().display_secret());

        // Or use your already existing (from hex or bech32)
        let nostr_seckey = std::env::var("NOSTR_SECKEY")?;
        let keys = Keys::parse(&nostr_seckey)?;

        // Show bech32 public key
        let bech32_pubkey: String = keys.public_key().to_bech32()?;
        println!("Bech32 PubKey: {}", bech32_pubkey);

        // Configure client to use proxy for `.onion` relays
        //let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
        let connection: Connection = Connection::new();
        //.proxy(addr)
        //.embedded_tor() // Use `.embedded_tor()` instead to enable the embedded tor client (require `tor` feature)
        //.target(ConnectionTarget::Onion);
        let opts = Options::new().connection(connection);

        // Create new client with custom options
        let client = Client::builder().signer(keys.clone()).opts(opts).build();

        // Add relays
        client.add_relay("wss://relay.damus.io").await?;
        client.add_relay("wss://nos.lol").await?;
        client
            .add_relay("ws://jgqaglhautb4k6e6i2g34jakxiemqp6z4wynlirltuukgkft2xuglmqd.onion")
            .await?;

        // Add read relay
        client.add_read_relay("wss://relay.nostr.info").await?;

        // Connect to relays
        client.connect().await;

        let metadata = Metadata::new()
            .name("Test Demo User Name")
            .display_name("Test Demo User Display Name")
            .about("This is just a description")
            .picture(Url::parse("https://upload.wikimedia.org/wikipedia/commons/thumb/3/3c/Czechoslovakia_1938_road_sign_-_Give_Way.svg/501px-Czechoslovakia_1938_road_sign_-_Give_Way.svg.png")?)
            .banner(Url::parse("https://upload.wikimedia.org/wikipedia/commons/thumb/3/3c/Czechoslovakia_1938_road_sign_-_Give_Way.svg/501px-Czechoslovakia_1938_road_sign_-_Give_Way.svg.png")?)
            .nip05("username@example.com")
            .lud16("username@yexmaple.com")
            .custom_field("custom_field", "my custom value");

        // Update metadata
        client.set_metadata(&metadata).await?;

        // Publish a text note

        let builder = EventBuilder::text_note("My first text note from rust-nostr!");
        client.send_event_builder(builder).await?;

        // Create a POW text note
        let builder = EventBuilder::text_note("POW text note from nostr-sdk").pow(20);
        client.send_event_builder(builder).await?; // Send to all relays
                                                   // client.send_event_builder_to(["wss://relay.damus.io"], builder).await?; // Send to specific relay

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut sloppy = Sloppy::new().await?;
    sloppy.run_survival_loop().await?;
    Ok(())
}
