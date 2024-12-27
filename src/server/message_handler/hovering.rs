use crate::server::{
    anaysis::get_kind_at_position,
    lsp::{errors::ResponseError, HoverRequest, HoverResponse},
    Server,
};

pub fn handle_hover_request(
    server: &mut Server,
    request: HoverRequest,
) -> Result<HoverResponse, ResponseError> {
    let node_kind = get_kind_at_position(
        &server.state,
        request.get_document_uri(),
        request.get_position(),
    );
    Ok(HoverResponse::new(
        request.get_id(),
        documentation(node_kind.unwrap_or("")),
    ))
}

fn documentation(kind: &str) -> String {
    match kind {
        "FILTER" => {
            r#"
### **FILTER**

The `FILTER` keyword is used to restrict the results by applying a boolean condition.

---

# **Example:**

```sparql
SELECT ?name WHERE {
  ?person foaf:name ?name .
  ?person foaf:age ?age .
  FILTER (?age > 20)
}
```"#
        }
        "PREFIX" => {
            r#"
### **PREFIX**

The `PREFIX` keyword defines a namespace prefix to simplify the use of URIs in the query.

---

**Example:**

```sparql
PREFIX foaf: <http://xmlns.com/foaf/0.1/>

SELECT ?name
WHERE {
  ?person foaf:name ?name .
}
```"#
        }
        _ => kind,
    }
    .to_string()
}
