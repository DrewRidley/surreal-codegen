use pretty_assertions_sorted::assert_eq_sorted;
use surreal_type_generator::{kind, var_map, QueryResult};

#[test]
fn basic_relate_statement() -> anyhow::Result<()> {
    let query = r#"
        RELATE user:john->works_at->company:acme;
    "#;

    let schema = r#"
        DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD name ON user TYPE string;

        DEFINE TABLE company SCHEMAFULL;
            DEFINE FIELD name ON company TYPE string;

        DEFINE TABLE works_at SCHEMAFULL
            TYPE RELATION;
    "#;

    let QueryResult { return_types, .. } =
        surreal_type_generator::step_3_codegen::query_to_return_type(query, schema)?;

    let expected = vec![kind!([kind!({
        id: kind!(Record["works_at"]),
        in: kind!(Record["user"]),
        out: kind!(Record["company"])
    })])];

    pretty_assertions_sorted::assert_eq_sorted!(return_types, expected);

    Ok(())
}

#[test]
fn relate_with_content() -> anyhow::Result<()> {
    let query = r#"
        RELATE user:john->works_at->company:acme
        CONTENT {
            start_date: time::now(),
            position: "Engineer"
        };
    "#;

    let schema = r#"
        DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD name ON user TYPE string;

        DEFINE TABLE company SCHEMAFULL;
            DEFINE FIELD name ON company TYPE string;

        DEFINE TABLE works_at SCHEMAFULL
            TYPE RELATION;
            DEFINE FIELD start_date ON works_at TYPE datetime;
            DEFINE FIELD position ON works_at TYPE string;
    "#;

    let QueryResult { return_types, .. } =
        surreal_type_generator::step_3_codegen::query_to_return_type(query, schema)?;

    let expected = vec![kind!([kind!({
        id: kind!(Record["works_at"]),
        in: kind!(Record["user"]),
        out: kind!(Record["company"]),
        start_date: kind!(Datetime),
        position: kind!(String)
    })])];

    pretty_assertions_sorted::assert_eq_sorted!(return_types, expected);

    Ok(())
}

#[test]
fn relate_with_set() -> anyhow::Result<()> {
    let query = r#"
        RELATE user:john->works_at->company:acme
        SET
            start_date = time::now(),
            position = "Engineer";
    "#;

    let schema = r#"
        DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD name ON user TYPE string;

        DEFINE TABLE company SCHEMAFULL;
            DEFINE FIELD name ON company TYPE string;

        DEFINE TABLE works_at SCHEMAFULL
            TYPE RELATION;
            DEFINE FIELD start_date ON works_at TYPE datetime;
            DEFINE FIELD position ON works_at TYPE string;
    "#;

    let QueryResult { return_types, .. } =
        surreal_type_generator::step_3_codegen::query_to_return_type(query, schema)?;

    let expected = vec![kind!([kind!({
        id: kind!(Record["works_at"]),
        in: kind!(Record["user"]),
        out: kind!(Record["company"]),
        start_date: kind!(Datetime),
        position: kind!(String)
    })])];

    pretty_assertions_sorted::assert_eq_sorted!(return_types, expected);

    Ok(())
}

#[test]
fn relate_multiple_sources() -> anyhow::Result<()> {
    let query = r#"
        RELATE [user:john, user:jane]->works_at->company:acme;
    "#;

    let schema = r#"
        DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD name ON user TYPE string;

        DEFINE TABLE company SCHEMAFULL;
            DEFINE FIELD name ON company TYPE string;

        DEFINE TABLE works_at SCHEMAFULL
            TYPE RELATION;
    "#;

    let QueryResult { return_types, .. } =
        surreal_type_generator::step_3_codegen::query_to_return_type(query, schema)?;

    let expected = vec![kind!([kind!({
        id: kind!(Record["works_at"]),
        in: kind!(Record["user"]),
        out: kind!(Record["company"])
    })])];

    pretty_assertions_sorted::assert_eq_sorted!(return_types, expected);

    Ok(())
}

#[test]
fn relate_multiple_targets() -> anyhow::Result<()> {
    let query = r#"
        RELATE user:john->works_at->[company:acme, company:startup];
    "#;

    let schema = r#"
        DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD name ON user TYPE string;

        DEFINE TABLE company SCHEMAFULL;
            DEFINE FIELD name ON company TYPE string;

        DEFINE TABLE works_at SCHEMAFULL
            TYPE RELATION;
    "#;

    let QueryResult { return_types, .. } =
        surreal_type_generator::step_3_codegen::query_to_return_type(query, schema)?;

    let expected = vec![kind!([kind!({
        id: kind!(Record["works_at"]),
        in: kind!(Record["user"]),
        out: kind!(Record["company"])
    })])];

    pretty_assertions_sorted::assert_eq_sorted!(return_types, expected);

    Ok(())
}

#[test]
fn relate_with_variable_content() -> anyhow::Result<()> {
    let query = r#"
        RELATE user:john->works_at->company:acme CONTENT $data;
    "#;

    let schema = r#"
        DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD name ON user TYPE string;

        DEFINE TABLE company SCHEMAFULL;
            DEFINE FIELD name ON company TYPE string;

        DEFINE TABLE works_at SCHEMAFULL
            TYPE RELATION;
            DEFINE FIELD start_date ON works_at TYPE datetime;
            DEFINE FIELD position ON works_at TYPE string;
    "#;

    let QueryResult {
        return_types,
        variables,
        ..
    } = surreal_type_generator::step_3_codegen::query_to_return_type(query, schema)?;

    let relation_vars = kind!({
        id: kind!(Opt(kind!(Record["works_at"]))),
        start_date: kind!(Datetime),
        position: kind!(String)
    });

    assert_eq_sorted!(
        variables,
        var_map! {
            data: kind!(Either [
                relation_vars.clone(),
                kind!([relation_vars])
            ])
        }
    );

    let expected = vec![kind!([kind!({
        id: kind!(Record["works_at"]),
        in: kind!(Record["user"]),
        out: kind!(Record["company"]),
        start_date: kind!(Datetime),
        position: kind!(String)
    })])];

    pretty_assertions_sorted::assert_eq_sorted!(return_types, expected);

    Ok(())
}

#[test]
fn relate_with_return_fields() -> anyhow::Result<()> {
    let query = r#"
        RELATE user:john->works_at->company:acme
        CONTENT {
            start_date: time::now(),
            position: "Engineer"
        }
        RETURN start_date, position;
    "#;

    let schema = r#"
        DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD name ON user TYPE string;

        DEFINE TABLE company SCHEMAFULL;
            DEFINE FIELD name ON company TYPE string;

        DEFINE TABLE works_at SCHEMAFULL
            TYPE RELATION;
            DEFINE FIELD start_date ON works_at TYPE datetime;
            DEFINE FIELD position ON works_at TYPE string;
    "#;

    let QueryResult { return_types, .. } =
        surreal_type_generator::step_3_codegen::query_to_return_type(query, schema)?;

    let expected = vec![kind!([kind!({
        start_date: kind!(Datetime),
        position: kind!(String)
    })])];

    pretty_assertions_sorted::assert_eq_sorted!(return_types, expected);

    Ok(())
}

#[test]
fn relate_multiple_with_content() -> anyhow::Result<()> {
    let query = r#"
        RELATE [user:john, user:jane]->works_at->[company:acme, company:startup]
        CONTENT {
            start_date: time::now(),
            position: "Engineer"
        };
    "#;

    let schema = r#"
        DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD name ON user TYPE string;

        DEFINE TABLE company SCHEMAFULL;
            DEFINE FIELD name ON company TYPE string;

        DEFINE TABLE works_at SCHEMAFULL
            TYPE RELATION;
            DEFINE FIELD start_date ON works_at TYPE datetime;
            DEFINE FIELD position ON works_at TYPE string;
    "#;

    let QueryResult { return_types, .. } =
        surreal_type_generator::step_3_codegen::query_to_return_type(query, schema)?;

    let expected = vec![kind!([kind!({
        id: kind!(Record["works_at"]),
        in: kind!(Record["user"]),
        out: kind!(Record["company"]),
        start_date: kind!(Datetime),
        position: kind!(String)
    })])];

    pretty_assertions_sorted::assert_eq_sorted!(return_types, expected);

    Ok(())
}
