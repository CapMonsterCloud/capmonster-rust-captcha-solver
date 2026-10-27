# CapMonster Cloud Rust SDK: Async AI Captcha Solver & Anti-Bot Bypass for Rust

> **🔄 Migrating from 2Captcha or Capsolver?** CapMonster Cloud is a fast, cost-effective **2captcha alternative** and **capsolver alternative** for Rust: reCAPTCHA v3 from **$0.90 per 1,000 solves**, which is **up to 38% cheaper than 2Captcha** and **10% cheaper than Capsolver**, with an API response time of **≤ 2 s**. Already have software built for 2Captcha? CapMonster Cloud [supports the 2Captcha API format](https://docs.capmonster.cloud/docs/external-services/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme). Moving your Rust code takes just a few lines: see the **Migration examples** below. Full comparison and disclaimer are in the comparison section below.

<p align="center">
  <a href="https://capmonster.cloud/en/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme">
    <img src="https://img.shields.io/badge/CapMonster%20Cloud-Rust%20Captcha%20Solver-00B2FF?style=for-the-badge&logo=rust&logoColor=white" alt="CapMonster Cloud Rust SDK - AI captcha solver for Rust" height="40">
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

`capmonstercloud-client` is the official Rust crate for the CapMonster Cloud captcha solving API. Use it for captcha bypass: solve and bypass reCAPTCHA v2/v3 (including Enterprise), Cloudflare Turnstile, GeeTest, DataDome, Amazon WAF, FunCaptcha and other captcha types from async Rust code: the SDK sends a `createTask` request, polls `getTaskResult` and returns the solution token. It is built on **tokio** with `rustls` for TLS, so no OpenSSL is required. As the official CapMonster SDK for Rust, it is a cost-effective 2captcha alternative and capsolver alternative.

**[👉 Get your Free API Key & Free Trial Balance on CapMonster Cloud](https://dash.capmonster.cloud/Account/SignUp?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme)**

---

## ⚡ Key Features of the Rust Captcha Solver SDK

- ⚡ **Fast Solving:** Automated AI models solve captchas in 300 ms to 6 seconds depending on the system load.
- 🤖 **Zero Human Workers:** 100% automated machine learning pipeline with no manual latencies.
- 🦀 **Type-Safe Requests:** required parameters are constructor arguments, closed value sets are enums, inconsistent requests are rejected locally before anything is sent.
- 🛡️ **Modern Anti-Bot Bypass:** Full support for Cloudflare Turnstile, reCAPTCHA Enterprise, GeeTest, DataDome and Amazon WAF.
- 💰 **Cost-Effective:** Low-cost pricing structure based on pay-as-you-go balance.
- 🔌 **Fits Your Stack:** Use the returned token with `fantoccini`, `thirtyfour`, `chromiumoxide`, `reqwest` and other Rust scraping and browser-automation tools.
- 📉 **Cheaper Than Alternatives:** reCAPTCHA v3 from $0.90 per 1,000 solves, versus $1.45 at 2Captcha and $1.00 at Capsolver.
- 🔄 **Easy Migration:** replace your 2Captcha or Capsolver solver code and keep the rest of your scraper or bot unchanged.

---

## 📊 CapMonster Cloud vs 2Captcha vs Capsolver

Looking for a 2captcha alternative or a capsolver alternative for Rust web scraping and automation? Compare price, speed and success rate.

### reCAPTCHA v3

| Parameter | CapMonster Cloud | 2Captcha | CapSolver | DeathByCaptcha |
| :--- | :---: | :---: | :---: | :---: |
| Price per 1,000 solves (from) | **$0.90** | $1.45 | $1.00 | $2.89 |
| API response time | **≤ 2 s** | ≤ 2 s | < 3 s | ≤ 3 s |
| Solving success rate | **up to 99%** | ~77–92% | ~95–98% | ~64–90% |

*Disclaimer: 2Captcha, CapSolver, and DeathByCaptcha are trademarks of their respective owners. CapMonster Cloud is not affiliated with these entities. Performance metrics and pricing comparisons are based on publicly available data and internal benchmarking conducted on 27.09.2026.*

### Cloudflare Turnstile

| Parameter | CapMonster Cloud | 2Captcha | Anti-Captcha | DeathByCaptcha |
| :--- | :---: | :---: | :---: | :---: |
| Price per 1,000 solves (from) | **$1.30** | $1.45 | $2.00 | $2.89 |
| Time to solve | **6.24 s\*** | 16.9 s | 20.33 s | ~13 s |
| Solving success rate | **99%\*** | 96% | 96% | 96% |

*Disclaimer: 2Captcha, Anti-Captcha, and DeathByCaptcha are trademarks of their respective owners. CapMonster Cloud is not affiliated with these entities. Performance metrics and pricing comparisons are based on publicly available data and internal benchmarking as of 12.10.2026.*

**What this means in practice:**

- **reCAPTCHA v3:** $0.90 versus $1.45 at 2Captcha is up to 38% cheaper, and $0.90 versus $1.00 at Capsolver is 10% cheaper.
- **Cloudflare Turnstile:** $1.30 versus $1.45 at 2Captcha is 10% cheaper, and a solving time of 6.24 s versus 16.9 s is more than 60% faster.

---

## 📦 Installation (Cargo)

```toml
[dependencies]
capmonstercloud-client = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Minimum supported Rust version: 1.88. TLS is `rustls` (no OpenSSL needed).

---

## 🚀 Quick Start & Usage Examples: Bypass Captchas in Rust

### 1. Solve reCAPTCHA v2 in Rust (async)

```rust
use capmonstercloud_client::requests::RecaptchaV2Request;
use capmonstercloud_client::{CapMonsterClient, ClientOptions};

#[tokio::main]
async fn main() -> capmonstercloud_client::Result<()> {
    // Initialize client with your API key from the dashboard
    let client = CapMonsterClient::new(ClientOptions::new("YOUR_CAPMONSTER_API_KEY"))?;

    // Build the task payload
    let request = RecaptchaV2Request::new(
        "[https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high](https://lessons.zennolab.com/captchas/recaptcha/v2_simple.php?level=high)",
        "6Lcg7CMUAAAAANphynKgn9YAgA4tQ2KI_iqRyTwd",
    );

    // Send task and receive token (the `solution` object as serde_json::Value)
    let solution = client.solve_captcha(&request).await?;
    println!("reCAPTCHA Token: {}", solution["gRecaptchaResponse"]);
    Ok(())
}
```

### 2. Solve Cloudflare Turnstile in Rust (with a proxy)

```rust
use capmonstercloud_client::requests::{TurnstileRequest, TurnstileStandard};
use capmonstercloud_client::{CapMonsterClient, ClientOptions, ProxyInfo, ProxyType};

#[tokio::main]
async fn main() -> capmonstercloud_client::Result<()> {
    let client = CapMonsterClient::new(ClientOptions::new("YOUR_CAPMONSTER_API_KEY"))?;

    let proxy = ProxyInfo::new(ProxyType::Http, "203.0.113.10", 8080).with_auth("login", "password");
    let request = TurnstileRequest::new(
        "[https://target-website.com/login](https://target-website.com/login)",
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

## 🔄 Migrating from 2Captcha or Capsolver

Already use 2Captcha or Capsolver in your Rust scraper or bot? If your code calls their HTTP APIs with `reqwest` or a community crate, replace only the solver part with the CapMonster Cloud SDK and keep the rest of your code unchanged: you still send the website URL and sitekey and receive the token.

### After: CapMonster Cloud

```rust
use capmonstercloud_client::requests::RecaptchaV2Request;
use capmonstercloud_client::{CapMonsterClient, ClientOptions};

#[tokio::main]
async fn main() -> capmonstercloud_client::Result<()> {
    let client = CapMonsterClient::new(ClientOptions::new("YOUR_CAPMONSTER_API_KEY"))?;

    let request = RecaptchaV2Request::new(
        "[https://target-website.com/page-with-recaptcha](https://target-website.com/page-with-recaptcha)",
        "YOUR_SITE_KEY",
    );

    let solution = client.solve_captcha(&request).await?;
    let token = solution["gRecaptchaResponse"].as_str().unwrap_or_default();
    println!("{token}");
    Ok(())
}
```

The SDK creates the task, polls the result and returns the token, so the createTask / getTaskResult polling code you wrote for 2Captcha or Capsolver is no longer needed. Use the token exactly as before.

### Keep existing 2Captcha-based software

Software that already speaks the 2Captcha API can work with CapMonster Cloud without code changes. See the [guide for connecting other services' APIs](https://docs.capmonster.cloud/docs/external-services/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme).

---

## 🛡️ Supported CAPTCHA Types & Anti-Bot Systems

Use this Rust captcha solver for captcha bypass in web scraping and automation: reCAPTCHA bypass (v2, v2 Enterprise, v3), Cloudflare Turnstile solver, DataDome solver, FunCaptcha solver, Amazon WAF bypass, GeeTest solver and image-to-text recognition.

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
[ CapMonsterClient ] ──► POST createTask to [https://api.capmonster.cloud](https://api.capmonster.cloud)
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

## ❓ FAQ

**Is CapMonster Cloud a 2Captcha alternative for Rust?**
Yes. It is a captcha solving service with an official Rust SDK. For reCAPTCHA v3 it starts at $0.90 per 1,000 solves versus $1.45 at 2Captcha, which is up to 38% cheaper. For Cloudflare Turnstile it starts at $1.30 versus $1.45, which is 10% cheaper.

**Is CapMonster Cloud a Capsolver alternative for Rust?**
Yes. For reCAPTCHA v3 it is 10% cheaper than Capsolver ($0.90 versus $1.00 per 1,000 solves) and shows an API response time of ≤ 2 s versus < 3 s. See the comparison table above and its disclaimer.

**How do I migrate from 2Captcha or Capsolver to CapMonster Cloud in Rust?**
Add `capmonstercloud-client` to `Cargo.toml`, create a `CapMonsterClient` with your API key, build the matching request and call `client.solve_captcha(&request)`. See the migration example above. Software that already supports the 2Captcha API can also be connected through the [CapMonster Cloud guide](https://docs.capmonster.cloud/docs/external-services/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme).

**How do I bypass reCAPTCHA v2 in Rust?**
Create a client, build a `RecaptchaV2Request` with the page URL and sitekey, call `client.solve_captcha(&request).await?` and read `solution["gRecaptchaResponse"]`. See the [reCAPTCHA v2 documentation](https://docs.capmonster.cloud/docs/captchas/no-captcha-task/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme).

**How do I bypass Cloudflare Turnstile in Rust?**
Use `TurnstileRequest` with `TurnstileStandard` (optionally with a proxy), as in example 2 above. According to the [Turnstile documentation](https://docs.capmonster.cloud/docs/captchas/turnstile-task/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme), all Turnstile subtypes are supported automatically: manual, non-interactive and invisible.

**Which captcha types does the Rust SDK support?**
See the table above and the [Supported Captchas Overview](https://docs.capmonster.cloud/docs/captchas/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme). Each type has its own request struct in `src/requests/`.

**How do I get an API key?**
[Create a CapMonster Cloud account](https://dash.capmonster.cloud/Account/SignUp?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme), then copy your API key from the Dashboard. The [Getting Started Guide](https://docs.capmonster.cloud/docs/getting-start/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme) walks through account setup and balance top-up.

**How fast is captcha solving?**
Depending on the system load, a response arrives in 300 ms to 6 seconds.

**Do I need a proxy?**
It depends on the captcha type. The table above marks proxy support as Optional or Required for each type (for example, FunCaptcha and DataDome require one).

**How long is a solution token valid?**
Most tokens (reCAPTCHA, Turnstile) remain valid for 120 seconds, so inject and submit them immediately.

**Which Rust version and TLS backend are used?**
Minimum supported Rust version is 1.88. TLS is `rustls`, so OpenSSL is not needed.

**Can I use it with browser automation?**
Yes. The SDK returns the solution token, and you inject it into the page with the tool of your choice (`fantoccini`, `thirtyfour`, `chromiumoxide`, ...).

---

## 🏷️ Versioning & releases

The crate follows [SemVer](https://semver.org/); user-visible changes are listed in [CHANGELOG.md](./CHANGELOG.md).
While the major version is `0`:

| Change | Bump |
| :--- | :--- |
| Bugfix, new optional field / metadata key, docs | patch (`0.1.0` → `0.1.1`) |
| New request type, or any breaking change (renamed/removed public item, a parameter becoming required) | minor (`0.1.1` → `0.2.0`) |

Release flow (automated by [`.github/workflows/release.yml`](./.github/workflows/release.yml)): in a PR bump `version` in `Cargo.toml`
and add a `## [X.Y.Z]` section to `CHANGELOG.md`; after the merge to `main` the workflow publishes to
[crates.io](https://crates.io/crates/capmonstercloud-client), tags `vX.Y.Z` and creates a GitHub Release. The version is also sent in the
`User-Agent` of every request (`Zennolab.CapMonsterCloud.Client.Rust/X.Y.Z`).

## 🧩 Adding a captcha type

1. `src/requests/<snake_name>.rs`: a `#[derive(Serialize)]` struct with `new()` + `with_*` setters, `impl CaptchaRequest` (`TASK_TYPE`, `CLASS` for CustomTask, `timeouts()`, optional `validate()`) and a test asserting the exact JSON. Copy `recaptcha_v2.rs` (plain task) or `altcha.rs` (CustomTask with typed metadata).
2. Register it in `src/requests/mod.rs` (`mod` + `pub use`). Nothing else: the client is generic over `CaptchaRequest`.
3. `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test` (unit tests and the mocked-API tests need neither network nor API key).

---

## ⚖️ Responsible Use

CapMonster Cloud is intended for legitimate business purposes only, such as testing your own website or a website you are authorized to test, and recognizing images you provide. Read more about [ethical and transparent captcha automation](https://capmonster.cloud/en/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme).

---

## 📚 Official Documentation & Links

- 📖 [Getting Started Guide](https://docs.capmonster.cloud/docs/getting-start/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme)
- 🎯 [Supported Captchas Overview](https://docs.capmonster.cloud/docs/captchas/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme)
- ⚙️ [API Methods Reference](https://docs.capmonster.cloud/docs/api/methods/create-task/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme)
- 💬 [Support & Community](https://capmonster.cloud/en/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme#support)

---

## 📄 License

[MIT](./LICENSE) © [CapMonster Cloud](https://capmonster.cloud/en/?utm_source=github&utm_medium=referral&utm_campaign=rust_repo_readme)
