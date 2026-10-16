//! Run: `CAPMONSTER_API_KEY=<key> cargo run --example image_to_text -- <image.png>`

use capmonstercloud_client::requests::{ImageToTextRequest, TextModule};
use capmonstercloud_client::{CapMonsterClient, ClientOptions};

#[tokio::main]
async fn main() -> capmonstercloud_client::Result<()> {
    let api_key = std::env::var("CAPMONSTER_API_KEY").expect("set CAPMONSTER_API_KEY");
    let path = std::env::args()
        .nth(1)
        .expect("usage: image_to_text <image file>");
    let image = std::fs::read(path).expect("cannot read the image");

    let client = CapMonsterClient::new(ClientOptions::new(api_key))?;
    println!("balance: {}", client.get_balance().await?);

    let request = ImageToTextRequest::new(image).with_module_name(TextModule::Universal);
    println!("{}", client.solve_captcha(&request).await?);
    Ok(())
}
