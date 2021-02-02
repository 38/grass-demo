mod heap;
mod inner;
mod outer;

use crate::algorithm::markers::Sorted;
use crate::properties::WithRegion;

use inner::{SortedIntersectIter, Context, State};

pub trait SortedIntersect: Iterator + Sorted + Sized
where
    Self::Item: WithRegion + Clone,
{
    fn sorted_intersect<U: WithRegion + Clone, Other: Iterator<Item = U> + Sorted>(
        self,
        other: Other,
    ) -> SortedIntersectIter<Self, Other> {
        SortedIntersectIter {
            context_a: Context::from_iter(self),
            context_b: Context::from_iter(other),
            state: State::FrontierA(0, 0, None),
        }
    }

    fn sorted_left_outer_intersect<U: WithRegion + Clone, Other: Iterator<Item = U> + Sorted>(
        self,
        other: Other
    ) -> outer::LeftOuterJoinIter<Self, Other> {
        outer::LeftOuterJoinIter::new(self, other)
    }
}

impl<I: Iterator + Sorted> SortedIntersect for I where I::Item: WithRegion + Clone {}

