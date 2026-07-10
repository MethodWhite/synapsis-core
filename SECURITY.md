# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.5.x   | :white_check_mark: |
| < 0.5   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in synapsis-core, please report it privately.

**Do not create a public GitHub issue.**

Email: methodwhite101@gmail.com

You should expect a response within 48 hours. If the issue is confirmed, a patch will be released as soon as possible.

## Disclosure Policy

- The vulnerability will be investigated and confirmed.
- A fix will be prepared and tested.
- A new version will be released with the fix.
- The vulnerability will be publicly disclosed after the fix is released.

## PQC Security

synapsis-core includes optional post-quantum cryptography support via `pqc` feature flag.
Any vulnerability in the PQC implementation should be treated as critical.
