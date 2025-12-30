#!/bin/bash
# Applies Apache-2.0 Header to Rust files if missing.

HEADER="// Copyright (c) 2025 KeyForge Contributors
//
// Licensed under the Apache License, Version 2.0 (the \"License\");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an \"AS IS\" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License."

find libs -name "*.rs" -print0 | while IFS= read -r -d '' file; do
    if ! grep -q "Licensed under the Apache License" "$file"; then
        echo "Applying header to $file"
        echo "$HEADER" | cat - "$file" > temp && mv temp "$file"
    fi
done
