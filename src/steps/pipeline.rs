use crate::hlist::IntoHList;
use crate::steps::{RunMacros, Step};

pub struct Pipeline;

impl Pipeline {
    #[allow(private_bounds)]
    pub fn define_step<H, Capability>(self, inputs: impl IntoHList<H>) -> impl Step<Capability, Inputs = H>
    where
        H: RunMacros<Capability>,
    {
        struct Container<H>(H);

        impl<C, H> Step<C> for Container<H>
        where
            H: RunMacros<C>,
        {
            type Inputs = H;

            fn into_contents(self) -> Self::Inputs {
                self.0
            }
        }

        Container(inputs.into_hlist())
    }
}
