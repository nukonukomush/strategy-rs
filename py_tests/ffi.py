import os
from ctypes import *

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

class SimplePosition(c_int):
    pass

class CrossState(c_int):
    pass

class ZoneId(c_int):
    pass

class TransactionId(c_longlong):
    pass

class TickId(c_longlong):
    pass

type_map = {
    c_int: "i32",
    c_double: "f64",
    SimplePosition: "simple_position",
    CrossState: "cross",
    TransactionId: "tid",
    TickId: "tick_id",
    Time: "time",
    ZoneId: "zone_id",
}

def default(T):
    defaults = {
        c_int: lambda: 0,
        c_double: lambda: 0.0,
        Option(c_double): Option(c_double).none,
        CrossState: 0,
        SimplePosition: 0,
        ZoneId: 0,
        # MaybeInRange(c_int): MaybeInRange(c_int).none, 
        # MaybeInRange(c_double): MaybeInRange(c_double).none, 
        # MaybeInRange(Option(c_double)): MaybeInRange(Option(c_double)).none, 
        # MaybeInRange(CrossState): MaybeInRange(CrossState).none, 
        # MaybeInRange(SimplePosition): MaybeInRange(SimplePosition).none, 
    }
    add = { MaybeInRange(k):MaybeInRange(k).out_of_range for k in defaults.keys() }
    for k, v in add.items():
        defaults[k] = v
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


def MaybeInRange_eq(self, other):
    if other is None or not isinstance(other, self.__class__):
        return False
    if self.is_in_range == other.is_in_range:
        if self.is_in_range:
            return self.value == other.value
        else:
            return True
    else:
        return False

def MaybeInRange_repr(self):
    if self.is_in_range:
        return "InRange({})".format(str(self.value))
    else:
        return "OutOfRange"

def MaybeInRange_nullable(self):
    if self.is_in_range:
        return self.value
    return None

def MaybeInRange_from_nullable(T, n):
    if n is None:
        return MaybeInRange(T).out_of_range()
    return MaybeInRange(T).in_range(n)

maybeinrange_types = {}
for T, t_str in type_map.items():
    def def_maybeinrange(t):
        T = t
        maybeinrange_t_str = "MaybeInRange_{}".format(t_str)
        maybeinrange_types[T] = type(maybeinrange_t_str, (Structure,), {
            "__eq__": MaybeInRange_eq,
            "__repr__": MaybeInRange_repr,
            "nullable": MaybeInRange_nullable,
        })
        maybeinrange_types[T]._fields_ = [
            ("is_in_range", c_byte),
            ("value", T),
        ]
        maybeinrange_types[T].in_range = lambda v: maybeinrange_types[T](1, v)
        maybeinrange_types[T].out_of_range = lambda : maybeinrange_types[T](0, default(T))
        maybeinrange_types[T].from_nullable = MaybeInRange_from_nullable
        maybeinrange_types[T].T = T
    def_maybeinrange(T)

def MaybeInRange(T):
    if T in maybeinrange_types:
        return maybeinrange_types[T]
    else:
        raise TypeError("type {} is not available for MaybeInRange.".format(T))

tmap = { MaybeInRange(k):"maybe_in_range_" + v for k,v in type_map.items() }
for k, v in tmap.items():
    type_map[k] = v
# type_map[MaybeInRange(c_double)] = "maybe_in_range_f64"
# type_map[MaybeInRange(c_int)] = "maybe_in_range_i32"

def MaybeFixed_eq(self, other):
    if other is None or not isinstance(other, self.__class__):
        return False
    if self.is_fixed == other.is_fixed:
        if self.is_fixed:
            return self.value == other.value
        else:
            return True
    else:
        return False

def MaybeFixed_repr(self):
    if self.is_fixed:
        return "Fixed({})".format(str(self.value))
    else:
        return "NotFixed"

def MaybeFixed_nullable(self):
    if self.is_fixed:
        return self.value
    return None

def MaybeFixed_from_nullable(T, n):
    if n is None:
        return MaybeFixed(T).not_fixed()
    return MaybeFixed(T).fixed(n)

maybefixed_types = {}
for T, t_str in type_map.items():
    def def_maybefixed(t):
        T = t
        maybefixed_t_str = "MaybeFixed_{}".format(t_str)
        maybefixed_types[T] = type(maybefixed_t_str, (Structure,), {
            "__eq__": MaybeFixed_eq,
            "__repr__": MaybeFixed_repr,
            "nullable": MaybeFixed_nullable,
        })
        maybefixed_types[T]._fields_ = [
            ("is_fixed", c_byte),
            ("value", T),
        ]
        maybefixed_types[T].fixed = lambda v: maybefixed_types[T](1, v)
        maybefixed_types[T].not_fixed = lambda : maybefixed_types[T](0, default(T))
        maybefixed_types[T].from_nullable = MaybeFixed_from_nullable
        maybefixed_types[T].T = T
    def_maybefixed(T)

def MaybeFixed(T):
    if T in maybefixed_types:
        return maybefixed_types[T]
    else:
        raise TypeError("type {} is not available for MaybeFixed.".format(T))

def MaybeValue(T):
    return MaybeFixed(MaybeInRange(T))

def value(T, v):
    return MaybeValue(T).fixed(MaybeInRange(T).in_range(v))

def out_of_range(T):
    return MaybeValue(T).fixed(MaybeInRange(T).out_of_range())

def not_fixed(T):
    return MaybeValue(T).not_fixed()


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
        [TickId, c_longlong, c_double, c_double],
        [TickId, c_longlong, Time, Time],
        [TickId, c_longlong, ZoneId, c_int],
    ]:
        get_func("indicator", "value", S1, V1).argtypes = [c_void_p, S2]
        get_func("indicator", "value", S1, V1).restype = MaybeValue(V2)
        # get_func("indicator", "value", S1, V1).restype = MaybeFixed(MaybeInRange(V2))

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
        [TickId, c_longlong, c_double, c_double],
        [TickId, c_longlong, Time, Time],
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

    def __init__(self, S, V, source_1, source_2):
        self._S = S
        self._V = V
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(source_1._ptr.f_ptr, source_2._ptr.f_ptr)

    def value(self, i):
        return get_func("indicator", "value", self._S, CrossState)(self._ptr.f_ptr, i)

class Func:
    def __init__(self, V, value_func, *sources):
        self.V = V
        self.sources = sources
        self.value_func = value_func

    def value(self, i):
        args = [source.value(i) for source in self.sources]
        not_fixed_args = [arg for arg in args if arg.is_fixed == 0]
        out_of_range_args = [arg for arg in args if arg.is_fixed == 1 and arg.value.is_in_range == 0]
        if len(out_of_range_args) != 0:
            return out_of_range(self.V)
        elif len(not_fixed_args) != 0:
            return not_fixed(self.V)
        else:
            v = self.value_func(*[arg.value.value for arg in args])
            return value(self.V, v)

class Slope(Indicator):
    _cls_ = "slope"
    for S1, S2, V1, V2 in [
        [Time, Time, c_double, c_double],
        [TransactionId, c_longlong, c_double, c_double],
    ]:
        get_func(_cls_, "new", S1, V1).argtypes = [c_void_p]
        get_func(_cls_, "new", S1, V1).restype = Ptr
        get_func(_cls_, "destroy", S1, V1).argtypes = [Ptr]
        get_func(_cls_, "destroy", S1, V1).restype = None

    def __init__(self, S, V, source):
        self._S = S
        self._V = V
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(source._ptr.f_ptr)

class IterFunc:
    def __init__(self, S, V1, V2, source, offset, func):
        self.S = S
        self.V1 = V1
        self.V2 = V2
        self.source = source
        self.offset = offset
        self.func = func
        self.vec = Vec(self.S, self.V2, [], offset)
        self._ptr = self.vec._ptr

    def __next(self):
        v = self.source.value(self.offset)
        if v.is_fixed:
            self.offset += 1
            if v.value.is_in_range:
                v2 = self.func(v.value.value)
                self.vec.add(v2)
                return value(self.V2, v2)
            else:
                return out_of_range(self.V2)
        else:
            return not_fixed(self.V2)

    def value(self, i):
        while self.offset <= i:
            v = self.__next();
            if v.is_fixed == 0 or v.value.is_in_range == 0:
                break
        return self.vec.value(i)

class TimeToTick(Indicator):
    _cls_ = "tick"
    for S1, S2, V1, V2 in [
        [TickId, TickId, c_double, c_double],
    ]:
        get_func(_cls_, "new", S1, V1).argtypes = [c_void_p, c_void_p]
        get_func(_cls_, "new", S1, V1).restype = Ptr
        get_func(_cls_, "destroy", S1, V1).argtypes = [Ptr]
        get_func(_cls_, "destroy", S1, V1).restype = None

    def __init__(self, S, V, values, time):
        self._S = S
        self._V = V
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(
            values._ptr.f_ptr,
            time._ptr.f_ptr
        )

class Zone(Indicator):
    _cls_ = "zone"
    for S1, S2, V1, V2 in [
        [TickId, TickId, c_double, c_double],
    ]:
        get_func(_cls_, "new", S1, V1).argtypes = [c_void_p, POINTER(c_void_p),
                                                   c_int, POINTER(c_void_p), c_int]
        get_func(_cls_, "new", S1, V1).restype = Ptr
        get_func(_cls_, "destroy", S1, V1).argtypes = [Ptr]
        get_func(_cls_, "destroy", S1, V1).restype = None

    def __init__(self, S, V, price, positive_lines, negative_lines):
        self._S = S
        self._V = V
        positive_length = len(positive_lines)
        positive_line_ptrs = [i._ptr.f_ptr for i in positive_lines]
        p_lines_ptr = POINTER(c_void_p)((c_void_p * positive_length)(*positive_line_ptrs))
        negative_length = len(negative_lines)
        negative_line_ptrs = [i._ptr.f_ptr for i in negative_lines]
        n_lines_ptr = POINTER(c_void_p)((c_void_p * negative_length)(*negative_line_ptrs))
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(
            price._ptr.f_ptr,
            p_lines_ptr,
            positive_length,
            n_lines_ptr,
            negative_length,
        )

    def value(self, i):
        return get_func("indicator", "value", self._S, ZoneId)(self._ptr.f_ptr, i)

class Envelope(Indicator):
    _cls_ = "envelope"
    for S1, S2, V1, V2 in [
        [Time, Time, c_double, c_double],
        [TickId, c_longlong, c_double, c_double],
    ]:
        get_func(_cls_, "new", S1, V1).argtypes = [c_void_p, c_double]
        get_func(_cls_, "new", S1, V1).restype = Ptr
        get_func(_cls_, "destroy", S1, V1).argtypes = [Ptr]
        get_func(_cls_, "destroy", S1, V1).restype = None

    def __init__(self, S, V, source, deviation_in_percents):
        self._S = S
        self._V = V
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(source._ptr.f_ptr, deviation_in_percents)

class Ema(Indicator):
    _cls_ = "ema"
    for S1, S2, V1, V2 in [
        [Time, Time, c_double, c_double],
        [TickId, c_longlong, c_double, c_double],
    ]:
        get_func(_cls_, "new", S1, V1).argtypes = [c_void_p, c_void_p, c_int, c_double, c_int]
        get_func(_cls_, "new", S1, V1).restype = Ptr
        get_func(_cls_, "destroy", S1, V1).argtypes = [Ptr]
        get_func(_cls_, "destroy", S1, V1).restype = None

    def __init__(self, S, V, source, first, n_period, accuracy, capacity):
        self._S = S
        self._V = V
        self._ptr = get_func(self._cls_, "new", self._S, self._V)(
            source._ptr.f_ptr,
            first._ptr.f_ptr,
            n_period,
            accuracy,
            capacity,
        )

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
