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

use hurl_core::ast::Expr;

use crate::runner::error::{RunnerError, RunnerErrorKind};
use crate::runner::value::Value;

/// Evaluates the expression `expr` with `variables` map and `http_response`, returns a
/// [`Value`] on success or an [`RunnerError`] .
pub fn eval_expr(expr: &Expr, variables: &HashMap<String, Value>) -> Result<Value, RunnerError> {
    if let Some(value) = variables.get(expr.variable.name.as_str()) {
        Ok(value.clone())
    } else {
        let kind = RunnerErrorKind::TemplateVariableNotDefined {
            name: expr.variable.name.clone(),
        };
        Err(RunnerError::new(expr.variable.source_info, kind, false))
    }
}
