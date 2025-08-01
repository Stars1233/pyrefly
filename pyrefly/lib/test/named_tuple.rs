/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::testcase;

testcase!(
    test_named_tuple,
    r#"
from typing import NamedTuple, assert_type
class Pair(NamedTuple):
    x: int
    y: str
p: Pair = Pair(1, "")
p = Pair(x=1, y="")
x, y = p
assert_type(x, int)
assert_type(y, str)
assert_type(p[0], int)
assert_type(p[1], str)
assert_type(p[:2], tuple[int, str])
p["oops"]  # E: Cannot index into `Pair`
p.x = 1  # E: Cannot set field `x`
    "#,
);

testcase!(
    test_named_tuple_delete,
    r#"
from typing import NamedTuple, assert_type
class Pair(NamedTuple):
    x: int
    y: str
p: Pair = Pair(1, "")
del p.x  # E: Cannot delete field `x`
del p[0]  # E: Cannot delete item in `Pair`
    "#,
);

testcase!(
    test_named_tuple_functional_attr_types,
    r#"
from typing import NamedTuple, Any, assert_type
from collections import namedtuple
Point1 = namedtuple('Point1', ['x', 'y'])
Point2 = namedtuple('Point2', ('x', 'y'))
Point3 = namedtuple('Point3', 'x y')
Point4 = namedtuple('Point4', 'x, y')
Point5 = NamedTuple('Point5', [('x', int), ('y', int)])
Point6 = NamedTuple('Point6', (('x', int), ('y', int)))
assert_type(Point1(1, 2).x, Any)
assert_type(Point2(1, 2).x, Any)
assert_type(Point3(1, 2).x, Any)
assert_type(Point4(1, 2).x, Any)
assert_type(Point5(1, 2).x, int)
assert_type(Point5(x=1, y=2).x, int)
assert_type(Point6(1, 2).x, int)
    "#,
);

testcase!(
    test_named_tuple_functional_defaults_and_constructor,
    r#"
from typing import NamedTuple
from collections import namedtuple
Point1 = namedtuple("Point1", ["x", "y"])
p1_1 = Point1(x=1, y=1)
p1_2 = Point1(2.3, "")
p1_3 = Point1(2.3)  # E: Missing argument `y` in function `Point1.__new__`
Point2 = namedtuple("Point2", ["x", "y"], defaults=(1, 2))
p1_1 = Point2(x=1, y=1)
p1_2 = Point2(1, 1)
p1_3 = Point2()  # Okay
Point3 = NamedTuple('Point3', [('x', int), ('y', int)])
Point3(1, 2)
Point3(1)  # E: Missing argument `y` in function `Point3.__new__`
    "#,
);

testcase!(
    test_named_tuple_functional_rename,
    r#"
from collections import namedtuple
NT1 = namedtuple("NT1", ["a", "a"])  # E: Duplicate field `a`
NT2 = namedtuple("NT2", ["abc", "def"], rename=False)  # E: `def` is not a valid identifier
NT3 = namedtuple("NT3", ["abc", "def"], rename=True)
NT4 = namedtuple("NT4", ["def", "ghi"], rename=True)
NT3(abc="", _1="")
NT4(_0="", ghi="")
    "#,
);

testcase!(
    test_qualifiers,
    r#"
from typing import NamedTuple, Required, NotRequired, ReadOnly, ClassVar, Final
class MyTuple(NamedTuple):
    v: ClassVar[int]  # E: `ClassVar` may not be used for TypedDict or NamedTuple members
    w: Final[int]  # E: `Final` may not be used for TypedDict or NamedTuple members
    x: NotRequired[int]  # E: `NotRequired` may only be used for TypedDict members
    y: Required[int]  # E: `Required` may only be used for TypedDict members
    z: ReadOnly[int]  # E: `ReadOnly` may only be used for TypedDict members
    "#,
);

testcase!(
    test_named_tuple_functional_duplicate,
    r#"
from typing import NamedTuple
Point = NamedTuple('Point', [('x', int), ('x', int)])  # E: Duplicate field `x`
    "#,
);

testcase!(
    test_named_tuple_subtype,
    r#"
from typing import NamedTuple
class Pair(NamedTuple):
    x: int
    y: str
p: Pair = Pair(1, "")
def func1(x: tuple[int | str, ...]) -> None: ...
def func2(x: tuple[int, str]) -> None: ...
def func3(x: tuple[str, str]) -> None: ...
func1(p)
func2(p)
func3(p)  # E: Argument `Pair` is not assignable to parameter `x` with type `tuple[str, str]` in function `func3
    "#,
);

testcase!(
    test_named_tuple_match,
    r#"
from typing import NamedTuple, assert_type
class Pair(NamedTuple):
    x: int
    y: int
def test(p: Pair):
    match p:
        case Pair(x, y):
            assert_type(x, int)
            assert_type(y, int)
    match p:
        case x, y:
            assert_type(x, int)
            assert_type(y, int)
    "#,
);

testcase!(
    test_named_tuple_iter,
    r#"
from typing import NamedTuple, reveal_type
class Pair(NamedTuple):
    x: int
    y: str

class Pair2[T](NamedTuple):
    x: int
    y: T

def test(p: Pair, p2: Pair2[bytes]):
    reveal_type(p.__iter__)  # E: BoundMethod[Pair, (self: Pair) -> Iterable[int | str]]
    reveal_type(p2.__iter__)  # E: BoundMethod[Pair2[bytes], (self: Pair2[bytes]) -> Iterable[bytes | int]]
    "#,
);

testcase!(
    bug = "NamedTuple extends tuple[Any, ...], making it a subtype of too many things",
    test_named_tuple_subclass,
    r#"
from typing import NamedTuple, Sequence, Never
class Pair(NamedTuple):
    x: int
    y: str
p: Pair = Pair(1, "")
x1: Sequence[int|str] = p # should succeed
x2: Sequence[Never] = p # should fail
    "#,
);

testcase!(
    test_named_tuple_multiple_inheritance,
    r#"
from typing import NamedTuple
class Foo: pass
class Pair(NamedTuple, Foo):  # E: Named tuples do not support multiple inheritance
    x: int
    y: int
class Pair2(NamedTuple):
    x: int
    y: int
class Pair3(Pair2, Foo):  # E: Named tuples do not support multiple inheritance
    pass
    "#,
);

testcase!(
    test_named_tuple_init_requiredness,
    r#"
from typing import NamedTuple
class Pair(NamedTuple):
    x: int
    y: str = "y"
Pair(x=5)
Pair(y="foo")  # E: Missing argument `x` in function `Pair.__new__`
    "#,
);

testcase!(
    test_named_tuple_default,
    r#"
from collections import namedtuple
x = 2
Tup = namedtuple("Tup", ["a", "b"], defaults=(None, x))
"#,
);

testcase!(
    test_named_tuple_dunder_unpack,
    r#"
from typing import NamedTuple
class A(NamedTuple):
    a: int
    b: str
    def __repr__(self) -> str:
        return "A"

def test(x: A) -> None:
    a, b = x
"#,
);

testcase!(
    bug =
        "Field names cannot start with an underscore, unless they were generated with rename=True",
    test_named_tuple_underscore_field_name,
    r#"
from typing import NamedTuple
from collections import namedtuple
class A(NamedTuple):
    a: int
    b: str
    _c: str  # Not OK
B = namedtuple("B", ["a", "b", "_c"])  # E: NamedTuple field name may not start with an underscore
C = namedtuple("C", ["a", "b", "_c"], rename=True)  # OK
"#,
);

testcase!(
    test_named_tuple_subclass_with_qualified_annotations,
    r#"
from typing import NamedTuple, ClassVar, Final, assert_type
class Foo(NamedTuple):
    x: int
    y: str
# Named tuple members (defined directly in a NamedTuple class) cannot use these
# qualifiers, but subclasses can have fields that do.
class Bar(Foo):
    z: ClassVar[int] = 7
    w: Final[int] = 7
assert_type(Bar.z, int)
assert_type(Bar(1, "y").w, int)
"#,
);

testcase!(
    get_named_tuple_elements,
    r#"
from typing import NamedTuple, ClassVar, Final, assert_type
class Foo(NamedTuple):
    x: int = 1
    z: int = 2
    y: str # E: NamedTuple field 'y' without a default may not follow NamedTuple field with a default
"#,
);

testcase!(
    test_named_tuple_override_error,
    r#"
from typing import NamedTuple

class A(NamedTuple):
    x: int

class B(A):
    x: int  # E: Cannot override named tuple element `x`
    y: int

class C(B):
    y: int  # OK
"#,
);

testcase!(
    test_named_tuple_invalid_field,
    r#"
# Used to crash, see https://github.com/facebook/pyrefly/issues/701
from dataclasses import InitVar
from typing import NamedTuple

class Cls(NamedTuple):
    fld: InitVar # E: Expected a type argument for `InitVar`
"#,
);

testcase!(
    bug = "Raise an error on InitVar, or allow it fully",
    test_named_tuple_initvar,
    r#"
# InitVar isn't meant to be used with NamedTuple.
# Pyright/Python treat this field as int, Mypy as InitVar, Pyrefly as Any.

from dataclasses import InitVar
from typing import Any, assert_type, NamedTuple

class Cls(NamedTuple):
    fld: InitVar[int]

v = Cls(1)
v = Cls("no")
assert_type(v[0], Any)

for y in v:
    print(assert_type(y, Any))
"#,
);

testcase!(
    test_named_tuple_override_self,
    r#"
from typing import NamedTuple, Self

class A(NamedTuple):
    x: int
    y: str

class B(A):
    def __new__(cls, x: int, y: str) -> Self:
        return super().__new__(cls, x, y)
    
    def __init__(self, x: int, y: str) -> None:
        return super().__init__(x, y)
"#,
);
