/*
 * Hurl (https://hurl.dev)
 * Copyright (C) 2024 Orange
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *          http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
use std::collections::HashMap;

use hurl_core::ast::*;

use crate::http;
use crate::runner::error::{RunnerError, RunnerErrorKind};
use crate::runner::filter::eval_filters;
use crate::runner::predicate::eval_predicate;
use crate::runner::query::eval_query;
use crate::runner::result::AssertResult;
use crate::runner::Value;
use crate::util::path::ContextDir;

impl AssertResult {
    /// Evaluates an assert and returns `None` if assert is succeeded or an `Error` if failed.
    pub fn error(&self) -> Option<RunnerError> {
        match self {
            AssertResult::Version {
                actual,
                expected,
                source_info,
            } => {
                if expected.as_str() == "HTTP"
                    || expected.as_str() == "HTTP/*"
                    || actual == expected
                {
                    None
                } else {
                    let kind = RunnerErrorKind::AssertVersion {
                        actual: actual.to_string(),
                    };
                    Some(RunnerError::new(*source_info, kind, false))
                }
            }
            AssertResult::Status {
                actual,
                expected,
                source_info,
            } => {
                if actual == expected {
                    None
                } else {
                    let kind = RunnerErrorKind::AssertStatus {
                        actual: actual.to_string(),
                    };
                    Some(RunnerError::new(*source_info, kind, false))
                }
            }
            AssertResult::Header {
                actual,
                expected,
                source_info,
            } => match actual {
                Err(e) => Some(e.clone()),
                Ok(s) => {
                    if s == expected {
                        None
                    } else {
                        let kind = RunnerErrorKind::AssertHeaderValueError { actual: s.clone() };
                        Some(RunnerError::new(*source_info, kind, false))
                    }
                }
            },
            AssertResult::Body {
                actual,
                expected,
                source_info,
            } => match expected {
                Err(e) => Some(e.clone()),
                Ok(expected) => match actual {
                    Err(e) => Some(e.clone()),
                    Ok(actual) => {
                        if actual == expected {
                            None
                        } else {
                            let actual = actual.to_string();
                            let expected = expected.to_string();
                            let kind = RunnerErrorKind::AssertBodyValueError { actual, expected };
                            Some(RunnerError::new(*source_info, kind, false))
                        }
                    }
                },
            },
            AssertResult::Explicit { actual: Err(e), .. } => Some(e.clone()),
            AssertResult::Explicit {
                predicate_result: Some(Err(e)),
                ..
            } => Some(e.clone()),
            _ => None,
        }
    }
    pub fn line(&self) -> usize {
        match self {
            AssertResult::Version { source_info, .. } => source_info.start.line,
            AssertResult::Status { source_info, .. } => source_info.start.line,
            AssertResult::Header { source_info, .. } => source_info.start.line,
            AssertResult::Body { source_info, .. } => source_info.start.line,
            AssertResult::Explicit { source_info, .. } => source_info.start.line,
        }
    }
}

/// Evaluates an explicit `assert`, given a set of `variables`, a HTTP response and a context
/// directory `context_dir`.
pub fn eval_explicit_assert(
    assert: &Assert,
    variables: &HashMap<String, Value>,
    http_response: &http::Response,
    context_dir: &ContextDir,
) -> AssertResult {
    let query_result = eval_query(&assert.query, variables, http_response);

    let actual = if assert.filters.is_empty() {
        query_result
    } else if let Ok(optional_value) = query_result {
        match optional_value {
            None => Err(RunnerError {
                source_info: assert
                    .filters
                    .first()
                    .expect("at least one filter")
                    .1
                    .source_info,
                kind: RunnerErrorKind::FilterMissingInput,
                assert: true,
            }),
            Some(value) => {
                let filters = assert
                    .filters
                    .iter()
                    .map(|(_, f)| f.clone())
                    .collect::<Vec<_>>();
                match eval_filters(&filters, &value, variables, true) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(e),
                }
            }
        }
    } else {
        query_result
    };

    let source_info = assert.predicate.predicate_func.source_info;
    let predicate_result = match &actual {
        Err(_) => None,
        Ok(actual) => Some(eval_predicate(
            &assert.predicate,
            variables,
            actual,
            context_dir,
        )),
    };

    AssertResult::Explicit {
        actual,
        source_info,
        predicate_result,
    }
}

#[cfg(test)]
pub mod tests {
    use hurl_core::ast::SourceInfo;
    use std::path::Path;

    use super::super::query;
    use super::*;
    use crate::http::xml_three_users_http_response;
    use crate::runner::Number;

    // `xpath "//user" count == 3`
    pub fn assert_count_user() -> Assert {
        let whitespace = Whitespace {
            value: String::from(" "),
            source_info: SourceInfo::new(Pos::new(1, 1), Pos::new(1, 1)),
        };
        let predicate = Predicate {
            not: false,
            space0: whitespace.clone(),
            predicate_func: PredicateFunc {
                source_info: SourceInfo::new(Pos::new(1, 22), Pos::new(1, 24)),
                value: PredicateFuncValue::Equal {
                    space0: whitespace.clone(),
                    value: PredicateValue::Number(hurl_core::ast::Number::Integer(3)),
                    operator: true,
                },
            },
        };
        Assert {
            line_terminators: vec![],
            space0: whitespace.clone(),
            query: query::tests::xpath_users(),
            filters: vec![(
                whitespace.clone(),
                Filter {
                    source_info: SourceInfo::new(Pos::new(1, 16), Pos::new(1, 21)),
                    value: FilterValue::Count,
                },
            )],
            space1: whitespace.clone(),
            predicate,
            line_terminator0: LineTerminator {
                space0: whitespace.clone(),
                comment: None,
                newline: whitespace,
            },
        }
    }

    #[test]
    fn test_invalid_xpath() {}

    #[test]
    fn test_eval() {
        let variables = HashMap::new();
        let current_dir = std::env::current_dir().unwrap();
        let file_root = Path::new("file_root");
        let context_dir = ContextDir::new(current_dir.as_path(), file_root);
        assert_eq!(
            eval_explicit_assert(
                &assert_count_user(),
                &variables,
                &xml_three_users_http_response(),
                &context_dir
            ),
            AssertResult::Explicit {
                actual: Ok(Some(Value::Number(Number::Integer(3)))),
                source_info: SourceInfo::new(Pos::new(1, 22), Pos::new(1, 24)),
                predicate_result: Some(Ok(())),
            }
        );
    }
}
