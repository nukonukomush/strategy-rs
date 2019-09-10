
#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum SimpleSignal {
    Nothing,
    OpenLong,
    OpenShort,
    CloseLong,
    CloseShort,
    CloseLongAndOpenShort,
    CloseShortAndOpenLong,
}

