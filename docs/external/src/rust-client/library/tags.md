---
title: Tags
sidebar_position: 8
---

# Working with Note Tags

Note tags are used to specify how notes should be executed and who can consume them. They also serve as a fuzzy filter mechanism for retrieving note updates during sync operations.

## Basic tag operations

```rust
use miden_objects::note::NoteTag;

// Add a tag
client.add_note_tag(NoteTag::from(12345u32)).await?;

// Remove a tag
client.remove_note_tag(NoteTag::from(12345u32)).await?;

// List all tags
let tags = client.get_note_tags().await?;
for record in &tags {
    println!("Tag: {:?}, Source: {:?}", record.tag(), record.source());
}
```

## Managing multiple tags

```rust
use miden_objects::note::NoteTag;

// Add multiple tags
let tags_to_add = [123u32, 456, 789];
for tag in tags_to_add {
    client.add_note_tag(NoteTag::from(tag)).await?;
}

// List them
let all_tags = client.get_note_tags().await?;
println!("Tracking {} tags", all_tags.len());

// Remove some
client.remove_note_tag(NoteTag::from(123u32)).await?;
client.remove_note_tag(NoteTag::from(456u32)).await?;
```

## Tag sources and sync behavior

Tags can come from different sources, tracked via `NoteTagRecord`:

| Source | Description |
|--------|-------------|
| `Account` | Automatically added for accounts tracked by the client |
| `Note` | Automatically added for expected notes being tracked |
| `User` | Manually added by the user via `add_note_tag()` |

During sync operations, the client uses all tracked tags to retrieve note-related updates from the network for notes with matching tags.

## Important notes

- Tags are `NoteTag` values (wrapping `u32`)
- Tags for tracked accounts and notes are managed automatically by the client
- Only user-added tags can be removed — system-generated tags (Account, Note sources) cannot be removed manually
- Use tags with [note transport](./note-transport.md) for increased privacy: add random tags instead of relying on default account-derived tags
