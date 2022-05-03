// Copyright (c) 2019 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#[cfg(not(test))]
#[macro_export]
macro_rules! ucs2_str {
    ($source_str:expr) => {{
        let mut ucs2_str = [0u16; $source_str.len() + 1];
        let result = $crate::encoder::encode($source_str, &mut ucs2_str);
        result.unwrap();
        ucs2_str
    }};
}
