# export_exists check does not force exports on transitive dep

`a` imports `value` from `b`. `b` does `from c import Foo`. `a` only
uses `value`, not `Foo`.

`from c import Foo` emits a `Binding::Import` with an `ImportFallback`;
existence is verified only at solve time, when the binding's value is
demanded. Since `a` never references `Foo`, the demand never fires and
`c`'s export set is never forced.

`c` reaches only `Step::Load` — the binder reads its file contents to
resolve the import target, but doesn't materialize its export set.

This is the same pattern as `test_unused_import_from_same_module` but
simplified: `b` imports from `c` but only exports an unrelated value.

## Files

`a.py`:
```python
from b import value
x = value
```

`b.py`:
```python
from c import Foo
value: int = 42
```

`c.py`:
```python
class Foo:
    x: int = 1
```

## Check `a.py`

```expected
a: Solutions
b: Answers
c: Load

(159 builtin demands hidden)
a -> b::Load(module_exists)
a -> b::Exports(export_exists)
a -> b::Exports(get_deprecated)
a -> b::KeyExport(Name("value"))
  b -> c::Load(module_exists)
```
