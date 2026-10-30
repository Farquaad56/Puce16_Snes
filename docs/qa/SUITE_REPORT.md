# SUITE REPORT (E07-11)

**Date:** 2026-10-28

## Summary

suite: 1 pass, 0 fail, 2 skip, 139 new

Verdict counts: NEW: 139, PASS: 1, SKIP: 2

> **Note:** Verdicts `NEW` are normal (the baseline is empty) and will be validated at step E07-12.

## All ROM Results

| name | epic | verdict | detail |
|------|------|---------|--------|
| blargg_spc_spc_dsp6 | E06 | NEW | frame_hash=0x042565ef385d9730 state_hash=0xa45099a350f25186 audio_hash=0x81d16ec40d51bc35 |
| blargg_spc_spc_mem_access_times | E03 | NEW | frame_hash=0x14a3b1d279380ecf state_hash=0xf143aa77ea88b058 audio_hash=0x69a53d5a0a363c15 |
| blargg_spc_spc_smp | E06 | NEW | frame_hash=0xa83c1d5bc806018a state_hash=0x54830940023ce61b audio_hash=0x81d16ec40d51bc35 |
| blargg_spc_spc_timer | E06 | NEW | frame_hash=0xf8d7e6681149bbb2 state_hash=0x288214d3b5f3abaf audio_hash=0x81d16ec40d51bc35 |
| gilyon_cputest_cputest_basic | E02 | NEW | frame_hash=0x86d8e6a909fbf2f7 state_hash=0x683e1c4a551ff228 audio_hash=0x69a53d5a0a363c15 |
| gilyon_cputest_cputest_full | E02 | NEW | frame_hash=0x53dfb112c87cf49a state_hash=0x35b127b3559406df audio_hash=0x69a53d5a0a363c15 |
| gilyon_spctest_spctest | E06 | NEW | frame_hash=0xc63ddd7d28754a27 state_hash=0xf3ea2641c2a5f908 audio_hash=0x81d16ec40d51bc35 |
| higan_93143_hblank_dma_vram_hvdma | E03 | NEW | frame_hash=0x8c1ff9de7764aaca state_hash=0xdfa9e93d89562119 audio_hash=0xb051896b18fd6a25 |
| higan_93143_hblank_dma_vram_hvdma_max | E03 | NEW | frame_hash=0xa3c23fddd7341d4a state_hash=0xd303742426367e24 audio_hash=0xb051896b18fd6a25 |
| higan_lidnariq_ppu_bus_activity_ppubusact | E05 | NEW | frame_hash=0xe676e6cd9a4c7ce8 state_hash=0x7e345f9ac05a1286 audio_hash=0x69a53d5a0a363c15 |
| higan_lidnariq_smp_clock_speed_measurement_smpspeed | E03 | NEW | frame_hash=0xc616495f95472267 state_hash=0x83a1f0157eadd42f audio_hash=0xb051896b18fd6a25 |
| higan_tepples_tellinglys_tellinglys | E04 | NEW | frame_hash=0x3708e3953af433b2 state_hash=0x3c9c82bb0dc2e4ea audio_hash=0x69a53d5a0a363c15 |
| higan_undisbeliever_inidisp_hdma_2100_glitch_2ch_0a | E05 | NEW | frame_hash=0x4de48c1a88c8ddb0 state_hash=0x68e6c5c91fd36d21 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_hdma_2100_glitch_2ch_81 | E05 | NEW | frame_hash=0x59d3856ff4d06bf9 state_hash=0x468be5aa65f78fc8 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_hdma_2100_glitch | E05 | NEW | frame_hash=0xaf7f252c53939c69 state_hash=0x9280d5d67d9a5ab3 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_hdma_21ff_2100_0f_glitch | E05 | NEW | frame_hash=0xaf7f252c53939c69 state_hash=0xf0548c13878655f1 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_hdma_21ff_2100_glitch | E05 | NEW | frame_hash=0xa3b2a129d8514908 state_hash=0xf0548c13878655f1 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_hdma_21ff_glitch | E05 | NEW | frame_hash=0xaf7f252c53939c69 state_hash=0xf375b89ad37a9414 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_hdmaen_latch_test | E05 | NEW | frame_hash=0x1b3b48fb160d9d4a state_hash=0xdacbf2fc98d88a4b audio_hash=0x69a53d5a0a363c15 |
| higan_undisbeliever_inidisp_hdmaen_latch_test_2 | E05 | NEW | frame_hash=0x1b3b48fb160d9d4a state_hash=0xca71f8137fd743ca audio_hash=0x69a53d5a0a363c15 |
| higan_undisbeliever_inidisp_inidisp_brightness_delay | E05 | NEW | frame_hash=0x7a8515b1ba242e32 state_hash=0xbe3f83b2556ddfa1 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_inidisp_d7_glitch_test | E05 | NEW | frame_hash=0xaf7f252c53939c69 state_hash=0xa547de6529036271 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_inidisp_enable_display_mid_frame | E05 | NEW | frame_hash=0xb239d966dad6f57c state_hash=0x23df15337c624e7d audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_inidisp_forgot_to_force_blank | E05 | NEW | frame_hash=0xbbc5ebebc00ef9ca state_hash=0xe4fef19ce1a3e4f3 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_inidisp_hammer_0f | E05 | NEW | frame_hash=0xaf7f252c53939c69 state_hash=0xbc8e6da7c88a8af4 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_inidisp_hammer_0f00 | E05 | NEW | frame_hash=0xaf7f252c53939c69 state_hash=0x2c80c7ba06021cd7 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_inidisp_hammer_0f0f | E05 | NEW | frame_hash=0xaf7f252c53939c69 state_hash=0x46c5e92be9a4b7fe audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_inidisp_hammer_0f8f | E05 | NEW | frame_hash=0xaf7f252c53939c69 state_hash=0xf6bccb720f923f28 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_inidisp_hammer_0f8f_fast | E05 | NEW | frame_hash=0xaf7f252c53939c69 state_hash=0xadd42011647b347e audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_inidisp_hammer_0f_long | E05 | NEW | frame_hash=0xaf7f252c53939c69 state_hash=0x48e2caef892ce489 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_inidisp_hammer_8f0f | E05 | NEW | frame_hash=0xb3d415c7c3801d4a state_hash=0x31aaa589ec36cbd3 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_scpu_a_dma_bug_1 | E05 | NEW | frame_hash=0xe2ed74afbf377b4a state_hash=0x08a0d5ca991b8022 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_scpu_a_dma_bug_2 | E05 | NEW | frame_hash=0xa7936b49a7f77b4a state_hash=0x42466930b056ecb9 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_scpu_a_dma_bug_3 | E05 | NEW | frame_hash=0x5f33dcae9f5855ca state_hash=0x5b228628ff5ecf4e audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_scpu_a_dma_bug_5 | E05 | NEW | frame_hash=0x92a70d1b3bebb1ca state_hash=0x1f9ae0ff050034ad audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_scpu_a_dma_bug_ch0 | E05 | NEW | frame_hash=0xa7936b49a7f77b4a state_hash=0x36465dd45132eba8 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_scpu_a_dma_bug_fix | E05 | NEW | frame_hash=0xa7936b49a7f77b4a state_hash=0xc63de8fc26e9b26d audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_scpu_a_dma_bug_fix2 | E05 | NEW | frame_hash=0xa7936b49a7f77b4a state_hash=0xe849115265d4c8a1 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_scpu_a_dma_bug_r2 | E05 | NEW | frame_hash=0xe2ed74afbf377b4a state_hash=0xdab9e7cd083c80b4 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_scpu_a_dma_bug_strange | E05 | NEW | frame_hash=0xe2ed74afbf377b4a state_hash=0x2780c6c91e8ddf71 audio_hash=0x5a9123c141aa9535 |
| higan_undisbeliever_inidisp_scpu_a_dma_bug_two_regs | E05 | NEW | frame_hash=0xa7936b49a7f77b4a state_hash=0x4259aa4d5259c4ef audio_hash=0x5a9123c141aa9535 |
| peterlemon_bank_hiromfastrom_bankhiromfastrom | E03 | NEW | frame_hash=0xf5a07341a234b498 state_hash=0x1f66809c0bb4ad7d audio_hash=0xb051896b18fd6a25 |
| peterlemon_bank_hiromslowrom_bankhiromslowrom | E03 | NEW | frame_hash=0xb7e190514ac765f8 state_hash=0x7032a26ebeee7ad2 audio_hash=0xb051896b18fd6a25 |
| peterlemon_bank_loromfastrom_bankloromfastrom | E03 | NEW | frame_hash=0x631b86a18a985312 state_hash=0xf892812284e2349b audio_hash=0xb051896b18fd6a25 |
| peterlemon_bank_loromslowrom_bankloromslowrom | E03 | NEW | frame_hash=0x34cba4a4c9dbe7a2 state_hash=0x00ba45e7fc3e52dc audio_hash=0xb051896b18fd6a25 |
| peterlemon_bank_wram_bankwram | E03 | NEW | frame_hash=0xc887f10ff9425f10 state_hash=0x1680c6cc66a65199 audio_hash=0xb051896b18fd6a25 |
| peterlemon_compress_lz77_lz77wramgfx_lz77wramgfx | E04 | NEW | frame_hash=0xedc4925ecd5c555c state_hash=0xc13d398268886cf9 audio_hash=0x875dbd0c337ba585 |
| peterlemon_cputest_cpu_adc_cpuadc | E02 | NEW | frame_hash=0xe4e811a2e9683208 state_hash=0x26de1791734c4a4d audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_and_cpuand | E02 | NEW | frame_hash=0x4fe9bf206758d944 state_hash=0x164a299bd1b12e3d audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_asl_cpuasl | E02 | NEW | frame_hash=0xd6aa45ec9e1f7ebe state_hash=0xe79b0e9fee796f3e audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_bit_cpubit | E02 | NEW | frame_hash=0x4840080b866716b2 state_hash=0xffd3e8f78a8e5426 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_bra_cpubra | E02 | NEW | frame_hash=0x2ba15d90a6569bac state_hash=0x933365883f41dba1 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_cmp_cpucmp | E02 | NEW | frame_hash=0xa844abf12b756e78 state_hash=0x2c0e23e4eb603bea audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_dec_cpudec | E02 | NEW | frame_hash=0x4e3c577d2d28118c state_hash=0xbbc7821324b7b61c audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_eor_cpueor | E02 | NEW | frame_hash=0x7304918838b4a9d4 state_hash=0x1f72aeb3083524f9 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_inc_cpuinc | E02 | NEW | frame_hash=0x13b78ef92fbb8448 state_hash=0xf63685e2a23bb6f6 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_jmp_cpujmp | E02 | NEW | frame_hash=0x382ae7ea9f84c44e state_hash=0x277b6790f2a68764 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_ldr_cpuldr | E02 | NEW | frame_hash=0x37c35b138b975808 state_hash=0x25c639160f27ab85 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_lsr_cpulsr | E02 | NEW | frame_hash=0xe0dc4afa00499c18 state_hash=0x420e5c5f728ebfb1 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_mov_cpumov | E02 | NEW | frame_hash=0xa10eb2e9c841085c state_hash=0x522d724c2ceb9b83 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_msc_cpumsc | E02 | NEW | frame_hash=0x43dca2917a0ae1d0 state_hash=0xfb19587fb8bd44f0 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_ora_cpuora | E02 | NEW | frame_hash=0xe55eb8a1db400206 state_hash=0x328eea12b5bd8130 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_phl_cpuphl | E02 | NEW | frame_hash=0x556090a64e99c73a state_hash=0xced2c1fe5cf768fd audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_psr_cpupsr | E02 | NEW | frame_hash=0xbdb6ec11d7ecadb4 state_hash=0xd4209c444bae716e audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_ret_cpuret | E02 | NEW | frame_hash=0xab0a0827f9e87a5a state_hash=0xa0a3aa8c996e6366 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_rol_cpurol | E02 | NEW | frame_hash=0x7680a66e7d3896b0 state_hash=0xded018122949e47d audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_ror_cpuror | E02 | NEW | frame_hash=0x905a2d59592eb600 state_hash=0x9d3fbbc960bbf98e audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_sbc_cpusbc | E02 | NEW | frame_hash=0x7293bf632c5acf50 state_hash=0x417f6138367245c3 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_str_cpustr | E02 | NEW | frame_hash=0x51c634c5fa2a9054 state_hash=0x68b547f22283f3bb audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_cpu_trn_cputrn | E02 | NEW | frame_hash=0x04f99c13e14dcc7e state_hash=0xeb26b628d9bc4da4 audio_hash=0xb051896b18fd6a25 |
| peterlemon_cputest_spc700_adc_spc700adc | E02 | NEW | frame_hash=0xf59903f94d43c99e state_hash=0x7b738a9f8b822c1a audio_hash=0xdd4641cfae2b245c |
| peterlemon_cputest_spc700_and_spc700and | E02 | NEW | frame_hash=0x0fc2b84214ffc25a state_hash=0x2a7a88977943cd60 audio_hash=0xdd4641cfae2b245c |
| peterlemon_cputest_spc700_dec_spc700dec | E02 | NEW | frame_hash=0xb84deb77e3013292 state_hash=0xbf30722aed117cb4 audio_hash=0x495705b6dc053724 |
| peterlemon_cputest_spc700_eor_spc700eor | E02 | NEW | frame_hash=0x8833b369af6ca94e state_hash=0x192f701116443af0 audio_hash=0xdd4641cfae2b245c |
| peterlemon_cputest_spc700_inc_spc700inc | E02 | NEW | frame_hash=0x5bb93178fde702ec state_hash=0xe0ddb3c93de3b4ae audio_hash=0x495705b6dc053724 |
| peterlemon_cputest_spc700_ora_spc700ora | E02 | NEW | frame_hash=0x981c32f8910df800 state_hash=0x4b54626179fd6f64 audio_hash=0xdd4641cfae2b245c |
| peterlemon_cputest_spc700_sbc_spc700sbc | E02 | NEW | frame_hash=0xe7847bb6e0e9f79e state_hash=0xac03cacc183f0726 audio_hash=0xdd4641cfae2b245c |
| peterlemon_games_monsterfarmjump_monsterfarmjump | E04 | NEW | frame_hash=0xd9145550a69be744 state_hash=0x3d35c945f6fa5eda audio_hash=0x2475791f9a2b6bd5 |
| peterlemon_helloworld_helloworld | E04 | NEW | frame_hash=0xb38912443b404147 state_hash=0xad67c8d0365e4d8d audio_hash=0xb051896b18fd6a25 |
| peterlemon_input_controllerlatency_controllerlatency | E04 | NEW | frame_hash=0xb3d415c7c3801d4a state_hash=0x7592ae3c66c2320e audio_hash=0xb051896b18fd6a25 |
| peterlemon_input_xbandkeyboard_xbandkeyboard | E04 | NEW | frame_hash=0xffffa1c3e31ba827 state_hash=0x7931f50483d07a90 audio_hash=0xb051896b18fd6a25 |
| peterlemon_plotline_mode7_plotlinemode7 | E04 | NEW | frame_hash=0x8b112c234919ef4a state_hash=0xdf992c3da662bc06 audio_hash=0xb051896b18fd6a25 |
| peterlemon_plotpixel_mode7_plotpixelmode7 | E04 | NEW | frame_hash=0x528423ddbd60ea12 state_hash=0xcc503497bcd02501 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_bgmap_8x8_2bpp_8x8bg1map2bpp32x328pal_8x8bg1map2bpp32x328pal | E05 | NEW | frame_hash=0xcc9c40eb2ef27506 state_hash=0x7bd60ce95bc07908 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_bgmap_8x8_2bpp_8x8bg2map2bpp32x328pal_8x8bg2map2bpp32x328pal | E05 | NEW | frame_hash=0x9c965b5f32b13844 state_hash=0x4a061d926c03b0d5 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_bgmap_8x8_2bpp_8x8bg3map2bpp32x328pal_8x8bg3map2bpp32x328pal | E05 | NEW | frame_hash=0x9c965b5f32b13844 state_hash=0x81be14cb00e9457b audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_bgmap_8x8_2bpp_8x8bg4map2bpp32x328pal_8x8bg4map2bpp32x328pal | E05 | NEW | frame_hash=0x9c965b5f32b13844 state_hash=0x18510a57ac75da37 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_bgmap_8x8_4bpp_8x8bgmap4bpp32x328pal_8x8bgmap4bpp32x328pal | E05 | NEW | frame_hash=0x5cb1056265875ae8 state_hash=0xe657a43d8ff7a534 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_bgmap_8x8_8bpp_32x32_8x8bgmap8bpp32x32 | E05 | NEW | frame_hash=0x5b189f5c7c91ef53 state_hash=0x87ed51f17b4ec514 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_bgmap_8x8_8bpp_32x64_8x8bgmap8bpp32x64 | E05 | NEW | frame_hash=0x3486e7627b671f36 state_hash=0x9e600a0082036898 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_bgmap_8x8_8bpp_64x32_8x8bgmap8bpp64x32 | E05 | NEW | frame_hash=0x3486e7627b671f36 state_hash=0xbc388e538a4b8f73 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_bgmap_8x8_8bpp_64x64_8x8bgmap8bpp64x64 | E05 | NEW | frame_hash=0x3486e7627b671f36 state_hash=0x1c44e04f9ae1ce42 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_bgmap_8x8_8bpp_tileflip_8x8bgmaptileflip | E05 | NEW | frame_hash=0xb7acecbd49b0254a state_hash=0x5f35ed0dcd5881f1 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_blend_hicolor_hicolor1241dlair_hicolor1241dlair | E05 | NEW | frame_hash=0x10364159e8d61e2c state_hash=0x05cf87fcc6c011f6 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_blend_hicolor_hicolor3840_hicolor3840 | E05 | NEW | frame_hash=0x53c8571cb2e4609a state_hash=0x3d851a4e476409b2 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_blend_hicolor_hicolor575myst_hicolor575myst | E05 | NEW | frame_hash=0xf8ad65f3304d9157 state_hash=0x66d21c2befb7ab9d audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_greenspace_greenspace | E05 | NEW | frame_hash=0xa3c23fddd7341d4a state_hash=0x3755cd0feb7a5933 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor128pertilerow_hicolor128pertilerow | E05 | NEW | frame_hash=0xfdc92a03b73695bc state_hash=0xc472f40f183b895f audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor128pertilerow_test_dq128pertilerow | E05 | NEW | frame_hash=0x8e9cc59464d744a7 state_hash=0x00a57c8f08822d8d audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor128pertilerow_test_rgb_24bits_palette_color_test_chart128pertilerow | E05 | NEW | frame_hash=0xfdc92a03b73695bc state_hash=0xc472f40f183b895f audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor128pertilerow_test_lenna128pertilerow | E05 | NEW | frame_hash=0x8d86c0c8c8ff95ab state_hash=0xace8fe7ec8ce307c audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor128pertilerow_test_mandrill128pertilerow | E05 | NEW | frame_hash=0x1da457c1b893da9d state_hash=0x5c10dfcf84c9af84 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor64pertilerow_hicolor64pertilerow | E05 | NEW | frame_hash=0xcd45f48b0f8b879c state_hash=0x2382a8c1583d9cfe audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor64pertilerow_test_dq64pertilerow | E05 | NEW | frame_hash=0x84263fa25a0b61d6 state_hash=0x83425e3fce6e8909 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor64pertilerow_test_rgb_24bits_palette_color_test_chart64pertilerow | E05 | NEW | frame_hash=0xcd45f48b0f8b879c state_hash=0x2382a8c1583d9cfe audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor64pertilerow_test_lenna64pertilerow | E05 | NEW | frame_hash=0x03f5b0e611236b93 state_hash=0xbc8d7b5acca260e1 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor64pertilerow_test_mandrill64pertilerow | E05 | NEW | frame_hash=0xddeddcb28099f4a8 state_hash=0x23e5b73ae230f5e2 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor64pertilerowpseudohires_hicolor64pertilerowpseudohires | E05 | NEW | frame_hash=0xad49c5b8b7e9e55b state_hash=0x3268e5718d170141 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor64pertilerowpseudohires_test_rgb_24bits_palette_color_test_chart64pertilerowhires | E05 | NEW | frame_hash=0x4720e13a742aa121 state_hash=0x771b2c208771381a audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor64pertilerowpseudohires_test_lenna64pertilerowhires | E05 | NEW | frame_hash=0xf9ee79a6bdacfa6c state_hash=0xc8019079bb140f18 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_hicolor64pertilerowpseudohires_test_mandrill64pertilerowhires | E05 | NEW | frame_hash=0xad49c5b8b7e9e55b state_hash=0x3268e5718d170141 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_mode7hdma_mode7hdma | E05 | NEW | frame_hash=0x70035850eb00369f state_hash=0x5a1a48d41a7dab7a audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_redspace9bithdma_redspace9bithdma | E05 | NEW | frame_hash=0xcfc6b6ef0818f34a state_hash=0xe4ce481dd0acc5f0 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_redspacehdma_redspacehdma | E05 | NEW | frame_hash=0xa4b0eec2bf6f9d4a state_hash=0x8c1b991f5e62ece5 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_redspaceindirecthdma_redspaceindirecthdma | E05 | NEW | frame_hash=0xa4b0eec2bf6f9d4a state_hash=0x531ee73af87618bf audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_hdma_wavehdma_wavehdma | E05 | NEW | frame_hash=0xfe6468b90ed8ef97 state_hash=0xbde39ef01037b8d1 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_interlace_interlacefont_interlacefont | E05 | NEW | frame_hash=0xba89827ea5d3d1b5 state_hash=0x7b2fff9fe534d10f audio_hash=0x2475791f9a2b6bd5 |
| peterlemon_ppu_interlace_interlacemoogle_interlacemoogle | E05 | NEW | frame_hash=0xedc4925ecd5c555c state_hash=0xbe102aa78a3c90e2 audio_hash=0x2475791f9a2b6bd5 |
| peterlemon_ppu_interlace_interlacemysthdma_interlacemysthdma | E05 | NEW | frame_hash=0xe2fcaa34619723a4 state_hash=0x96ba5059e5cac1ee audio_hash=0xdccc973a9e636e65 |
| peterlemon_ppu_interlace_interlacerpg_interlacerpg | E05 | NEW | frame_hash=0x8edea2ff8a72d15b state_hash=0x3162fb7e077087f5 audio_hash=0xdccc973a9e636e65 |
| peterlemon_ppu_interlace_interlacescroll_interlacescroll | E05 | NEW | frame_hash=0x26635a3406f4d524 state_hash=0x593679546653c6f5 audio_hash=0x2475791f9a2b6bd5 |
| peterlemon_ppu_interlace_interlacesimpsonshdma_interlacesimpsonshdma | E05 | NEW | frame_hash=0x926fc10e0d565e4e state_hash=0x4e67d24c177bb844 audio_hash=0xdccc973a9e636e65 |
| peterlemon_ppu_mode7_perspective_perspective | E05 | NEW | frame_hash=0xb7bf2ab3cd6c1828 state_hash=0x3089988129e6ec6e audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_mode7_rotzoom_rotzoom | E05 | NEW | frame_hash=0x32ac1edb3722a219 state_hash=0xeb3c99ae88c791ca audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_mode7_starwars_starwars | E05 | NEW | frame_hash=0x81c9aad0b9cddfc3 state_hash=0x334a9d9ac019022e audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_mosaic_mode3_mosaicmode3 | E05 | NEW | frame_hash=0x0b7f288ca3998777 state_hash=0x4c133f61695ab58d audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_mosaic_mode5_mosaicmode5 | E05 | NEW | frame_hash=0xedc4925ecd5c555c state_hash=0x7c0b050607e867d4 audio_hash=0x2475791f9a2b6bd5 |
| peterlemon_ppu_rings_rings | E05 | NEW | frame_hash=0xac391731ad161e9c state_hash=0x6cb1deca135b5900 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_window_windowhdma_windowhdma | E05 | NEW | frame_hash=0xdd15a2efcebc42a2 state_hash=0x0c8a34fd8ea174f8 audio_hash=0xb051896b18fd6a25 |
| peterlemon_ppu_window_windowmultihdma_windowmultihdma | E05 | NEW | frame_hash=0x9340acfd4462dd6f state_hash=0xfffd05769d5b75b1 audio_hash=0xb051896b18fd6a25 |
| peterlemon_spc700_axel_f_axel_f | E06 | NEW | frame_hash=0xb3d415c7c3801d4a state_hash=0x1e3e147ef364adc9 audio_hash=0x773f07e3395b7097 |
| peterlemon_spc700_ffviiprelude_ffviiprelude | E06 | NEW | frame_hash=0xb3d415c7c3801d4a state_hash=0x7cd90cc086e763ae audio_hash=0x3eb5a90e3c96ff48 |
| peterlemon_spc700_italotest_italotest | E06 | NEW | frame_hash=0xb3d415c7c3801d4a state_hash=0x8e90c03f28ccc95e audio_hash=0x6b04d6a48f1e0af2 |
| peterlemon_spc700_pitchmod_pitchmod | E06 | NEW | frame_hash=0xb3d415c7c3801d4a state_hash=0xd4e753a1549ea5a1 audio_hash=0xe22eb43f03b66192 |
| peterlemon_spc700_playbrrsample_playbrrsample | E06 | NEW | frame_hash=0xb3d415c7c3801d4a state_hash=0xdc0ec0a5a28a9e2a audio_hash=0x1daf49ca26c9ff7a |
| peterlemon_spc700_playnoise_playnoise | E06 | NEW | frame_hash=0xb3d415c7c3801d4a state_hash=0xcd387e81e59ca621 audio_hash=0x2b6c44efbcbc4395 |
| peterlemon_spc700_playtwosong_playtwosong | E06 | NEW | frame_hash=0xb3d415c7c3801d4a state_hash=0x8d35a66dc17c9e09 audio_hash=0x31180c01b9562445 |
| peterlemon_spc700_speechsynth_speechsynth | E06 | NEW | frame_hash=0xb3d415c7c3801d4a state_hash=0x20c7a151ea6e26bd audio_hash=0x2eea808a7e8c9b3d |
| peterlemon_spc700_twinkle_twinkle | E06 | NEW | frame_hash=0xb3d415c7c3801d4a state_hash=0xac0f4dc639e55193 audio_hash=0xd85135a241c22ae2 |
| game_smw | E07 | PASS |  |
| game_alttp: | UNKNOWN | SKIP | file missing |
| game_allstars: | UNKNOWN | SKIP | file missing |

## Failed ROMs by Epic

None.

## UNKNOWN (documente a l'etape E07-12, decision utilisateur du 2026-10-28)

| ROM | epopee | cause |
|---|---|---|
| blargg_spc_spc_mem_access_times | E03 | `Failed 02`, CRC affiche 53B06F2C. Positions des acces bus du SPC700 conformes a l'oracle (256/256, `e07_11b_access_cycle_positions`), mais la table attendue par la ROM est introuvable: ecart non localise. Derogation explicite a la regle "E03 doit passer". ROM non figee dans la baseline. |
| blargg_spc_spc_smp | E06 | Meme ecran et meme CRC que ci-dessus (controle commun). Non fige. |
| blargg_spc_spc_dsp6 | E06 | Passe Echo/*, `attack->decay during gain`, `decay->sustain during gain`, `envelope rates`; `Failed 02` sur `Envelope/gain $E0 threshold` (CRC E7EEFEC8). Le timing des registres DSP (ENVX/OUTX/ENDX par horloge, buffers partages, latch ADSR1) reproduit la reference blargg: 0 ecart sur 26 510 lectures rejouees (E07-11e). La ROM echoue pourtant a l'identique et ne depend pas de la phase DSP/SMP: cause probable cote SMP, meme famille que les deux lignes ci-dessus. Option A retenue (decision utilisateur du 2026-10-28, `docs/qa/inbox/E07-11e-Q1.md`): ordre d'enveloppe de la spec 07b conserve (`Dsp::set_reference_order` desactive). Manifeste inchange (300 frames), non fige. |

## Derogation E05 (decision utilisateur du 2026-10-28)

Les 5 ROM `peterlemon_ppu_hdma_hicolor128pertilerow_*` (hicolor128pertilerow, dq, rgb chart, lenna, mandrill) sont figees EN L'ETAT dans la baseline: image correcte mais fins artefacts horizontaux (lignes de tearing HDMA visibles sur dq, lenna, mandrill). Pas de tache de tuning creee; a reprendre si on veut un rendu exact (timing HDMA par ligne). Un changement de timing HDMA fera donc echouer ces 5 baselines et obligera a les re-figer.

Pistes si reprise (SPC700): fournir la table/CRC attendus ou une sortie Mesen/bsnes/Snes9x de la ROM, puis regler ligne par ligne (adresse des acces fantomes pour les cycles `.`, instant du tick timer/DSP dans `apu/io.rs`).
## E09-00: sprites decales d'une ligne (formule `row = (r - y) & 0xFF`)

Correction de la formule de rangee des sprites (SMW cache ses sprites a Y=240; avec l'ancienne formule la rangee 15 d'un 16x16 retombait sur la ligne 0). Seule la couche OBJ et les drapeaux range/time over dependent de cette formule. Comparaison image a image ancienne/nouvelle formule sur les 139 ROM: 25 images changent (les 3 jeux `game_*` n'ont pas de hash fige), toutes par decalage d'une ligne des sprites.

ROM re-figees dans `tests/baseline.json` (22):
- `higan_lidnariq_ppu_bus_activity_ppubusact`
- `higan_undisbeliever_inidisp_hdma_2100_glitch`, `..._hdma_2100_glitch_2ch_0a`, `..._hdma_2100_glitch_2ch_81`, `..._hdma_21ff_2100_0f_glitch`, `..._hdma_21ff_2100_glitch`, `..._hdma_21ff_glitch`
- `higan_undisbeliever_inidisp_inidisp_d7_glitch_test`, `..._inidisp_enable_display_mid_frame`, `..._inidisp_hammer_0f`, `..._hammer_0f00`, `..._hammer_0f0f`, `..._hammer_0f8f`, `..._hammer_0f8f_fast`, `..._hammer_0f_long`
- `peterlemon_ppu_hdma_hicolor128pertilerow_{hicolor128pertilerow,test_dq128pertilerow,test_rgb_24bits_palette_color_test_chart128pertilerow,test_lenna128pertilerow,test_mandrill128pertilerow}` (OAM par defaut: sprites parasites decales d'une ligne; la derogation E05 ci-dessus reste valable pour les artefacts HDMA)
- `peterlemon_ppu_interlace_interlacerpg_interlacerpg`, `peterlemon_ppu_mode7_starwars_starwars`

Les tests de rendu de reference ont ete alignes: `docs/tools/ppu_ref.py` (formule + autotest) et `ppu_scenes.expected` regeneres (23 scenes, moteur et reference independante d'accord), `e04_10_hash` (reste sur la ligne 0 pour eviter un range_over precoce avec l'OAM a zero), `e05_16_compose` (Y decales d'une unite).
