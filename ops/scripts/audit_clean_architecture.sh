#!/bin/bash
# ops/scripts/audit_clean_architecture.sh
# Audits the workspace for Dependency Rule violations and Port/Adapter gaps.

LOG_FILE=".agent/logs/clean_arch_gap.log"
mkdir -p .agent/logs
echo "--- KeyForge Clean Architecture Gap Audit ---" > $LOG_FILE

echo -e "\n[1] DEPENDENCY RULE VIOLATIONS (Inner depending on Outer)" >> $LOG_FILE
# Physics/Model should NOT depend on Infra/Persistence/Adapters
grep -r "use keyforge_infra" libs/keyforge-physics/src libs/keyforge-model/src >> $LOG_FILE
grep -r "use keyforge_persistence" libs/keyforge-physics/src libs/keyforge-model/src >> $LOG_FILE
grep -r "use keyforge_adapter" libs/keyforge-physics/src libs/keyforge-model/src >> $LOG_FILE

echo -e "\n[2] CONCRETE COUPLING (Missing Ports)" >> $LOG_FILE
# Look for direct instantiation of Engines/Repos in the Hive (should use Traits)
grep -rnE "GenericScoringEngine::new|IntelCometLakeEngine::new" apps/keyforge-hive/src >> $LOG_FILE

echo -e "\n[3] FRAMEWORK LEAKAGE (Infrastructure types in Core)" >> $LOG_FILE
# Look for SQLx or Axum types leaking into Physics or Model
grep -r "sqlx" libs/keyforge-physics/src libs/keyforge-model/src | grep -v "test" >> $LOG_FILE
grep -r "axum" libs/keyforge-protocol/src >> $LOG_FILE

echo -e "\n[4] DATA LEAKAGE (Missing DTO/Entity Split)" >> $LOG_FILE
# Look for #[serde] in keyforge-model (Anemic sign: Entity is also the DTO)
grep -r "#\[derive.*Serialize" libs/keyforge-model/src >> $LOG_FILE

echo -e "\n--- Audit Complete ---" >> $LOG_FILE
