use crate::steps::{RunMacros, Step};

pub struct Pipeline;

impl Pipeline {
    pub fn step<H>(self, inputs: H) -> impl Step<Inputs = H>
    where
        H: RunMacros,
    {
        struct Container<H>(H);

        impl<H> Step for Container<H>
        where
            H: RunMacros,
        {
            type Inputs = H;

            fn into_contents(self) -> Self::Inputs {
                self.0
            }
        }

        Container(inputs)
    }
}
