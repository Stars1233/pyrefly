/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::test::util::TestEnv;
use crate::testcase;

testcase!(
    test_lambda,
    r#"
from typing import Callable, reveal_type
f1 = lambda x: 1
reveal_type(f1)  # E: revealed type: (x: Unknown) -> Literal[1]
f2 = lambda x: reveal_type(x)  # E: revealed type: Unknown
f3: Callable[[int], int] = lambda x: 1
reveal_type(f3)  # E: revealed type: (int) -> int
f4: Callable[[int], int] = lambda x: reveal_type(x)  # E: revealed type: int
f5: Callable[[int], int] = lambda x: x
f6: Callable[[int], int] = lambda x: "foo"  # E: `(x: int) -> Literal['foo']` is not assignable to `(int) -> int`
f7: Callable[[int, int], int] = lambda x: 1  # E: `(x: int) -> Literal[1]` is not assignable to `(int, int) -> int`
f8: Callable[[int], int] = lambda x: x + "foo" # E: Argument `Literal['foo']` is not assignable to parameter `value` with type `int`
"#,
);

testcase!(
    test_callable_ellipsis_upper_bound,
    r#"
from typing import Callable
def test(f: Callable[[int, str], None]) -> Callable[..., None]:
    return f
"#,
);

testcase!(
    test_callable_ellipsis_lower_bound,
    r#"
from typing import Callable
def test(f: Callable[..., None]) -> Callable[[int, str], None]:
    return f
"#,
);

testcase!(
    test_callable_invalid_annotation,
    r#"
from typing import Callable, assert_type, Any
def test(x: Callable[int]):  # E: `Callable` requires exactly two arguments but 1 was found
    assert_type(x, Callable[..., Any])
"#,
);

testcase!(
    test_callable_constructor,
    r#"
from typing import Callable, Self, NoReturn
class C1:
    def __init__(self, x: str) -> None: pass
class C2:
    def __new__(cls, x: str) -> Self:
        return super(C2, cls).__new__(cls)
class C3:
    def __init__(self, x: str) -> None: pass
    def __new__(cls, x: str) -> Self:
        return super(C3, cls).__new__(cls)
class C4: pass
class C5:
    # The __init__ should be ignored
    def __new__(cls, x: int) -> int:
        return 1
    def __init__(self, x: str) -> None: pass
class C6:
    def __new__(cls, *args, **kwargs) -> Self:
        return super(C6, cls).__new__(cls)
    def __init__(self, x: int) -> None: pass
class CustomMeta(type):
    def __call__(cls) -> NoReturn:
        raise NotImplementedError("Class not constructable")
class C7(metaclass=CustomMeta):
    def __new__(cls, x: int) -> Self:
        return super(C7, cls).__new__(cls)

x1: Callable[[], int] = int
x2: Callable[[str], C1] = C1
x3: Callable[[str], C2] = C2
x4: Callable[[str], C3] = C3
x5: Callable[[], C4] = C4
x6: Callable[[int], int] = C5
x7: Callable[[int], C6] = C6
x8: Callable[[], NoReturn] = C7

x9: Callable[[], str] = int  # E:
x10: Callable[[], C2] = C2  # E:
x11: Callable[[int], C3] = C3  # E:
x12: Callable[[int], C5] = C5  # E:
"#,
);

testcase!(
    test_callable_constructor_unannotated_metaclass_call,
    r#"
from typing import Self, Callable
class Meta(type):
    # This is unannotated, so we should treat it as compatible and use the signature of __new__
    def __call__(cls, x: str):
        raise TypeError("Cannot instantiate class")
class MyClass(metaclass=Meta):
    def __new__(cls, x: int) -> Self:
        return super().__new__(cls)
x1: Callable[[int], MyClass] = MyClass  # OK
x2: Callable[[str], MyClass] = MyClass  # E: `type[MyClass]` is not assignable to `(str) -> MyClass`
    "#,
);

testcase!(
    test_callable_unpack,
    r#"
from typing import Callable
def test(f: Callable[[bool, *tuple[int, str], bool], None]) -> Callable[[*tuple[bool, int, str, bool]], None]:
    return f
"#,
);

testcase!(
    test_callable_unpack_vararg,
    r#"
from typing import Protocol
class P1(Protocol):
    def __call__(self, *args: int): ...
class P2(Protocol):
    def __call__(self, *args: *tuple[int, int]): ...
class P3(Protocol):
    def __call__(self, *args: *tuple[int, str]): ...
class P4(Protocol):
    def __call__(self, *args: *tuple[int, ...]): ...
class P5(Protocol):
    def __call__(self, x: int, y: int, /): ...
class P6(Protocol):
    def __call__(self, x: int, /, *args: *tuple[int]): ...
class P7(Protocol):
    def __call__(self, x: int, y: int = 2, /): ...

def test(p1: P1, p2: P2, p3: P3, p4: P4, p5: P5, p6: P6, p7: P7):
    x1: P2 = p1
    x2: P1 = p2  # E: `P2` is not assignable to `P1`
    x3: P2 = p3  # E: `P3` is not assignable to `P2`
    x4: P2 = p4
    x5: P4 = p2  # E: `P2` is not assignable to `P4`
    x6: P5 = p2
    x7: P2 = p5
    x8: P2 = p6
    x9: P6 = p2
    x10: P2 = p7
"#,
);

testcase!(
    test_callable_unparameterized,
    r#"
from typing import Callable, assert_type, Any
def test(f: Callable):
    assert_type(f, Callable[..., Any])
"#,
);

testcase!(
    test_callable_subtype_vararg_and_positional,
    r#"
from typing import Protocol
class P1(Protocol):
    def __call__(self, a: int, b: str) -> None: ...

class P2(Protocol):
    def __call__(self, *args: int | str) -> None: ...

class P3(Protocol):
    def __call__(self, *args: int | str, a: int, b: str) -> None: ...

class P4(Protocol):
    def __call__(self, *args: int | str, a: int = 1, b: str = "") -> None: ...

def test(p2: P2, p3: P3, p4: P4):
    # this one doesn't work because a/b can be passed by name
    x1: P1 = p2  # E: `P2` is not assignable to `P1`
    # this one doesn't work because a/b isn't always passed by name
    x2: P1 = p3  # E: `P3` is not assignable to `P1`
    x3: P1 = p4  # OK
"#,
);

testcase!(
    test_callable_annot_too_few_args,
    r#"
from typing import Callable
def test(f: Callable[[int], None]):
    f() # E: Expected 1 more positional argument
"#,
);

testcase!(
    test_callable_annot_too_many_args,
    r#"
from typing import Callable
def test(f: Callable[[], None]):
    f(
      1, # E: Expected 0 positional arguments
      2
    )
"#,
);

testcase!(
    test_callable_annot_keyword_args,
    r#"
from typing import Callable
def test(f: Callable[[], None]):
    f(
      x=1, # E: Unexpected keyword argument `x`
      y="hello" # E: Unexpected keyword argument `y`
    )
"#,
);

testcase!(
    test_callable_ellipsis_keyword_args,
    r#"
from typing import Callable
def test(f: Callable[..., None]):
    f(x=1, y="hello") # OK
"#,
);

testcase!(
    test_callable_annot_upper_bound,
    r#"
from typing import Callable
def test(f: Callable[[int, int], None]) -> None: ...

def f1(x: int, y: int) -> None: ...
test(f1) # OK

# Lower bound has too many args
def f2(x: int, y: int, z: int) -> None: ...
test(f2) # E: Argument `(x: int, y: int, z: int) -> None` is not assignable to parameter `f` with type `(int, int) -> None`

# Lower bound has too few args
def f3(x: int) -> None: ...
test(f3) # E: Argument `(x: int) -> None` is not assignable to parameter `f` with type `(int, int) -> None`

# Lower bound has wrong arg types
def f4(x: str, y: int) -> None: ...
test(f4) # E: Argument `(x: str, y: int) -> None` is not assignable to parameter `f` with type `(int, int) -> None`

# Lower bound has variadic args of compatible type
def f5(*args: int) -> None: ...
test(f5) # OK

# Lower bound has variadic args of incompatible type
def f6(*args: str) -> None: ...
test(f6) # E: Argument `(*args: str) -> None` is not assignable to parameter `f` with type `(int, int) -> None`

# Lower bound has extra kwargs of arbitrary type
class Arbitrary: pass
def f7(x: int, y: int, **kwargs: Arbitrary) -> None: ...
test(f7) # OK

# Lower bound has extra args with defaults
def f7(x: int, y: int, z: int = 0) -> None: ...
test(f7) # OK
"#,
);

testcase!(
    test_positional_param_keyword_arg,
    r#"
def test(x: int, y: str): ...
test(1, "hello") # OK
test(x=1, y="hello") # OK
test(y="hello", x=1) # OK
test(1, y="hello") # OK
test(1) # E: Missing argument `y`
test(1, "hello", x=2) # E: Multiple values for argument `x`
"#,
);

testcase!(
    test_positional_only_params,
    r#"
def test(x: int, y: str, /): ...
test(1, "hello") # OK
test(1) # E: Missing positional argument `y`
test(1, y="hello") # E: Expected argument `y` to be positional
test(1, "hello", 2) # E: Expected 2 positional arguments, got 3
"#,
);

testcase!(
    test_historical_positional_only_params,
    r#"
def f1(__x: str): ...
f1("hello") # OK
f1(__x="hello") # E: Expected argument `__x` to be positional

def f2(__x: str, /, __y: str, __z__: str): ...
f2(__x="hello", __y="my", __z__="world") # E: Expected argument `__x` to be positional
f2("hello", __y="my", __z__="world") # OK

def f3(__x: str, *, __y__: str, __z: str): ...
f3(__x="hello", __y__="my", __z="world") # OK

def f4(x: str, __y: str): ... # E: Positional-only parameter `__y` cannot appear after keyword parameters

class C:
    def f5(self, __x: str): ...

    def f6(self, x: str, __y: str): ... # E: Positional-only parameter `__y` cannot appear after keyword parameters

    @classmethod
    def f7(cls, __x: str): ...

c = C()
c.f5("hello") # OK
c.f5(__x="hello") # E: Expected argument `__x` to be positional
C.f7("hello") # OK
C.f7(__x="hello") # E: Expected argument `__x` to be positional
"#,
);

testcase!(
    test_keyword_only_params,
    r#"
def test(*, x: int, y: str): ...
test(x=1, y="hello") # OK
test(1, "hello") # E: Expected argument `x` to be passed by name # E: Expected argument `y` to be passed by name
test(x=1) # E: Missing argument `y`
test(y="hello") # E: Missing argument `x`
"#,
);

testcase!(
    test_extra_positional_args,
    r#"
def test(*, x: int): ...
test(1, 2)  # E: Expected argument `x` to be passed by name  # E: Expected 0 positional arguments, got 2
    "#,
);

testcase!(
    test_missing_self_and_kwonly,
    r#"
class A:
    def f(*, x): ...
A().f(1)  # E: Expected argument `x` to be passed by name  # E: Expected 0 positional arguments, got 2 (including implicit `self`)
    "#,
);

testcase!(
    test_varargs,
    r#"
def test(*args: int): ...
test(1, 2, "foo", 4) # E: Argument `Literal['foo']` is not assignable to parameter `*args` with type `int`
"#,
);

testcase!(
    test_kwargs,
    r#"
def test(**kwargs: int): ...
test(x=1, y="foo", z=2) # E: Keyword argument `y` with type `Literal['foo']` is not assignable to parameter `**kwargs` with type `int` in function `test`
"#,
);

testcase!(
    test_args_kwargs_type,
    r#"
from typing import assert_type
def test(*args: int, **kwargs: int) -> None:
    assert_type(args, tuple[int, ...])
    assert_type(kwargs, dict[str, int])
"#,
);

testcase!(
    test_defaults,
    r#"
def test(x: int, y: int = 0, z: str = ""): ...
test() # E: Missing argument `x`
test(0, 1) # OK
test(0, 1, "foo") # OK
test(0, 1, "foo", 2) # E: Expected 3 positional arguments
"#,
);

testcase!(
    test_defaults_posonly,
    r#"
def test(x: int, y: int = 0, z: str = "", /): ...
test() # E: Missing positional argument `x`
test(0, 1) # OK
test(0, 1, "foo") # OK
test(0, 1, "foo", 2) # E: Expected 3 positional arguments
"#,
);

testcase!(
    test_bad_default,
    r#"
def f(x: int = ""):  # E: Default `Literal['']` is not assignable to parameter `x` with type `int`
    pass
    "#,
);

testcase!(
    test_infer_param_type_from_default,
    r#"
from typing import Any, assert_type
def f(x, y = "", z = None):
    assert_type(x, Any)
    assert_type(y, Any | str)
    assert_type(z, Any | None)
    "#,
);

testcase!(
    test_default_ellipsis,
    r#"
def stub(x: int = ...): ... # OK
def err(x: int = ...): pass # E: Default `Ellipsis` is not assignable to parameter `x` with type `int`
"#,
);

testcase!(
    test_splat_tuple,
    r#"
def test(x: int, y: int, z: int): ...
test(*(1, 2, 3)) # OK
test(*(1, 2)) # E: Missing argument `z`
test(*(1, 2, 3, 4)) # E: Expected 3 positional arguments, got 4
"#,
);

testcase!(
    test_splat_iterable,
    r#"
def test(x: int, y: int, z: int): ...
test(*[1, 2, 3]) # OK
test(*[1, 2]) # OK
test(*[1, 2, 3, 4]) # OK
test(*[1], 2) # E: Expected 3 positional arguments, got 4
test(1, 2, 3, *[4]) # OK
"#,
);

testcase!(
    test_splat_unpacked_args,
    r#"
from typing import assert_type

def test1(*args: *tuple[int, int, int]): ...
test1(*(1, 2, 3)) # OK
test1(*(1, 2)) # E: Unpacked argument `tuple[Literal[1], Literal[2]]` is not assignable to parameter `*args` with type `tuple[int, int, int]` in function `test1`
test1(*(1, 2, 3, 4)) # E: Unpacked argument `tuple[Literal[1], Literal[2], Literal[3], Literal[4]]` is not assignable to parameter `*args` with type `tuple[int, int, int]` in function `test1`

def test2[*T](*args: *tuple[int, *T, int]) -> tuple[*T]: ...
assert_type(test2(*(1, 2, 3)), tuple[int])
assert_type(test2(*(1, 2)), tuple[()])
assert_type(test2(*(1, 2, 3, 4)), tuple[int, int])
assert_type(test2(1, 2, *(3, 4), 5), tuple[int, int, int])
assert_type(test2(1, *(2, 3), *("4", 5)), tuple[int, int, str])
assert_type(test2(1, *[2, 3], 4), tuple[int, ...])
test2(1, *(2, 3), *(4, "5"))  # E: Unpacked argument `tuple[Literal[1], Literal[2], Literal[3], Literal[4], Literal['5']]` is not assignable to parameter `*args` with type `tuple[int, *@_, int]` in function `test2`
"#,
);

testcase!(
    test_splat_union,
    r#"
from typing import Iterable

def test(x: int, y: int, z: int): ...

def fixed_same_len_ok(xs: tuple[int, int, int] | tuple[int, int, int]):
    test(*xs) # OK

def fixed_same_len_type_err(xs: tuple[int, int, int] | tuple[int, int, str]):
    test(*xs) # E: Argument `int | str` is not assignable to parameter `z` with type `int`

def fixed_same_len_too_few(xs: tuple[int, int] | tuple[int, int]):
    test(*xs) # E: Missing argument `z`

def fixed_diff_len(xs: tuple[int, int] | tuple[int, int, int]):
    test(*xs) # OK (treated as Iterable[int])

def mixed_same_type(xs: tuple[int, int] | Iterable[int]):
    test(*xs) # OK (treated as Iterable[int])

def mixed_type_err(xs: tuple[int, int] | Iterable[str]):
    test(*xs) # E: Argument `int | str` is not assignable to parameter `x` with type `int` # E: Argument `int | str` is not assignable to parameter `y` with type `int` # E: Argument `int | str` is not assignable to parameter `z` with type `int`
"#,
);

// Normally, positional arguments can not come after keyword arguments. Splat args are an
// exception. However, splat args are still evaluated first, so they consume positional params
// before any keyword arguments.
// See https://github.com/python/cpython/issues/104007
testcase!(
    test_splat_keyword_first,
    r#"
def test(x: str, y: int, z: int): ...
test(x="", *(0, 1)) # E: Argument `Literal[0]` is not assignable to parameter `x` with type `str` # E: Multiple values for argument `x` # E: Missing argument `z`
"#,
);

testcase!(
    test_splat_kwargs,
    r#"
def f(x: int, y: int, z: int): ...
def test(kwargs: dict[str, int]):
    f(**kwargs) # OK
    f(1, **kwargs) # OK
"#,
);

testcase!(
    test_splat_kwargs_mixed_with_keywords,
    r#"
def f(x: str, y: int, z: int): ...
def test(kwargs: dict[str, int]):
    f("foo", **kwargs) # OK
    f(x="foo", **kwargs) # OK
    f(**kwargs) # E: Unpacked keyword argument `int` is not assignable to parameter `x` with type `str` in function `f`
"#,
);

testcase!(
    test_splat_kwargs_multi,
    r#"
def f(x: int, y: int, z: int): ...
def test(kwargs1: dict[str, int], kwargs2: dict[str, str]):
    f(**kwargs1, **kwargs2) # E: Unpacked keyword argument `str` is not assignable to parameter `x` with type `int` in function `f` # E: Unpacked keyword argument `str` is not assignable to parameter `y` with type `int` in function `f` # E: Unpacked keyword argument `str` is not assignable to parameter `z` with type `int` in function `f`
"#,
);

testcase!(
    test_splat_kwargs_mapping,
    r#"
from typing import Mapping
def f(x: int, y: int, z: int): ...
def test(kwargs: Mapping[str, int]):
    f(**kwargs) # OK
"#,
);

testcase!(
    test_splat_kwargs_subclass,
    r#"
class Counter[T](dict[T, int]): ...
def f(**kwargs: int): ...
def test(c: Counter[str]):
    f(**c)
"#,
);

testcase!(
    test_splat_kwargs_wrong_key,
    r#"
def f(x: int): ...
def test(kwargs: dict[int, str]):
    f(**kwargs) # E: Expected argument after ** to have `str` keys, got: int # E: Missing argument `x`
"#,
);

testcase!(
    test_splat_kwargs_to_kwargs_param,
    r#"
def f(**kwargs: int): ...
def g(**kwargs: str): ...
def test(kwargs: dict[str, int]):
    f(**kwargs) # OK
    g(**kwargs) # E: Unpacked keyword argument `int` is not assignable to parameter `**kwargs` with type `str` in function `g`
"#,
);

testcase!(
    test_callable_async,
    r#"
from typing import Any, Awaitable, Callable, Coroutine

async def f(x: int) -> int: ...
def test_corountine() -> Callable[[int], Coroutine[Any, Any, int]]:
    return f
def test_awaitable() -> Callable[[int], Awaitable[int]]:
    return f
def test_sync() -> Callable[[int], int]:
    return f  # E: Returned type `(x: int) -> Coroutine[Unknown, Unknown, int]` is not assignable to declared return type `(int) -> int`
"#,
);

testcase!(
    test_assignability_both_typed_dicts,
    r#"
from typing import TypedDict, Unpack, Protocol
class TD1(TypedDict):
    x: int
class TD2(TypedDict):
    x: int
    y: int
class P1(Protocol):
    def __call__(self, **kwargs: Unpack[TD1]): ...
class P2(Protocol):
    def __call__(self, **kwargs: Unpack[TD2]): ...
def test(accept_td1: P1, accept_td2: P2):
    a: P1 = accept_td2  # E: `P2` is not assignable to `P1`
    b: P2 = accept_td1
"#,
);

testcase!(
    test_assignability_one_typed_dict,
    r#"
from typing import TypedDict, Unpack, NotRequired, Protocol
class TD(TypedDict):
    string: str
    number: NotRequired[int]
class P1(Protocol):
    def __call__(self, **kwargs: Unpack[TD]): ...
class P2(Protocol):
    def __call__(self, *, string: str, number: int = ...): ...
class P3(Protocol):
    def __call__(self, string: str, number: int = ...): ...
def test(accept_td: P1, kwonly_args: P2, regular_args: P3):
    a: P2 = accept_td
    b: P3 = accept_td  # E: `P1` is not assignable to `P3`
    c: P1 = kwonly_args  # E: `P2` is not assignable to `P1`
    d: P1 = regular_args   # E: `P3` is not assignable to `P1`
"#,
);

testcase!(
    test_assignability_typed_dict_and_regular_kwargs,
    r#"
from typing import TypedDict, Unpack, NotRequired, Protocol
class TD(TypedDict):
    string: str
    number: NotRequired[int]
class P1(Protocol):
    def __call__(self, **kwargs): ...
class P2(Protocol):
    def __call__(self, **kwargs: Unpack[TD]): ...
class P3(Protocol):
    def __call__(self, **kwargs: int | str): ...
class P4(Protocol):
    def __call__(self, **kwargs: int): ...
def test(unannotated: P1, unpacked: P2, annotated: P3, annotated_wrong: P4):
    a: P2 = unannotated
    b: P2 = annotated
    c: P2 = annotated_wrong  # E: `P4` is not assignable to `P2`
"#,
);

testcase!(
    test_assignability_typed_dict_wrong_kwarg,
    r#"
from typing import TypedDict, Protocol, Required, NotRequired, Unpack
class TD(TypedDict):
    v1: Required[int]
    v2: NotRequired[str]
    v3: Required[str]
def func1(**kwargs: Unpack[TD]) -> None: ...
class P1(Protocol):
    def __call__(self, *, v1: int, v2: int, v3: str) -> None:...
class P2(Protocol):
    def __call__(self, *, v1: int) -> None: ...
class P3(Protocol):
    def __call__(self, *, v1: int, v2: str, v4: str) -> None: ...
x: P1 = func1  # E: `(**kwargs: Unpack[TypedDict[TD]]) -> None` is not assignable to `P1`
y: P2 = func1  # E: `(**kwargs: Unpack[TypedDict[TD]]) -> None` is not assignable to `P2`
z: P3 = func1  # E: `(**kwargs: Unpack[TypedDict[TD]]) -> None` is not assignable to `P3`
"#,
);

testcase!(
    test_function_vs_callable,
    r#"
from typing import assert_type, Callable
def f(x: int) -> int:
    return x
# This assertion (correctly) fails because x is a positional parameter rather than a positional-only one.
# This test verifies that we produce a sensible error message that shows the mismatch.
assert_type(f, Callable[[int], int])  # E: assert_type((x: int) -> int, (int) -> int) failed
    "#,
);

testcase!(
    test_function_name_in_error,
    TestEnv::one("foo", "def f(x: int): ..."),
    r#"
import foo
foo.f("")  # E: in function `foo.f`

def f(x: int): ...
f("")  # E: in function `f`

class A:
    def f(self, x: int): ...
    @classmethod
    def g(cls, x: int): ...
    @staticmethod
    def h(x: int): ...
A().f("")  # E: in function `A.f`
A.f(A(), "")  # E: in function `A.f`
A.g("")  # E: in function `A.g`
A.h("")  # E: in function `A.h`

class B(A):
    pass
B().f("")  # E: in function `A.f`
    "#,
);

testcase!(
    test_args_kwargs_assignment,
    r#"
from typing import TypedDict, Unpack
def test1(*cmd: str, **keywords: str) -> None:
    cmd = ("mycmd",)
    cmd = (1,)  # E: `tuple[Literal[1]]` is not assignable to variable `cmd` with type `tuple[str, ...]`
    keywords = {"key": "value"}
    keywords = {"key": 0}  # E: `dict[str, int]` is not assignable to variable `keywords` with type `dict[str, str]`
class MyDict(TypedDict):
    x: int
    y: int
def test2(my_dict: MyDict, *cmd: *tuple[str, str], **keywords: Unpack[MyDict]) -> None:
    cmd = ("mycmd", "mycmd2")
    cmd = ("mycmd",)  # E: `tuple[Literal['mycmd']]` is not assignable to variable `cmd` with type `tuple[str, str]`
    keywords = my_dict
    keywords = { "x": 1 }  # E: Missing required key `y` for TypedDict `MyDict`
"#,
);

testcase!(
    test_never_callable,
    r#"
from typing import Never

def f(x: Never) -> Never:
    return x()
"#,
);

testcase!(
    test_param_matching_rhs_empty,
    r#"
from typing import Callable
def foo(f: Callable[[], None]) -> None: ...

def optional_pos_only_ok(x: int = 0, /) -> None: ...
foo(optional_pos_only_ok)

def optional_pos_ok(x: int = 0) -> None: ...
foo(optional_pos_ok)

def optional_kw_only_ok(*, x: int = 0) -> None: ...
foo(optional_kw_only_ok)

def optional_all_default_ok(x: int = 0, /, y: int = 1, *, z: int = 2) -> None: ...
foo(optional_all_default_ok)

def varargs_ok(*args: int) -> None: ...
foo(varargs_ok)

def kwargs_ok(**kwargs: int) -> None: ...
foo(kwargs_ok)

def varargs_kwargs_ok(*args: int, **kwargs: int) -> None: ...
foo(varargs_kwargs_ok)

def varargs_bad(*args: int, x: int) -> None: ...
foo(varargs_bad)  # E: not assignable to parameter `f`

def varargs_kwargs_bad(*args: int, x: int, **kwargs: int) -> None: ...
foo(varargs_kwargs_bad)  # E: not assignable to parameter `f`
"#,
);

testcase!(
    test_callable_class,
    r#"
from typing import Callable
class C:
    def __call__(self, x: int) -> int:
        return 1
def test(cls: C):
    x: Callable[[int], int] = cls
"#,
);

testcase!(
    test_callable_class_functools_partial,
    r#"
from __future__ import annotations
from functools import partial
from typing import Callable, Match

def bar(a: Match[str], b: int) -> str:
    return f'{a}{b}'

def zoo(a: Callable[[Match[str]], str]) -> None:
    return None

zoo(partial(bar, b=99))
"#,
);

testcase!(
    test_callable_class_substitute_self,
    r#"
from typing import Callable, Self, assert_type

def ret[T](f: Callable[[], T]) -> T: ...

class Meta(type):
    def __call__(self, *args, **kwargs) -> Self: ... # TODO: error invalid Self

# metaclass __call__
class A(metaclass=Meta):
    pass

# __new__
class B:
    def __new__(cls, *args, **kwargs) -> Self: ...

# __init__
class C:
    def __init__(self, *args, **kwargs) -> None: ...

assert_type(ret(A), A) # mypy/pyright agree, but maybe Any since metaclass Self is illegal?
assert_type(ret(B), B)
assert_type(ret(C), C)
"#,
);

testcase!(
    test_callable_class_self_confusion,
    r#"
from typing import Callable, Self, assert_type

class A:
    def __new__(cls) -> Self: ...

class B[T]:
    def __new__(self, f: Callable[[], T]) -> Self: ...

assert_type(B(A), B[A])
"#,
);

testcase!(
    test_call_self,
    r#"
from typing import assert_type
class Foo:
    def __call__(self, a: int) -> int:
        return a
    def bar(self, b: int) -> None:
        assert_type(self(b), int)
    "#,
);

testcase!(
    test_ellipsis_body,
    r#"
from typing import Any, assert_type
def f(): ...
# This is technically wrong (`g()` returns `None`), but `...` is often used to stub out the bodies
# of things like overload signatures and abstractmethods. For simplicity, we just always allow this
# stubbing behavior.
def g() -> str: ...
assert_type(f(), None)
assert_type(g(), str)
    "#,
);

testcase!(
    test_posonly_kwargs_duplicate_ok,
    r#"
def f(x: int, /, **kwargs: str):
    pass
f(0, x="1")
    "#,
);

testcase!(
    test_not_a_class_object,
    r#"
isinstance(1, "not a class object")  # E: Expected class object
issubclass(str, "not a class object")  # E: Expected class object
    "#,
);

testcase!(
    test_generic_function_is_callable,
    r#"
from typing import Any, Callable
def f[T](*, x: T) -> T:
    return x
def g(f: Callable[..., Any]):
    pass
g(f)
    "#,
);

testcase!(
    bug = "There should be no errors",
    test_return_generic_callable,
    r#"
from typing import assert_type, Callable
def f[T]() -> Callable[[T], T]:
    return lambda x: x

g = f()
assert_type(g(0), int)
assert_type(g(""), str)  # E: assert_type(int, str)  # E: `Literal['']` is not assignable to parameter with type `int`

@f()
def h(x: int) -> int:
    return x
assert_type(h(0), int)  # E: assert_type(Any, int)
    "#,
);
