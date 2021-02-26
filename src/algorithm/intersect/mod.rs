mod heap;
mod inner;
mod outer;

use crate::{ChromName, algorithm::markers::Sorted};
use crate::properties::WithRegion;

use inner::{SortedIntersectIter, Context, State};

pub trait SortedIntersect: Iterator + Sorted + Sized {
    fn sorted_intersect<C: ChromName, U: WithRegion<C> + Clone, Other: Iterator<Item = U> + Sorted>(
        self,
        other: Other,
    ) -> SortedIntersectIter<C, Self, Other> 
    where
        Self::Item: WithRegion<C> + Clone,
    {
        SortedIntersectIter {
            context_a: Context::from_iter(self),
            context_b: Context::from_iter(other),
            state: State::FrontierA(0, 0, None),
        }
    }

    fn sorted_left_outer_intersect<C: ChromName, U: WithRegion<C> + Clone, Other: Iterator<Item = U> + Sorted>(
        self,
        other: Other
    ) -> outer::LeftOuterJoinIter<C, Self, Other> 
    where
        Self::Item: WithRegion<C> + Clone,
    {
        outer::LeftOuterJoinIter::new(self, other)
    }
}

impl<I: Iterator + Sorted> SortedIntersect for I {}

