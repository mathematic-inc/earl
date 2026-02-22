use anyhow::{Result, bail};
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;
use sxd_document::parser;
use sxd_xpath::{Context as XPathContext, Factory, Value as XPathValue};

use earl_core::schema::ResultExtract;

use earl_core::decode::DecodedBody;

pub fn extract_result(extract: Option<&ResultExtract>, decoded: &DecodedBody) -> Result<Value> {
    let Some(extract) = extract else {
        return Ok(decoded.to_json_value());
    };

    match extract {
        ResultExtract::JsonPointer { json_pointer } => extract_json_pointer(decoded, json_pointer),
        ResultExtract::Regex { regex } => extract_regex(decoded, regex),
        ResultExtract::CssSelector { css_selector } => extract_css(decoded, css_selector),
        ResultExtract::XPath { xpath } => extract_xpath(decoded, xpath),
    }
}

fn extract_json_pointer(decoded: &DecodedBody, pointer: &str) -> Result<Value> {
    let json = decoded
        .as_json()
        .ok_or_else(|| anyhow::anyhow!("json_pointer extraction requires decoded JSON body"))?;

    json.pointer(pointer)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("json_pointer `{pointer}` did not match response body"))
}

fn extract_regex(decoded: &DecodedBody, pattern: &str) -> Result<Value> {
    let text = decoded
        .as_text()
        .ok_or_else(|| anyhow::anyhow!("regex extraction requires decoded text/html/xml body"))?;

    let regex = Regex::new(pattern)?;
    let captures = regex
        .captures(text)
        .ok_or_else(|| anyhow::anyhow!("regex pattern did not match response body"))?;

    if captures.len() > 1 {
        Ok(Value::String(
            captures
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default(),
        ))
    } else {
        Ok(Value::String(
            captures
                .get(0)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default(),
        ))
    }
}

fn extract_css(decoded: &DecodedBody, selector: &str) -> Result<Value> {
    let html = decoded.as_text().ok_or_else(|| {
        anyhow::anyhow!("css_selector extraction requires decoded html/text body")
    })?;

    let doc = Html::parse_document(html);
    let selector = Selector::parse(selector)
        .map_err(|err| anyhow::anyhow!("invalid css selector `{selector}`: {err}"))?;

    let mut results = Vec::new();
    for node in doc.select(&selector) {
        let text = node.text().collect::<Vec<_>>().join(" ").trim().to_string();
        if !text.is_empty() {
            results.push(Value::String(text));
        }
    }

    Ok(Value::Array(results))
}

fn extract_xpath(decoded: &DecodedBody, xpath: &str) -> Result<Value> {
    let xml = decoded
        .as_text()
        .ok_or_else(|| anyhow::anyhow!("xpath extraction requires decoded xml/text body"))?;

    let package = parser::parse(xml).map_err(|err| anyhow::anyhow!("invalid XML: {err}"))?;
    let doc = package.as_document();

    let factory = Factory::new();
    let xpath = factory
        .build(xpath)?
        .ok_or_else(|| anyhow::anyhow!("failed to compile xpath expression"))?;

    let context = XPathContext::new();
    let value = xpath.evaluate(&context, doc.root())?;

    match value {
        XPathValue::Nodeset(nodes) => {
            let values = nodes
                .document_order()
                .into_iter()
                .map(|node| Value::String(node.string_value()))
                .collect();
            Ok(Value::Array(values))
        }
        XPathValue::Boolean(v) => Ok(Value::Bool(v)),
        XPathValue::Number(v) => {
            if let Some(n) = serde_json::Number::from_f64(v) {
                Ok(Value::Number(n))
            } else {
                bail!("xpath returned invalid non-finite number")
            }
        }
        XPathValue::String(v) => Ok(Value::String(v)),
    }
}
