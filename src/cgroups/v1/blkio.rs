use std::{
    fs::{self},
    path::Path,
};

use crate::cgroups::{common, v1::Controller};
use oci_spec::{LinuxBlockIo, LinuxResources};

const CGROUP_BLKIO_THROTTLE_READ_BPS: &str = "blkio.throttle.read_bps_device";
const CGROUP_BLKIO_THROTTLE_WRITE_BPS: &str = "blkio.throttle.write_bps_device";
const CGROUP_BLKIO_THROTTLE_READ_IOPS: &str = "blkio.throttle.read_iops_device";
const CGROUP_BLKIO_THROTTLE_WRITE_IOPS: &str = "blkio.throttle.write_iops_device";

pub struct Blkio {}

impl Controller for Blkio {
    fn apply(
        linux_resources: &LinuxResources,
        cgroup_root: &Path,
        pid: nix::unistd::Pid,
    ) -> anyhow::Result<()> {
        log::debug!("Apply blkio cgroup config");
        fs::create_dir_all(cgroup_root)?;

        if let Some(blkio) = &linux_resources.block_io {
            Self::apply(cgroup_root, blkio)?;
        }

        common::write_cgroup_file(&cgroup_root.join("cgroup.procs"), &pid.to_string())?;
        Ok(())
    }
}

impl Blkio {
    fn apply(root_path: &Path, blkio: &LinuxBlockIo) -> anyhow::Result<()> {
        for trbd in &blkio.blkio_throttle_read_bps_device {
            common::write_cgroup_file(
                &root_path.join(CGROUP_BLKIO_THROTTLE_READ_BPS),
                &format!("{}:{} {}", trbd.major, trbd.minor, trbd.rate),
            )?;
        }

        for twbd in &blkio.blkio_throttle_write_bps_device {
            common::write_cgroup_file(
                &root_path.join(CGROUP_BLKIO_THROTTLE_WRITE_BPS),
                &format!("{}:{} {}", twbd.major, twbd.minor, twbd.rate),
            )?;
        }

        for trid in &blkio.blkio_throttle_read_iops_device {
            common::write_cgroup_file(
                &root_path.join(CGROUP_BLKIO_THROTTLE_READ_IOPS),
                &format!("{}:{} {}", trid.major, trid.minor, trid.rate),
            )?;
        }

        for twid in &blkio.blkio_throttle_write_iops_device {
            common::write_cgroup_file(
                &root_path.join(CGROUP_BLKIO_THROTTLE_WRITE_IOPS),
                &format!("{}:{} {}", twid.major, twid.minor, twid.rate),
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Write, path::PathBuf};

    use super::*;
    use oci_spec::{LinuxBlockIo, LinuxThrottleDevice};

    struct BlockIoBuilder {
        block_io: LinuxBlockIo,
    }

    impl BlockIoBuilder {
        fn new() -> Self {
            let block_io = LinuxBlockIo {
                blkio_weight: Some(0),
                blkio_leaf_weight: Some(0),
                blkio_weight_device: vec![],
                blkio_throttle_read_bps_device: vec![],
                blkio_throttle_write_bps_device: vec![],
                blkio_throttle_read_iops_device: vec![],
                blkio_throttle_write_iops_device: vec![],
            };

            Self { block_io }
        }

        fn with_read_bps(mut self, throttle: Vec<LinuxThrottleDevice>) -> Self {
            self.block_io.blkio_throttle_read_bps_device = throttle;
            self
        }

        fn with_write_bps(mut self, throttle: Vec<LinuxThrottleDevice>) -> Self {
            self.block_io.blkio_throttle_write_bps_device = throttle;
            self
        }

        fn with_read_iops(mut self, throttle: Vec<LinuxThrottleDevice>) -> Self {
            self.block_io.blkio_throttle_read_iops_device = throttle;
            self
        }

        fn with_write_iops(mut self, throttle: Vec<LinuxThrottleDevice>) -> Self {
            self.block_io.blkio_throttle_write_iops_device = throttle;
            self
        }

        fn build(self) -> LinuxBlockIo {
            self.block_io
        }
    }

    fn setup(testname: &str, throttle_type: &str) -> (PathBuf, PathBuf) {
        let tmp = create_temp_dir(testname).expect("create temp directory for test");
        let throttle_file = set_fixture(&tmp, throttle_type, "")
            .unwrap_or_else(|_| panic!("set fixture for {}", throttle_type));

        (tmp, throttle_file)
    }

    fn set_fixture(
        temp_dir: &std::path::Path,
        filename: &str,
        val: &str,
    ) -> anyhow::Result<PathBuf> {
        let full_path = temp_dir.join(filename);

        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&full_path)?
            .write_all(val.as_bytes())?;

        Ok(full_path)
    }

    fn create_temp_dir(test_name: &str) -> anyhow::Result<PathBuf> {
        std::fs::create_dir_all(std::env::temp_dir().join(test_name))?;
        Ok(std::env::temp_dir().join(test_name))
    }

    #[test]
    fn test_set_blkio_read_bps() {
        let (test_root, throttle) =
            setup("test_set_blkio_read_bps", CGROUP_BLKIO_THROTTLE_READ_BPS);

        let blkio = BlockIoBuilder::new()
            .with_read_bps(vec![LinuxThrottleDevice {
                major: 8,
                minor: 0,
                rate: 102400,
            }])
            .build();

        Blkio::apply(&test_root, &blkio).expect("apply blkio");
        let content = fs::read_to_string(throttle)
            .unwrap_or_else(|_| panic!("read {} content", CGROUP_BLKIO_THROTTLE_READ_BPS));

        assert_eq!("8:0 102400", content);
    }

    #[test]
    fn test_set_blkio_write_bps() {
        let (test_root, throttle) =
            setup("test_set_blkio_write_bps", CGROUP_BLKIO_THROTTLE_WRITE_BPS);

        let blkio = BlockIoBuilder::new()
            .with_write_bps(vec![LinuxThrottleDevice {
                major: 8,
                minor: 0,
                rate: 102400,
            }])
            .build();

        Blkio::apply(&test_root, &blkio).expect("apply blkio");
        let content = fs::read_to_string(throttle)
            .unwrap_or_else(|_| panic!("read {} content", CGROUP_BLKIO_THROTTLE_WRITE_BPS));

        assert_eq!("8:0 102400", content);
    }

    #[test]
    fn test_set_blkio_read_iops() {
        let (test_root, throttle) =
            setup("test_set_blkio_read_iops", CGROUP_BLKIO_THROTTLE_READ_IOPS);

        let blkio = BlockIoBuilder::new()
            .with_read_iops(vec![LinuxThrottleDevice {
                major: 8,
                minor: 0,
                rate: 102400,
            }])
            .build();

        Blkio::apply(&test_root, &blkio).expect("apply blkio");
        let content = fs::read_to_string(throttle)
            .unwrap_or_else(|_| panic!("read {} content", CGROUP_BLKIO_THROTTLE_READ_IOPS));

        assert_eq!("8:0 102400", content);
    }

    #[test]
    fn test_set_blkio_write_iops() {
        let (test_root, throttle) = setup(
            "test_set_blkio_write_iops",
            CGROUP_BLKIO_THROTTLE_WRITE_IOPS,
        );

        let blkio = BlockIoBuilder::new()
            .with_write_iops(vec![LinuxThrottleDevice {
                major: 8,
                minor: 0,
                rate: 102400,
            }])
            .build();

        Blkio::apply(&test_root, &blkio).expect("apply blkio");
        let content = fs::read_to_string(throttle)
            .unwrap_or_else(|_| panic!("read {} content", CGROUP_BLKIO_THROTTLE_WRITE_IOPS));

        assert_eq!("8:0 102400", content);
    }
}
