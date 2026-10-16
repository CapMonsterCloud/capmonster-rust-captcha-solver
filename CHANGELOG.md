# Changelog

All notable changes to this crate are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versions follow [SemVer](https://semver.org/)
(while the major version is 0, a breaking change bumps the **minor** number).

## [0.1.0]

### Added
- Async client: `createTask`, `getTaskResult`, `getBalance`, and `solve_captcha` with a per-type polling schedule.
- Typed requests: reCAPTCHA v2 / v2 Enterprise / v3 / v3 Enterprise, reCAPTCHA click and Recognition ComplexImageTask,
  FunCaptcha, GeeTest v3/v4, Turnstile (standard, Cloudflare Challenge, Waiting Room), Amazon WAF, Binance, MTCaptcha,
  Yidun, Prosopo, ImageToText, and CustomTask: DataDome, Imperva, TSPD, HUNT, Basilisk, TenDI, Alibaba, Friendly Captcha, Altcha.
- Local validation of requests before sending; secrets are masked in `Debug` output.
