sbc_user := env("SBC_USER", "root")
sbc_pwd := env("SBC_PWD", "mAxsuw-1sizhe-tojvew")
sbc_host := env("SBC_HOST", "192.168.1.101")

[doc('Build central for linux/aarch64')]
central-build:
    cd edge-central && cargo build --release --target aarch64-unknown-linux-musl

[doc('Execute central')]
central-exec:
    sshpass -p {{sbc_pwd}} ssh {{sbc_user}}@{{sbc_host}}

[doc("Build central and transfer it to the SBC and restart and show its output")]
central-build-roll:
    just central-build
    sshpass -p {{sbc_pwd}} scp edge-central/target/aarch64-unknown-linux-musl/release/main {{sbc_user}}@{{sbc_host}}:~/central
    sshpass -p {{sbc_pwd}} ssh {{sbc_user}}@{{sbc_host}} '~/central'