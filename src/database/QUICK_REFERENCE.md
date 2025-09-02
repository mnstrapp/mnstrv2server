# Database Macros Quick Reference

A quick reference guide for the database macros system.

## Quick Start

```rust
use crate::database::traits::DatabaseResource;
use sqlx::{postgres::PgRow, Error};

// 1. Define your struct
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

// 2. Implement DatabaseResource
impl DatabaseResource for User {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        Ok(User {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
            name: row.try_get("name")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            archived_at: row.try_get("archived_at")?,
        })
    }
    fn has_id() -> bool { true }
    fn is_archivable() -> bool { true }
    fn is_updatable() -> bool { true }
    fn is_creatable() -> bool { true }
    fn is_expirable() -> bool { false }
    fn is_verifiable() -> bool { false }
}
```

## Macro Reference

### Query Macros

| Macro | Purpose | Returns |
|-------|---------|---------|
| `find_all_resources_where_fields!` | Find all resources | `Result<Vec<Resource>, Error>` |
| `find_all_unarchived_resources_where_fields!` | Find unarchived resources | `Result<Vec<Resource>, Error>` |
| `find_all_archived_resources_where_fields!` | Find archived resources | `Result<Vec<Resource>, Error>` |
| `find_one_resource_where_fields!` | Find single resource | `Result<Resource, Error>` |
| `find_one_unarchived_resource_where_fields!` | Find single unarchived | `Result<Resource, Error>` |
| `find_one_archived_resource_where_fields!` | Find single archived | `Result<Resource, Error>` |

### CRUD Macros

| Macro | Purpose | Returns |
|-------|---------|---------|
| `insert_resource!` | Create resource | `Result<Resource, Error>` |
| `update_resource!` | Update by ID | `Result<Resource, Error>` |
| `delete_resource_where_fields!` | Delete resources | `Result<(), Error>` |

### Join Macros

| Macro | Purpose | Returns |
|-------|---------|---------|
| `join_all_resources_where_fields_on!` | JOIN two resources | `Result<Vec<Resource>, Error>` |

## Common Patterns

### Basic CRUD

```rust
// Create
let params = vec![
    ("email", "user@example.com".into()),
    ("name", "John Doe".into())
];
let user = insert_resource!(User, params).await?;

// Read
let params = vec![("id", user.id.clone().into())];
let found_user = find_one_resource_where_fields!(User, params).await?;

// Update
let update_params = vec![("name", "Jane Doe".into())];
let updated_user = update_resource!(User, user.id, update_params).await?;

// Delete
let delete_params = vec![("id", user.id.into())];
delete_resource_where_fields!(User, delete_params).await?;
```

### Finding Resources

```rust
// Find by single field
let params = vec![("email", "user@example.com".into())];
let user = find_one_resource_where_fields!(User, params).await?;

// Find by multiple fields
let params = vec![
    ("organization_id", "org_123".into()),
    ("status", "active".into())
];
let users = find_all_resources_where_fields!(User, params).await?;

// Find only active (unarchived) resources
let params = vec![("organization_id", "org_123".into())];
let active_users = find_all_unarchived_resources_where_fields!(User, params).await?;

// Find only deleted (archived) resources
let params = vec![("organization_id", "org_123".into())];
let deleted_users = find_all_archived_resources_where_fields!(User, params).await?;
```

### Joins

```rust
// Join users with roles
let params = vec![("organization_id", "org_123".into())];
let users_with_roles = join_all_resources_where_fields_on!(User, Role, params).await?;
```

## DatabaseValue Conversions

```rust
// Strings
let value: DatabaseValue = "hello".into();
let value: DatabaseValue = String::from("world").into();

// Numbers
let value: DatabaseValue = 42i64.into();
let value: DatabaseValue = 3.14f64.into();

// Booleans
let value: DatabaseValue = true.into();

// DateTime
let value: DatabaseValue = OffsetDateTime::now_utc().into();

// Null
let value = DatabaseValue::None;
```

## Error Handling

```rust
// Basic error handling
match find_one_resource_where_fields!(User, params).await {
    Ok(user) => println!("Found user: {:?}", user),
    Err(e) => eprintln!("Database error: {}", e),
}

// With custom error types
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

async fn find_user_safe(email: &str) -> Result<User, UserError> {
    let params = vec![("email", email.into())];

    match find_one_resource_where_fields!(User, params).await {
        Ok(user) => Ok(user),
        Err(sqlx::Error::RowNotFound) => {
            Err(UserError::NotFound(email.to_string()))
        }
        Err(e) => Err(UserError::Database(e))
    }
}
```

## Performance Tips

### Indexing

```sql
-- Common indexes for performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_organization_id ON users(organization_id);
CREATE INDEX idx_users_archived_at ON users(archived_at);

-- Composite index for common queries
CREATE INDEX idx_users_org_status ON users(organization_id, status)
WHERE archived_at IS NULL;
```

### Connection Pooling

```rust
// Configure connection pool
PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(30))
```

### Query Logging

```rust
// Enable SQL query logging
std::env::set_var("RUST_LOG", "sqlx::query=debug");
```

## Troubleshooting

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `relation "users" does not exist` | Table name mismatch | Check naming: `User` → `users` |
| `trait bound User: DatabaseResource is not satisfied` | Missing trait impl | Implement `DatabaseResource` |
| `connection pool exhausted` | Too many connections | Increase pool size |
| `the trait bound String: Into<DatabaseValue> is not satisfied` | Type conversion | Use explicit conversion |

### Debug Queries

```rust
// Enable query logging to see generated SQL
std::env::set_var("SQLX_OFFLINE", "false");
std::env::set_var("RUST_LOG", "sqlx::query=debug");
```

## Table Naming

| Rust Struct | Database Table |
|-------------|----------------|
| `User` | `users` |
| `OrganizationLocation` | `organization_locations` |
| `BillingPlan` | `billing_plans` |

## Trait Methods

| Method | Purpose | Example |
|--------|---------|---------|
| `has_id()` | Auto-generate UUID | `true` for most resources |
| `is_archivable()` | Support soft delete | `true` for user data |
| `is_updatable()` | Auto-update timestamp | `true` for most resources |
| `is_creatable()` | Auto-create timestamp | `true` for most resources |
| `is_expirable()` | Auto-expire in 30 days | `false` for user data |
| `is_verifiable()` | Support verification | `false` (reserved) |

## Best Practices

1. ✅ Always handle errors with `?` or `match`
2. ✅ Use unarchived variants for active data
3. ✅ Add database indexes for performance
4. ✅ Use transactions for multiple operations
5. ✅ Validate input before database operations
6. ✅ Test your database operations thoroughly
7. ✅ Monitor query performance with logging
8. ✅ Keep schema and trait implementations in sync

## Environment Variables

```bash
# Required
DATABASE_URL=postgresql://user:pass@localhost:5432/database

# Optional (for debugging)
RUST_LOG=sqlx::query=debug
SQLX_OFFLINE=false
```
