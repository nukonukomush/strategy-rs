#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SimplePosition {
    Nothing,
    Long,
    Short,
}

pub mod ffi {
    use super::*;

    #[repr(C)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum CSimplePosition {
        Nothing = 0,
        Long = 1,
        Short = -1,
    }
    impl Default for CSimplePosition {
        fn default() -> Self {
            CSimplePosition::Nothing
        }
    }

    impl From<SimplePosition> for CSimplePosition {
        fn from(s: SimplePosition) -> Self {
            match s {
                SimplePosition::Nothing => CSimplePosition::Nothing,
                SimplePosition::Long => CSimplePosition::Long,
                SimplePosition::Short => CSimplePosition::Short,
            }
        }
    }

    impl std::convert::Into<SimplePosition> for CSimplePosition {
        fn into(self) -> SimplePosition {
            match self {
                CSimplePosition::Nothing => SimplePosition::Nothing,
                CSimplePosition::Long => SimplePosition::Long,
                CSimplePosition::Short => SimplePosition::Short,
            }
        }
    }
}
