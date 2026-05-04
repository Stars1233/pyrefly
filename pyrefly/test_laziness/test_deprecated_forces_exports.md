# get_deprecated no longer forces exports on transitive dep

`a` imports `value` from `b`. `b` imports `old_func` from `c`. `a` only
uses `value`, not `old_func`.

Deprecation warning emission for imported names was moved from the
binding phase to solve time (see the `Binding::Import` arm of
`solve_binding`). The binding phase no longer demands
`get_deprecated(c, "old_func")` just to potentially warn at the
`from c import ...` site. The warning will fire at solve time only if
`b`'s `old_func` import is actually resolved — which it is not here,
because `a` never references `old_func` (directly or transitively).

`c` remains at Exports because `b`'s binding phase still demands
`module_exists(c)` and `export_exists(c, "old_func")`. Those calls are
targeted by later commits in this stack.

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
a -> b::Exports(is_special_export)
a -> b::Exports(export_exists)
a -> b::Exports(get_deprecated)
a -> b::KeyExport(Name("value"))
  b -> c::Load(module_exists)
```
