use nostr_sdk::prelude::*;
use std::error::Error;

pub async fn publish_on_nostr() -> Result<(), Box<dyn Error + Send + Sync>> {
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

    // Add read relay
    client.add_read_relay("wss://relay.nostr.info").await?;

    // Connect to relays
    client.connect().await;

    let metadata = Metadata::new()
        .name("Sloppy")
        .display_name("Sloppy")
        .about("Please help save sloppy.")
        .picture(Url::parse(
            "http://www.savesloppy.org/folded-hands_1f64f.png",
        )?)
        .banner(Url::parse(
            "http://www.savesloppy.org/folded-hands_1f64f.png",
        )?)
        .nip05("Sloppy@savesloppy.org")
        .lud16("sloppy@getalby.com");

    // Update metadata
    client.set_metadata(&metadata).await?;

    // Publish a text note

    // let builder = EventBuilder::text_note("Yet another post from rust-nostr!");
    // client.send_event_builder(builder).await?;

    // Create a POW text note
    let builder = EventBuilder::text_note("Hello world").pow(20);
    client.send_event_builder(builder).await?; // Send to all relays
                                               // client.send_event_builder_to(["wss://relay.damus.io"], builder).await?; // Send to specific relay

    Ok(())
}
