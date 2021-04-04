Split a string without another allocation

Helpfull for some types that need to be parsed from a string
and get split into smaller parts like an `Url` or a `Vec` containing lines
which need to be owned by the parent type.

## Note

First try to store references, for example `&str` which is more efficient.

At the moment if you create a `SharedString` the underlying bytes cannot be
mutated.

## Example

```rust
use shared_string::SharedString;
// or SharedSyncString if `Sync` is required

struct Name {
    firstname: SharedString,
    middlename: SharedString,
    lastname: SharedString
    // to be faster than string
    // you should use at least 3 fields
}

impl Name {
    pub fn new(fullname: impl Into<SharedString>) -> Option<Self> {
        let mut split = fullname.into().split(b' ');
        Some(Self {
            firstname: split.next()?,
            middlename: split.next()?,
            lastname: split.next()?
        })
    }
}

let name = Name::new("Bartholomew Jojo Simpson").unwrap();
assert_eq!(name.firstname, "Bartholomew");
assert_eq!(name.middlename, "Jojo");
assert_eq!(name.lastname, "Simpson");
```

## Performance

`SharedString` can increase the perfomance in situations such as the example
above by over 30%. See `benches/*` for benchmarks.