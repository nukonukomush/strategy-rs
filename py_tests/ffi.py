import os
from ctypes import *

class SimplePosition(c_int):
    pass

type_map = {
    c_int: "i32",
    c_double: "f64",
    # SimplePosition: "simpleposition",
}

def default(T):
    defaults = {
        c_int: lambda: 0,
        c_double: lambda: 0.0,
        Option(c_double): Option(c_double).none,
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

def get_func(cls, method, T):
    t_str = get_rust_type(T)
    name = "{}_{}_{}".format(cls, method, t_str)
    return getattr(mydll, name)

def Option_eq(self, other):
    # print(self.__class__, other.__class__)
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
    print(self.__class__, other.__class__)
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


class Ptr(Structure):
    _fields_ = [
        ("b_ptr", c_void_p),
        ("f_ptr", c_void_p),
        ("i_ptr", c_void_p),
    ]

get_func("indicator", "value", c_double).argtypes = [c_void_p, Time]
get_func("indicator", "value", c_double).restype = MaybeValue(c_double)
get_func("indicator", "value", c_int).argtypes = [c_void_p, Time]
get_func("indicator", "value", c_int).restype = MaybeValue(c_int)
get_func("indicator", "value", Option(c_double)).argtypes = [c_void_p, Time]
get_func("indicator", "value", Option(c_double)).restype = MaybeValue(Option(c_double))
# getattr(mydll, "indicator_value_{}".format("simpleposition")).argtypes = [c_void_p, Time]
# getattr(mydll, "indicator_value_{}".format("simpleposition")).restype = Option(c_int)

class Indicator:
    _cls_ = None
    for T in [
        c_double,
        c_int,
        Option(c_double),
    ]:
        get_func("indicator", "value", T).argtypes = [c_void_p, Time]
        get_func("indicator", "value", T).restype = MaybeValue(T)

    def value(self, i):
        return get_func("indicator", "value", self._T)(self._ptr.f_ptr, i)

    def __del__(self):
        get_func(self._cls_, "destroy", self._T)(self._ptr)
        self._ptr = None


class Vec(Indicator):
    _cls_ = "vec"
    for T in [
        c_double,
    ]:
        get_func(_cls_, "new", T).argtypes = [Time, POINTER(T), c_int]
        get_func(_cls_, "new", T).restype = Ptr
        get_func(_cls_, "destroy", T).argtypes = [Ptr]
        get_func(_cls_, "destroy", T).restype = None

    def __init__(self, offset, T, vec):
        self._T = T
        length = len(vec)
        arr = (T * length)(*vec)
        ptr = POINTER(T)(arr)
        self._ptr = get_func(self._cls_, "new", self._T)(offset, ptr, length)

class Storage(Indicator):
    _cls_ = "storage"
    for T in [
        c_double,
        # SimplePosition,
    ]:
        get_func(_cls_, "new", T).argtypes = [Time]
        get_func(_cls_, "new", T).restype = Ptr
        get_func(_cls_, "destroy", T).argtypes = [Ptr]
        get_func(_cls_, "destroy", T).restype = None
        get_func(_cls_, "add", T).argtypes = [Ptr, Time, T]
        get_func(_cls_, "add", T).restype = None

    def __init__(self, T, granularity):
        self._T = T
        self._ptr = get_func(self._cls_, "new", self._T)(granularity)

    def value(self, i):
        return get_func("indicator", "value", Option(self._T))(self._ptr.f_ptr, i)

    def add(self, time, value):
        get_func(self._cls_, "add", self._T)(self._ptr, time, value)


class Cached(Indicator):
    _cls_ = "cached"
    for T in [
        c_double,
    ]:
        get_func(_cls_, "new", T).argtypes = [c_int, c_void_p]
        get_func(_cls_, "new", T).restype = Ptr
        get_func(_cls_, "destroy", T).argtypes = [Ptr]
        get_func(_cls_, "destroy", T).restype = None

    def __init__(self, T, capacity, source):
        self._T = T
        self._ptr = get_func(self._cls_, "new", self.T)(capacity, source._ptr.f_ptr)


class Sma(Indicator):
    _cls_ = "sma"
    for T in [
        c_double,
    ]:
        get_func(_cls_, "new", T).argtypes = [c_void_p, c_int]
        get_func(_cls_, "new", T).restype = Ptr
        get_func(_cls_, "destroy", T).argtypes = [Ptr]
        get_func(_cls_, "destroy", T).restype = None

    def __init__(self, T, source, period):
        self._T = T
        self._ptr = get_func(self._cls_, "new", self._T)(source._ptr.f_ptr, period)




# CrossState = c_int
# getattr(mydll, "indicator_value_cross").argtypes = [c_void_p, Time]
# getattr(mydll, "indicator_value_cross").restype = Option(c_int)
# class Cross:
#     for T, t_str in {
#         c_double: "f64",
#     }.items():
#         getattr(mydll, "cross_new_{}".format(t_str)).argtypes = [c_void_p, c_void_p]
#         getattr(mydll, "cross_new_{}".format(t_str)).restype = Ptr
#         getattr(mydll, "cross_destroy_{}".format(t_str)).argtypes = [Ptr]
#         getattr(mydll, "cross_destroy_{}".format(t_str)).restype = None

#     def __init__(self, T, source_1, source_2):
#         self.T = T
#         self.ptr = getattr(mydll, "cross_new_{}".format(get_rust_type(self.T)))(source_1.ptr.f_ptr, source_2.ptr.f_ptr)

#     def value(self, i):
#         return getattr(mydll, "indicator_value_cross")(self.ptr.f_ptr, i)

#     def __del__(self):
#         getattr(mydll, "cross_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
#         self.ptr = None

# class Cmpl:
#     for T, t_str in {
#         c_double: "f64",
#     }.items():
#         getattr(mydll, "cmpl_new_{}".format(t_str)).argtypes = [c_void_p, c_int, c_int]
#         getattr(mydll, "cmpl_new_{}".format(t_str)).restype = Ptr
#         getattr(mydll, "cmpl_destroy_{}".format(t_str)).argtypes = [Ptr]
#         getattr(mydll, "cmpl_destroy_{}".format(t_str)).restype = None

#     def __init__(self, T, source, max_loop, capacity):
#         self.T = T
#         self.ptr = getattr(mydll, "cmpl_new_{}".format(get_rust_type(self.T)))(source.ptr.f_ptr, max_loop, capacity)

#     def value(self, i):
#         return getattr(mydll, "indicator_value_{}".format(get_rust_type(self.T)))(self.ptr.f_ptr, i)

#     def __del__(self):
#         getattr(mydll, "cmpl_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
#         self.ptr = None

# class Func:
#     def __init__(self, T, value_func, *sources):
#         self.T = T
#         self.sources = sources
#         self.value_func = value_func

#     def value(self, i):
#         return Option(self.T).from_nullable(self.T, self.value_nullable(i))

#     def value_nullable(self, i):
#         args = [source.value(i).nullable() for source in self.sources]
#         none_args = [arg for arg in args if arg is None]
#         if len(none_args) == 0:
#             return self.value_func(*args)
#         return None

# class Slope:
#     for T, t_str in {
#         c_double: "f64",
#     }.items():
#         getattr(mydll, "slope_new_{}".format(t_str)).argtypes = [c_void_p]
#         getattr(mydll, "slope_new_{}".format(t_str)).restype = Ptr
#         getattr(mydll, "slope_destroy_{}".format(t_str)).argtypes = [Ptr]
#         getattr(mydll, "slope_destroy_{}".format(t_str)).restype = None

#     def __init__(self, T, source):
#         self.T = T
#         self.ptr = getattr(mydll, "slope_new_{}".format(get_rust_type(self.T)))(source.ptr.f_ptr)

#     def value(self, i):
#         return getattr(mydll, "indicator_value_{}".format(get_rust_type(self.T)))(self.ptr.f_ptr, i)

#     def __del__(self):
#         getattr(mydll, "slope_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
#         self.ptr = None


# TrailingStopSignal = c_int
# getattr(mydll, "indicator_value_trailingstopsignal").argtypes = [c_void_p, Time]
# getattr(mydll, "indicator_value_trailingstopsignal").restype = Option(c_int)
# class TrailingStop:
#     getattr(mydll, "trailingstop_new").argtypes = [c_void_p, c_void_p, c_double]
#     getattr(mydll, "trailingstop_new").restype = Ptr
#     getattr(mydll, "trailingstop_destroy").argtypes = [Ptr]
#     getattr(mydll, "trailingstop_destroy").restype = None

#     def __init__(self, T, price, position, stop_level):
#         self.T = T
#         self.ptr = getattr(mydll, "trailingstop_new")(price.ptr.f_ptr, position.ptr.f_ptr, stop_level)

#     def value(self, i):
#         return getattr(mydll, "indicator_value_trailingstopsignal")(self.ptr.f_ptr, i)

#     def __del__(self):
#         getattr(mydll, "trailingstop_destroy")(self.ptr)
#         self.ptr = None
