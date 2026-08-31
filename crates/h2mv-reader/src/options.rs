use h2mv_format::ReaderLimits;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LockPolicy {
    Required,
    #[default]
    BestEffort,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct OpenOptions {
    pub lock_policy: LockPolicy,
    pub limits: ReaderLimits,
}
impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            lock_policy: LockPolicy::BestEffort,
            limits: ReaderLimits::default(),
        }
    }
}
