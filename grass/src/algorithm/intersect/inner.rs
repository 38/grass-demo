use std::marker::PhantomData;

use super::heap::RegionHeap;
use crate::algorithm::Sorted;
use crate::properties::WithRegion;
use crate::ChromName;

pub(super) struct Context<C: ChromName, I: Iterator + Sorted>
where
    I::Item: WithRegion<C> + Clone,
{
    iter: I,
    peek_buffer: Option<I::Item>,
    frontier: Vec<I::Item>,
    active_regions: RegionHeap<C, I::Item>,
    _p: PhantomData<C>,
}

impl<C: ChromName, I: Iterator + Sorted> Context<C, I>
where
    I::Item: WithRegion<C> + Clone,
{
    pub(super) fn from_iter(mut iter: I) -> Self {
        let peek_buffer = iter.next();
        Self {
            iter,
            peek_buffer,
            frontier: Vec::new(),
            active_regions: Default::default(),
            _p: PhantomData,
        }
    }

    fn skip_util_chrom(&mut self, target: &C) {
        while let Some(head) = self.peek_buffer.as_ref() {
            if head.chrom() < target {
                self.peek_buffer = self.iter.next();
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<&I::Item> {
        self.peek_buffer.as_ref()
    }

    fn remove_inactive_regions(&mut self, chrom: &C, active_limit: u32) {
        while let Some(top) = self.active_regions.peek() {
            if top.chrom() < chrom || top.end() <= active_limit {
                self.active_regions.pop();
            } else {
                break;
            }
        }
    }

    fn push_frontier(&mut self) -> Option<u32> {
        let new_frontier = self.peek_buffer.as_ref()?.begin();
        let chrom = self.peek_buffer.as_ref()?.chrom().clone();

        while let Some(region) = self.peek_buffer.as_ref() {
            if region.begin() == new_frontier && &chrom == region.chrom() {
                self.frontier.push(self.peek_buffer.take().unwrap());
                self.peek_buffer = self.iter.next();
            } else {
                break;
            }
        }
        self.remove_inactive_regions(&chrom, new_frontier);
        Some(new_frontier)
    }

    fn flush_frontier(&mut self) {
        for item in self.frontier.drain(0..self.frontier.len()) {
            self.active_regions.push(item);
        }
    }

    fn ingest_active_regions(&mut self, chrom: &C, active_limit: u32) {
        while let Some(region) = self.peek_buffer.as_ref() {
            if region.begin() <= active_limit && region.chrom() == chrom {
                self.active_regions.push(self.peek_buffer.take().unwrap());
                self.peek_buffer = self.iter.next();
            } else {
                break;
            }
        }
        self.remove_inactive_regions(chrom, active_limit);
    }
}

#[derive(Debug)]
pub enum State {
    FrontierA(usize, usize, Option<usize>),
    FrontierB(usize, usize, Option<usize>),
}

impl State {
    fn next<
        C: ChromName,
        A: WithRegion<C> + Clone,
        B: WithRegion<C> + Clone,
        IA: Iterator<Item = A> + Sorted,
        IB: Iterator<Item = B> + Sorted,
    >(
        &mut self,
        ctx: (&mut Context<C, IA>, &mut Context<C, IB>),
    ) -> Option<(A, B)> {
        match self {
            Self::FrontierA(f_idx, h_idx, b_idx) if b_idx.is_none() => {
                let ret = if *f_idx >= ctx.0.frontier.len() || ctx.1.active_regions.len() == 0 {
                    return None;
                } else {
                    let a = ctx.0.frontier[*f_idx].clone();
                    let b = ctx.1.active_regions.as_slice()[*h_idx].clone();
                    (a, b)
                };

                if *f_idx == 0 && ret.1.begin() == ret.0.begin() && ctx.0.active_regions.len() > 0 {
                    *b_idx = Some(0);
                } else {
                    *h_idx += 1;

                    if *h_idx >= ctx.1.active_regions.len() {
                        *f_idx += 1;
                        *h_idx = 0;
                    }
                }
                Some(ret)
            }
            Self::FrontierA(f_idx, h_idx, b_idx_ref) => {
                let b_idx = b_idx_ref.unwrap();
                let a = ctx.0.active_regions.as_slice()[b_idx].clone();
                let b = ctx.1.active_regions.as_slice()[*h_idx].clone();
                if b_idx == ctx.0.active_regions.len() - 1 {
                    *h_idx += 1;
                    if *h_idx >= ctx.1.active_regions.len() {
                        *f_idx += 1;
                        *h_idx = 0;
                    }
                    *b_idx_ref = None;
                } else {
                    *b_idx_ref = Some(b_idx + 1);
                }
                Some((a, b))
            }
            Self::FrontierB(f_idx, h_idx, b_idx) => {
                let mut tmp_state = Self::FrontierA(*f_idx, *h_idx, *b_idx);
                let ret = tmp_state.next((ctx.1, ctx.0)).map(|(b, a)| (a, b));
                match tmp_state {
                    Self::FrontierA(f, h, b) => {
                        *f_idx = f;
                        *h_idx = h;
                        *b_idx = b;
                    }
                    _ => unreachable!(),
                }
                ret
            }
        }
    }
}

pub struct SortedIntersectIter<C: ChromName, IA: Iterator + Sorted, IB: Iterator + Sorted>
where
    IA::Item: WithRegion<C> + Clone,
    IB::Item: WithRegion<C> + Clone,
{
    pub(super) context_a: Context<C, IA>,
    pub(super) context_b: Context<C, IB>,
    pub(super) state: State,
}

impl<C, IA, IB> Sorted for SortedIntersectIter<C, IA, IB>
where
    C: ChromName,
    IA: Iterator + Sorted,
    IB: Iterator + Sorted,
    IA::Item: WithRegion<C> + Clone,
    IB::Item: WithRegion<C> + Clone,
{
}

impl<C, IA, IB> Iterator for SortedIntersectIter<C, IA, IB>
where
    C: ChromName,
    IA: Iterator + Sorted,
    IB: Iterator + Sorted,
    IA::Item: WithRegion<C> + Clone,
    IB::Item: WithRegion<C> + Clone,
{
    type Item = (IA::Item, IB::Item);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(next) = self.state.next((&mut self.context_a, &mut self.context_b)) {
                return Some(next);
            }

            self.context_a.flush_frontier();
            self.context_b.flush_frontier();

            let (frontier_a, frontier_b) = loop {
                let peek_a = self.context_a.peek();
                let peek_b = self.context_b.peek();

                if peek_a.is_none() && peek_b.is_none() {
                    return None;
                }

                let chrom_cmp = if let (Some(peek_a), Some(peek_b)) = (peek_a, peek_b) {
                    peek_a.chrom().cmp(&peek_b.chrom())
                } else {
                    std::cmp::Ordering::Equal
                };

                match chrom_cmp {
                    std::cmp::Ordering::Less => {
                        self.context_a
                            .skip_util_chrom(&peek_b.as_ref().unwrap().chrom());
                        self.context_a.frontier.clear();
                        self.context_a.active_regions.data.clear();
                    }
                    std::cmp::Ordering::Greater => {
                        self.context_b
                            .skip_util_chrom(&peek_a.as_ref().unwrap().chrom());
                        self.context_b.frontier.clear();
                        self.context_b.active_regions.data.clear();
                    }
                    std::cmp::Ordering::Equal => {
                        break (peek_a.map(|x| x.begin()), peek_b.map(|x| x.begin()));
                    }
                }
            };

            self.state =
                if frontier_a.unwrap_or(std::u32::MAX) <= frontier_b.unwrap_or(std::u32::MAX) {
                    let frontier = self.context_a.push_frontier()?;
                    self.context_b
                        .ingest_active_regions(&self.context_a.frontier[0].chrom(), frontier);
                    State::FrontierA(0, 0, None)
                } else {
                    let frontier = self.context_b.push_frontier()?;
                    self.context_a
                        .ingest_active_regions(&self.context_b.frontier[0].chrom(), frontier);
                    State::FrontierB(0, 0, None)
                };
        }
    }
}
