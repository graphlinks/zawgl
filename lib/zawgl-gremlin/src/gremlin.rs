// MIT License
//
// Copyright (c) 2022 Alexandre RICCIARDI
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::collections::HashMap;
use std::convert::TryFrom;
use serde_json::json;

pub trait ToJson {
    fn to_json(&self) -> serde_json::Value;
}

#[derive(Debug, Clone)]
pub enum GStep {
    AddV(String),
    V(Option<GValueOrVertex>),
    InV,
    Has(String, GPredicate),
    AddE(String),
    E(Option<GValue>),
    OutE(Vec<String>),
    As(String),
    From(GValueOrVertex),
    To(GValueOrVertex),
    Match(Vec<Vec<GStep>>),
    SetProperty(String, GValue),
    SetDynProperty(String, Vec<GStep>),
    Commit,
    Empty,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GSource {
    TxCommit,
    Empty,
}

pub enum GBytecode {
    Steps(Vec<GStep>),
    Source(GSource),
}

#[derive(Debug, PartialEq, Clone)]
pub enum GValue {
    Integer(GInteger),
    Double(GDouble),
    String(String),
    Bool(bool),
}

impl GValue {
    pub fn as_str(&self) -> Option<&str> {
        match &self {
            GValue::String(sval) => {Some(sval)}
            _ => {None}
        }
    }
}

impl ToJson for GValue {
    fn to_json(&self) -> serde_json::Value {
        match self {
            GValue::Integer(v) => {
                v.to_json()
            },
            GValue::String(v) => {
                json!(v)
            },
            GValue::Bool(v) => {
                json!(v)
            }
            GValue::Double(v) => {
                v.to_json()
            }
        }
    }
}

impl TryFrom<GValue> for u64 {
    type Error = &'static str;

    fn try_from(value: GValue) -> Result<Self, Self::Error> {
        match &value {
            GValue::Integer(v) => {
                match v {
                    GInteger::I32(ivalue) => {
                        Ok(ivalue.0 as u64)
                    },
                    GInteger::I64(ivalue) => {
                        u64::try_from(ivalue.0).map_err(|_| "Error casting to unsigned integer")
                    }
                }
            },
            GValue::String(sv) => {
                sv.parse().map_err(|_| "Error parsing integer")
            }
            _ => {
                Err("Cannot parse GValue as Integer")
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GProperty {
    pub name: String,
    pub values: Vec<(GInt64, GValue)>,
}

impl ToJson for GProperty {
    fn to_json(&self) -> serde_json::Value {
        let mut array = Vec::new();
        for e in &self.values {
            array.push(json!({
                "id": e.0.to_json(),
                "value": e.1.to_json(),
            }));
        }
        json!({
            self.name.clone(): array
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GProperties {
    pub properties: Vec<GProperty>, 
}

impl GProperties {
    pub fn new() -> Self {
        GProperties { properties: Vec::new() }
    }
}

impl ToJson for GProperties {
    fn to_json(&self) -> serde_json::Value {
        let mut props = HashMap::new();
        for p in &self.properties {
            let mut array = Vec::new();
            for e in &p.values {
                array.push(json!({
                    "id": e.0.to_json(),
                    "value": e.1.to_json(),
                }));
            }
            props.insert(p.name.clone(), json!(array));
        }
        json!(props)
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GInteger {
    I64(GInt64),
    I32(GInt32),
}

impl ToJson for GInteger {
    fn to_json(&self) -> serde_json::Value {
        match self {
            GInteger::I64(v) => {
                v.to_json()
            },
            GInteger::I32(v) => {
                v.to_json()
            }
        }
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GDouble(pub f64);

impl ToJson for GDouble {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "@type": "g:Double",
            "@value": self.0,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GInt64(pub i64);

impl GInt64 {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "@type": "g:Int64",
            "@value": self.0,
        })
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GInt32(pub i32);

impl GInt32 {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "@type": "g:Int32",
            "@value": self.0,
        })
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct GList<T: ToJson> {
    pub values: Vec<T>,
}

impl <T: ToJson> GList<T> {
    pub fn new() -> Self {
        GList{values: Vec::new()}
    }
}
impl <T: ToJson> ToJson for GList<T> {
    
    fn to_json(&self) -> serde_json::Value {
        let mut array = Vec::new();
        for e in &self.values {
            array.push(e.to_json());
        }
        json!({
            "@type": "g:List",
            "@value": array
        })
    }
}

pub struct GEdge {
    pub id: GInt64,
    pub label: String,
    pub in_v_label: String,
    pub out_v_abel: String,
    pub in_v: GInt64,
    pub out_v: GInt64,
    pub properties: GProperties,
}

impl ToJson for GEdge {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "@type": "g:Edge",
            "@value": {
                "id": self.id.to_json(),
                "label": self.label,
                "inVLabel": self.in_v_label,
                "outVLabel": self.out_v_abel,
                "inV": self.in_v.to_json(),
                "outV": self.out_v.to_json(),
                "properties": self.properties.to_json(),
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum GPredicate {
    Value(GValue),
    Within(GList<GValue>),
}

pub struct GremlinRequest {
    pub request_id: String,
    pub data: Option<GremlinRequestData>,
    pub session: Option<GremlinSession>,
}

pub struct GremlinRequestData {
    pub steps: Vec<GStep>,
}

pub struct GremlinSession {
    pub session_id: String,
    pub manage_transaction: bool,
    pub maintain_state_after_exception: bool,
    pub commit: bool, 
}

pub struct GremlinResponse {
    pub request_id: String,
    pub status: GStatus,
    pub result: GResult,
}

impl ToJson for GremlinResponse {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "requestId": self.request_id,
            "status": self.status.to_json(),
            "result": self.result.to_json(),
        })
    }
}

pub struct GStatus {
    pub message: String,
    pub code: i32,
    pub attributes: GMap,
}

impl GStatus {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "message": self.message,
            "code": self.code,
            "attributes": self.attributes.to_json(),
        })
    }
}

pub struct GTraverser {
    pub bulk: GInt64,
    pub value: GItem,
}

impl ToJson for GTraverser {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "@type": "g:Traverser",
            "@value": {
                "bulk": self.bulk.to_json(),
                "value": self.value.to_json(),
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GVertex {
    pub id: GValue,
    pub label: String,
    pub properties: GProperties,
}

pub enum GItem {
    Vertex(GVertex),
    Edge(GEdge),
}


#[derive(Debug, PartialEq, Clone)]
pub enum GValueOrVertex {
    Value(GValue),
    Vertex(GVertex),
}

impl ToJson for GValueOrVertex {
    fn to_json(&self) -> serde_json::Value {
        match self {
            GValueOrVertex::Vertex(v) => {
                v.to_json()
            },
            GValueOrVertex::Value(v) => {
                v.to_json()
            }
        }
    }
}

impl ToJson for GItem {
    fn to_json(&self) -> serde_json::Value {
        match self {
            GItem::Vertex(v) => {
                v.to_json()
            },
            GItem::Edge(e) => {
                e.to_json()
            }
        }
    }
}

impl ToJson for GVertex {
    fn to_json(&self) -> serde_json::Value {
        json!({
            "@type": "g:Vertex",
            "@value": {
                "id": self.id.to_json(),
                "label": self.label,
                "properties": self.properties.to_json(),
            }
        })
    }
}

pub struct GMap {
    pub map: HashMap<String, String>,
}

impl GMap {
    pub fn new() -> Self {
        GMap{map: HashMap::new()}
    }

    fn to_json(&self) -> serde_json::Value {
        let mut res = Vec::new();
        for e in &self.map {
            res.push(e.0);
            res.push(e.1);
        }
        json!({
            "@type": "g:Map",
            "@value": res
        })
    }
}

pub struct GResult {
    pub data: GList<GTraverser>,
    pub meta: GMap,
}

impl GResult {
    pub fn new() -> Self {
        GResult{data: GList::new(), meta: GMap::new()}
    }

    fn to_json(&self) -> serde_json::Value {
        json!({
            "data": self.data.to_json(),
            "meta": self.meta.to_json()
        })
    }
}