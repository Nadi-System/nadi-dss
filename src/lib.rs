#![allow(improper_ctypes)]
use nadi_core::nadi_plugin::nadi_plugin;

#[nadi_plugin]
mod dss {
    use hecdss_sys::*;
    use nadi_core::abi_stable::std_types::{RNone, ROption, RSome};
    use nadi_core::nadi_plugin::{env_func, node_func};
    use nadi_core::prelude::*;
    use nadi_core::timeseries::{MaskedSeries, Series};
    use std::ffi::{c_char, c_longlong, CStr, CString};

    /// List the catalog of the dss file
    ///
    /// env list_catalog("dsslib/routed-main-stem.dss")
    #[env_func(paths = "/*/*/*/*/*/*/")]
    fn list_catalog(dssfile: String, paths: String) -> Result<Vec<String>, String> {
        let fnamec = CString::new(dssfile).unwrap();
        let filen = Box::new([0; 300]);
        let filename = Box::into_raw(filen);
        let mut cats = vec![];
        unsafe {
            // the codes in this unsafe section is very similar to writing C code for DSS handling
            let mut dummy = 0;
            let exists = zfileName(filename as *mut c_char, 300, fnamec.as_ptr(), &mut dummy);
            zsetMessageLevel(0, 0);
            if exists == 0 {
                return Err("DSS file does not exist".to_string());
            }
            let ifltab = Box::new([0; 250]);
            let tab: *mut c_longlong = Box::into_raw(ifltab) as *mut _;
            let status = hec_dss_zopen(tab, filename as *const c_char);
            if status != (STATUS_OKAY as i32) {
                return Err(format!("Error opening DSS file, {}", status));
            }
            let catalogue = zstructCatalogNew();
            let number_paths = zcatalog(
                tab,
                CString::new(paths).unwrap().as_ptr() as *const c_char,
                catalogue,
                1,
            );
            if number_paths > 0 {
                let cat = *catalogue;
                let n = cat.numberPathnames as usize;
                let names: &[*mut c_char] = std::slice::from_raw_parts(cat.pathnameList, n);
                for i in 0..n {
                    let c_str: &CStr = CStr::from_ptr(names[i]);
                    let name = c_str.to_str().unwrap();
                    cats.push(name.to_string());
                }
            }
        }
        Ok(cats)
    }

    #[node_func]
    fn load_series(
        node: &mut NodeInner,
        dssfile: String,
        path: String,
        name: String,
    ) -> Result<(), String> {
        let fnamec = CString::new(dssfile).unwrap();
        let pathc = CString::new(path).unwrap();
        let filen = Box::new([0; 300]);
        let filename = Box::into_raw(filen);
        let series: Series;
        unsafe {
            // the codes in this unsafe section is very similar to writing C code for DSS handling
            let mut dummy = 0;
            let exists = zfileName(filename as *mut c_char, 300, fnamec.as_ptr(), &mut dummy);
            zsetMessageLevel(0, 0);
            if exists == 0 {
                return Err("DSS file does not exist".to_string());
            }
            let ifltab = Box::new([0; 250]);
            let tab: *mut c_longlong = Box::into_raw(ifltab) as *mut _;
            let status = hec_dss_zopen(tab, filename as *const c_char);
            if status != (STATUS_OKAY as i32) {
                return Err(format!("Error opening DSS file, {}", status));
            }
            let ts = zstructTsNew(pathc.as_ptr());
            let status = ztsRetrieve(tab, ts, 0, 1, 0);
            if status != (STATUS_OKAY as i32) {
                return Err(format!(
                    "Error retrieving timeseris from DSS file, {}",
                    status
                ));
            }
            let ts = *ts;
            let val_count = ts.numberValues;
            let ts_vals = std::slice::from_raw_parts(ts.floatValues, val_count as usize);
            let values: Vec<ROption<f64>> = ts_vals
                .iter()
                .map(|v| {
                    // zisMissingFloat returns int
                    if zisMissingFloat(*v) == 1 {
                        RNone
                    } else {
                        RSome(*v as f64)
                    }
                })
                .collect();
            series = MaskedSeries::floats(values).into();
        }
        node.set_series(&name, series);
        Ok(())
    }
}
