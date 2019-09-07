use super::*;
use crate::seq::*;
use crate::position::*;

pub struct TrailingStop<G, IPrice, IPos> {
    price: IPrice,
    position: IPos,
    stop_level: f64,
    phantom: std::marker::PhantomData<G>,
}

impl<G, IPrice, IPos> TrailingStop<G, IPrice, IPos>
where
    IPrice: Indicator<G, f64>,
    IPos: Indicator<G, SimplePosition>,
{
    pub fn new(price: IPrice, position: IPos, stop_level: f64) -> Self {
        Self {
            price: price,
            position: position,
            stop_level: stop_level,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<G, IPrice, IPos> Indicator<G, TrailingStopSignal> for TrailingStop<G, IPrice, IPos>
where
    G: Granularity + Copy + Ord,
    IPrice: Indicator<G, f64>,
    IPos: Indicator<G, SimplePosition>,
{
    fn value(&self, time: Time<G>) -> Option<TrailingStopSignal> {
        use SimplePosition::*;
        use TrailingStopSignal::*;
        let pos = self.position.value(time)?;
        let price = self.price.value(time)?;
        // TODO: performance tuning
        let order_begin = {
            let mut i = time - 1;
            while self.position.value(i) == Some(pos.clone()) {
                i = i - 1;
            }
            i + 1
        };
        let prices_in_order = order_begin
            .range_to_end(time + 1)
            .filter_map(|i| self.price.value(i));

        let signal = match pos {
            Nothing => Continue,
            Long => {
                let max = prices_in_order.fold(-1.0 / 0.0, f64::max);
                if max - price > self.stop_level {
                    Stop
                } else {
                    Continue
                }
            }
            Short => {
                let min = prices_in_order.fold(1.0 / 0.0, f64::min);
                if price - min > self.stop_level {
                    Stop
                } else {
                    Continue
                }
            }
        };
        Some(signal)
    }
    fn granularity(&self) -> G {
        self.price.granularity()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TrailingStopSignal {
    Continue,
    Stop,
}

pub mod ffi {
    use super::*;
    use crate::indicator::ffi::*;
    use crate::indicator::*;
    use crate::position::ffi::*;
    use crate::position::*;
    use crate::time::ffi::*;
    use std::cell::RefCell;
    use std::mem::drop;
    use std::os::raw::*;
    use std::ptr;
    use std::rc::Rc;

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum CTrailingStopSignal {
        Continue = 0,
        Stop = 1,
    }
    impl Default for CTrailingStopSignal {
        fn default() -> Self {
            CTrailingStopSignal::Continue
        }
    }

    impl From<TrailingStopSignal> for CTrailingStopSignal {
        fn from(s: TrailingStopSignal) -> Self {
            match s {
                TrailingStopSignal::Continue => CTrailingStopSignal::Continue,
                TrailingStopSignal::Stop => CTrailingStopSignal::Stop,
            }
        }
    }

    #[repr(C)]
    pub struct Ptr {
        b_ptr: *mut Rc<
            RefCell<TrailingStop<VarGranularity, IndicatorPtr<f64>, IndicatorPtr<SimplePosition>>>,
        >,
        t_ptr: *mut IndicatorPtr<TrailingStopSignal>,
    }

    macro_rules! define_trailingstop_methods {
        ($new:ident, $destroy:ident) => {
            #[no_mangle]
            pub unsafe extern "C" fn $new(
                price: *mut IndicatorPtr<f64>,
                position: *mut IndicatorPtr<SimplePosition>,
                stop_level: f64,
            ) -> Ptr {
                let price = (*price).clone();
                let position = (*position).clone();
                let ptr = Rc::new(RefCell::new(TrailingStop::new(price, position, stop_level)));
                Ptr {
                    b_ptr: Box::into_raw(Box::new(ptr.clone())),
                    t_ptr: Box::into_raw(Box::new(IndicatorPtr(ptr))),
                }
            }

            #[no_mangle]
            pub unsafe extern "C" fn $destroy(ptr: Ptr) {
                destroy(ptr.b_ptr);
                destroy(ptr.t_ptr);
            }
        };
    }

    define_trailingstop_methods!(trailingstop_new, trailingstop_destroy);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::SimplePosition::*;
    use crate::vec::*;
    use TrailingStopSignal::*;

    #[test]
    fn test_1() {
        let offset = Time::new(0, S5);
        let price = vec![1.0, 2.0, -3.0, 8.0, 3.0];
        let price = VecIndicator::new(offset, price);
        let position = vec![Long; 5];
        let position = VecIndicator::new(offset, position);
        let expect = vec![Continue, Continue, Stop, Continue, Stop];
        let trailing_stop = TrailingStop::new(price, position, 4.0);

        let result = (0..5)
            .map(|i| trailing_stop.value(offset + i).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(result, expect);
    }
}
