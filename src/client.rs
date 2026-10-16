//! `CapMonsterClient`: createTask / getTaskResult / getBalance and the high-level `solve_captcha`.

use std::time::Duration;

use serde::Deserialize;
use serde_json::{json, Value};
use tokio::time::{sleep, Instant};

use crate::error::{Error, Result};
use crate::proxy::{ClientProxy, ProxyType};
use crate::requests::CaptchaRequest;
use crate::timeouts::GetResultTimeouts;

/// Default API endpoint.
pub const DEFAULT_SERVICE_URL: &str = "https://api.capmonster.cloud";
/// Default `softId` sent with every task.
pub const DEFAULT_SOFT_ID: u32 = 152;

/// Client settings. Build with [`ClientOptions::new`] and chain `with_*` methods.
#[derive(Clone)]
pub struct ClientOptions {
    pub api_key: String,
    pub service_url: String,
    pub soft_id: u32,
    /// Timeout of ONE HTTP request (not of the whole solving).
    pub client_timeout: Duration,
    /// Proxy for the SDK's own requests to the API (http/https only).
    pub client_proxy: Option<ClientProxy>,
    /// If set, replaces the per-captcha-type polling schedule (mostly useful for tests).
    pub timeouts_override: Option<GetResultTimeouts>,
}

impl ClientOptions {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            service_url: DEFAULT_SERVICE_URL.to_string(),
            soft_id: DEFAULT_SOFT_ID,
            client_timeout: Duration::from_secs(20),
            client_proxy: None,
            timeouts_override: None,
        }
    }

    pub fn with_service_url(mut self, url: impl Into<String>) -> Self {
        self.service_url = url.into();
        self
    }

    pub fn with_soft_id(mut self, soft_id: u32) -> Self {
        self.soft_id = soft_id;
        self
    }

    pub fn with_client_timeout(mut self, timeout: Duration) -> Self {
        self.client_timeout = timeout;
        self
    }

    pub fn with_client_proxy(mut self, proxy: ClientProxy) -> Self {
        self.client_proxy = Some(proxy);
        self
    }

    pub fn with_timeouts_override(mut self, timeouts: GetResultTimeouts) -> Self {
        self.timeouts_override = Some(timeouts);
        self
    }
}

// Debug is written by hand so that the API key never ends up in logs.
impl std::fmt::Debug for ClientOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientOptions")
            .field("api_key", &"***")
            .field("service_url", &self.service_url)
            .field("soft_id", &self.soft_id)
            .field("client_timeout", &self.client_timeout)
            .field("client_proxy", &self.client_proxy)
            .field("timeouts_override", &self.timeouts_override)
            .finish()
    }
}

/// Raw envelope shared by all three API methods; every field is optional because
/// each method fills only its own subset.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    error_id: i64,
    error_code: Option<String>,
    error_description: Option<String>,
    task_id: Option<u64>,
    status: Option<String>,
    solution: Option<Value>,
    balance: Option<f64>,
}

impl ApiResponse {
    fn code(&self) -> String {
        self.error_code.clone().unwrap_or_default()
    }
    fn description(&self) -> String {
        self.error_description.clone().unwrap_or_default()
    }
}

/// State of a task returned by [`CapMonsterClient::get_task_result`].
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Processing,
    /// The solution object; its shape depends on the captcha type.
    Ready(Value),
}

/// Asynchronous CapMonster Cloud client. Cheap to clone (the HTTP pool is shared).
#[derive(Debug, Clone)]
pub struct CapMonsterClient {
    options: ClientOptions,
    http: reqwest::Client,
}

impl CapMonsterClient {
    pub fn new(options: ClientOptions) -> Result<Self> {
        let user_agent = format!(
            "Zennolab.CapMonsterCloud.Client.Rust/{}",
            env!("CARGO_PKG_VERSION")
        );
        let mut builder = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(options.client_timeout);

        if let Some(p) = &options.client_proxy {
            if !matches!(p.proxy_type, ProxyType::Http | ProxyType::Https) {
                return Err(Error::UnsupportedProxyType(
                    p.proxy_type.as_str().to_string(),
                ));
            }
            let url = format!("{}://{}:{}", p.proxy_type.as_str(), p.address, p.port);
            let mut proxy = reqwest::Proxy::all(url)?;
            if let (Some(login), Some(password)) = (&p.login, &p.password) {
                proxy = proxy.basic_auth(login, password);
            }
            builder = builder.proxy(proxy);
        }

        Ok(Self {
            options,
            http: builder.build()?,
        })
    }

    pub fn options(&self) -> &ClientOptions {
        &self.options
    }

    /// Account balance in USD.
    pub async fn get_balance(&self) -> Result<f64> {
        let resp = self
            .post("getBalance", json!({ "clientKey": self.options.api_key }))
            .await?;
        if resp.error_id != 0 {
            return Err(Error::GetBalance {
                code: resp.code(),
                description: resp.description(),
            });
        }
        resp.balance
            .ok_or_else(|| Error::MalformedResponse("getBalance: no `balance` field".into()))
    }

    /// Creates a task and returns its id (low-level; most users want [`Self::solve_captcha`]).
    pub async fn create_task<R: CaptchaRequest>(&self, request: &R) -> Result<u64> {
        let body = json!({
            "clientKey": self.options.api_key,
            "task": request.to_task()?,
            "softId": self.options.soft_id,
        });
        let resp = self.post("createTask", body).await?;
        if resp.error_id != 0 {
            return Err(Error::CreateTask {
                code: resp.code(),
                description: resp.description(),
            });
        }
        resp.task_id
            .ok_or_else(|| Error::MalformedResponse("createTask: no `taskId` field".into()))
    }

    /// One `getTaskResult` call (low-level).
    pub async fn get_task_result(&self, task_id: u64) -> Result<TaskStatus> {
        let body = json!({ "clientKey": self.options.api_key, "taskId": task_id });
        let resp = match self.post("getTaskResult", body).await {
            Ok(r) => r,
            // The service sometimes answers 500 while the task is still being processed.
            Err(Error::Status { status: 500, .. }) => return Ok(TaskStatus::Processing),
            Err(e) => return Err(e),
        };
        if resp.error_id != 0 {
            return Err(Error::GetResult {
                code: resp.code(),
                description: resp.description(),
            });
        }
        match resp.status.as_deref() {
            Some("ready") => Ok(TaskStatus::Ready(resp.solution.unwrap_or(Value::Null))),
            _ => Ok(TaskStatus::Processing),
        }
    }

    /// Creates a task, polls until it is solved and returns the `solution` object.
    ///
    /// Returns `Error::Timeout` if the captcha type's maximum waiting time is exceeded.
    pub async fn solve_captcha<R: CaptchaRequest>(&self, request: &R) -> Result<Value> {
        let timeouts = self
            .options
            .timeouts_override
            .unwrap_or_else(|| request.timeouts());
        let task_id = self.create_task(request).await?;

        let now = Instant::now();
        // checked_add: a huge user-supplied timeout must not panic
        let deadline = now
            .checked_add(timeouts.timeout)
            .unwrap_or_else(|| now + Duration::from_secs(365 * 24 * 3600));
        sleep(timeouts.first_request_delay).await;
        while Instant::now() < deadline {
            if let TaskStatus::Ready(solution) = self.get_task_result(task_id).await? {
                return Ok(solution);
            }
            sleep(timeouts.requests_interval).await;
        }
        Err(Error::Timeout(timeouts.timeout))
    }

    async fn post(&self, method: &'static str, body: Value) -> Result<ApiResponse> {
        let url = format!(
            "{}/{}",
            self.options.service_url.trim_end_matches('/'),
            method
        );
        let resp = self.http.post(url).json(&body).send().await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(Error::Status {
                method,
                status: status.as_u16(),
            });
        }
        // The API sometimes returns JSON with a wrong content-type, so parse the text ourselves.
        let text = resp.text().await?;
        Ok(serde_json::from_str(&text)?)
    }
}
