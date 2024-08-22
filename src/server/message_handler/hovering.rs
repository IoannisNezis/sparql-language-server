use indoc::indoc;

use crate::{
    analysis::get_kind_at_position,
    lsp::{HoverRequest, HoverResponse},
    server::ServerState,
};

fn documentation(kind: &str) -> String {
    match kind {
        "FILTER" => indoc!(
            "
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
```"
        ),
        "PREFIX" => indoc!(
            "
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
```"
        ),
        _ => kind,
    }
    .to_string()
}

pub fn handle_hover_request(request: &HoverRequest, state: &ServerState) -> HoverResponse {
    let node_kind = get_kind_at_position(
        &state.analysis_state,
        request.get_document_uri(),
        request.get_position(),
    );
    HoverResponse::new(request.get_id(), documentation(node_kind.unwrap_or("")))
}
