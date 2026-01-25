import os
import re

def process_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    original = content

    # 1. Remove file-level #[cfg_attr(test, allow(...))]
    content = re.sub(r'#\[cfg_attr\(\s*test,\s*allow\(.*\)\s*\)\]\s*', '', content, flags=re.DOTALL)

    # 2. Replace manual #[cfg(test)] mod tests { with #[keyforge_testing_macros::kf_test] mod tests {
    # This pattern handles cases where #[cfg(test)] is on the line immediately preceding mod tests {
    content = re.sub(r'#\[cfg\(test\)\]\s*mod tests \{', '#[keyforge_testing_macros::kf_test]\nmod tests {', content)

    # 3. Purge manual #[allow(...)] immediately preceding #[keyforge_testing_macros::kf_test]
    # or #[cfg(test)] if it somehow remained.
    # Note: Using a slightly different approach to avoid multi-line string issues in the re.sub replacement part
    content = re.sub(r'#\[allow\(.*\)\]\s*(#\[keyforge_testing_macros::kf_test\])', r'\
\1', content, flags=re.DOTALL)

    if content != original:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

targets = [
    "apps/keyforge-agent/src/agent/calibration.rs",
    "apps/keyforge-agent/src/agent/compute.rs",
    "apps/keyforge-agent/src/agent/network/breaker.rs",
    "apps/keyforge-agent/src/hw_detect.rs",
    "apps/keyforge-agent/src/identity.rs",
    "apps/keyforge-agent/src/models.rs",
    "apps/keyforge-agent/tests/agent_integration.rs",
    "apps/keyforge-agent/tests/calibration_integration.rs",
    "apps/keyforge-assetmgr/src/lib.rs",
    "apps/keyforge-cli/src/cli_parsers.rs",
    "apps/keyforge-cli/src/cmd/update.rs",
    "apps/keyforge-cli/src/update.rs",
    "apps/keyforge-cli/tests/integration/commands.rs",
    "apps/keyforge-cli/tests/integration/io.rs",
    "apps/keyforge-cli/tests/integration/search.rs",
    "apps/keyforge-cli/tests/integration/security.rs",
    "apps/keyforge-hive/src/api/validation.rs",
    "apps/keyforge-hive/src/auth.rs",
    "apps/keyforge-ui/src-tauri/src/commands/library.rs",
    "libs/keyforge-adapter/src/conversion/config.rs",
    "libs/keyforge-adapter/src/conversion/geometry.rs",
    "libs/keyforge-adapter/src/conversion/layout.rs",
    "libs/keyforge-adapter/src/parsing.rs",
    "libs/keyforge-adapter/src/utils/padding.rs",
    "libs/keyforge-compute/src/biometrics.rs",
    "libs/keyforge-compute/src/hardware.rs",
    "libs/keyforge-evolution/src/supervisor/annealing.rs",
    "libs/keyforge-evolution/src/supervisor/optimizer.rs",
    "libs/keyforge-evolution/src/supervisor/state.rs",
    "libs/keyforge-evolution/src/supervisor/strategies/annealing.rs",
    "libs/keyforge-evolution/src/supervisor/strategies/group.rs",
    "libs/keyforge-evolution/src/verify.rs",
    "libs/keyforge-export/src/qmk.rs",
    "libs/keyforge-export/src/util.rs",
    "libs/keyforge-export/src/via.rs",
    "libs/keyforge-export/src/zmk.rs",
    "libs/keyforge-infra/src/asset/caching_provider.rs",
    "libs/keyforge-infra/src/asset/valkey_provider.rs",
    "libs/keyforge-infra/src/config.rs",
    "libs/keyforge-infra/src/fs/init.rs",
    "libs/keyforge-infra/src/net/client.rs",
    "libs/keyforge-infra/src/util/common.rs",
    "libs/keyforge-infra/src/util/corpus.rs",
    "libs/keyforge-model/src/asset.rs",
    "libs/keyforge-model/src/config/aggregate.rs",
    "libs/keyforge-model/src/config/constraints.rs",
    "libs/keyforge-model/src/config/definitions.rs",
    "libs/keyforge-model/src/config/search.rs",
    "libs/keyforge-model/src/config/source.rs",
    "libs/keyforge-model/src/config/utils.rs",
    "libs/keyforge-model/src/config/weights.rs",
    "libs/keyforge-model/src/corpus.rs",
    "libs/keyforge-model/src/cost_model.rs",
    "libs/keyforge-model/src/error.rs",
    "libs/keyforge-model/src/geometry/kle.rs",
    "libs/keyforge-model/src/job.rs",
    "libs/keyforge-model/src/keyboard.rs",
    "libs/keyforge-model/src/keycodes.rs",
    "libs/keyforge-model/src/layout.rs",
    "libs/keyforge-model/src/rubric.rs",
    "libs/keyforge-model/src/types.rs",
    "libs/keyforge-model/src/utils/mod.rs",
    "libs/keyforge-model/src/validator.rs",
    "libs/keyforge-persistence/src/project.rs",
    "libs/keyforge-physics/src/analysis/fingerprint.rs",
    "libs/keyforge-physics/src/analysis/heuristics.rs",
    "libs/keyforge-physics/src/analysis/mod.rs",
    "libs/keyforge-physics/src/engines/arm_neon.rs",
    "libs/keyforge-physics/src/engines/arm_sve.rs",
    "libs/keyforge-physics/src/engines/intel_avx512.rs",
    "libs/keyforge-physics/src/engines/intel_comet_lake.rs",
    "libs/keyforge-physics/src/engines/wasm_simd.rs",
    "libs/keyforge-physics/src/error.rs",
    "libs/keyforge-physics/src/kernel/compute/analysis.rs",
    "libs/keyforge-physics/src/kernel/compute/delta.rs",
    "libs/keyforge-physics/src/kernel/compute/flow.rs",
    "libs/keyforge-physics/src/kernel/mechanics.rs",
    "libs/keyforge-physics/src/kernel/stages/costs.rs",
    "libs/keyforge-physics/src/kernel/stages/geometry.rs",
    "libs/keyforge-physics/src/verify.rs",
    "libs/keyforge-protocol/src/assets.rs",
    "libs/keyforge-protocol/src/error.rs",
    "libs/keyforge-protocol/src/job.rs",
    "libs/keyforge-protocol/src/lib.rs",
    "libs/keyforge-protocol/src/node.rs",
    "libs/keyforge-protocol/src/serde_utils.rs",
    "libs/keyforge-protocol/src/telemetry.rs",
    "libs/keyforge-security/src/lib.rs",
    "libs/keyforge-testing/src/lib.rs",
    "libs/keyforge-wasm/src/lib.rs",
    "libs/keyforge-model/src/testing.rs"
]

modified_count = 0
for t in targets:
    if os.path.exists(t):
        if process_file(t):
            modified_count += 1

print(f"Purged boilerplate in {modified_count} files.")