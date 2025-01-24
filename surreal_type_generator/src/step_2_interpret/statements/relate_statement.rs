use crate::{
    kind,
    step_2_interpret::{get_statement_fields, schema::QueryState, utils::get_value_table},
    Kind,
};
use surrealdb::sql::{statements::RelateStatement, Data, Fields, Output, Table};

pub fn get_relate_statement_return_type(
    relate: &RelateStatement,
    state: &mut QueryState,
) -> Result<Kind, anyhow::Error> {
    // Get the relation table name
    let relation_table = get_value_table(&relate.kind, state)?;

    // Handle output types
    let return_type = match &relate.output {
        Some(Output::After) | None => get_relate_fields(relate, state, None)?,
        Some(Output::Before) => Kind::Null, // Relations are new, so before is always null
        Some(Output::Null) => Kind::Null,
        Some(Output::None) => Kind::Null,
        Some(Output::Fields(fields)) => get_relate_fields(relate, state, Some(fields))?,
        Some(Output::Diff) => anyhow::bail!("Relate with returned diff is not currently supported"),
        Some(_) => anyhow::bail!("Unsupported output type for RELATE statement"),
    };

    // Handle content validation and variable inference
    if let Some(data) = &relate.data {
        validate_data_type(state, &relation_table, data)?;
    }

    // Relations always return arrays since they can create multiple relationships
    Ok(kind!([return_type]))
}

fn get_relate_fields(
    relate: &RelateStatement,
    state: &mut QueryState,
    fields: Option<&Fields>,
) -> Result<Kind, anyhow::Error> {
    let relation_table = get_value_table(&relate.kind, state)?;
    let in_table = get_value_table(&relate.from, state)?;
    let out_table = get_value_table(&relate.with, state)?;

    // Get base relation fields from schema
    let mut relation_fields = state.table_select_fields(&relation_table)?;

    // Add standard relation fields - using Table type and vec! for Record kinds
    relation_fields.insert("id".into(), Kind::Record(vec![Table::from(relation_table)]));
    relation_fields.insert("in".into(), Kind::Record(vec![Table::from(in_table)]));
    relation_fields.insert("out".into(), Kind::Record(vec![Table::from(out_table)]));

    // If specific fields were requested, filter to just those
    if let Some(fields) = fields {
        let filtered_fields = get_statement_fields(
            &[relate.kind.clone()],
            state,
            Some(fields),
            |fields, state| {
                state.set_local("this", kind!(Obj fields.clone()));
            },
        )?;

        match filtered_fields {
            Kind::Literal(surrealdb::sql::Literal::Object(fields)) => Ok(kind!(Obj fields)),
            _ => anyhow::bail!("Expected object type for relation fields"),
        }
    } else {
        Ok(kind!(Obj relation_fields))
    }
}

fn validate_data_type(
    state: &mut QueryState,
    relation_table: &str,
    data: &Data,
) -> Result<(), anyhow::Error> {
    match data {
        Data::ContentExpression(value) => {
            if let surrealdb::sql::Value::Param(param) = value {
                let table = state
                    .schema
                    .schema
                    .tables
                    .get(relation_table)
                    .ok_or_else(|| anyhow::anyhow!("Unknown relation table: {}", relation_table))?;

                let create_fields = kind!(Obj table.compute_create_fields()?);

                state.infer(
                    &param.0.to_string(),
                    kind!(Either [create_fields.clone(), kind!([create_fields])]),
                );
            }
            Ok(())
        }
        // TODO: Handle SET expressions
        _ => Ok(()),
    }
}
