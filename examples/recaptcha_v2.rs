//! Run: `CAPMONSTER_API_KEY=<key> cargo run --example recaptcha_v2`

use capmonstercloud_client::requests::RecaptchaV2Request;
use capmonstercloud_client::{CapMonsterClient, ClientOptions};

#[tokio::main]
async fn main() -> capmonstercloud_client::Result<()> {
    let api_key = std::env::var("CAPMONSTER_API_KEY").expect("set CAPMONSTER_API_KEY");
    let client = CapMonsterClient::new(ClientOptions::new(api_key))?;

    println!("balance: {}", client.get_balance().await?);

    let request = RecaptchaV2Request::new(
        "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high",
        "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd",
    );
    let solution = client.solve_captcha(&request).await?;
    println!("{solution}");
    Ok(())
}
