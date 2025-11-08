// -------------------------------------------------------------------------
// SPDX-FileCopyrightText: Copyright © 2025 The Typhon Project
// SPDX-FileName: crates/typhon-compiler/build.rs
// SPDX-FileType: SOURCE
// SPDX-License-Identifier: Apache-2.0
// -------------------------------------------------------------------------
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// -------------------------------------------------------------------------

fn main() {
    // If LLVM_SYS_181_PREFIX isn't set, try to auto-detect common installation locations
    if std::env::var_os("LLVM_SYS_181_PREFIX").is_none() {
        // Common LLVM installation paths
        let common_paths = vec![
            "/usr/local/opt/llvm@18",
            "/usr/local/opt/llvm",
            "/opt/homebrew/opt/llvm@18",
            "/opt/homebrew/opt/llvm",
            "/usr/lib/llvm-18",
            "/usr/local/lib/llvm-18",
        ];

        for path in common_paths {
            let path = std::path::Path::new(path);
            if path.exists() {
                println!("cargo:rustc-env=LLVM_SYS_181_PREFIX={}", path.display());
                break;
            }
        }
    }

    // Always print the config used
    println!(
        "cargo:warning=Using LLVM from: {}",
        std::env::var("LLVM_SYS_181_PREFIX").unwrap_or_else(|_| "system path".to_string())
    );
}
