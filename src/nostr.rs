use nostr_sdk::prelude::*;
use std::error::Error;

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
