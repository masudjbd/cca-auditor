use auditor_core::samples::ResourceSample;

/// Largest-Triangle-Three-Buckets downsampling.
/// Reduces a time-series to `target_points` while preserving visual fidelity.
/// Reference: Sveinn Steinarsson, "Downsampling Time Series for Visual Representation" (2013)
///
/// Operates on `cpu_pct` as the y-axis dimension. Maintains same RSS/timestamp positions.
pub fn lttb_downsample(
    mut samples: Vec<ResourceSample>,
    target_points: usize,
) -> Vec<ResourceSample> {
    let n = samples.len();
    if n <= target_points || target_points < 3 {
        return samples;
    }

    // Sort by timestamp to ensure monotonic x-axis
    samples.sort_by_key(|s| s.timestamp.unix_timestamp());

    let bucket_size = (n - 2) as f64 / (target_points - 2) as f64;
    let mut result = Vec::with_capacity(target_points);

    // First point is always included
    result.push(samples[0].clone());

    let mut a_idx = 0;

    for i in 0..(target_points - 2) {
        // Calculate the average point of the next bucket (used to find the third point of the triangle)
        let avg_range_start = ((i + 1) as f64 * bucket_size).floor() as usize + 1;
        let avg_range_end = (((i + 2) as f64 * bucket_size).floor() as usize + 1).min(n);
        let avg_range = avg_range_end - avg_range_start;
        if avg_range == 0 {
            continue;
        }
        let mut avg_x = 0.0_f64;
        let mut avg_y = 0.0_f64;
        for j in avg_range_start..avg_range_end {
            avg_x += samples[j].timestamp.unix_timestamp() as f64;
            avg_y += samples[j].cpu_pct;
        }
        avg_x /= avg_range as f64;
        avg_y /= avg_range as f64;

        // Bucket of points to choose one from
        let bucket_start = (i as f64 * bucket_size).floor() as usize + 1;
        let bucket_end = (((i + 1) as f64 * bucket_size).floor() as usize + 1).min(n);

        // Point a (previously selected)
        let point_a_x = samples[a_idx].timestamp.unix_timestamp() as f64;
        let point_a_y = samples[a_idx].cpu_pct;

        let mut max_area = -1.0_f64;
        let mut max_idx = bucket_start;

        for j in bucket_start..bucket_end {
            let x = samples[j].timestamp.unix_timestamp() as f64;
            let y = samples[j].cpu_pct;
            let area = ((point_a_x - avg_x) * (y - point_a_y)
                - (point_a_x - x) * (avg_y - point_a_y))
                .abs()
                * 0.5;
            if area > max_area {
                max_area = area;
                max_idx = j;
            }
        }

        result.push(samples[max_idx].clone());
        a_idx = max_idx;
    }

    // Last point is always included
    result.push(samples[n - 1].clone());

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use auditor_core::samples::ResourceSample;
    use time::OffsetDateTime;

    fn make_samples(count: usize) -> Vec<ResourceSample> {
        let base = OffsetDateTime::now_utc();
        (0..count)
            .map(|i| ResourceSample {
                pid: 1,
                cpu_pct: (i % 100) as f64,
                rss_bytes: 1024 * 1024 * (i as u64),
                gpu_mem_bytes: None,
                timestamp: base + time::Duration::seconds(i as i64),
            })
            .collect()
    }

    #[test]
    fn passes_through_when_smaller_than_target() {
        let s = make_samples(50);
        let result = lttb_downsample(s.clone(), 200);
        assert_eq!(result.len(), 50);
    }

    #[test]
    fn downsamples_to_target() {
        let s = make_samples(1000);
        let result = lttb_downsample(s, 100);
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn preserves_first_and_last() {
        let s = make_samples(1000);
        let first_ts = s[0].timestamp;
        let last_ts = s[s.len() - 1].timestamp;
        let result = lttb_downsample(s, 50);
        assert_eq!(result[0].timestamp, first_ts);
        assert_eq!(result[result.len() - 1].timestamp, last_ts);
    }
}
