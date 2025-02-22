use std::collections::BTreeMap;

use crate::{kind, Kind};
use surrealdb::sql::{Ident, Literal, Param, Part, Thing, Value};

use super::schema::{QueryState, TableFields};

pub fn get_value_table(
    what_value: &Value,
    state: &mut QueryState,
) -> Result<String, anyhow::Error> {
    match what_value {
        Value::Table(table) => Ok(table.0.clone()),
        Value::Array(array) => {
            // For arrays, just take the table type from the first element
            // since all elements must be from the same table
            if let Some(first) = array.0.first() {
                get_value_table(first, state)
            } else {
                anyhow::bail!("Empty array in relation")
            }
        }
        Value::Param(Param {
            0: Ident { 0: param_ident, .. },
            ..
        }) => match state.get(param_ident.as_str()) {
            Some(Kind::Record(tables)) => Ok(tables[0].0.clone()),
            // Add this case to handle raw record IDs stored in variables
            Some(kind) => Ok(match kind {
                Kind::Record(tables) => tables[0].0.clone(),
                Kind::Option(box Kind::Record(tables)) => tables[0].0.clone(),
                Kind::Literal(Literal::Object(fields)) => {
                    if let Some(Kind::Record(tables)) = fields.get("id") {
                        tables[0].0.clone()
                    } else {
                        anyhow::bail!("Expected record type for param: {}", param_ident)
                    }
                }
                _ => anyhow::bail!("Expected record type for param: {}", param_ident),
            }),
            None => anyhow::bail!("Unknown parameter: {}", param_ident),
        },
        Value::Thing(Thing { tb, .. }) => Ok(tb.clone()),
        Value::Idiom(idiom) => {
            // For graph traversal paths, get the last Graph part's target table
            if let Some(Part::Graph(graph)) = idiom.0.last() {
                Ok(graph.what.0[0].to_string())
            } else {
                anyhow::bail!("Expected graph traversal to end with a target table")
            }
        }
        _ => anyhow::bail!("Expected record type, got: {}", what_value),
    }
}

pub fn get_what_fields(
    what_value: &Value,
    state: &mut QueryState,
) -> Result<TableFields, anyhow::Error> {
    let table_name = get_value_table(what_value, state)?;
    Ok(state.table_select_fields(&table_name)?)
}

pub fn merge_into_map_recursively(
    map: &mut BTreeMap<String, Kind>,
    parts: &[Part],
    return_type: Kind,
) -> Result<(), anyhow::Error> {
    if parts.is_empty() {
        return Ok(());
    }

    match &parts[0] {
        Part::Field(field_name) => {
            if parts.len() == 1 {
                map.insert(field_name.0.clone(), return_type);
            } else {
                // check if the return type is a double optional, because something like xyz.abc returns option<option<string>> if xyz and abc are both optional
                if is_double_optional(&return_type) {
                    let next_map = map
                        .entry(field_name.to_string())
                        .or_insert_with(|| kind!(Opt(kind!({}))));

                    match next_map {
                        Kind::Option(box Kind::Literal(Literal::Object(nested_fields))) => {
                            merge_into_map_recursively(
                                nested_fields,
                                &parts[1..],
                                match return_type {
                                    Kind::Option(return_type) => *return_type,
                                    _ => anyhow::bail!("Expected Option, got {:?}", return_type),
                                },
                            )?
                        }
                        // Kind::Literal(Literal::Object(nested_fields)) => {
                        //     merge_into_map_recursively(
                        //         nested_fields,
                        //         &parts[1..],
                        //         kind!(Opt(return_type)),
                        //     )?
                        // }
                        // TODO: If we have something like SELECT *, xyz.abc FROM xyz, it will fail because it thinks `xyz` is already a record
                        // it instead needs to replace here
                        // Kind::Option(box Kind::Record(tables)) => {
                        //     merge_into_map_recursively(
                        //         tables[0].1.clone(),
                        //         &parts[1..],
                        //         return_type.expect_option()?,
                        //     )?
                        // }
                        _ => anyhow::bail!("Unsupported field return type: {:?}", next_map),
                    }
                } else {
                    let next_map = map
                        .entry(field_name.to_string())
                        .or_insert_with(|| kind!({}));

                    match next_map {
                        Kind::Literal(Literal::Object(nested_fields)) => {
                            merge_into_map_recursively(nested_fields, &parts[1..], return_type)?
                        }
                        _ => anyhow::bail!("Unsupported field return type: {:?}", next_map),
                    }
                }
            }
        }
        Part::All => {
            if let Some(Part::Field(ident)) = parts.get(1) {
                map.insert(ident.to_string(), kind!(Arr return_type));
            } else {
                map.insert("*".to_string(), kind!(Arr return_type));
            }
        }
        _ => anyhow::bail!(
            "Unsupported part in merge_into_map_recursively: {:?}",
            parts
        ),
    }

    Ok(())
}

pub fn is_double_optional(return_type: &Kind) -> bool {
    match return_type {
        Kind::Option(return_type) => match **return_type {
            Kind::Option(_) => true,
            _ => false,
        },
        _ => false,
    }
}
