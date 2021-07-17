use std::ops::Range;

use plotters::{evcxr::SVGWrapper, prelude::*};

use crate::{
    algorithm::{Components, ComponentsIter, Point},
    chromset::LexicalChromRef,
    properties::{WithRegion, WithRegionCore},
    records::Bed5,
};

pub struct DepthIter<I: Iterator>
where
    I::Item: WithRegion<LexicalChromRef> + Clone,
{
    last: Option<Point<LexicalChromRef, I::Item>>,
    iter: ComponentsIter<LexicalChromRef, I>,
}

impl<I: Iterator> Iterator for DepthIter<I>
where
    I::Item: WithRegion<LexicalChromRef> + Clone,
{
    type Item = Bed5<LexicalChromRef, usize>;
    fn next(&mut self) -> Option<Self::Item> {
        let last = self.last.take()?;
        if let Some(next) = self.iter.next() {
            let (last_chr, last_pos) = last.position();
            let (next_chr, next_pos) = next.position();
            if last_chr == next_chr {
                let result = Bed5::new(last_chr, last_pos, next_pos, ".", last.depth);
                self.last = Some(next);
                return Some(result);
            } else {
                self.last = Some(next);
                return self.next();
            }
        }
        None
    }
}

pub trait DepthExt
where
    Self: IntoIterator + Sized,
    <Self as IntoIterator>::Item: WithRegion<LexicalChromRef> + Clone,
{
    fn coverage(self) -> DepthIter<Self::IntoIter> {
        let mut iter = self.into_iter().components();
        let last = iter.next();
        DepthIter { iter, last }
    }

    fn plot_coverage(self) -> SVGWrapper {
        struct DepthData {
            min_bucket: Option<u32>,
            bucket_size: u32,
            bucket_list: Vec<usize>,
            incompleted_bucket: (u32, usize),
        }
        impl Default for DepthData {
            fn default() -> Self {
                Self {
                    min_bucket: None,
                    bucket_size: 1,
                    bucket_list: vec![],
                    incompleted_bucket: (0, 0),
                }
            }
        }
        impl DepthData {
            fn double_bucket_size(&mut self) {
                let mut new_buckets = vec![];
                for i in 0..self.bucket_list.len() / 2 {
                    new_buckets.push(self.bucket_list[i * 2] + self.bucket_list[i * 2 + 1]);
                }
                if self.bucket_list.len() % 2 > 0 {
                    self.incompleted_bucket.0 += self.bucket_size;
                    self.incompleted_bucket.1 += self.bucket_list[self.bucket_list.len() - 1];
                }
                self.bucket_list = new_buckets;
                self.bucket_size *= 2;
            }

            fn right_most(&self) -> u32 {
                self.min_bucket.unwrap_or(0)
                    + self.bucket_size * self.bucket_list.len() as u32
                    + self.incompleted_bucket.0
            }

            fn flush_pending_bucket(&mut self, force: bool) {
                if self.bucket_list.len() >= 1024 {
                    self.double_bucket_size();
                }

                if force || self.incompleted_bucket.0 < self.bucket_size {
                    return;
                }

                self.bucket_list.push(self.incompleted_bucket.1);

                self.incompleted_bucket.0 = 0;
                self.incompleted_bucket.1 = 0;
            }

            fn append_interval_step(&mut self, interval: (u32, u32), value: usize) -> (u32, u32) {
                if self.min_bucket.is_none() {
                    self.min_bucket = Some(interval.0);
                }

                // First, we need to flush zeros before the interval begins
                while self.right_most() < interval.0 {
                    let incr = (interval.0 - self.right_most()).min(self.bucket_size);
                    self.incompleted_bucket.0 += incr;
                    self.flush_pending_bucket(false);
                }

                // After that, we should be in the bucket that overlaps with current interval
                let incr = (interval.1 - interval.0).min(self.bucket_size);
                self.incompleted_bucket.0 += incr;
                self.incompleted_bucket.1 += incr as usize * value;
                self.flush_pending_bucket(false);

                (interval.0 + incr, interval.1)
            }
            fn append_interval(&mut self, mut interval: (u32, u32), value: usize) {
                while interval.1 - interval.0 > 0 {
                    interval = self.append_interval_step(interval, value);
                }
            }

            fn get_y_range(&self) -> Range<f64> {
                let max =
                    *self.bucket_list.iter().max().unwrap_or(&1) as f64 / self.bucket_size as f64;
                -1.0..max
            }

            fn get_x_range(&self) -> Range<u32> {
                let min = self.min_bucket.unwrap_or(0);
                let max = self.right_most();
                min..max
            }

            fn into_data_points(self) -> impl Iterator<Item = (u32, f64)> {
                let mut current = self.min_bucket.unwrap_or(0);
                let bucket_size = self.bucket_size;
                self.bucket_list.into_iter().map(move |value| {
                    let ret = (current, value as f64 / bucket_size as f64);
                    current += bucket_size;
                    ret
                })
            }
        }
        let mut data = vec![];
        let mut chrom_list = vec![];
        let mut last_chrom = None;
        for depth in self.coverage() {
            if last_chrom
                .as_ref()
                .map_or(true, |last_chrom| last_chrom != depth.chrom())
            {
                last_chrom = Some(depth.chrom().clone());
                data.last_mut()
                    .map(|data: &mut DepthData| data.flush_pending_bucket(true));
                data.push(Default::default());
                chrom_list.push(depth.chrom().clone());
            }
            let interval = (depth.begin(), depth.end());
            data.last_mut()
                .map(|data: &mut DepthData| data.append_interval(interval, depth.score.unwrap()));
        }

        data.last_mut()
            .map(|data: &mut DepthData| data.flush_pending_bucket(true));

        evcxr_figure((800, 600), move |root| {
            let cols = 3.min(data.len()).max(1);

            let splitted = root.split_evenly(((data.len() + cols - 1) / cols, cols));

            for (root, (data, chrom)) in splitted
                .into_iter()
                .zip(data.into_iter().zip(chrom_list.into_iter()))
            {
                let xs = data.get_x_range();
                let ys = data.get_y_range();

                let mut chart = plotters::prelude::ChartBuilder::on(&root)
                    .caption(format!("{:?}", chrom), ("sans", 15))
                    .x_label_area_size(50)
                    .y_label_area_size(50)
                    .build_cartesian_2d(xs, ys)?;

                chart.configure_mesh().draw()?;

                chart.draw_series(plotters::series::AreaSeries::new(
                    data.into_data_points(),
                    -0.0,
                    &plotters::prelude::RED,
                ))?;
            }

            Ok(())
        })
    }
}

impl<T> DepthExt for T
where
    T: IntoIterator + Sized,
    T::Item: WithRegion<LexicalChromRef> + Clone,
{
}
