use crate::spec::Target;

pub fn target() -> Target {
    let mut base = super::x86_64_apple_darwin::base_target("x86_64h");
    // x86_64h is core2-avx cpu without a few of the features which would
    // otherwise be guaranteed. This imitates clang's logic below:
    // - https://github.com/llvm/llvm-project/blob/e8933455/clang/lib/Driver/ToolChains/Arch/X86.cpp#L81
    // - https://github.com/llvm/llvm-project/blob/e8933455/clang/lib/Driver/ToolChains/Arch/X86.cpp#L137
    base.options.cpu = "core2-avx".into();
    // FIXME: these should be overridden by `-Ctarget-cpu=native`
    base.options.features = "-rdrnd,-aes,-pclmul,-rtm,-fsgsbase".into();
    base
}
