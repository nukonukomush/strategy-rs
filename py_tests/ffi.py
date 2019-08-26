import os
from ctypes import *

type_map = {
    c_int: "i32",
    c_double: "f64",
}

def default(T):
    return 0

def get_rust_type(T):
    if T in type_map:
        return type_map[T]
    else:
        # TODO: raise Error
        pass

dirname = os.path.dirname(os.path.abspath(__file__))
# mydll = cdll.LoadLibrary("{}/../target/debug/libstrategy.dylib".format(dirname))
mydll = cdll.LoadLibrary("{}/libstrategy.dylib".format(dirname))

def Option_eq(self, other):
    # print(self.__class__, other.__class__)
    if other is None or not isinstance(other, self.__class__):
        return False
    if self.is_some == other.is_some:
        if self.is_some:
            return self.value == other.value
        else:
            return True
    else:
        return False

def Option_repr(self):
    if self.is_some:
        return "Some({})".format(str(self.value))
    else:
        return "None"

def Option_nullable(self):
    if self.is_some:
        return self.value
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
            ("value", T),
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


class Ptr(Structure):
    _fields_ = [
        ("b_ptr", c_void_p),
        ("t_ptr", c_void_p),
    ]

getattr(mydll, "indicator_value_{}".format("f64")).argtypes = [c_void_p, Time]
getattr(mydll, "indicator_value_{}".format("f64")).restype = Option(c_double)

class Vec:
    for T, t_str in {
        c_double: "f64",
    }.items():
        getattr(mydll, "vec_new_{}".format(t_str)).argtypes = [Time, POINTER(T), c_int]
        getattr(mydll, "vec_new_{}".format(t_str)).restype = Ptr
        getattr(mydll, "vec_destroy_{}".format(t_str)).argtypes = [Ptr]
        getattr(mydll, "vec_destroy_{}".format(t_str)).restype = None

    def __init__(self, offset, T, vec):
        self.T = T
        length = len(vec)
        arr = (T * length)(*vec)
        ptr = POINTER(T)(arr)
        self.ptr = getattr(mydll, "vec_new_{}".format(get_rust_type(self.T)))(offset, ptr, length)

    def value(self, i):
        return getattr(mydll, "indicator_value_{}".format(get_rust_type(self.T)))(self.ptr.t_ptr, i)

    def __del__(self):
        getattr(mydll, "vec_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
        self.ptr = None

class Hash:
    for T, t_str in {
        c_double: "f64",
    }.items():
        getattr(mydll, "hash_new_{}".format(t_str)).argtypes = [c_int]
        getattr(mydll, "hash_new_{}".format(t_str)).restype = Ptr
        getattr(mydll, "hash_destroy_{}".format(t_str)).argtypes = [Ptr]
        getattr(mydll, "hash_destroy_{}".format(t_str)).restype = None
        getattr(mydll, "hash_set_{}".format(t_str)).argtypes = [Ptr, Time, T]
        getattr(mydll, "hash_set_{}".format(t_str)).restype = None

    def __init__(self, T, granularity):
        self.T = T
        self.ptr = getattr(mydll, "hash_new_{}".format(get_rust_type(self.T)))(granularity)

    def value(self, i):
        return getattr(mydll, "indicator_value_{}".format(get_rust_type(self.T)))(self.ptr.t_ptr, i)

    def __del__(self):
        getattr(mydll, "hash_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
        self.ptr = None

    def set(self, time, value):
        getattr(mydll, "hash_set_{}".format(get_rust_type(self.T)))(self.ptr, time, value)


class Sma:
    for T, t_str in {
        c_double: "f64",
    }.items():
        getattr(mydll, "sma_new_{}".format(t_str)).argtypes = [c_void_p, c_int]
        getattr(mydll, "sma_new_{}".format(t_str)).restype = Ptr
        getattr(mydll, "sma_destroy_{}".format(t_str)).argtypes = [Ptr]
        getattr(mydll, "sma_destroy_{}".format(t_str)).restype = None

    def __init__(self, T, source, period):
        self.T = T
        self.ptr = getattr(mydll, "sma_new_{}".format(get_rust_type(self.T)))(source.ptr.t_ptr, period)

    def value(self, i):
        return getattr(mydll, "indicator_value_{}".format(get_rust_type(self.T)))(self.ptr.t_ptr, i)

    def __del__(self):
        getattr(mydll, "sma_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
        self.ptr = None

class Cached:
    for T, t_str in {
        c_double: "f64",
    }.items():
        getattr(mydll, "cached_new_{}".format(t_str)).argtypes = [c_int, c_void_p]
        getattr(mydll, "cached_new_{}".format(t_str)).restype = Ptr
        getattr(mydll, "cached_destroy_{}".format(t_str)).argtypes = [Ptr]
        getattr(mydll, "cached_destroy_{}".format(t_str)).restype = None

    def __init__(self, T, capacity, source):
        self.T = T
        self.ptr = getattr(mydll, "cached_new_{}".format(get_rust_type(self.T)))(capacity, source.ptr.t_ptr)

    def value(self, i):
        return getattr(mydll, "indicator_value_{}".format(get_rust_type(self.T)))(self.ptr.t_ptr, i)

    def __del__(self):
        getattr(mydll, "cached_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
        self.ptr = None


CrossState = c_int
getattr(mydll, "indicator_value_cross").argtypes = [c_void_p, Time]
getattr(mydll, "indicator_value_cross").restype = Option(c_int)
class Cross:
    for T, t_str in {
        c_double: "f64",
    }.items():
        getattr(mydll, "cross_new_{}".format(t_str)).argtypes = [c_void_p, c_void_p]
        getattr(mydll, "cross_new_{}".format(t_str)).restype = Ptr
        getattr(mydll, "cross_destroy_{}".format(t_str)).argtypes = [Ptr]
        getattr(mydll, "cross_destroy_{}".format(t_str)).restype = None

    def __init__(self, T, source_1, source_2):
        self.T = T
        self.ptr = getattr(mydll, "cross_new_{}".format(get_rust_type(self.T)))(source_1.ptr.t_ptr, source_2.ptr.t_ptr)

    def value(self, i):
        return getattr(mydll, "indicator_value_cross")(self.ptr.t_ptr, i)

    def __del__(self):
        getattr(mydll, "cross_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
        self.ptr = None

class Cmpl:
    for T, t_str in {
        c_double: "f64",
    }.items():
        getattr(mydll, "cmpl_new_{}".format(t_str)).argtypes = [c_void_p, c_int, c_int]
        getattr(mydll, "cmpl_new_{}".format(t_str)).restype = Ptr
        getattr(mydll, "cmpl_destroy_{}".format(t_str)).argtypes = [Ptr]
        getattr(mydll, "cmpl_destroy_{}".format(t_str)).restype = None

    def __init__(self, T, source, max_loop, capacity):
        self.T = T
        self.ptr = getattr(mydll, "cmpl_new_{}".format(get_rust_type(self.T)))(source.ptr.t_ptr, max_loop, capacity)

    def value(self, i):
        return getattr(mydll, "indicator_value_{}".format(get_rust_type(self.T)))(self.ptr.t_ptr, i)

    def __del__(self):
        getattr(mydll, "cmpl_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
        self.ptr = None

class Func:
    def __init__(self, T, value_func, *sources):
        self.T = T
        self.sources = sources
        self.value_func = value_func

    def value(self, i):
        return Option(self.T).from_nullable(self.T, self.value_nullable(i))

    def value_nullable(self, i):
        args = [source.value(i).nullable() for source in self.sources]
        none_args = [arg for arg in args if arg is None]
        if len(none_args) == 0:
            return self.value_func(*args)
        return None

class Slope:
    for T, t_str in {
        c_double: "f64",
    }.items():
        getattr(mydll, "slope_new_{}".format(t_str)).argtypes = [c_void_p]
        getattr(mydll, "slope_new_{}".format(t_str)).restype = Ptr
        getattr(mydll, "slope_destroy_{}".format(t_str)).argtypes = [Ptr]
        getattr(mydll, "slope_destroy_{}".format(t_str)).restype = None

    def __init__(self, T, source):
        self.T = T
        self.ptr = getattr(mydll, "slope_new_{}".format(get_rust_type(self.T)))(source.ptr.t_ptr)

    def value(self, i):
        return getattr(mydll, "indicator_value_{}".format(get_rust_type(self.T)))(self.ptr.t_ptr, i)

    def __del__(self):
        getattr(mydll, "slope_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
        self.ptr = None
