# Design: KeyForge Export

**Responsibility:** Generating firmware configuration files.
**Tier:** 3 (The Adapter)

## 1. Exporter Trait

A simple Strategy pattern for different firmware targets.

```rust
pub trait Exporter {
    fn generate(&self, layout_name: &str, keys: &[String]) -> Result<String>;
}
```

## 2. Supported Targets

* **QMK:** Generates `keymap.c` arrays.
* **ZMK:** Generates `.keymap` device tree overlays.
* **VIA:** Generates JSON definitions.
