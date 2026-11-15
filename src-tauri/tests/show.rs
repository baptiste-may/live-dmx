//! Integration test harness for the `show` module (state, history, patch, configure, MVR, serialization).

mod show {
    mod configure;
    mod history;
    mod identity;
    mod mvr;
    mod output;
    mod patch {
        mod batch;
        mod groups;
    }
    mod recent;
    mod serialization;
    mod state;
}
