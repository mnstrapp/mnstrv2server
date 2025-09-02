# Database Macros Documentation

This module provides a comprehensive set of Rust macros for database operations using SQLx with PostgreSQL. The macros are designed to work with any struct that implements the `DatabaseResource` trait.

## Overview

The database macros system consists of several categories:

- **Query Macros**: For finding and retrieving resources
- **Insert Macros**: For creating new resources
- **Update Macros**: For modifying existing resources
- **Delete Macros**: For removing resources (soft delete via archiving)
- **Join Macros**: For complex queries with table joins
- **Supporting Types**: DatabaseValue enum and DatabaseResource trait

## DatabaseResource Trait

All macros require your struct to implement the `DatabaseResource` trait:

```rust
pub trait DatabaseResource {
    fn from_row(row: &PgRow) -> Result<Self, Error> where Self: Sized;
    fn has_id() -> bool;
    fn is_archivable() -> bool;
    fn is_updatable() -> bool;
    fn is_creatable() -> bool;
    fn is_expirable() -> bool;
    fn is_verifiable() -> bool;
}
```

### Trait Methods

- `from_row()`: Converts a database row to your struct
- `has_id()`: Whether the resource has an ID field (auto-generated UUID)
- `is_archivable()`: Whether the resource supports soft deletion via `archived_at`
- `is_updatable()`: Whether the resource has `updated_at` timestamps
- `is_creatable()`: Whether the resource has `created_at` timestamps
- `is_expirable()`: Whether the resource has `expires_at` timestamps
- `is_verifiable()`: Whether the resource supports verification

## Query Macros

### `find_all_resources_where_fields!`

Finds all resources matching the specified field conditions.

**Signature:**

```rust
find_all_resources_where_fields!($resource:ty, $params:expr)
```

**Parameters:**

- `$resource`: The resource type (must implement DatabaseResource)
- `$params`: Vector of `(&str, DatabaseValue)` tuples for field conditions

**Returns:** `Result<Vec<Resource>, Error>`

**Example:**

```rust
let params = vec![
    ("user_id", "123".into()),
    ("status", "active".into())
];
let results = find_all_resources_where_fields!(User, params).await?;
```

### `find_all_unarchived_resources_where_fields!`

Finds all unarchived resources matching the specified field conditions.

**Signature:**

```rust
find_all_unarchived_resources_where_fields!($resource:ty, $params:expr)
```

**Parameters:**

- `$resource`: The resource type (must implement DatabaseResource)
- `$params`: Vector of `(&str, DatabaseValue)` tuples for field conditions

**Returns:** `Result<Vec<Resource>, Error>`

**Example:**

```rust
let params = vec![("organization_id", "456".into())];
let active_users = find_all_unarchived_resources_where_fields!(User, params).await?;
```

### `find_all_archived_resources_where_fields!`

Finds all archived resources matching the specified field conditions.

**Signature:**

```rust
find_all_archived_resources_where_fields!($resource:ty, $params:expr)
```

**Parameters:**

- `$resource`: The resource type (must implement DatabaseResource)
- `$params`: Vector of `(&str, DatabaseValue)` tuples for field conditions

**Returns:** `Result<Vec<Resource>, Error>`

**Example:**

```rust
let params = vec![("organization_id", "456".into())];
let deleted_users = find_all_archived_resources_where_fields!(User, params).await?;
```

### `find_one_resource_where_fields!`

Finds a single resource matching the specified field conditions.

**Signature:**

```rust
find_one_resource_where_fields!($resource:ty, $params:expr)
```

**Parameters:**

- `$resource`: The resource type (must implement DatabaseResource)
- `$params`: Vector of `(&str, DatabaseValue)` tuples for field conditions

**Returns:** `Result<Resource, Error>`

**Example:**

```rust
let params = vec![("email", "user@example.com".into())];
let user = find_one_resource_where_fields!(User, params).await?;
```

### `find_one_unarchived_resource_where_fields!`

Finds a single unarchived resource matching the specified field conditions.

**Signature:**

```rust
find_one_unarchived_resource_where_fields!($resource:ty, $params:expr)
```

**Parameters:**

- `$resource`: The resource type (must implement DatabaseResource)
- `$params`: Vector of `(&str, DatabaseValue)` tuples for field conditions

**Returns:** `Result<Resource, Error>`

**Example:**

```rust
let params = vec![("id", "789".into())];
let active_user = find_one_unarchived_resource_where_fields!(User, params).await?;
```

### `find_one_archived_resource_where_fields!`

Finds a single archived resource matching the specified field conditions.

**Signature:**

```rust
find_one_archived_resource_where_fields!($resource:ty, $params:expr)
```

**Parameters:**

- `$resource`: The resource type (must implement DatabaseResource)
- `$params`: Vector of `(&str, DatabaseValue)` tuples for field conditions

**Returns:** `Result<Resource, Error>`

**Example:**

```rust
let params = vec![("id", "789".into())];
let deleted_user = find_one_archived_resource_where_fields!(User, params).await?;
```

## Insert Macros

### `insert_resource!`

Creates a new resource in the database.

**Signature:**

```rust
insert_resource!($resource:ty, $params:expr)
```

**Parameters:**

- `$resource`: The resource type (must implement DatabaseResource)
- `$params`: Vector of `(&str, DatabaseValue)` tuples for field values

**Returns:** `Result<Resource, Error>`

**Features:**

- Auto-generates UUID if `has_id()` returns true
- Auto-sets `created_at` timestamp if `is_creatable()` returns true
- Auto-sets `updated_at` timestamp if `is_updatable()` returns true
- Auto-sets `expires_at` timestamp (30 days from now) if `is_expirable()` returns true
- Handles proper SQL type casting for different DatabaseValue types
- Returns the created resource

**Example:**

```rust
let params = vec![
    ("email", "newuser@example.com".into()),
    ("name", "John Doe".into()),
    ("password_hash", "hashed_password".into())
];
let new_user = insert_resource!(User, params).await?;
```

## Update Macros

### `update_resource!`

Updates an existing resource by ID.

**Signature:**

```rust
update_resource!($resource:ty, $id:expr, $params:expr)
```

**Parameters:**

- `$resource`: The resource type (must implement DatabaseResource)
- `$id`: The ID of the resource to update
- `$params`: Vector of `(&str, DatabaseValue)` tuples for field updates

**Returns:** `Result<Resource, Error>`

**Features:**

- Auto-updates `updated_at` timestamp if `is_updatable()` returns true
- Auto-updates `expires_at` timestamp (30 days from now) if `is_expirable()` returns true
- Handles proper SQL type casting for different DatabaseValue types
- Returns the updated resource

**Example:**

```rust
let params = vec![
    ("name", "Jane Doe".into()),
    ("email", "jane@example.com".into())
];
let updated_user = update_resource!(User, "user_id_123", params).await?;
```

## Delete Macros

### `delete_resource_where_fields!`

Deletes resources matching the specified field conditions.

**Signature:**

```rust
delete_resource_where_fields!($resource:ty, $params:expr)
```

**Parameters:**

- `$resource`: The resource type (must implement DatabaseResource)
- `$params`: Vector of `(&str, DatabaseValue)` tuples for field conditions

**Returns:** `Result<(), Error>`

**Features:**

- Performs soft delete (sets `archived_at` timestamp) if `is_archivable()` returns true
- Performs hard delete if `is_archivable()` returns false
- Supports multiple field conditions

**Example:**

```rust
let params = vec![("organization_id", "456".into())];
delete_resource_where_fields!(User, params).await?;
```

## Join Macros

### `join_all_resources_where_fields_on!`

Performs a JOIN query between two resources.

**Signature:**

```rust
join_all_resources_where_fields_on!($resource:ty, $join_resource:ty, $params:expr)
```

**Parameters:**

- `$resource`: The primary resource type (must implement DatabaseResource)
- `$join_resource`: The resource to join with (must implement DatabaseResource)
- `$params`: Vector of `(&str, DatabaseValue)` tuples for WHERE conditions

**Returns:** `Result<Vec<Resource>, Error>`

**Features:**

- Automatically determines table names and join conditions
- Assumes foreign key relationship: `{join_resource}_id` in the primary table
- Supports WHERE conditions on the joined result

**Example:**

```rust
let params = vec![("organization_id", "456".into())];
let users_with_roles = join_all_resources_where_fields_on!(User, Role, params).await?;
```

## DatabaseValue Enum

The `DatabaseValue` enum provides type-safe database values with proper SQL encoding:

```rust
pub enum DatabaseValue {
    None,           // NULL value
    Str(&'static str),      // Static string
    String(String),         // Owned string
    Text(String),           // Text field
    Int(String),            // Integer
    Int64(String),          // Big integer
    Float(String),          // Float
    Boolean(String),        // Boolean
    DateTime(String),       // DateTime
}
```

### From Implementations

The enum provides convenient `From` implementations for common types:

```rust
// String conversions
let value: DatabaseValue = "hello".into();
let value: DatabaseValue = String::from("hello").into();

// Numeric conversions
let value: DatabaseValue = 42i64.into();
let value: DatabaseValue = 3.14f64.into();

// Boolean conversion
let value: DatabaseValue = true.into();

// DateTime conversion
let value: DatabaseValue = OffsetDateTime::now_utc().into();
```

## Usage Patterns

### Basic CRUD Operations

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

### Finding Multiple Resources

```rust
// Find all active users in an organization
let params = vec![
    ("organization_id", "org_123".into()),
    ("status", "active".into())
];
let active_users = find_all_unarchived_resources_where_fields!(User, params).await?;
```

### Complex Queries with Joins

```rust
// Find all users with their roles in a specific organization
let params = vec![("organization_id", "org_123".into())];
let users_with_roles = join_all_resources_where_fields_on!(User, Role, params).await?;
```

## Advanced Usage Patterns

### Batch Operations

```rust
// Batch insert multiple resources
async fn create_multiple_users(users: Vec<(String, String)>) -> Result<Vec<User>, Error> {
    let mut created_users = Vec::new();

    for (email, name) in users {
        let params = vec![
            ("email", email.into()),
            ("name", name.into())
        ];
        let user = insert_resource!(User, params).await?;
        created_users.push(user);
    }

    Ok(created_users)
}
```

### Conditional Updates

```rust
// Update only if certain conditions are met
async fn conditional_update(user_id: &str, new_name: &str) -> Result<Option<User>, Error> {
    // First check if user exists and meets conditions
    let check_params = vec![
        ("id", user_id.into()),
        ("status", "active".into())
    ];

    match find_one_unarchived_resource_where_fields!(User, check_params).await {
        Ok(_) => {
            // User exists and is active, proceed with update
            let update_params = vec![("name", new_name.into())];
            let updated_user = update_resource!(User, user_id, update_params).await?;
            Ok(Some(updated_user))
        }
        Err(_) => Ok(None) // User doesn't exist or isn't active
    }
}
```

### Transaction Support

```rust
use sqlx::Transaction;
use sqlx::Postgres;

async fn transfer_ownership(
    mut tx: Transaction<'_, Postgres>,
    user_id: &str,
    new_org_id: &str
) -> Result<(), Error> {
    // Update user's organization
    let user_params = vec![("organization_id", new_org_id.into())];
    update_resource!(User, user_id, user_params).await?;

    // Update user's role in the new organization
    let role_params = vec![
        ("user_id", user_id.into()),
        ("organization_id", new_org_id.into()),
        ("role", "member".into())
    ];
    insert_resource!(UserRole, role_params).await?;

    tx.commit().await?;
    Ok(())
}
```

### Error Handling Patterns

```rust
// Graceful error handling with custom error types
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Validation error: {0}")]
    Validation(String),
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

## Performance Considerations

### Indexing Strategy

For optimal performance, ensure your database tables have appropriate indexes:

```sql
-- Primary key (usually auto-created)
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_organization_id ON users(organization_id);
CREATE INDEX idx_users_archived_at ON users(archived_at);

-- Composite indexes for common query patterns
CREATE INDEX idx_users_org_status ON users(organization_id, status)
WHERE archived_at IS NULL;
```

### Query Optimization

```rust
// Use specific field selection instead of SELECT *
// (Note: This would require macro modifications)

// Use LIMIT for large result sets
let params = vec![("organization_id", "org_123".into())];
let users = find_all_unarchived_resources_where_fields!(User, params).await?;
let limited_users = users.into_iter().take(100).collect::<Vec<_>>();
```

### Connection Pooling

The macros use connection pooling automatically. Configure your pool size based on your application needs:

```rust
// In your connection.rs file
pub async fn get_connection() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .connect(&database_url)
        .await
        .unwrap()
}
```

## Troubleshooting

### Common Issues

#### 1. Table Name Mismatch

**Problem:** Macro can't find the table

```
Error: relation "users" does not exist
```

**Solution:** Check your table naming convention

```rust
// Your struct: User -> table: users
// Your struct: OrganizationLocation -> table: organization_locations
```

#### 2. Missing DatabaseResource Implementation

**Problem:** Compilation error about missing trait

```
error[E0277]: the trait bound `User: DatabaseResource` is not satisfied
```

**Solution:** Implement the trait for your struct

```rust
impl DatabaseResource for User {
    // ... implementation
}
```

#### 3. Type Conversion Errors

**Problem:** DatabaseValue conversion fails

```
error[E0277]: the trait bound `String: Into<DatabaseValue>` is not satisfied
```

**Solution:** Use explicit conversion

```rust
let params = vec![
    ("name", DatabaseValue::String(name)),
    ("age", age.to_string().into())
];
```

#### 4. Connection Pool Exhaustion

**Problem:** Too many database connections

```
Error: connection pool exhausted
```

**Solution:** Increase pool size or add connection timeouts

```rust
PgPoolOptions::new()
    .max_connections(50)
    .acquire_timeout(Duration::from_secs(30))
```

### Debugging Queries

Enable SQLx query logging to see generated SQL:

```rust
// In your main.rs or lib.rs
use sqlx::migrate::Migrator;

#[tokio::main]
async fn main() {
    // Enable query logging
    std::env::set_var("SQLX_OFFLINE", "false");
    std::env::set_var("RUST_LOG", "sqlx::query=debug");

    // ... rest of your application
}
```

## Migration Guide

### From Raw SQL

If you're migrating from raw SQL queries:

**Before:**

```rust
let user = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE email = $1",
    email
)
.fetch_one(&pool)
.await?;
```

**After:**

```rust
let params = vec![("email", email.into())];
let user = find_one_resource_where_fields!(User, params).await?;
```

### From ORM

If you're migrating from an ORM like Diesel:

**Before:**

```rust
let users = users::table
    .filter(users::organization_id.eq(org_id))
    .filter(users::archived_at.is_null())
    .load::<User>(&conn)?;
```

**After:**

```rust
let params = vec![("organization_id", org_id.into())];
let users = find_all_unarchived_resources_where_fields!(User, params).await?;
```

## Best Practices

1. **Always handle errors**: Use `?` operator or `match` expressions
2. **Use appropriate macros**: Choose between archived/unarchived variants based on your needs
3. **Validate input**: Ensure your DatabaseValue conversions are correct
4. **Use transactions**: For multiple related operations, consider using database transactions
5. **Implement DatabaseResource properly**: Ensure all trait methods return appropriate values for your resource type
6. **Add indexes**: Create database indexes for frequently queried fields
7. **Monitor performance**: Use query logging to identify slow queries
8. **Handle large datasets**: Use pagination or limiting for large result sets
9. **Test thoroughly**: Write tests for your database operations
10. **Document your schema**: Keep your database schema and trait implementations in sync

## Table Naming Convention

The macros automatically convert your Rust struct names to database table names:

- `User` → `users`
- `OrganizationLocation` → `organization_locations`
- `BillingPlan` → `billing_plans`

The conversion uses camelCase to snake_case conversion and pluralization.

## Contributing

When contributing to the database macros system:

1. **Add tests** for new macros or functionality
2. **Update documentation** for any API changes
3. **Follow the existing patterns** for macro implementation
4. **Consider backward compatibility** when making changes
5. **Add examples** for new features

## License

This database macros system is part of the Sidekick project and follows the same licensing terms.
