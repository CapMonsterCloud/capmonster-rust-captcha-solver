# CapMonster Cloud Rust SDK: AI Captcha Solver & Anti-Bot Bypass

<p align="center">
  <a href="https://capmonster.cloud/en/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme">
    <img src="https://img.shields.io/badge/CapMonster%20Cloud-Rust%20Captcha%20Solver-00B2FF?style=for-the-badge&logo=rust&logoColor=white" alt="CapMonster Cloud Rust SDK" height="40">
  </a>
</p>

<p align="center">
  <strong>Fast, AI-driven CAPTCHA solving client for Rust. Seamlessly bypass Cloudflare Turnstile, reCAPTCHA v2/v3/Enterprise, DataDome, GeeTest, and Amazon WAF in web scraping and browser automation.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/capmonstercloud-client"><img src="https://img.shields.io/crates/v/capmonstercloud-client.svg?style=flat-square&color=blue" alt="crates.io version"></a>
  <a href="https://crates.io/crates/capmonstercloud-client"><img src="https://img.shields.io/crates/d/capmonstercloud-client.svg?style=flat-square" alt="crates.io downloads"></a>
  <a href="https://docs.rs/capmonstercloud-client"><img src="https://img.shields.io/docsrs/capmonstercloud-client?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/CapMonsterCloud/capmonster-rust-captcha-solver/stargazers"><img src="https://img.shields.io/github/stars/CapMonsterCloud/capmonster-rust-captcha-solver?style=flat-square&color=yellow" alt="GitHub Stars"></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/License-MIT-orange.svg?style=flat-square" alt="License: MIT"></a>
</p>

---

Official asynchronous Rust SDK for [CapMonster Cloud](https://capmonster.cloud/en/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme). Plug ultra-fast AI captcha recognition into your **tokio** based scrapers and browser-automation workflows (`fantoccini`, `thirtyfour`, `chromiumoxide`, `reqwest`, ...) without human-in-the-loop delays.

**[👉 Get your Free API Key & Free Trial Balance on CapMonster Cloud](https://dash.capmonster.cloud/Account/SignUp?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme)**

---

## ⚡ Highlights & Benchmarks

- ⚡ **Sub-Second Speed:** Automated AI models solve captchas in 300 ms to 6 seconds depending on the system load.
- 🤖 **Zero Human Workers:** 100% automated machine learning pipeline with no manual latencies.
- 🦀 **Type-Safe Requests:** required parameters are constructor arguments, closed value sets are enums, inconsistent requests are rejected locally before anything is sent.
- 🛡️ **Modern Anti-Bot Bypass:** Full support for Cloudflare Turnstile, reCAPTCHA Enterprise, GeeTest, DataDome and Amazon WAF.
- 💰 **Cost-Effective:** Low-cost pricing structure based on pay-as-you-go balance.

---

## 📦 Installation

```toml
[dependencies]
capmonstercloud-client = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Minimum supported Rust version: 1.88. TLS is `rustls` (no OpenSSL needed).

---

## 🚀 Quick Start & Usage Examples

### 1. Asynchronous reCAPTCHA v2 Solving

```rust
use capmonstercloud_client::requests::RecaptchaV2Request;
use capmonstercloud_client::{CapMonsterClient, ClientOptions};

#[tokio::main]
async fn main() -> capmonstercloud_client::Result<()> {
    // Initialize client with your API key from the dashboard
    let client = CapMonsterClient::new(ClientOptions::new("YOUR_CAPMONSTER_API_KEY"))?;

    // Build the task payload
    let request = RecaptchaV2Request::new(
        "https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high",
        "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd",
    );

    // Send task and receive token (the `solution` object as serde_json::Value)
    let solution = client.solve_captcha(&request).await?;
    println!("reCAPTCHA Token: {}", solution["gRecaptchaResponse"]);
    Ok(())
}
```

### 2. Cloudflare Turnstile with a proxy

```rust
use capmonstercloud_client::requests::{TurnstileRequest, TurnstileStandard};
use capmonstercloud_client::{CapMonsterClient, ClientOptions, ProxyInfo, ProxyType};

#[tokio::main]
async fn main() -> capmonstercloud_client::Result<()> {
    let client = CapMonsterClient::new(ClientOptions::new("YOUR_CAPMONSTER_API_KEY"))?;

    let proxy = ProxyInfo::new(ProxyType::Http, "203.0.113.10", 8080).with_auth("login", "password");
    let request = TurnstileRequest::new(
        "https://target-website.com/login",
        "0x4AAAAAAABnPIDnK2k_e-2",
        TurnstileStandard::new().with_proxy(proxy),
    );

    let solution = client.solve_captcha(&request).await?;
    // inject solution["token"] into the cf-turnstile-response field of your browser session and submit
    println!("Turnstile token: {}", solution["token"]);
    Ok(())
}
```

### 3. Account balance and low-level calls

```rust
println!("balance: {}", client.get_balance().await?);

// solve_captcha() = create_task() + polling get_task_result()
let task_id = client.create_task(&request).await?;
match client.get_task_result(task_id).await? {
    TaskStatus::Processing => println!("not ready yet"),
    TaskStatus::Ready(solution) => println!("{solution}"),
}
```

`ClientOptions` knobs: `with_service_url`, `with_soft_id` (default `152`), `with_client_timeout` (one HTTP request, default 20 s), `with_client_proxy` (http/https proxy for the SDK's own requests), `with_timeouts_override`.
Errors are a single `capmonstercloud_client::Error` enum (`Validation`, `Http`, `CreateTask`, `GetResult`, `Timeout`, ...).

---

## 🛡️ Supported CAPTCHA Types

All task types conform to the official [CapMonster Cloud API Documentation](https://docs.capmonster.cloud/docs/captchas/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme):

| Protection Type | Task Request Type | Proxy |
| :--- | :--- | :---: |
| **Cloudflare Turnstile / Challenge / Waiting Room** | `TurnstileRequest` (`TurnstileStandard`, `TurnstileChallengeToken`, `TurnstileChallengeCookie`, `TurnstileWaitRoom`) | Optional / Required for cookie modes |
| **reCAPTCHA v2** | `RecaptchaV2Request` | Optional |
| **reCAPTCHA v2 Enterprise** | `RecaptchaV2EnterpriseRequest` | Optional |
| **reCAPTCHA v3** | `RecaptchaV3ProxylessRequest` | — |
| **reCAPTCHA v3 Enterprise** | `RecaptchaV3EnterpriseRequest` | — |
| **GeeTest (v3, v4)** | `GeetestRequest` (`GeetestVersion::V3` / `V4`) | Optional |
| **Amazon WAF** | `AmazonWafRequest` (`AmazonVisibleCaptcha`, `AmazonChallengeAndCaptcha`, `AmazonChallengeOnly`) | Optional |
| **FunCaptcha (Arkose Labs)** | `FuncaptchaRequest` | 🌐 Required |
| **DataDome** | `DataDomeCustomTaskRequest` | 🌐 Required |
| **Imperva / Incapsula** | `ImpervaCustomTaskRequest` | 🌐 Required |
| **TSPD** | `TspdCustomTaskRequest` | 🌐 Required |
| **HUNT** | `HuntCustomTaskRequest` | 🌐 Required |
| **Basilisk / TenDI / Alibaba / Friendly Captcha / Altcha** | `BasiliskCustomTaskRequest`, `TenDiCustomTaskRequest`, `AlibabaCustomTaskRequest`, `FriendlyCustomTaskRequest`, `AltchaCustomTaskRequest` | Optional |
| **Binance CAPTCHA** | `BinanceTaskRequest` | Optional |
| **MTCaptcha / Yidun / Prosopo** | `MtCaptchaRequest`, `YidunRequest`, `ProsopoTaskRequest` | Optional |
| **Standard Text CAPTCHA** | `ImageToTextRequest` (+ `TextModule`) | — |
| **Complex Image Tasks** | `RecaptchaComplexImageTaskRequest`, `RecognitionComplexImageTaskRequest` | — |

One source file per type lives in [`src/requests/`](./src/requests).

---

## 🛠️ How It Works

```text
[ Your service ]
         │
         ▼
[ Extract sitekey & website URL ]
         │
         ▼
[ CapMonsterClient ] ──► POST createTask to https://api.capmonster.cloud
         │
         ▼
[ Neural Network Processing ] ──► Poll getTaskResult (300ms - 6s)
         │
         ▼
[ Receive Token & Autosubmit ] ──► Complete bypass
```

---

## ⚙️ Best Practices

- **Token Expiration:** Most tokens (reCAPTCHA, Turnstile) remain valid for 120 seconds. Inject and submit the solution immediately after receiving the response.
- **Proxy Usage:** For custom tasks and strict protections, use the same proxy in your scraping session and the CapMonster task payload.
- **Balance Monitoring:** Ensure your account balance remains positive in the [Dashboard](https://dash.capmonster.cloud/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme).
- **Secrets in logs:** `Debug` output of `ClientOptions` and proxies masks the API key and passwords.

---

## 🧩 Adding a captcha type

1. `src/requests/<snake_name>.rs`: a `#[derive(Serialize)]` struct with `new()` + `with_*` setters, `impl CaptchaRequest` (`TASK_TYPE`, `CLASS` for CustomTask, `timeouts()`, optional `validate()`) and a test asserting the exact JSON. Copy `recaptcha_v2.rs` (plain task) or `altcha.rs` (CustomTask with typed metadata).
2. Register it in `src/requests/mod.rs` (`mod` + `pub use`). Nothing else: the client is generic over `CaptchaRequest`.
3. `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test` (unit tests and the mocked-API tests need neither network nor API key).

---

## 📚 Official Documentation & Links

- 📖 [Getting Started Guide](https://docs.capmonster.cloud/docs/getting-start/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme)
- 🎯 [Supported Captchas Overview](https://docs.capmonster.cloud/docs/captchas/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme)
- ⚙️ [API Methods Reference](https://docs.capmonster.cloud/docs/api/methods/create-task/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme)
- 💬 [Support & Community](https://capmonster.cloud/en/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme#support)

---

## 📄 License

[MIT](./LICENSE) © [ZennoLab](https://zennolab.com/) / [CapMonster Cloud](https://capmonster.cloud/en/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme)
