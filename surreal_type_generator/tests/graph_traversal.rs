use surreal_type_generator::{kind, var_map, Kind, Literal, QueryResult};

#[test]
fn graph_traversal() -> anyhow::Result<()> {
    let query = r#"
        SELECT ->memberOf->org.* as orgs FROM user;
    "#;

    let schema = r#"
        DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD name ON user TYPE string;
            DEFINE FIELD email ON user TYPE string;

        DEFINE TABLE memberOf SCHEMAFULL
            TYPE RELATION FROM user TO org;

        DEFINE TABLE org SCHEMAFULL;
            DEFINE FIELD name ON org TYPE string;
            DEFINE FIELD revenue ON org TYPE float;
    "#;

    let QueryResult {
        statements,
        return_types,
        ..
    } = surreal_type_generator::step_3_codegen::query_to_return_type(query, schema)?;

    // Build expected type - updated to match actual structure
    let org_fields = kind!({
        id: kind!(Record["org"]),
        name: kind!(String),
        revenue: kind!(Float)
    });

    let orgs_array = kind!([org_fields]);

    let outer_obj = kind!({
        orgs: orgs_array
    });

    let expected = vec![kind!([outer_obj])];

    pretty_assertions_sorted::assert_eq_sorted!(return_types, expected);

    Ok(())
}

#[test]
fn graph_traversal_with_auth_and_subquery() -> anyhow::Result<()> {
    let query = r#"
        SELECT *,
            (SELECT * FROM bucket WHERE workspace = workspace.id) AS buckets
        FROM $auth->memberOf->workspace;
    "#;

    let schema = r#"
        DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD name ON user TYPE string;
            DEFINE FIELD email ON user TYPE string;

        DEFINE TABLE memberOf SCHEMAFULL
            TYPE RELATION FROM user TO workspace;

        DEFINE TABLE workspace SCHEMAFULL;
            DEFINE FIELD name ON workspace TYPE string;

        DEFINE TABLE bucket SCHEMAFULL;
            DEFINE FIELD workspace ON bucket TYPE record<workspace>;
            DEFINE FIELD name ON bucket TYPE string;
    "#;

    // Print the parsed query structure
    let parsed = surrealdb::sql::parse(query)?;
    println!("\n=== PARSED QUERY ===");
    println!("{:#?}", parsed);

    let globals = var_map! {
        auth: kind!(Record["user"])
    };

    let result = surreal_type_generator::step_3_codegen::query_to_return_type_with_globals(
        query, schema, &globals,
    );

    // Print the error if we get one
    if let Err(ref e) = result {
        println!("\n=== ERROR ===");
        println!("{:#?}", e);
    }

    let QueryResult {
        statements,
        return_types,
        ..
    } = result?;

    // Build expected type
    let bucket_fields = kind!({
        id: kind!(Record["bucket"]),
        workspace: kind!(Record["workspace"]),
        name: kind!(String)
    });

    let workspace_fields = kind!({
        id: kind!(Record["workspace"]),
        name: kind!(String),
        buckets: kind!([bucket_fields])
    });

    let expected = vec![kind!([workspace_fields])];

    println!("\n=== ACTUAL TYPES ===");
    println!("{:#?}", return_types);
    println!("\n=== EXPECTED TYPES ===");
    println!("{:#?}", expected);

    pretty_assertions_sorted::assert_eq_sorted!(return_types, expected);

    Ok(())
}
