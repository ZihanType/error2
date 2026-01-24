use crate::{Location, private};

/// Internal trait for transforming middle-stage errors to target errors.
///
/// This trait is used by the derive macro's generated code and is not
/// intended for direct use.
pub trait MiddleToTarget<Middle, Target> {
    /// Transforms a middle-stage error to the target type.
    fn middle_to_target(self, middle: Middle, location: Location) -> Target;
}

/// Internal trait for transforming source errors to target errors.
///
/// This trait is used by the derive macro's generated code and is not
/// intended for direct use.
pub trait SourceToTarget<M, Source, Middle, Target>
where
    Source: Into<Middle>,
{
    /// Transforms a source error to the target type.
    fn source_to_target(self, source: Source, location: Location) -> Target;
}

impl<Source, Middle, Target, C> SourceToTarget<private::ViaPartial, Source, Middle, Target> for C
where
    Source: Into<Middle>,
    C: MiddleToTarget<Middle, Target>,
{
    #[inline]
    fn source_to_target(self, source: Source, location: Location) -> Target {
        let middle: Middle = source.into();
        self.middle_to_target(middle, location)
    }
}
