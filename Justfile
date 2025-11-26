sbc_user := env("SBC_USER", "root")
sbc_pwd := env("SBC_PWD", "mAxsuw-1sizhe-tojvew")
sbc_host := env("SBC_HOST", "192.168.1.101")

[doc('Execute central')]
central-exec:
    sshpass -p {{sbc_pwd}} ssh {{sbc_user}}@{{sbc_host}}

[doc("Build central and transfer it to the SBC and restart and show its output")]
central-build-roll:
    dagger -c "build-central --arch linux/arm64 | export central" 
    sshpass -p {{sbc_pwd}} scp central {{sbc_user}}@{{sbc_host}}:~/central
    sshpass -p {{sbc_pwd}} ssh {{sbc_user}}@{{sbc_host}} '~/central'