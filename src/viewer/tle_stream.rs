use crate::tle::TLE;
use crate::traits::Observable;
use crate::BoxResult;
use reqwest::*;
use std::collections::HashMap;

pub struct TleStream {
    consumed: bool,
    url: String,
}

impl TleStream {
    pub fn new(url: &String) -> Self {
        Self {
            consumed: false,
            url: url.clone(),
        }
    }

    pub fn next(&mut self) -> BoxResult<HashMap<String, TLE>> {
        if self.consumed {
            // TODO: someday, periodically check URL for new data...
            // for now: never return results more than once
            bail!("No update");
        }

        let mut result = HashMap::<String, TLE>::new();

        let r = blocking::get(&self.url)?.text()?;

        let mut i = 0;
        let mut tle_lines = vec![];
        tle_lines.resize(3, "".to_string());

        for l in r.lines() {
            if l.is_empty() {
                continue;
            }

            tle_lines[i] = l.to_string();

            i = (i + 1) % 3;

            if i == 0 {
                let tle_result = TLE::from_lines(
                    tle_lines[1].as_bytes(),
                    tle_lines[2].as_bytes(),
                    tle_lines[0].as_bytes(),
                );

                if let Ok(tle) = tle_result {
                    result.insert(tle.name(), tle);
                } else if let Err(e) = tle_result {
                    println!("{}", e)
                }
            }
        }

        self.consumed = true;
        Ok(result)
    }
}
