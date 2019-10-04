use super::*;
use MaybeFixed::*;
use MaybeInRange::*;

pub struct Envelope<I> {
    source: I,
    deviation: f64,
}

impl<I> Envelope<I>
where
    I: FuncIndicator,
{
    pub fn new(source: I, deviation_in_percents: f64) -> Self {
        Self {
            source: source,
            deviation: 1.0 + deviation_in_percents / 100.0,
        }
    }
}

impl<I> Indicator for Envelope<I>
where
    I: Indicator<Val = f64>,
{
    type Seq = I::Seq;
    type Val = I::Val;
}

impl<I> FuncIndicator for Envelope<I>
where
    I: FuncIndicator<Val = f64>,
{
    #[inline]
    fn value(&self, seq: Self::Seq) -> MaybeValue<Self::Val> {
        self.source.value(seq).map2(|v| v * self.deviation)
    }
}

impl<I> IterIndicator for Envelope<I>
where
    I: IterIndicator<Val = f64>,
{
    #[inline]
    fn next(&mut self) -> MaybeValue<Self::Val> {
        self.source.next().map2(|v| v * self.deviation)
    }

    #[inline]
    fn offset(&self) -> Self::Seq {
        self.source.offset()
    }
}

#[cfg(feature = "ffi")]
mod ffi {
    use super::*;
    use crate::granularity::ffi::*;
    use crate::indicator::ffi::*;
    use crate::time::ffi::*;

    type IPtr<S, V> = Ptr<S, V, Envelope<FuncIndicatorPtr<S, V>>>;

    macro_rules! define_new {
        ($s:ty, $cs:ty, $v:ty, $cv:ty, $name:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $name(
                source: *mut FuncIndicatorPtr<$s, $v>,
                deviation_in_percents: f64,
            ) -> IPtr<$s, $v> {
                let source = (*source).clone();
                let ptr = Envelope::new(source, deviation_in_percents).into_sync_ptr();
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    f_ptr: Box::into_raw(Box::new(FuncIndicatorPtr(ptr))),
                }
            }
        };
    }

    define_new!(GTime<Var>, CTime, f64, f64, envelope_new_time_f64);
    define_new!(TickId, i64, f64, f64, envelope_new_tick_id_f64);

    define_destroy!(IPtr<GTime<Var>, f64>, envelope_destroy_time_f64);
    define_destroy!(IPtr<TickId, f64>, envelope_destroy_tick_id_f64);
}
