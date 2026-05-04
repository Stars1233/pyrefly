# get_deprecated does not force exports on transitive dep

`a` imports `value` from `b`. `b` imports `old_func` from `c`. `a` only
uses `value`, not `old_func`.

Deprecation warning emission for imported names happens at solve time
(see the `Binding::Import` arm of `solve_binding`), not during binding.
The warning fires only if the import is actually resolved — which
`old_func` is not here, because `a` never references it.

`c` reaches only `Step::Load`: the binder reads its file contents to
resolve the import target, but never forces its export set to be
materialized.

## Files

`a.py`:
```python
from b import value
x = value
```

`b.py`:
```python
from c import old_func
value: int = 42
```

`c.py`:
```python
def old_func() -> None: ...
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
