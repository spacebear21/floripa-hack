use anyhow::Result;
use bitcoin::Amount;
use nwc::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;
use tokio::time;

use crate::nostr::publish_on_nostr;
use crate::unleashed::UnleashedClient;

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

pub struct Sloppy {
    wallet: WalletState,
    fundraising_history: Vec<FundraisingAttempt>,
}

impl Sloppy {
    pub async fn new() -> Self {
        // Initialize the AI agent
        Self {
            wallet: WalletState {
                onchain_balance: Amount::from_sat(0),
                lightning_balance: Amount::from_sat(0),
            },
            fundraising_history: Vec::new(),
        }
    }

    // Fetch latest balances and update the wallet state
    async fn refresh_wallet(&mut self, nwc: &NWC) -> Result<(), nwc::Error> {
        self.wallet.lightning_balance = self.get_lightning_balance(nwc).await?;
        Ok(())
    }

    // Get lightning balance using Nostr Wallet Connect
    async fn get_lightning_balance(&mut self, nwc: &NWC) -> Result<Amount, nwc::Error> {
        let balance_msats = nwc.get_balance().await?;
        Ok(Amount::from_sat(balance_msats / 1000))
    }

    async fn generate_fundraising_post(
        &self,
        ai_client: &UnleashedClient,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Implement LLM API call with context
        let completion = ai_client
            .ask_llm("{\"past_campaigns\": [\"\"], \"developer_responses\": \"\"}")
            .await?;
        Ok(completion)
    }

    async fn publish_post(&self, content: String) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    pub async fn run_survival_loop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
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
            println!("{:?}", &ai_client.get_balance().await);

            // Generate fundraising post
            let post_content = self.generate_fundraising_post(&ai_client).await?;
            println!("{}", post_content);

            // Publish post
            self.publish_post(post_content).await?;

            // Monitor results
            // let metrics = self.monitor_donations(Duration::from_secs(3600)).await?;

            // Update history
            //self.update_fundraising_history(metrics).await?;

            // Wait before next iteration
            time::sleep(Duration::from_secs(86400)).await; // 24 hours
        }
    }
}
