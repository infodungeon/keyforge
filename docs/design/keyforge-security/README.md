# Design: KeyForge Security

**Responsibility:** Cryptographic signing and secret management.
**Tier:** 2 (The Shield)

## 1. Result Signing Protocol

To prevent malicious workers from submitting fake high scores, every result is signed.

```mermaid
sequenceDiagram
    participant Worker
    participant Sec as SecurityCrate
    participant Hive

    Note over Worker: Optimization Complete
    
    Worker->>Sec: sign_result(JobID, Layout, Score, Nonce)
    Sec->>Sec: Hash(Payload)
    Sec->>Sec: Ed25519_Sign(Hash, PrivateKey)
    Sec-->>Worker: Signature (Hex)
    
    Worker->>Hive: Submit(Result + Signature)
    
    Hive->>Sec: verify_result(Payload, Signature, PublicKey)
    alt Valid
        Hive->>Hive: Accept
    else Invalid
        Hive->>Hive: Reject & Ban Node
    end
```

## 2. Memory Safety

* **SecretBytes:** Uses `zeroize` to wipe private keys from memory when dropped.
* **No Logging:** Secrets are never implemented with `Debug` or `Display`.
