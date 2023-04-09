superseded by github.com/parth/db-rs

Very rough draft, just getting some ideas down.

# hmdb

An embedded database with the following properties:

+ Read Optimized
+ Persistent
+ Transactional
+ In-Memory
+ Key Value Store
+ Schema defined and enforced in Rust

Collection of traits that when applied to the appropriate struct allow for concurrent access to a collection
of `HashMap`s. Writes are written to a disk using an append only log. Log can be compacted using a snapshot. Snapshots
are written atomically, appends to logs are limited in scope, if they become corrupted. Compaction can happen
automatically on a separate thread or when the application decides it's an appropriate time to do so. Database can be
configured for different consistency guarantees (buffered logs) and can be configured for environments in which multiple
non-cooperative processes are sharing a data directory (file locks + no log buffer). Optimized for latency and a compact
on-disk format.

Everything you know about your schema, you can express to the database at compile time. You can specify what tables
exist and what keys and values those tables have in rust. The database supports anything that implements `serde` traits,
and uses `bincode` for the on-disk format (battle tested local optima for performance and compactness). No need to
manage mapping to and from db-specific types yourself.

## Target Usage

Define a schema:

```rust
schema! {
    SchemaV1 {
        accounts: <Username, Account>,
        files: <Uuid, EncryptedFileMetadata>
    }
}
```

Under the hood, this generates the struct `SchemaV1` which you can use like this:

```rust
fn main() {
    let db = SchemaV1::init("data.db");
}
```

`SchemaV1` is your type, but it will have on it `impl`'d various traits from this crate. These traits
include `Initialize` which allow you to start your database from disk, initialize will use an associated type that
represents your `OnDiskFormat`.

You can interact directly with your tables:

```rust
fn main() {
    let db = SchemaV1::init("data.db");

    db.accounts.insert(Username::from("parth"), Account { ... });
    let account = db.accounts.get(Username::from("parth"));

    db.files.insert(meta.id, meta);
    let file = db.files.get(meta.id);
}
```

These types are not implicitly inferred based on usage, they are specified by you in one location that represents the
schema.

If you wanted to evolve your schema you would do something like this:

```rust
schema! {
    SchemaV1 {
        accounts: <Username, Account>,
        files: <Uuid, EncryptedFileMetadata>
    }
}

schema! {
    SchemaV2 {
        accounts: <Username, AccountV2>,
        files: <Uuid, EncryptedFileMetadata>
    }
}

fn main() {
    let old = SchemaV1::init("data.db");
    let new = SchemaV2::init("data-v2.db");

    old.accounts
        .iter()
        .map(|key, value| new.accounts.insert(key, value.into()));

    old.files
        .iter()
        .map(|key, value| new.files.insert(key, value.into()));

    // Migration successful, safe to delete data.db
}
```

Basic transaction experience:

```rust
schema! {
    SchemaV1 {
        accounts: <Username, Account>,
        files: <Uuid, EncryptedFileMetadata>
    }
}

fn main() {
    let db = SchemaV1::init("data.db");

    db.transaction(|accounts, files| {});
}
```

The basic and most primitive version will just lock everything for transactions. A more sophisticated implementation
could allow you to specify which tables to lock. Long term target state probably involves no locks and some optimistic
concurrency.

While this is already expected to be a significant speedup for use
withing [lockbook](https://github.com/lockbook/lockbook), further gains can be made by experimenting with state of the
art:
+ [Zero Copy (super fast) serializers](https://github.com/rkyv/rkyv)
+ [Lock-free Read Optimized (eventually consistent) Map](https://github.com/jonhoo/evmap)
