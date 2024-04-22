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
use chrono::{DateTime, SecondsFormat, Utc};
use hurl_core::ast::SourceInfo;
use serde_json::Number;

use crate::http::{
    Call, Certificate, Cookie, Header, HttpVersion, Param, Request, RequestCookie, Response,
    ResponseCookie, Timings,
};
use crate::runner::{AssertResult, CaptureResult, EntryResult, HurlResult, Input};
use crate::util::logger;

impl HurlResult {
    /// Serializes an [`HurlResult`] to a JSON representation.
    ///
    /// Note: `content` is passed to this method to save asserts and
    /// errors messages (with lines and columns). This parameter will be removed
    /// soon and the original content will be accessible through the [`HurlResult`] instance.
    pub fn to_json(&self, content: &str, filename: &Input) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "filename".to_string(),
            serde_json::Value::String(filename.to_string()),
        );
        let entries = self
            .entries
            .iter()
            .map(|e| e.to_json(filename, content))
            .collect();
        map.insert("entries".to_string(), serde_json::Value::Array(entries));
        map.insert("success".to_string(), serde_json::Value::Bool(self.success));
        map.insert(
            "time".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.time_in_ms as u64)),
        );
        let cookies = self.cookies.iter().map(|e| e.to_json()).collect();
        map.insert("cookies".to_string(), serde_json::Value::Array(cookies));
        serde_json::Value::Object(map)
    }
}

impl EntryResult {
    fn to_json(&self, filename: &Input, content: &str) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        map.insert(
            "index".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.entry_index)),
        );
        map.insert(
            "line".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.source_info.start.line)),
        );
        let calls = self.calls.iter().map(|c| c.to_json()).collect();
        map.insert("calls".to_string(), calls);
        let captures = self.captures.iter().map(|c| c.to_json()).collect();
        map.insert("captures".to_string(), captures);
        let asserts = self
            .asserts
            .iter()
            .map(|a| a.to_json(filename, content, self.source_info))
            .collect();
        map.insert("asserts".to_string(), asserts);
        map.insert(
            "time".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.time_in_ms as u64)),
        );
        serde_json::Value::Object(map)
    }
}

impl Call {
    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("request".to_string(), self.request.to_json());
        map.insert("response".to_string(), self.response.to_json());
        map.insert("timings".to_string(), self.timings.to_json());
        serde_json::Value::Object(map)
    }
}

impl Request {
    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "method".to_string(),
            serde_json::Value::String(self.method.clone()),
        );
        map.insert(
            "url".to_string(),
            serde_json::Value::String(self.url.clone()),
        );
        let headers = self.headers.iter().map(|h| h.to_json()).collect();
        map.insert("headers".to_string(), headers);
        let cookies = self.cookies().iter().map(|e| e.to_json()).collect();
        map.insert("cookies".to_string(), serde_json::Value::Array(cookies));
        let query_string = self
            .query_string_params()
            .iter()
            .map(|e| e.to_json())
            .collect();
        map.insert(
            "queryString".to_string(),
            serde_json::Value::Array(query_string),
        );
        serde_json::Value::Object(map)
    }
}

impl Response {
    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("httpVersion".to_string(), self.version.to_json());
        map.insert(
            "status".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.status)),
        );
        let headers = self.headers.iter().map(|h| h.to_json()).collect();
        map.insert("headers".to_string(), headers);
        let cookies = self.cookies().iter().map(|e| e.to_json()).collect();
        map.insert("cookies".to_string(), serde_json::Value::Array(cookies));
        if let Some(certificate) = &self.certificate {
            map.insert("certificate".to_string(), certificate.to_json());
        }
        serde_json::Value::Object(map)
    }
}

impl Header {
    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "name".to_string(),
            serde_json::Value::String(self.name.clone()),
        );
        map.insert(
            "value".to_string(),
            serde_json::Value::String(self.value.clone()),
        );
        serde_json::Value::Object(map)
    }
}

impl HttpVersion {
    fn to_json(self) -> serde_json::Value {
        let value = match self {
            HttpVersion::Http10 => "HTTP/1.0",
            HttpVersion::Http11 => "HTTP/1.1",
            HttpVersion::Http2 => "HTTP/2",
            HttpVersion::Http3 => "HTTP/3",
        };
        serde_json::Value::String(value.to_string())
    }
}

impl Param {
    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "name".to_string(),
            serde_json::Value::String(self.name.clone()),
        );
        map.insert(
            "value".to_string(),
            serde_json::Value::String(self.value.clone()),
        );
        serde_json::Value::Object(map)
    }
}

impl RequestCookie {
    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "name".to_string(),
            serde_json::Value::String(self.name.clone()),
        );
        map.insert(
            "value".to_string(),
            serde_json::Value::String(self.value.clone()),
        );
        serde_json::Value::Object(map)
    }
}

impl ResponseCookie {
    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "name".to_string(),
            serde_json::Value::String(self.name.clone()),
        );
        map.insert(
            "value".to_string(),
            serde_json::Value::String(self.value.clone()),
        );

        if let Some(expires) = &self.expires() {
            map.insert(
                "expires".to_string(),
                serde_json::Value::String(expires.to_string()),
            );
        }
        if let Some(max_age) = &self.max_age() {
            map.insert(
                "max_age".to_string(),
                serde_json::Value::String(max_age.to_string()),
            );
        }
        if let Some(domain) = &self.domain() {
            map.insert(
                "domain".to_string(),
                serde_json::Value::String(domain.to_string()),
            );
        }
        if let Some(path) = &self.path() {
            map.insert(
                "path".to_string(),
                serde_json::Value::String(path.to_string()),
            );
        }
        if self.has_secure() {
            map.insert("secure".to_string(), serde_json::Value::Bool(true));
        }
        if self.has_httponly() {
            map.insert("httponly".to_string(), serde_json::Value::Bool(true));
        }
        if let Some(samesite) = &self.samesite() {
            map.insert(
                "samesite".to_string(),
                serde_json::Value::String(samesite.to_string()),
            );
        }
        serde_json::Value::Object(map)
    }
}

impl Certificate {
    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "subject".to_string(),
            serde_json::Value::String(self.subject.clone()),
        );
        map.insert(
            "issue".to_string(),
            serde_json::Value::String(self.issuer.clone()),
        );
        map.insert("start_date".to_string(), json_date(self.start_date));
        map.insert("expire_date".to_string(), json_date(self.expire_date));
        map.insert(
            "serial_number".to_string(),
            serde_json::Value::String(self.serial_number.clone()),
        );
        serde_json::Value::Object(map)
    }
}

impl Timings {
    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "begin_call".to_string(),
            serde_json::Value::String(self.begin_call.to_rfc3339_opts(SecondsFormat::Micros, true)),
        );
        map.insert(
            "end_call".to_string(),
            serde_json::Value::String(self.end_call.to_rfc3339_opts(SecondsFormat::Micros, true)),
        );
        let value = self.name_lookup.as_micros() as u64;
        map.insert(
            "name_lookup".to_string(),
            serde_json::Value::Number(Number::from(value)),
        );
        let value = self.connect.as_micros() as u64;
        map.insert(
            "connect".to_string(),
            serde_json::Value::Number(Number::from(value)),
        );
        let value = self.app_connect.as_micros() as u64;
        map.insert(
            "app_connect".to_string(),
            serde_json::Value::Number(Number::from(value)),
        );
        let value = self.pre_transfer.as_micros() as u64;
        map.insert(
            "pre_transfer".to_string(),
            serde_json::Value::Number(Number::from(value)),
        );
        let value = self.start_transfer.as_micros() as u64;
        map.insert(
            "start_transfer".to_string(),
            serde_json::Value::Number(Number::from(value)),
        );
        let value = self.total.as_micros() as u64;
        map.insert(
            "total".to_string(),
            serde_json::Value::Number(Number::from(value)),
        );
        serde_json::Value::Object(map)
    }
}

impl CaptureResult {
    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "name".to_string(),
            serde_json::Value::String(self.name.clone()),
        );
        map.insert("value".to_string(), self.value.to_json());
        serde_json::Value::Object(map)
    }
}

impl AssertResult {
    fn to_json(
        &self,
        filename: &Input,
        content: &str,
        entry_src_info: SourceInfo,
    ) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        let success = self.error().is_none();
        map.insert("success".to_string(), serde_json::Value::Bool(success));

        if let Some(err) = self.error() {
            let message = logger::error_string(
                &filename.to_string(),
                content,
                &err,
                Some(entry_src_info),
                false,
            );
            map.insert("message".to_string(), serde_json::Value::String(message));
        }
        map.insert(
            "line".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.line())),
        );

        serde_json::Value::Object(map)
    }
}

impl Cookie {
    fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert(
            "domain".to_string(),
            serde_json::Value::String(self.domain.clone()),
        );
        map.insert(
            "include_subdomain".to_string(),
            serde_json::Value::String(self.include_subdomain.clone()),
        );
        map.insert(
            "path".to_string(),
            serde_json::Value::String(self.path.clone()),
        );
        map.insert(
            "https".to_string(),
            serde_json::Value::String(self.https.clone()),
        );
        map.insert(
            "expires".to_string(),
            serde_json::Value::String(self.expires.clone()),
        );
        map.insert(
            "name".to_string(),
            serde_json::Value::String(self.name.clone()),
        );
        map.insert(
            "value".to_string(),
            serde_json::Value::String(self.value.clone()),
        );
        serde_json::Value::Object(map)
    }
}

fn json_date(value: DateTime<Utc>) -> serde_json::Value {
    serde_json::Value::String(value.to_string())
}
