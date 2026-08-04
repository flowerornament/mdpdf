# Wrapping

## Narrow cells with long identifiers

| Concept | Spec anchor | Implementation | Verdict | Prose |
|---|---|---|---|---|
| plan family | Medium Def 3.2, line 388, governs plan registries | `Herald.Medium.PlanRegistry` and `registry_evaluator.ex:1`, call these plans registries | MATCHES | in architecture prose |
| schema catalog | Medium Def 3.4, line 407, governs versioned node-schema definitions and digests | `Herald.Medium.NodeSchemaCatalog`, `lib/herald/domains/medium/node_schema/catalog.ex:1`, resolves governed schema refs | MATCHES | schema catalog |

## Short code spans stay intact

Short spans such as `foo.bar`, `Vec<T>`, `a::b`, and `x = 1` are common enough
that they must survive byte-for-byte.

## Fenced blocks stay verbatim

```rust
let very_long_identifier_that_must_stay_verbatim = SomeVeryLongTypeName::new();
```
