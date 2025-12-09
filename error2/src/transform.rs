use crate::{Location, private};

pub trait MiddleToTarget<Middle, Target> {
    fn middle_to_target(self, middle: Middle, location: Location) -> Target;
}

pub trait SourceToTarget<M, Source, Middle, Target>
where
    Source: Into<Middle>,
{
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
