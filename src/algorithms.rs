use super::properties::{RegionOrder, WithRegion};
use super::records::Bed3;
use std::collections::BinaryHeap;

pub trait Intersect: WithRegion {
    fn intersect<T: WithRegion>(&self, other: &T) -> Bed3 {
        if self.chrom() != other.chrom() {
            return Bed3 {
                chrom: self.chrom().to_owned(),
                left: self.left(),
                right: self.right(),
            };
        }

        Bed3 {
            chrom: self.chrom().to_owned(),
            left: self.left().max(other.left()),
            right: self.right().min(other.right()),
        }
    }
}

impl<T: WithRegion> Intersect for T {}

pub fn intersect_sorted_file<FA, FB>(
    file_a: FA,
    mut file_b: FB,
    mut action: impl FnMut(&FA::Item, &FB::Item) -> bool,
) where
    FA: Iterator,
    FB: Iterator,
    FA::Item: WithRegion,
    FB::Item: WithRegion,
{
    let mut active_regions = BinaryHeap::<RegionOrder<FB::Item>>::new();
    let mut current_chrom = "".to_string();
    let mut right_limit = 0;

    let mut next_b = file_b.next();

    for cur_a in file_a {
        if cur_a.chrom() != current_chrom {
            current_chrom = cur_a.chrom().to_owned();
            right_limit = 0;
            active_regions.clear();
        }

        right_limit = right_limit.max(cur_a.right());

        while let Some(ref b) = next_b {
            if b.chrom() > current_chrom.as_ref() || right_limit <= b.left() {
                break;
            }

            active_regions.push(RegionOrder(next_b.unwrap()));
            next_b = file_b.next();
        }

        while let Some(ref top) = active_regions.peek() {
            if top.chrom() == cur_a.chrom() && cur_a.left() < top.right() {
                break;
            }
            active_regions.pop();
        }

        for RegionOrder(active_b) in active_regions.iter() {
            if cur_a.overlaps(active_b) {
                if !action(&cur_a, active_b) {
                    return;
                }
            }
        }
    }
}
