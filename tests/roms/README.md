# Test ROMs

Downloaded: 2026-10-28

## Homebrew test ROMs (N3)

These are homebrew test ROMs downloaded for validating the emulator.
They are not committed to git (see .gitignore).

### Sources

| Candidate | URL | Status | ROMs copied |
|---|---|---|---|
| PeterLemon/SNES (krom) | https://github.com/PeterLemon/SNES | downloaded | 164 |
| gilyon/snes-tests | https://github.com/gilyon/snes-tests/releases/tag/v1.4 | downloaded (v1.4) | 3 |
| undisbeliever/snes-test-roms | https://github.com/undisbeliever/snes-test-roms | source only | 0 |
| blargg SPC700/DSP tests | https://gitlab.com/higan/snes-test-roms (blargg-spc-6) | downloaded | 4 |
| bsnes/higan test ROMs | https://gitlab.com/higan/snes-test-roms | downloaded (subset) | 40 |

### File listing

| path | size | sha256 |
|---|---|---|
| homebrew/blargg/spc/spc_dsp6.sfc | 489472 | b12038f865536a33003c9887e6f86fcbcb526b7f1766526ab30cb74b7380aa9a |
| homebrew/blargg/spc/spc_mem_access_times.sfc | 69632 | 53dc8005959d54ccfc95b8d4222e1d51d21e3d9f3491058320f105edc1a4454a |
| homebrew/blargg/spc/spc_smp.sfc | 137216 | 2ed302c5b989675d4ab6b40001fd5166ac4c6e0188a6b1b6df878e4608999c03 |
| homebrew/blargg/spc/spc_timer.sfc | 67584 | f4799b994a91d78689073f4e31c3668ae46c100abd78177b9f4813506d6c9629 |
| homebrew/gilyon/cputest/cputest-basic.sfc | 262144 | d479bde706b9e16d76c0dfe98e2b2d3f5348f9dcb911e6a36b898f4201fc98bb |
| homebrew/gilyon/cputest/cputest-full.sfc | 262144 | f61de78d346a68c166d67472a414e5d207368889d37b977d1338f9221ca6400e |
| homebrew/gilyon/spctest/spctest.sfc | 131072 | 3909f03d55b07081a4e9ac437ba5179731ec6c8ac3c715f712daec0f398dde63 |
| homebrew/higan/93143-hblank-dma-vram/hvdma.sfc | 32768 | 8dfeece984c292682cce6a8b02e07a32d2f304737c538fe7882a87a5ac005bfc |
| homebrew/higan/93143-hblank-dma-vram/hvdma_max.sfc | 65536 | 9f279b369d13dcd467abd370d063c3b341826ea88f2dde087f805920ddb63f46 |
| homebrew/higan/lidnariq-ppu-bus-activity/ppubusact.sfc | 131072 | 08fdb7580b7d7d9a6d07ddf9ec46e426ac940f581e41f84890aebf2d5d38887c |
| homebrew/higan/lidnariq-smp-clock-speed-measurement/smpspeed.sfc | 32768 | 78c7ca086f717ec3ea87eb935c68761492a059ad2457467c07591a28a94dd56e |
| homebrew/higan/tepples-tellinglys/tellinglys.sfc | 262144 | a204e05181351a1ce8775976fd509f274fbc9a647607bdad70b847349941c85d |
| homebrew/higan/tukuyomi-bsnes-tests/demo_irqtest.smc | 131584 | 043dfde410ec2d07eec6dc3737373da8102a67489146c422c02f4c1961318775 |
| homebrew/higan/tukuyomi-bsnes-tests/demo_mode3.smc | 131584 | 452f7c2b1af59127317ad392ba5b5f357f80001951aa035a28f1373b1611d79b |
| homebrew/higan/tukuyomi-bsnes-tests/demo_nmitest.smc | 131584 | 835786f30392596bf8b2f51cedcbbba1836a7c5cc612b0eb077f8aa82d9281a3 |
| homebrew/higan/tukuyomi-bsnes-tests/test_irq.smc | 131072 | 50e34d80264eab957222fe224322e7c143ebd617049576c64a9c61fcc9a80e4b |
| homebrew/higan/tukuyomi-bsnes-tests/test_irq4200.smc | 131072 | 71ee7336cfb7ee393b4ce36d0ca01e5c5e869cf8044aa5017987cc3a166f47a1 |
| homebrew/higan/tukuyomi-bsnes-tests/test_nmi.smc | 131072 | f6416e0e8207ead3ceb3044e6505af2bc040b62e8f3724bcf1547f020df9ebcc |
| homebrew/higan/undisbeliever-inidisp/hdma-2100-glitch-2ch-0a.sfc | 131072 | f017025218801ca4bd1ee4369c88921a48678130ef3bf54e15788bd2814f1074 |
| homebrew/higan/undisbeliever-inidisp/hdma-2100-glitch-2ch-81.sfc | 131072 | 3c9636f8443b4754488487bf60bd209240268d7d9450b7644170ac150fc3bc52 |
| homebrew/higan/undisbeliever-inidisp/hdma-2100-glitch.sfc | 131072 | f9d889e52607a6eddddb1b8507e255cea076a212dbb826ebdf9ca4df492260d1 |
| homebrew/higan/undisbeliever-inidisp/hdma-21ff-2100-0f-glitch.sfc | 131072 | bed2af8e64d12a0b93cca396515612f8a9ef46cb24a13b01dad144937d45e0ca |
| homebrew/higan/undisbeliever-inidisp/hdma-21ff-2100-glitch.sfc | 131072 | ecc9d7aa558cbf34989698179d28cad4510e17905324fed5e27e673253f4d69e |
| homebrew/higan/undisbeliever-inidisp/hdma-21ff-glitch.sfc | 131072 | d9b6111d66e7ad5b6475417d203240b47fc14248f0a16c51d472426aff8e787a |
| homebrew/higan/undisbeliever-inidisp/hdmaen_latch_test.sfc | 131072 | 196a9ba7885073c9ce7210bc3de35fc82c824554c94742ff6c66816e5ff9ee2b |
| homebrew/higan/undisbeliever-inidisp/hdmaen_latch_test_2.sfc | 131072 | 047cda002127b3770a3915ebd7c1779ea0b68365d4afc0d555ca533674b73121 |
| homebrew/higan/undisbeliever-inidisp/inidisp_brightness_delay.sfc | 131072 | 3b396f4979f01ac68ffc9ea24c7417fdbd9406b49fccc6d35bf95353fc587a55 |
| homebrew/higan/undisbeliever-inidisp/inidisp_d7_glitch_test.sfc | 131072 | bfa2eab7ac8d7bb9792028f6a4b4520f668944ec35dd9f7940df6fc37fb77766 |
| homebrew/higan/undisbeliever-inidisp/inidisp_enable_display_mid_frame.sfc | 131072 | c8568e7f7ac13f94503cf0b3d67f623fbc216f7ae9c25c6169ee56219bc1e747 |
| homebrew/higan/undisbeliever-inidisp/inidisp_forgot_to_force_blank.sfc | 131072 | 140223df9a1ea7558cadafbd1d4ca48acefc1112b82998624f681c82b33d55d0 |
| homebrew/higan/undisbeliever-inidisp/inidisp_hammer_0f.sfc | 131072 | 10991b6449606140d1715443c3d6d388d98a137723b0267a696c188710ed4e89 |
| homebrew/higan/undisbeliever-inidisp/inidisp_hammer_0f00.sfc | 131072 | 316745d7fc1474b32f4e0eab7e85f4ca1d35b357df6d24356365ab00d5de3f6b |
| homebrew/higan/undisbeliever-inidisp/inidisp_hammer_0f0f.sfc | 131072 | 8a997ac38bf43273d4794901aa1c9799dd5bce8021db242356884b02a37bb7f8 |
| homebrew/higan/undisbeliever-inidisp/inidisp_hammer_0f8f.sfc | 131072 | 42b2307b15271bb581cc8bc8926db1f0d3b71f07517304a61d2fda7450d46911 |
| homebrew/higan/undisbeliever-inidisp/inidisp_hammer_0f8f_fast.sfc | 131072 | 7a11766b4023eb9531c728dffd68e7a452d9962c5bdb33339c6b68778d45f4b4 |
| homebrew/higan/undisbeliever-inidisp/inidisp_hammer_0f_long.sfc | 131072 | 41b4cecbdb5e08cd55dad26343029edd815c49a9a8cec792ecbab3ae2cc715a8 |
| homebrew/higan/undisbeliever-inidisp/inidisp_hammer_8f0f.sfc | 131072 | 1c5c3c43aa6677c9163249b7df12ee665186eb53732b9d49c4814ea8b63bdd2b |
| homebrew/higan/undisbeliever-inidisp/scpu-a-dma-bug-1.sfc | 131072 | e9ca995e598625fdb3183969e260b048663b080d9fc3eeafeae83da244e507ba |
| homebrew/higan/undisbeliever-inidisp/scpu-a-dma-bug-2.sfc | 131072 | e8a1a38239219f3dc2f93b9f30d08da0afd8bd8f4436d9c7e784d5ee8a3fe57f |
| homebrew/higan/undisbeliever-inidisp/scpu-a-dma-bug-3.sfc | 131072 | 77b6bf4c0b227a3de847c9f3894352ab34833226cbee0164989b87f1e66f3981 |
| homebrew/higan/undisbeliever-inidisp/scpu-a-dma-bug-5.sfc | 131072 | c98e282371a4f8f511f2da7acb1439cc665549d70e3d48c62252c6748a18738a |
| homebrew/higan/undisbeliever-inidisp/scpu-a-dma-bug-ch0.sfc | 131072 | 021ba6ca3ce3fafa0971f5be53a3aa101e980774a6cabfcf922ce357eea9c4c3 |
| homebrew/higan/undisbeliever-inidisp/scpu-a-dma-bug-fix.sfc | 131072 | 5f0d455f3eb949b7f346a7982a2a8f6b70fdc86c0d2c79d6a6b32c5725cdcb8a |
| homebrew/higan/undisbeliever-inidisp/scpu-a-dma-bug-fix2.sfc | 131072 | c96d21f22189527cfe44bdf6d0c1f26813044cab359a30f69396997775e78d26 |
| homebrew/higan/undisbeliever-inidisp/scpu-a-dma-bug-r2.sfc | 131072 | d40e29d683524e90fbb49681665216cced3b8acf684886998fe906def5d86523 |
| homebrew/higan/undisbeliever-inidisp/scpu-a-dma-bug-strange.sfc | 131072 | a11117c99363f5fec3339bce225340d5441a901bdcb2252fa2ede94c810f675e |
| homebrew/higan/undisbeliever-inidisp/scpu-a-dma-bug-two-regs.sfc | 131072 | 4d1968cfd14acf253e24114b857ca0855749f493e1ebe2169ac04f528f1082eb |
| homebrew/peterlemon/BANK/HiROMFastROM/BANKHiROMFastROM.sfc | 131072 | 676f859ffcbe5b903e30049db04c479c5dd41b8bc1bd14485d735f9d185dc8c5 |
| homebrew/peterlemon/BANK/HiROMSlowROM/BANKHiROMSlowROM.sfc | 131072 | 320caca1bc8f17d8f5396d7c210cdbc798978a601cf8b2cef17d70e6d398fc6f |
| homebrew/peterlemon/BANK/LoROMFastROM/BANKLoROMFastROM.sfc | 65536 | 7a8de9f87986b4667723dcef00003ab93aea84586d3e13cfc8459db4a59369c5 |
| homebrew/peterlemon/BANK/LoROMSlowROM/BANKLoROMSlowROM.sfc | 65536 | 3f2d0d9d8d3a05912c636830e6c78a823ea357bc99e0115fe46d0e9a7f6737c6 |
| homebrew/peterlemon/BANK/WRAM/BANKWRAM.sfc | 32768 | 6ee3c92304298e5b6b083f1309285d5b8a75ad4287220ba16d1eb7df01167b06 |
| homebrew/peterlemon/CHIP/GSU/2BPP/FillPoly/256x128/GSU2BPP256x128FillPoly.sfc | 32768 | f3cc15d0720591e73a8552bf486559612110830d0788c14aa525c39be2c94e75 |
| homebrew/peterlemon/CHIP/GSU/2BPP/FillPoly/256x160/GSU2BPP256x160FillPoly.sfc | 32768 | b0921d2a4f5b5368bf1ffaf5de0b29c5d3cda3d8e319e106f068469cc281a2f4 |
| homebrew/peterlemon/CHIP/GSU/2BPP/FillPoly/256x192/GSU2BPP256x192FillPoly.sfc | 32768 | d6739dc0aecf9e4e4e290ff76813237b752340ace728b833992bc84cb3b86fae |
| homebrew/peterlemon/CHIP/GSU/2BPP/PlotLine/256x128/GSU2BPP256x128PlotLine.sfc | 32768 | 849c4fec9a60f037d23e5fead27dd1a90439e461bb1c32349425abb87c3b7f93 |
| homebrew/peterlemon/CHIP/GSU/2BPP/PlotLine/256x160/GSU2BPP256x160PlotLine.sfc | 32768 | f830a46cf39b24cbccff71d93475931268548725cf889abd223e8d7abefcf717 |
| homebrew/peterlemon/CHIP/GSU/2BPP/PlotLine/256x192/GSU2BPP256x192PlotLine.sfc | 32768 | 0e66f75f75ac23b0f927376885ba2656cafb4454e2ee4ceacdc326815760751f |
| homebrew/peterlemon/CHIP/GSU/2BPP/PlotPixel/256x128/GSU2BPP256x128PlotPixel.sfc | 32768 | 8fd080c611fe109a3627b14eb7eabbc5f1c316d79213eb637a04453aa3ac3285 |
| homebrew/peterlemon/CHIP/GSU/2BPP/PlotPixel/256x160/GSU2BPP256x160PlotPixel.sfc | 32768 | 3d0c703b754150e4fd46c626ea70db74ada6cc37b23f12c35c12486302c4cc14 |
| homebrew/peterlemon/CHIP/GSU/2BPP/PlotPixel/256x192/GSU2BPP256x192PlotPixel.sfc | 32768 | 3d5f0ae5eb652d7ecbb90df222e2fe4e2b4d8c11b9a765b3e48e17664e19dcc4 |
| homebrew/peterlemon/CHIP/GSU/4BPP/FillPoly/256x128/GSU4BPP256x128FillPoly.sfc | 32768 | 4b4c6fbc9efc52c4294c9db32c57093a47016e79d74e565cdb9a124088ee0afb |
| homebrew/peterlemon/CHIP/GSU/4BPP/FillPoly/256x160/GSU4BPP256x160FillPoly.sfc | 32768 | 4d75dcc29e6bf626de81e7e46a58a1fb8e6c5e1d2b8e6d4f959b4c024e94549e |
| homebrew/peterlemon/CHIP/GSU/4BPP/FillPoly/256x192/GSU4BPP256x192FillPoly.sfc | 32768 | 1c7304196db6735b6dd72485d36bb81a9eb41efdfc3fd2ed4783b879ac266402 |
| homebrew/peterlemon/CHIP/GSU/4BPP/PlotLine/256x128/GSU4BPP256x128PlotLine.sfc | 32768 | a0c7f67f115faa9e25641a06d0da82f2f547b7ec7e3b5bcd222c3a2899f187cb |
| homebrew/peterlemon/CHIP/GSU/4BPP/PlotLine/256x160/GSU4BPP256x160PlotLine.sfc | 32768 | b0fb9b00f8ef7ba8214e8e74b75a8ceecd72a423f4382f76013dbe330bb2e819 |
| homebrew/peterlemon/CHIP/GSU/4BPP/PlotLine/256x192/GSU4BPP256x192PlotLine.sfc | 32768 | 7f65457d5a373757a6c75f72310f09c4bd65e0ed8ceae052ec9218c79ee01572 |
| homebrew/peterlemon/CHIP/GSU/4BPP/PlotPixel/256x128/GSU4BPP256x128PlotPixel.sfc | 32768 | cc077ae259dd1cff8e593c52637a71816b556e1a7afe07278f7a1ebcb1e7db18 |
| homebrew/peterlemon/CHIP/GSU/4BPP/PlotPixel/256x160/GSU4BPP256x160PlotPixel.sfc | 32768 | 1c5bb6bc5c6de2137fac3f7237337cc2c6a0e9a6a81421114162c0321c5b416b |
| homebrew/peterlemon/CHIP/GSU/4BPP/PlotPixel/256x192/GSU4BPP256x192PlotPixel.sfc | 32768 | 071a12fa7b01ed134b12c8496f398530d04f149bd452b064d1c6659d3a882264 |
| homebrew/peterlemon/CHIP/GSU/8BPP/FillPoly/256x128/GSU8BPP256x128FillPoly.sfc | 32768 | 40cec16462a3a6c01ed1e37aa59af6503d5d40decc02dde28b14069594e7844c |
| homebrew/peterlemon/CHIP/GSU/8BPP/FillPoly/256x160/GSU8BPP256x160FillPoly.sfc | 32768 | 818be754ec240910deccda3f003ff1fd1100392a126e497260f66056fa997ba4 |
| homebrew/peterlemon/CHIP/GSU/8BPP/FillPoly/256x192/GSU8BPP256x192FillPoly.sfc | 32768 | 477998c920ee3a8229cbd3adb61edbb16643b153e4bfbc14b1c93999a3e857b1 |
| homebrew/peterlemon/CHIP/GSU/8BPP/PlotLine/256x128/GSU8BPP256x128PlotLine.sfc | 32768 | 00fab18c604a873bd83d7e1df8db0acf39d2d51dcbf835653ff8e904759811c8 |
| homebrew/peterlemon/CHIP/GSU/8BPP/PlotLine/256x160/GSU8BPP256x160PlotLine.sfc | 32768 | 29bece3ad2ad6d2716d2f5ede8632042dd2185574c34b2691940e1ea33dd8941 |
| homebrew/peterlemon/CHIP/GSU/8BPP/PlotLine/256x192/GSU8BPP256x192PlotLine.sfc | 32768 | 0b33a1be398a20ca08609e60c022a5eaeae0df6f0f8331522e16b0c1783595e2 |
| homebrew/peterlemon/CHIP/GSU/8BPP/PlotPixel/256x128/GSU8BPP256x128PlotPixel.sfc | 32768 | 9a5d7fde5ff5fb00c636b4f73bfe65adcd6ea6335160096f7e1300cece58b8af |
| homebrew/peterlemon/CHIP/GSU/8BPP/PlotPixel/256x160/GSU8BPP256x160PlotPixel.sfc | 32768 | 847095374dd07f253abd740cd6adf5508c03e44725319001ea9f64c231cb10ac |
| homebrew/peterlemon/CHIP/GSU/8BPP/PlotPixel/256x192/GSU8BPP256x192PlotPixel.sfc | 32768 | 756d57496dc577d02ab0ccbcb880bcdbd43a4a54cc98cfd4899db13a1c9f8cb3 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/ADC/GSUADC.sfc | 32768 | 985792cf8b4ae9781e8414e78246a0784ded72ebcdba78ed7484f5982e142946 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/ADD/GSUADD.sfc | 32768 | 1ccb785ce9de227572dc1a2e8753625d8225d4fbf37489f71d78cd06d568d460 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/AND/GSUAND.sfc | 32768 | 26379f858df03313b9268427c563e1a75d3f953b211916490f895c6bbd4d9b95 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/ASR/GSUASR.sfc | 32768 | bb0f31f976f7757854d2e78f49b5b597dec2517978f8a56d41a42460a3a5cc30 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/BIC/GSUBIC.sfc | 32768 | 08217e529bc8fa6aa0f3da0233458997c79b8e17d308c7fe6607bbe95b29501a |
| homebrew/peterlemon/CHIP/GSU/GSUTest/CACHEINJECT/GSUCACHEINJECT.sfc | 32768 | d74e5619663fdf209bc4e8f4108da769dfe9f317a4f36eaa013f21562e2a75df |
| homebrew/peterlemon/CHIP/GSU/GSUTest/CMP/GSUCMP.sfc | 32768 | 67c787d4a11ac58fb70adae5de9a02137faaee119fab71e05865a934ad5e614f |
| homebrew/peterlemon/CHIP/GSU/GSUTest/DEC/GSUDEC.sfc | 32768 | 907e529d4f5e26fcce3fe1b709e1daa81c1e6b98a76248b2bbe212c1d46c9020 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/DIV2/GSUDIV2.sfc | 32768 | d9183540cbd8c36cf621189fec1aae8b1c9580f4a6de4c8e9ce11fc68ccca27b |
| homebrew/peterlemon/CHIP/GSU/GSUTest/FMULT/GSUFMULT.sfc | 32768 | 8b82b3219dc71b9b087511f8b83dcca990571910f15e24c1258a62384a5818cc |
| homebrew/peterlemon/CHIP/GSU/GSUTest/HIB/GSUHIB.sfc | 32768 | acec35771a834ee6967f08fba0363a1740616752f92dcecf255f12c459c60665 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/IBT/GSUIBT.sfc | 32768 | e00a40dfb32b45e63dc892ababd935189b81bda70cd5d614abfcf92ef210bd1a |
| homebrew/peterlemon/CHIP/GSU/GSUTest/INC/GSUINC.sfc | 32768 | e79a508570cbc3f6377270df43ed153dbe8dd54923a92015f5c1a40ce295cab3 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/IWT/GSUIWT.sfc | 32768 | bf91d835dfdb2579b72f0f57946130bfea797c4264bb93684d618b2a975279d4 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/LMULT/GSULMULT.sfc | 32768 | eb2c24fdd206a1b757a41da3ab2683045901c83cbbd2862f4ca09ecca1a6432f |
| homebrew/peterlemon/CHIP/GSU/GSUTest/LOB/GSULOB.sfc | 32768 | 8e32fa8a21a411553373dae4fa04cafbe065a8d66920e7011045cfd1e15ca906 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/LSR/GSULSR.sfc | 32768 | d9895a519d6806e130b184c9fd4340eac24b732ef19793d28667b17f6f5d9964 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/MERGE/GSUMERGE.sfc | 32768 | 1c07529317c66fbed75fb3bcc2a21be050356974dfd5a2efe23e92ddad989280 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/MOVE/GSUMOVE.sfc | 32768 | 463398ae02bdcdaafafc7778a8892d37a52098e4f085ba0d190880f15c9eb428 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/MOVES/GSUMOVES.sfc | 32768 | 8af62f4bfb03d2f1a1c974c2fbcfe9443b2864aaaff78b295c9f1eddc19e749d |
| homebrew/peterlemon/CHIP/GSU/GSUTest/MULT/GSUMULT.sfc | 32768 | 1f085c681e7a13166628ab9ca8c54cde915a72f8b8099d100a3cfe2c5c3bb50f |
| homebrew/peterlemon/CHIP/GSU/GSUTest/NOT/GSUNOT.sfc | 32768 | 9a8db6613ba00d9544f342e68895e68378ed6ea30554fd33eacb650d1a2c1eac |
| homebrew/peterlemon/CHIP/GSU/GSUTest/OR/GSUOR.sfc | 32768 | 98194438bd066e3995d58ea3e673c083db4227e05e4f7ff6fceb3204201446d6 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/ROL/GSUROL.sfc | 32768 | 3a1fa14475c817a534a4ef534af8e6eab33a14314c14f3820ad6738682b0a108 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/ROR/GSUROR.sfc | 32768 | c3e622136cdc0f549b0e23464d393521e22f5cf4a0c86b559f9b779d7a678b93 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/SBC/GSUSBC.sfc | 32768 | 1ec152bc9747c356027027a38e4ed6604666c4a2434f1f79120182239f160b13 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/SEX/GSUSEX.sfc | 32768 | 3a3b138316c31cf79f49a340532e93e7c9617ecd8ffcbccbc64f5313b66b2146 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/SUB/GSUSUB.sfc | 32768 | 456d3efbb1b0e3237060cd85581e0f973e05e0d7ef19a34fc7fd85052f34ebc9 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/SWAP/GSUSWAP.sfc | 32768 | 2aafc101d578dfd5c292e44b99baac18ac9141f5a23b12cc40ed99f736c22c03 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/UMULT/GSUUMULT.sfc | 32768 | 8c6640ca1ffcf93e7d8e425812fcc36fc133a51feca4cee3c39fdb8104e02fc6 |
| homebrew/peterlemon/CHIP/GSU/GSUTest/XOR/GSUXOR.sfc | 32768 | 3898c53fac2d8fa3005405ddf8434e7feb3856e076e5e677c33dbfde65ca50b2 |
| homebrew/peterlemon/CPUTest/CPU/ADC/CPUADC.sfc | 32768 | d6c4f5a284ce7842b8a3944b2bc51c8cde2f5932305f9b561334f4b7f49b5b4c |
| homebrew/peterlemon/CPUTest/CPU/AND/CPUAND.sfc | 32768 | b7438031625af35e86f2c1725a7157358525ffc83ef1f15c02e3654e52ac61c6 |
| homebrew/peterlemon/CPUTest/CPU/ASL/CPUASL.sfc | 32768 | 0abb736aaaf683a183355e712ccb98d170bb25e1e13db456b2c21d8be4de480f |
| homebrew/peterlemon/CPUTest/CPU/BIT/CPUBIT.sfc | 32768 | fdfc621bcd1f9dfcc0800b331d540af69ca70dd7e075632b144b46d9583fa15e |
| homebrew/peterlemon/CPUTest/CPU/BRA/CPUBRA.sfc | 32768 | 1101903669864e5917f6d1f427ee24ca95c481fd4566125576e0f5a247f0cb25 |
| homebrew/peterlemon/CPUTest/CPU/CMP/CPUCMP.sfc | 32768 | b24c5e15301a0b68806157af642b39d4d38c5dfdc335eb44ddcec0d9e0222672 |
| homebrew/peterlemon/CPUTest/CPU/DEC/CPUDEC.sfc | 32768 | d0d50a863b2b3dc6d5c14aad134c1b47dbd29bdc9af37ce34d68b552f3a3dee9 |
| homebrew/peterlemon/CPUTest/CPU/EOR/CPUEOR.sfc | 32768 | 4db10b39d39125dcb83cddd5f8ad329c95653eb1d3e266db7a58a1ad1c97a2bd |
| homebrew/peterlemon/CPUTest/CPU/INC/CPUINC.sfc | 32768 | d32d4a8128d07970e4d652d907b2fcb51ea2d79d311d5f4a452c357644167b18 |
| homebrew/peterlemon/CPUTest/CPU/JMP/CPUJMP.sfc | 32768 | 8be0c153f166e11d7d384cffffe5ae27ea9c1a57eb4a1b36a85759bab2fe59a3 |
| homebrew/peterlemon/CPUTest/CPU/LDR/CPULDR.sfc | 32768 | 4598006dd6ec9ce03fa6622e18178ef25a6d471926ca40abb8748c10006e92da |
| homebrew/peterlemon/CPUTest/CPU/LSR/CPULSR.sfc | 32768 | 01517b0bd134d93fe858d536f76ea36691b23b10462b19caef693c8b67c6b60d |
| homebrew/peterlemon/CPUTest/CPU/MOV/CPUMOV.sfc | 32768 | 98ae96bdc5035fe9074e5b7c2dc0a8da67cb94f3d9c5668f26f936d9f99b6ab5 |
| homebrew/peterlemon/CPUTest/CPU/MSC/CPUMSC.sfc | 32768 | 00fa9244fe5a086042c4ca8e9f425892dbf115b135d7a14630c88dee3fd18490 |
| homebrew/peterlemon/CPUTest/CPU/ORA/CPUORA.sfc | 32768 | e9e4fdd107faa16ec43c13756a465b02f72a576b79e6b00869931535e3922379 |
| homebrew/peterlemon/CPUTest/CPU/PHL/CPUPHL.sfc | 32768 | 27bb4a9e21d1e1021a7287cd48c612c87888ca83b4e30323c22d24ccb0764f76 |
| homebrew/peterlemon/CPUTest/CPU/PSR/CPUPSR.sfc | 32768 | bd85187c7da1006b7c7111a8921b9b13cc73043a89eb7353f3d4fff207273067 |
| homebrew/peterlemon/CPUTest/CPU/RET/CPURET.sfc | 32768 | 0f65834dae85684202643e648166c37c8ea7364a295992c92e99506d65c7247e |
| homebrew/peterlemon/CPUTest/CPU/ROL/CPUROL.sfc | 32768 | 2b9c67c63befa2d0419c9a9f9b7fe2963fbe3cf84a0663f0bd36107c5aa0b255 |
| homebrew/peterlemon/CPUTest/CPU/ROR/CPUROR.sfc | 32768 | ff73f1dd8c2dd8605f8fbc09ff4fda8312741cd382258ec09518763056792fec |
| homebrew/peterlemon/CPUTest/CPU/SBC/CPUSBC.sfc | 32768 | b9a688dcfcf9316227de98c305a260af35c80d0a0fd48bd227a21051159634a1 |
| homebrew/peterlemon/CPUTest/CPU/STR/CPUSTR.sfc | 32768 | f8bd66164c369d956145c747d71ce59c252693a735cc644a34521a599cf6847a |
| homebrew/peterlemon/CPUTest/CPU/TRN/CPUTRN.sfc | 32768 | 60776a7bfbdf3f1f0e57885e885c66b9c0ebfa7866f18fece8f8cefd82221ad1 |
| homebrew/peterlemon/CPUTest/SPC700/ADC/SPC700ADC.sfc | 32768 | 4fa785418a109c0ef052cb9b71f55d37ee56834fca170fe1a0da784ee3df0e2e |
| homebrew/peterlemon/CPUTest/SPC700/AND/SPC700AND.sfc | 32768 | a4e7a9d6a926f5b575b2c3006b301bf837ed36072c9617776aab527706c3242d |
| homebrew/peterlemon/CPUTest/SPC700/DEC/SPC700DEC.sfc | 32768 | fe9254720e62b2aa953a42f50827facc12f7549ad8ca3775c89a5b65f4b55fdf |
| homebrew/peterlemon/CPUTest/SPC700/EOR/SPC700EOR.sfc | 32768 | 061572e2ba5599f78c3573be3b538ba2ab67703529f7518eee8e8ce3e41372f1 |
| homebrew/peterlemon/CPUTest/SPC700/INC/SPC700INC.sfc | 32768 | 467376b5ab3cf83197553ec598f91645beef628dadbd43932b6e96d2fa17776b |
| homebrew/peterlemon/CPUTest/SPC700/ORA/SPC700ORA.sfc | 32768 | aefb416d0f681eed0975cfbee70fd3048f59154cd5502f550e48e80f789a0ab2 |
| homebrew/peterlemon/CPUTest/SPC700/SBC/SPC700SBC.sfc | 32768 | fcc6017db0560ce12ae029fa97947eedbd8eaf35dd32200c445281ca94cbae9b |
| homebrew/peterlemon/Compress/LZ77/LZ77WRAMGFX/LZ77WRAMGFX.sfc | 32768 | 8d01e89a6b35d396d10a5c03b7a2c99a4b2d691776b6e84a14a5a6732ec3381f |
| homebrew/peterlemon/Games/MonsterFarmJump/MonsterFarmJump.sfc | 524288 | 1bea742bf177900c71bd423593a658c2fc389f5b19b3eb30d7abddba641e18a3 |
| homebrew/peterlemon/HelloWorld/HelloWorld.sfc | 32768 | 63ed338621788ad4ab856bb782e16044193882b7840370b500c8359a41fb87ef |
| homebrew/peterlemon/INPUT/ControllerLatency/ControllerLatency.sfc | 32768 | 9e01faa75d9d09dc3c1bee3cbde76c468982dee3f27f55fae6de52be72b73767 |
| homebrew/peterlemon/INPUT/XBandKeyboard/XBandKeyboard.sfc | 32768 | 1722c82b4799a9f8254c37011edc3daed185ccc7e3fad190206fcfa91edf6638 |
| homebrew/peterlemon/MSU/AUDIO/AUDIO.sfc | 32768 | 86a6449cea5aa19b5fb44ed1652a1013fb95185999e4955dca09325fbbe24ec6 |
| homebrew/peterlemon/MSU/Touhou-BadApple!15FPS16ColInterlace/Touhou-BadApple!15FPS16ColInterlace.sfc | 32768 | 7f142c408e9db031f9be21c49412c6dc53835f20fe184d424f996facd42adc9a |
| homebrew/peterlemon/MSU/Touhou-BadApple!30FPS4ColInterlace/Touhou-BadApple!30FPS4ColInterlace.sfc | 32768 | f5647cab549f93a6eb5e9872267dd4959dca05f5a96c9069df4943749d3d1479 |
| homebrew/peterlemon/MSU/VIDEO15FPS15Col16Pal/VIDEO15FPS15Col16Pal.sfc | 32768 | df4446cfc2c74e2da25642ebaa2ffd05dd2630b2f4aead793ebdc193b7e4ce2d |
| homebrew/peterlemon/MSU/VIDEO15FPS15Col8Pal/VIDEO15FPS15Col8Pal.sfc | 32768 | 3a46dd7c4c4f8d2a9304052101edb26abcd8b9d0475beee0ee5d87d6b0d02031 |
| homebrew/peterlemon/MSU/VIDEO15FPS16Col/VIDEO15FPS16Col.sfc | 32768 | 1a2d85f41c1b74f9d9c57a50e4d17bb4138c39f6d5a5d27127cb648c88642c2e |
| homebrew/peterlemon/MSU/VIDEO15FPS16Col8Pal/VIDEO15FPS16Col8Pal.sfc | 32768 | 371ba251b8aa28180c82c86ae6f1979274534430023e0775f43166dc8a067164 |
| homebrew/peterlemon/MSU/VIDEO15FPS16ColInterlace/VIDEO15FPS16ColInterlace.sfc | 32768 | 83985572f732718d68d1eb60e6acbd9aadd9fe64fff7f0c8eb1383e916fe2c27 |
| homebrew/peterlemon/PPU/BGMAP/8x8/2BPP/8x8BG1Map2BPP32x328PAL/8x8BG1Map2BPP32x328PAL.sfc | 32768 | 66083b119131d4cd7b3c41301750342152d2c3a3cfce103de074a0ae53330146 |
| homebrew/peterlemon/PPU/BGMAP/8x8/2BPP/8x8BG2Map2BPP32x328PAL/8x8BG2Map2BPP32x328PAL.sfc | 32768 | 048fbb0ad29fa6821a127c10a51f8dc3bbc136ef0b678f1bd7b76ea5b6e2c8da |
| homebrew/peterlemon/PPU/BGMAP/8x8/2BPP/8x8BG3Map2BPP32x328PAL/8x8BG3Map2BPP32x328PAL.sfc | 32768 | 4fa8e6c48e0a3251088ae810fbd7a7022f52427837660669b05e66daf09c3494 |
| homebrew/peterlemon/PPU/BGMAP/8x8/2BPP/8x8BG4Map2BPP32x328PAL/8x8BG4Map2BPP32x328PAL.sfc | 32768 | 18e4d47a265aa3b787e57a6ffc2789cd5cace453e963aa5bc0932ee0c674e6cc |
| homebrew/peterlemon/PPU/BGMAP/8x8/4BPP/8x8BGMap4BPP32x328PAL/8x8BGMap4BPP32x328PAL.sfc | 32768 | 60d4c21e53ee731c15af870eb903fc13220dcbc9dac677ded1355c0b62e428f3 |
| homebrew/peterlemon/PPU/BGMAP/8x8/8BPP/32x32/8x8BGMap8BPP32x32.sfc | 98304 | 238d33f7ec2a24b73e3553dffeb300121c72df48e553eb0988a1e278e3a0af2d |
| homebrew/peterlemon/PPU/BGMAP/8x8/8BPP/32x64/8x8BGMap8BPP32x64.sfc | 98304 | 6dad09e729bbdf0da49b700855a86771aa10e25795eac8c55ee79ffeb6130696 |
| homebrew/peterlemon/PPU/BGMAP/8x8/8BPP/64x32/8x8BGMap8BPP64x32.sfc | 98304 | c7660e70f6ecef32289045122feb125714cba766d9e1515b44e3f9c1c3153a59 |
| homebrew/peterlemon/PPU/BGMAP/8x8/8BPP/64x64/8x8BGMap8BPP64x64.sfc | 98304 | 423fee52064fb581e7627f9ae63924d46df73056ec514cc93a1275a24d098b65 |
| homebrew/peterlemon/PPU/BGMAP/8x8/8BPP/TileFlip/8x8BGMapTileFlip.sfc | 32768 | 6107053b6b47baf036332897518a8f49ea57470cad3f6492e4ae763e61163b41 |
| homebrew/peterlemon/PPU/Blend/HiColor/HiColor1241DLair/HiColor1241DLair.sfc | 98304 | 2c887a0f6de4aa63498f08c6ba921120d7688f212121c4ce33a5efa25a0dddac |
| homebrew/peterlemon/PPU/Blend/HiColor/HiColor3840/HiColor3840.sfc | 98304 | b86c4a131809d923187ac14d2ee82d3aef53f7b61246f0aee8a6b2b28cab1256 |
| homebrew/peterlemon/PPU/Blend/HiColor/HiColor575Myst/HiColor575Myst.sfc | 98304 | 47316ae64e97fcc9d1a89611f3f6668cf036e1ca8bde5fc46c7bb07dafb48a19 |
| homebrew/peterlemon/PPU/GreenSpace/GreenSpace.sfc | 32768 | e3933942eb205f601d96ad9b4cb55d1de4bc44304a038f1ae77686a082ca9196 |
| homebrew/peterlemon/PPU/HDMA/HiColor128PerTileRow/HiColor128PerTileRow.sfc | 65536 | d80f7b9f7f764eb26d72618f68da1f24e4fb8a4563418824d645f34c6570d9ac |
| homebrew/peterlemon/PPU/HDMA/HiColor128PerTileRow/TEST/DQ128PerTileRow.sfc | 65536 | cb55abb2fde7ec87385b4e2e8bb30da65b990e303378278cc2251bec031679d7 |
| homebrew/peterlemon/PPU/HDMA/HiColor128PerTileRow/TEST/RGB_24bits_palette_color_test_chart128PerTileRow.sfc | 65536 | d80f7b9f7f764eb26d72618f68da1f24e4fb8a4563418824d645f34c6570d9ac |
| homebrew/peterlemon/PPU/HDMA/HiColor128PerTileRow/TEST/lenna128PerTileRow.sfc | 65536 | ef3e5ba2065644b4742f00fdaf8af532d5676ff2504a8b0754e659496cfb3d52 |
| homebrew/peterlemon/PPU/HDMA/HiColor128PerTileRow/TEST/mandrill128PerTileRow.sfc | 65536 | 544af28137908847441eaa43b66b79d5d7a6cbb8ce3a9917fcd5d77cd761a62f |
| homebrew/peterlemon/PPU/HDMA/HiColor64PerTileRow/HiColor64PerTileRow.sfc | 65536 | 3a45a7ebba613094a0a22a530d71d092a90ad13ea7f6cec92ac71a47ad036bde |
| homebrew/peterlemon/PPU/HDMA/HiColor64PerTileRow/TEST/DQ64PerTileRow.sfc | 65536 | d7bea83288fdbbe89ab6e2e9248e4ade5e2ddf7f5615006d90301ed72c8c189f |
| homebrew/peterlemon/PPU/HDMA/HiColor64PerTileRow/TEST/RGB_24bits_palette_color_test_chart64PerTileRow.sfc | 65536 | 3a45a7ebba613094a0a22a530d71d092a90ad13ea7f6cec92ac71a47ad036bde |
| homebrew/peterlemon/PPU/HDMA/HiColor64PerTileRow/TEST/lenna64PerTileRow.sfc | 65536 | 5362a1f68eb7737f0b857e3a2920a6835f2dc853062b5f00cf3b95e7dd21bf2c |
| homebrew/peterlemon/PPU/HDMA/HiColor64PerTileRow/TEST/mandrill64PerTileRow.sfc | 65536 | 174f499f25304b199c8296cec4b2156e7aa27c43f35b482768f2ca54c7c701e7 |
| homebrew/peterlemon/PPU/HDMA/HiColor64PerTileRowPseudoHiRes/HiColor64PerTileRowPseudoHiRes.sfc | 98304 | 01dace4535e9352aba93ea688e8d1c4f769482823b0dca0fda6606cbf3e51729 |
| homebrew/peterlemon/PPU/HDMA/HiColor64PerTileRowPseudoHiRes/TEST/RGB_24bits_palette_color_test_chart64PerTileRowHiRes.sfc | 98304 | 737f85d7224c18e370ab61658eb87a9062e934ea31c18be82bdc6639b7718f9f |
| homebrew/peterlemon/PPU/HDMA/HiColor64PerTileRowPseudoHiRes/TEST/lenna64PerTileRowHiRes.sfc | 98304 | 5f9d917efd692db1ecded8fc2b1c4cf95cef4b0570694997c45b06b4a5e8f45c |
| homebrew/peterlemon/PPU/HDMA/HiColor64PerTileRowPseudoHiRes/TEST/mandrill64PerTileRowHiRes.sfc | 98304 | 01dace4535e9352aba93ea688e8d1c4f769482823b0dca0fda6606cbf3e51729 |
| homebrew/peterlemon/PPU/HDMA/Mode7HDMA/Mode7HDMA.sfc | 32768 | e60a09ec6e5d57c1adc22b2f84fe24ca9259b60e0d3f7d986bd809151980543c |
| homebrew/peterlemon/PPU/HDMA/RedSpace9BitHDMA/RedSpace9BitHDMA.sfc | 32768 | ad06e07d45dc82d744d6d9fbe0a9b7897f3386fb475168c163824ee0997c1e44 |
| homebrew/peterlemon/PPU/HDMA/RedSpaceHDMA/RedSpaceHDMA.sfc | 32768 | f59bc2cc89b22cad7308fe65f5d9fee1025311c84a9f809e14ebf96ab9a4c4de |
| homebrew/peterlemon/PPU/HDMA/RedSpaceIndirectHDMA/RedSpaceIndirectHDMA.sfc | 32768 | bfc85c6c05399512e33c4ec5393e90eba6a4465bdcdb6c3f2624d2f7c2cf90ff |
| homebrew/peterlemon/PPU/HDMA/WaveHDMA/WaveHDMA.sfc | 98304 | b9ecef8a41628d74900cbe6334ae24680dcd6e5526292f3e68aa39be34518b31 |
| homebrew/peterlemon/PPU/Interlace/InterlaceFont/InterlaceFont.sfc | 32768 | 097e4c79fbf351c8e8b74c73ce3d60752ec5f8691185f46737aac786f45dd36c |
| homebrew/peterlemon/PPU/Interlace/InterlaceMoogle/InterlaceMoogle.sfc | 32768 | 322ae93b6294ee5e75df3ed11a4eef3bbccbf4e6f54933d7ca01f91d26576fe0 |
| homebrew/peterlemon/PPU/Interlace/InterlaceMystHDMA/InterlaceMystHDMA.sfc | 65536 | 0dfa5dec0394424c3d29d0685b7373c521c82f2334d9b275c06d13cf7703029b |
| homebrew/peterlemon/PPU/Interlace/InterlaceRPG/InterlaceRPG.sfc | 65536 | 39489ace933125e2aa9e71e6ea90420b96cbb5e64bc07e4b5abaa06a0c3bf133 |
| homebrew/peterlemon/PPU/Interlace/InterlaceScroll/InterlaceScroll.sfc | 32768 | 98dcf13d979775afb49b8585845c0cdfd024c6744406978d231d2bd8c865de59 |
| homebrew/peterlemon/PPU/Interlace/InterlaceSimpsonsHDMA/InterlaceSimpsonsHDMA.sfc | 65536 | 300013acbf4987a73ab83cd1fdc86b8f709482eda9c09dbe572dedbfa8600783 |
| homebrew/peterlemon/PPU/Mode7/Perspective/Perspective.sfc | 131072 | cf31c9af58af91bd3541b1ee4133fca3379ea93416a7ed718bc21d3a5044422a |
| homebrew/peterlemon/PPU/Mode7/RotZoom/RotZoom.sfc | 32768 | d5580870e152046ca838fa276568e10c671be63b1db0285030a53dc7864cc4c0 |
| homebrew/peterlemon/PPU/Mode7/StarWars/StarWars.sfc | 131072 | f94388694230db97b7026db91310bc3adde11d4f255fea4738938fa41f9148bf |
| homebrew/peterlemon/PPU/Mosaic/Mode3/MosaicMode3.sfc | 98304 | 6deebf7ddbe46392d900914dd69e6a060d868e9cc7436a486987c1fb868885db |
| homebrew/peterlemon/PPU/Mosaic/Mode5/MosaicMode5.sfc | 32768 | da0197247fa6c19f076351a2110608ad248e10d199310191c69e0a91b6fa339c |
| homebrew/peterlemon/PPU/Rings/Rings.sfc | 32768 | 9622cef24adb23093c8b66bbcf807ffb00ceb48ebfd02fa62b49130cdbc1e556 |
| homebrew/peterlemon/PPU/Window/WindowHDMA/WindowHDMA.sfc | 65536 | 27594315a409d11f0182838201901aedeaaf825e986a0c9999111f08cb83ffbf |
| homebrew/peterlemon/PPU/Window/WindowMultiHDMA/WindowMultiHDMA.sfc | 65536 | 1bacb0c050a42c8ba3a53268f1bf74cd8889234ca904e195490adf3bf682f363 |
| homebrew/peterlemon/PlotLine/Mode7/PlotLineMode7.sfc | 32768 | ac4301894011eb3d311d405d0034fe2d6b97aa42b68effe8895f61dfab888b90 |
| homebrew/peterlemon/PlotPixel/Mode7/PlotPixelMode7.sfc | 32768 | 95e0a82e2e0de259e74eb7425bdedea63a57fe4945a783c765d54b5ae4ed0a90 |
| homebrew/peterlemon/SPC700/Axel-F/Axel-F.sfc | 98304 | 96fdf1cf0ad6cd06cbc2d20127b112ffe88b0e7ac22fd351755ed9a59ce1eaf5 |
| homebrew/peterlemon/SPC700/FFVIIPrelude/FFVIIPrelude.sfc | 98304 | 29537a0825bce4801f0448aa81246f34839939c2b9de54ec82856b13350a48d5 |
| homebrew/peterlemon/SPC700/ItaloTest/ItaloTest.sfc | 32768 | 7464c65cba3145537951feaccfc2599da373a37be03ec7bb12068ba063cc850f |
| homebrew/peterlemon/SPC700/PitchMod/PitchMod.sfc | 32768 | 2a5cc3eba5d0f1e3c35afe32490c3398be4334adb1f3f0d3616bac0980a01a02 |
| homebrew/peterlemon/SPC700/PlayBRRSample/PlayBRRSample.sfc | 32768 | 0bf2ded4b9cfb2c23c985fae9e54a5075dd105cfd5cc98c11ee718af50d86c23 |
| homebrew/peterlemon/SPC700/PlayNoise/PlayNoise.sfc | 32768 | ae0841adaf3785a54c74852e9e0d8b7ac0bdfef5049f79e305dbeb137a8ff347 |
| homebrew/peterlemon/SPC700/PlayTwoSong/PlayTwoSong.sfc | 65536 | 5b09082d4f8d737117042fdd603ec9e7ca31ef8c9517c5c5188ba800aac1e676 |
| homebrew/peterlemon/SPC700/SpeechSynth/SpeechSynth.sfc | 98304 | 6bbbcb083db2788afc93804370e5d8052e6c513aa998b8f156a13af7409379d8 |
| homebrew/peterlemon/SPC700/Twinkle/Twinkle.sfc | 32768 | 74bd4116755d9eeb761591107ca3d683fb369f25895e43b2f7c05746f1276817 |

## Jeux commerciaux (fournis par l'utilisateur)

Place commercial ROMs in `tests/roms/games/` with these exact names:

- `smw.sfc` - Super Mario World
- `alttp.sfc` - The Legend of Zelda: A Link to the Past
- `smb_allstars.sfc` - Super Mario All-Stars
- `sfa2.sfc` - Street Fighter Alpha 2 (S-DD1)
- `starocean.sfc` - Star Ocean (S-DD1)

These are never committed to git.

Launch games with: `target/release/puce16-desktop tests/roms/games/<name>.sfc`
