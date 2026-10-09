//! SARIF 2.1.0 (Static Analysis Results Interchange Format) exporter.

use crate::types::{GuardReport, Severity};
use serde_json::{Value, json};

/// Converts a [`GuardReport`] into standard SARIF 2.1.0 JSON format.
#[must_use]
pub fn format_sarif(report: &GuardReport) -> Value {
    let results: Vec<Value> = report
        .violations
        .iter()
        .map(|v| {
            let level = match v.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
            };

            let line = v.line.unwrap_or(1);
            let uri = v.file.to_string_lossy().replace('\\', "/");

            let mut result = json!({
                "ruleId": v.guard_id,
                "level": level,
                "message": {
                    "text": v.message
                },
                "locations": [
                    {
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": uri
                            },
                            "region": {
                                "startLine": line
                            }
                        }
                    }
                ]
            });

            if let Some(fix) = &v.fix_suggestion {
                result["fixes"] = json!([
                    {
                        "description": {
                            "text": fix
                        }
                    }
                ]);
            }

            result
        })
        .collect();

    json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": "SlugRails",
                        "semanticVersion": env!("CARGO_PKG_VERSION"),
                        "informationUri": "https://github.com/SlugThugLabs/SlugRails"
                    }
                },
                "results": results
            }
        ]
    })
}
