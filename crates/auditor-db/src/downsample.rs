use auditor_core::samples::ResourceSample;

pub fn lttb_downsample(
    samples: Vec<ResourceSample>,
    target_points: usize,
) -> Vec<ResourceSample> {
    if samples.len() <= target_points {
        return samples;
    }
    samples
}
