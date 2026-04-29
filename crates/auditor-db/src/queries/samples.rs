use auditor_core::samples::ResourceSample;

pub fn insert_sample(_sample: &ResourceSample) -> crate::error::Result<()> {
    Ok(())
}

pub fn get_samples(_pid: u32, _from: i64, _to: i64) -> crate::error::Result<Vec<ResourceSample>> {
    Ok(vec![])
}
