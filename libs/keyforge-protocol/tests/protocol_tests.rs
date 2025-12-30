// Copyright (c) 2025 KeyForge Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use keyforge_protocol::config::Config;
use keyforge_protocol::config::CorpusSource;
use keyforge_protocol::Validator;
use std::str::FromStr;

#[test]
fn test_config_defaults() {
    let c = Config::default();
    assert!(c.search.validate().is_ok());
    assert!(c.weights.validate().is_ok());
}

#[test]
fn test_corpus_source() {
    let c = CorpusSource::default();
    assert_eq!(c.id, "text/en_std");
    assert_eq!(c.weight, 1.0);

    let c1 = CorpusSource::from_str("rust").unwrap();
    assert_eq!(c1.id, "rust");
    assert_eq!(c1.weight, 1.0);

    let c2 = CorpusSource::from_str("rust:0.5").unwrap();
    assert_eq!(c2.id, "rust");
    assert_eq!(c2.weight, 0.5);

    assert!(CorpusSource::from_str("rust:invalid").is_err());
    assert!(CorpusSource::from_str("rust:0.0").is_err());
    assert!(CorpusSource::from_str("rust:-1.0").is_err());
    assert!(CorpusSource::from_str("rust:NaN").is_err());
}
