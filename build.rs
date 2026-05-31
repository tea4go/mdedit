/// 构建脚本 - 将 lib/ 目录添加到链接搜索路径
///
/// 用于链接 Windows 平台的 shlwapi 等系统库。
fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}/lib", manifest_dir);
}
