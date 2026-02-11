# ClawShell 🛡️

![ClawShell Banner](docs/images/banner.png)

> **Powered by Runta. The essential safety harness for OpenClaw's PII & API data.**

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/runta-dev/ClawShell)
[![Version](https://img.shields.io/badge/version-1.0.0-orange.svg)]()

## 📖 Introduction

**ClawShell** is the security middleware designed to strap onto the **OpenClaw** ecosystem. Just as a safety harness prevents falls, ClawShell prevents data from slipping through the cracks.

It wraps your OpenClaw instance in a protective layer, utilizing intelligent traffic analysis to detect, intercept, and secure sensitive **Personally Identifiable Information (PII)** and **API Access Tokens** in real-time. It ensures that even if your application logic slips, your data remains secure.

## ✨ Key Features

### 🔒 1. PII Safety Net (DLP)
ClawShell acts as a final check for all outbound traffic, identifying sensitive information (e.g., phone numbers, government IDs, emails) before it leaves your perimeter.
- **Real-time Masking**: Redact PII data dynamically in HTTP responses and logs.
- **Compliance Enforcement**: seamlessly aligns your OpenClaw instance with GDPR, CCPA, and SOC2 requirements.
- **Custom Patterns**: Define specific sensitive data fields using Regex or NLP models to suit your domain.

### 🔑 2. API Token Secure Binding
Protecting API keys is critical. ClawShell ensures your tokens are tightly managed.
- **Leak Prevention**: Scans outbound bodies and headers to prevent accidental token exposure to clients.
- **Access Control**: Implements strict role-based token validation and rate limiting.
- **Anomaly Circuit Breaker**: Automatically cuts the connection if suspicious token usage or irregular geographic access is detected.

### 🚀 3. Seamless Integration
- **Sidecar / Gateway Mode**: Deploys alongside OpenClaw without requiring code changes—like strapping on a harness.
- **Runta Ecosystem**: Native integration with Runta's broader security suite.

## Architecture

```
                               ║ security boundary (Unix File System Permissions)
                               ║
                               ║  ┌─────────────────-─┐
                               ║  │  /etc/clawshell   │
                               ║  │  ┄ real API keys  │
                               ║  │  ┄ DLP patterns   │
                               ║  │  ┄ rate limits    │
                               ║  └────────┬─────────-┘
                               ║     reads │
                               ║  ┌────────┴─────────-┐
  ┌──────────────┐  REQUEST    ║  │                   │   REQUEST       ┌────────────┐
  │              ├──(virtual───╫─►│    ClawShell      ├──-(real key,───►│            │
  │   OpenClaw   │   key)      ║  │                   │   PII redacted) │   OpenAI   │
  │              │             ║  │  DLP scan         │                 │     or     │
  │ holds only   │  RESPONSE   ║  │  rate limit       │   RESPONSE      │  Anthropic │
  │ virtual keys │◄─-----------║◄─┤  real-key mapping │◄─-----------────┤            │
  │              │             ║  │                   │                 │            │
  └──────────────┘             ║  └──────────────────-┘                 └────────────┘
                               ║
```

OpenClaw only holds virtual keys and cannot access the 
real API keys(and sensitive data) stored in the privileged config.
ClawShell swaps virtual keys for real ones,
scans for PII, and enforces rate limits.

## Installation

```bash
npm install -g @runta/clawshell

clawshell onboard
```

## Build from Source

```bash
cargo build --release
ls -al target/release/clawshell
```

### Cross-compile on Linux/arm64

```bash
wget https://musl.cc/x86_64-linux-musl-cross.tgz -O /tmp/musl-cross.tgz
tar -xzf /tmp/musl-cross.tgz -C /tmp
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="/tmp/x86_64-linux-musl-cross/bin/x86_64-linux-musl-gcc" \
cargo build --release --target x86_64-unknown-linux-musl
```
