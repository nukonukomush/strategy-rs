import os
from ctypes import *

class SimplePosition(c_int):
    pass

class CrossState(c_int):
    pass

class TransactionId(c_longlong):
    pass

type_map = {
    c_int: "i32",
    c_double: "f64",
    SimplePosition: "simple_position",
    CrossState: "cross",
    TransactionId: "tid",
}

def default(T):
    defaults = {
        c_int: lambda: 0,
        c_double: lambda: 0.0,
        Option(c_double): Option(c_double).none,
        CrossState: 0,
        SimplePosition: 0,
    }
    return defaults[T]()

def get_rust_type(T):
    if T in type_map:
        return type_map[T]
    else:
        # TODO: raise Error
        pass

dirname = os.path.dirname(os.path.abspath(__file__))
# mydll = cdll.LoadLibrary("{}/../target/debug/libstrategy.dylib".format(dirname))
mydll = cdll.LoadLibrary("{}/libstrategy.dylib".format(dirname))

def get_func(cls, method, S, V):
    s_str = get_rust_type(S)
    v_str = get_rust_type(V)
    name = "{}_{}_{}_{}".format(cls, method, s_str, v_str)
    return getattr(mydll, name)

def Option_eq(self, other):
    if other is None or not isinstance(other, self.__class__):
        return False
    if self.is_some == other.is_some:
        if self.is_some:
            return self.content == other.content
        else:
            return True
    else:
        return False

def Option_repr(self):
    if self.is_some:
        return "Some({})".format(str(self.content))
    else:
        return "None"

def Option_nullable(self):
    if self.is_some:
        return self.content
    return None

def Option_from_nullable(T, n):
    if n is None:
        return Option(T).none()
    return Option(T).some(n)

option_types = {}
for T, t_str in type_map.items():
    def def_option(t):
        T = t
        option_t_str = "Option_{}".format(t_str)
        option_types[T] = type(option_t_str, (Structure,), {
            "__eq__": Option_eq,
            "__repr__": Option_repr,
            "nullable": Option_nullable,
        })
        option_types[T]._fields_ = [
            ("is_some", c_byte),
            ("content", T),
        ]
        option_types[T].some = lambda v: option_types[T](1, v)
        option_types[T].none = lambda : option_types[T](0, default(T))
        option_types[T].from_nullable = Option_from_nullable
        option_types[T].T = T
    def_option(T)

def Option(T):
    if T in option_types:
        return option_types[T]
    else:
        raise TypeError("type {} is not available for Option.".format(T))

type_map[Option(c_double)] = "option_f64"
type_map[Option(c_int)] = "option_i32"

def MaybeValue_eq(self, other):
    if other is None or not isinstance(other, self.__class__):
        return False
    if self.is_value == other.is_value:
        if self.is_value:
            return self.content == other.content
        else:
            return True
    else:
        return False

def MaybeValue_repr(self):
    if self.is_value:
        return "Value({})".format(str(self.content))
    else:
        return "OutOfRange"

def MaybeValue_nullable(self):
    if self.is_value:
        return self.content
    return None

def MaybeValue_from_nullable(T, n):
    if n is None:
        return MaybeValue(T).out_of_range()
    return MaybeValue(T).value(n)

maybevalue_types = {}
for T, t_str in type_map.items():
    def def_maybevalue(t):
        T = t
        maybevalue_t_str = "MaybeValue_{}".format(t_str)
        maybevalue_types[T] = type(maybevalue_t_str, (Structure,), {
            "__eq__": MaybeValue_eq,
            "__repr__": MaybeValue_repr,
            "nullable": MaybeValue_nullable,
        })
        maybevalue_types[T]._fields_ = [
            ("is_value", c_byte),
            ("content", T),
        ]
        maybevalue_types[T].value = lambda v: maybevalue_types[T](1, v)
        maybevalue_types[T].out_of_range = lambda : maybevalue_types[T](0, default(T))
        maybevalue_types[T].from_nullable = MaybeValue_from_nullable
        maybevalue_types[T].T = T
    def_maybevalue(T)

def MaybeValue(T):
    if T in maybevalue_types:
        return maybevalue_types[T]
    else:
        raise TypeError("type {} is not available for MaybeValue.".format(T))


from datetime import datetime
class Time(Structure):
    _fields_ = [
        ("time", c_longlong),
        ("granularity", c_longlong),
    ]

    def __init__(self, time, granularity):
        if isinstance(time, str):
            time = int(datetime.strptime(time, "%Y-%m-%d %H:%M:%S").timestamp())
        elif isinstance(time, datetime):
            time = int(time.timestamp())

        super(Time, self).__init__(time, granularity)

    def __add__(self, other):
        if isinstance(other, int):
            return Time(self.time + self.granularity * other, self.granularity)
        return None

    def dt(self):
        return datetime.fromtimestamp(self.time)

    def __eq__(self, other):
        if not isinstance(other, type(self)):
            return False
        if self.granularity != other.granularity:
            return False
        return self.time == other.time

    def __lt__(self, other):
        if not isinstance(other, type(self)):
            return False
        if self.granularity != other.granularity:
            return False
        return self.time < other.time

    def __le__(self, other):
        if not isinstance(other, type(self)):
            return False
        if self.granularity != other.granularity:
            return False
        return self.time <= other.time

    def __repr__(self):
        return "({}, {})".format(self.dt().strftime("%Y-%m-%d %H:%M:%S"), self.granularity)

    def range_to(self, end):
        def gen():
            t = self
            while t < end:
                yield t
                t += 1
        return gen()

type_map[Time] = "time"


class Ptr(Structure):
    _fields_ = [
        ("b_ptr", c_void_p),
        ("f_ptr", c_void_p),
    ]

class Indicator:
    _cls_ = None
    for S1, S2, V1, V2 in [
        [Time, Time, c_double, c_double],
        [Time, Time, c_int, c_int],
        [Time, Time, Option(c_double), Option(c_double)],
        [Time, Time, CrossState, c_int],
        [TransactionId, c_longlong, c_double, c_double],
    ]:
        get_func("indicator", "value", S1, V1).argtypes = [c_void_p, S2]
        get_func("indicator", "value", S1, V1).restype = MaybeValue(V2)

    def value(self, i):
        return get_func("indicator", "value", self._S, self._V)(self._ptr.f_ptr, i)

    def __del__(self):
        get_func(self._cls_, "destroy", self._S, self._V)(self._ptr)
        self._ptr = None


class Vec(Indicator):
    _cls_ = "vec"
    for S1, S2, V1, V2 in [
        [Time, Time, c_double, c_double],
        [TransactionId, c_longlong, c_double, c_double],
    ]:
        get_func(_cls_, "new", S1, V1).argtypes = [S2, POINTER(V2), c_int]
        get_func(_cls_, "new", S1, V1).restype = Ptr
        get_func(_cls_, "destroy", S1, V1).argtypes = [Ptr]
        get_func(_cls_, "destroy", S1, V1).restype = None
        get_func(_cls_, "add", S1, V1).argtypes = [Ptr, V2]
        get_func(_cls_, "add", S1, V1).restype = None

    def __init__(self, S, V, vec, offset):
        self._S = S
        self._V = V
        length = len(vec)
        arr = (V * length)(*vec)
        ptr = POINTER(V)(arr)
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(offset, ptr, length)

    def add(self, value):
        get_func(self._cls_, "add", self._S, self._V)(self._ptr, value)


class Storage(Indicator):
    _cls_ = "storage"
    for S1, S2, V1, V2 in [
        [Time, Time, c_double, c_double],
        [TransactionId, c_longlong, c_double, c_double],
    ]:
        get_func(_cls_, "new", S1, V1).argtypes = [S2]
        get_func(_cls_, "new", S1, V1).restype = Ptr
        get_func(_cls_, "destroy", S1, V1).argtypes = [Ptr]
        get_func(_cls_, "destroy", S1, V1).restype = None
        get_func(_cls_, "add", S1, V1).argtypes = [Ptr, S2, V2]
        get_func(_cls_, "add", S1, V1).restype = None

    def __init__(self, S, V, offset):
        self._S = S
        self._V = V
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(offset)

    def value(self, i):
        return get_func("indicator", "value", self._S, Option(self._V))(self._ptr.f_ptr, i)

    def add(self, time, value):
        get_func(self._cls_, "add", self._S, self._V)(self._ptr, time, value)


class Cached(Indicator):
    _cls_ = "cached"
    for S1, S2, V1, V2 in [
        [Time, Time, c_double, c_double],
        [TransactionId, c_longlong, c_double, c_double],
    ]:
        get_func(_cls_, "new", S1, V1).argtypes = [c_int, c_void_p]
        get_func(_cls_, "new", S1, V1).restype = Ptr
        get_func(_cls_, "destroy", S1, V1).argtypes = [Ptr]
        get_func(_cls_, "destroy", S1, V1).restype = None

    def __init__(self, S, V, capacity, source):
        self._S = S
        self._V = V
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(capacity, source._ptr.f_ptr)


class Sma(Indicator):
    _cls_ = "sma"
    for S1, S2, V1, V2 in [
        [Time, Time, c_double, c_double],
        [TransactionId, c_longlong, c_double, c_double],
    ]:
        get_func(_cls_, "new", S1, V1).argtypes = [c_void_p, c_int]
        get_func(_cls_, "new", S1, V1).restype = Ptr
        get_func(_cls_, "destroy", S1, V1).argtypes = [Ptr]
        get_func(_cls_, "destroy", S1, V1).restype = None

    def __init__(self, S, V, source, period):
        self._S = S
        self._V = V
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(source._ptr.f_ptr, period)


class Cmpl(Indicator):
    _cls_ = "cmpl"
    for S1, S2, V1, V2 in [
        [Time, Time, c_double, c_double],
        [TransactionId, c_longlong, c_double, c_double],
    ]:
        get_func(_cls_, "new", S1, V1).argtypes = [c_void_p, c_int]
        get_func(_cls_, "new", S1, V1).restype = Ptr
        get_func(_cls_, "destroy", S1, V1).argtypes = [Ptr]
        get_func(_cls_, "destroy", S1, V1).restype = None

    def __init__(self, S, V, source, capacity):
        self._S = S
        self._V = V
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(source._ptr.f_ptr, capacity)

class Cross:
    _cls_ = "cross"
    for S1, S2, V1, V2 in [
        [Time, Time, c_double, c_double],
        [TransactionId, c_longlong, c_double, c_double],
    ]:
        get_func(_cls_, "new", S1, V1).argtypes = [c_void_p, c_void_p]
        get_func(_cls_, "new", S1, V1).restype = Ptr
        get_func(_cls_, "destroy", S1, V1).argtypes = [Ptr]
        get_func(_cls_, "destroy", S1, V1).restype = None
    # for T, t_str in {
    #     c_double: "f64",
    # }.items():
    #     get_func(_cls_, "new", T).argtypes = [c_void_p, c_void_p]
    #     get_func(_cls_, "new", T).restype = Ptr
    #     get_func(_cls_, "destroy", T).argtypes = [Ptr]
    #     get_func(_cls_, "destroy", T).restype = None

    def __init__(self, S, V, source_1, source_2):
        self._S = S
        self._V = V
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(source_1._ptr.f_ptr, source_2._ptr.f_ptr)

    def value(self, i):
        return get_func("indicator", "value", self._S, CrossState)(self._ptr.f_ptr, i)

# class Func:
#     def __init__(self, T, value_func, *sources):
#         self.T = T
#         self.sources = sources
#         self.value_func = value_func

#     def value(self, i):
#         args = [source.value(i) for source in self.sources]
#         out_of_range_args = [arg for arg in args if arg.is_value == 0 ]
#         if len(out_of_range_args) == 0:
#             v = self.value_func(*[arg.content for arg in args])
#             return MaybeValue(self.T).value(v)
#         return MaybeValue(self.T).out_of_range()

# class Slope(Indicator):
#     _cls_ = "slope"
#     for T in [
#         c_double,
#     ]:
#         get_func(_cls_, "new", T).argtypes = [c_void_p]
#         get_func(_cls_, "new", T).restype = Ptr
#         get_func(_cls_, "destroy", T).argtypes = [Ptr]
#         get_func(_cls_, "destroy", T).restype = None

#     def __init__(self, T, source):
#         self._T = T
#         self._ptr = get_func(self._cls_, "new", self._T)(source._ptr.f_ptr)

# class IterFunc:
#     def __init__(self, T1, T2, source, offset, func):
#         self.T1 = T1
#         self.T2 = T2
#         self.source = source
#         self.offset = offset
#         self.func = func
#         self.vec = Vec(offset, self.T2, [])
#         self._ptr = self.vec._ptr

#     def __next(self):
#         v = self.source.value(self.offset)
#         if v.is_value:
#             self.offset += 1
#             v2 = self.func(v.content)
#             self.vec.add(v2)
#             return MaybeValue(self.T2).value(v2)
#         else:
#             return v

#     def value(self, i):
#         while self.offset <= i:
#             v = self.__next();
#             if v.is_value == 0:
#                 break
#         return self.vec.value(i)

# # class ViaIterMap(Indicator):
# #     _cls_ = "via_iter"
# #     for T in [
# #         c_double,
# #     ]:
# #         get_func(_cls_, "new", T).argtypes = [c_void_p, Time]
# #         get_func(_cls_, "new", T).restype = Ptr
# #         get_func(_cls_, "destroy", T).argtypes = [Ptr]
# #         get_func(_cls_, "destroy", T).restype = None

# #     def __init__(self, T, source):
# #         self._T = T
# #         self._ptr = get_func(self._cls_, "new", self._T)(source._ptr.f_ptr)


# # TrailingStopSignal = c_int
# # getattr(mydll, "indicator_value_trailingstopsignal").argtypes = [c_void_p, Time]
# # getattr(mydll, "indicator_value_trailingstopsignal").restype = Option(c_int)
# # class TrailingStop:
# #     getattr(mydll, "trailingstop_new").argtypes = [c_void_p, c_void_p, c_double]
# #     getattr(mydll, "trailingstop_new").restype = Ptr
# #     getattr(mydll, "trailingstop_destroy").argtypes = [Ptr]
# #     getattr(mydll, "trailingstop_destroy").restype = None

# #     def __init__(self, T, price, position, stop_level):
# #         self.T = T
# #         self.ptr = getattr(mydll, "trailingstop_new")(price.ptr.f_ptr, position.ptr.f_ptr, stop_level)

# #     def value(self, i):
# #         return getattr(mydll, "indicator_value_trailingstopsignal")(self.ptr.f_ptr, i)

# #     def __del__(self):
# #         getattr(mydll, "trailingstop_destroy")(self.ptr)
# #         self.ptr = None
