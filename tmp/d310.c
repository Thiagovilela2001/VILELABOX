==== FUN_14118d310 @ 14118d310 (prof 1)

/* WARNING: Function: __security_check_cookie replaced with injection: security_check_cookie */

undefined8
FUN_14118d310(longlong param_1,undefined8 *param_2,double param_3,int param_4,uint param_5,
             double param_6,uint param_7,longlong *param_8,int param_9)

{
  double dVar1;
  bool bVar2;
  int iVar3;
  double *pdVar4;
  longlong lVar5;
  longlong lVar6;
  longlong lVar7;
  double *pdVar8;
  uint uVar9;
  int iVar10;
  int *piVar11;
  undefined8 uVar12;
  ulonglong uVar13;
  ulonglong uVar14;
  int iVar15;
  uint uVar16;
  int iVar17;
  ulonglong uVar18;
  uint uVar19;
  int iVar20;
  double dVar21;
  double dVar22;
  undefined1 auVar23 [16];
  undefined1 extraout_var [56];
  undefined1 auVar24 [16];
  undefined1 auVar25 [16];
  undefined1 auVar26 [16];
  undefined1 auVar27 [16];
  undefined1 auVar28 [16];
  undefined1 auVar29 [16];
  undefined1 auVar30 [16];
  undefined1 auVar31 [16];
  undefined1 auVar32 [16];
  double dVar33;
  undefined1 auVar34 [64];
  double dVar35;
  double dVar36;
  undefined1 auVar37 [64];
  undefined1 auVar38 [64];
  undefined1 auVar39 [64];
  undefined1 auVar40 [64];
  undefined1 in_ZMM12 [64];
  undefined1 auVar41 [64];
  undefined1 in_ZMM13 [64];
  undefined1 auVar42 [64];
  undefined1 auVar43 [64];
  undefined1 auVar44 [64];
  undefined1 auVar45 [64];
  undefined1 auVar46 [64];
  undefined1 auStack_308 [32];
  longlong local_2e8;
  int local_2e0;
  double local_2d8;
  int local_2d0;
  uint local_2cc;
  uint local_2c8;
  uint local_2c4;
  uint local_2c0;
  uint local_2bc;
  double *local_2b8;
  uint local_2b0;
  longlong local_2a8;
  int *local_2a0;
  uint local_298;
  int local_294;
  undefined1 local_290 [24];
  int local_278;
  undefined1 local_270 [24];
  undefined4 local_258;
  double local_250;
  longlong local_248;
  longlong *local_240;
  longlong local_238;
  longlong local_230;
  double local_228;
  double local_218 [6];
  ulonglong uStack_1e8;
  ulonglong uStack_1e0;
  ulonglong local_1d8;
  ulonglong uStack_1d0;
  ulonglong uStack_1c8;
  ulonglong uStack_1c0;
  ulonglong local_1b8;
  ulonglong uStack_1b0;
  ulonglong uStack_1a8;
  ulonglong uStack_1a0;
  undefined1 local_198 [24];
  int local_180;
  undefined1 local_178 [24];
  int local_160;
  undefined1 local_158 [32];
  undefined1 local_138 [32];
  int local_118 [8];
  ulonglong local_f8;
  undefined1 local_c8 [16];
  undefined1 local_b8 [16];
  
  local_b8 = in_ZMM12._0_16_;
  local_c8 = in_ZMM13._0_16_;
  local_f8 = DAT_1459ca6c0 ^ (ulonglong)auStack_308;
  local_240 = param_8;
  local_2a8 = param_1;
  FUN_141166770(local_270);
  FUN_141166770(local_290);
  FUN_141166770(local_178);
  FUN_141166770(local_138);
  FUN_141166770(local_198);
  FUN_141166770(local_158);
  dVar35 = DAT_1454152a0;
  local_2d8 = *(double *)(param_1 + 0x1930);
  auVar38 = ZEXT864((ulonglong)local_2d8);
  local_2c8 = *(uint *)(param_1 + 0x1938);
  auVar31._8_8_ = 0;
  auVar31._0_8_ = *(ulonglong *)(param_1 + 0x1940);
  local_238 = *param_8;
  if (local_238 != 0) {
    local_2c4 = *(uint *)(param_2 + 3);
    dVar21 = (DAT_145415638 / param_3) * param_6;
    auVar45 = ZEXT864((ulonglong)dVar21);
    auVar39 = ZEXT864((ulonglong)DAT_145415208);
    dVar36 = dVar21 * DAT_145415208;
    iVar20 = (int)dVar36;
    auVar43 = ZEXT864((ulonglong)(dVar21 * DAT_1454154c0));
    uVar19 = (uint)(dVar21 * (double)(int)(param_5 - 1) +
                   (double)((int)(param_6 * (double)param_4 * DAT_145414e88 + DAT_1454152a0) + 1));
    uVar16 = *(uint *)(local_238 + -4 + (longlong)param_9 * 4);
    uVar9 = (uVar16 - 1) + (int)dVar21;
    if ((int)uVar9 <= (int)local_2c4) {
      uVar16 = uVar9;
    }
    iVar3 = (param_9 - param_5) + 1;
    local_2d0 = (int)(iVar3 + (iVar3 >> 0x1f & 3U)) >> 2;
    auVar26 = vpcmpgtd_avx(ZEXT416(param_7),ZEXT416(1));
    auVar26 = vshufps_avx(auVar26,auVar26,0);
    auVar23._8_8_ = 0;
    auVar23._0_8_ = DAT_145414da0;
    auVar31 = vblendvpd_avx(auVar23,auVar31,auVar26);
    local_250 = auVar31._0_8_;
    *(undefined4 *)(param_1 + 0x1938) = 0;
    local_2c0 = uVar16;
    local_298 = uVar19;
    local_294 = iVar20;
    local_228 = dVar36;
    pdVar4 = (double *)FUN_1411667d0(local_270,*(undefined4 *)(param_2 + 3),0);
    local_2b8 = pdVar4;
    if (pdVar4 != (double *)0x0) {
      if (*(int *)(param_1 + 0x16f8) == 4) {
        FUN_141166570(param_1 + 0x1700,*param_2,pdVar4,local_258);
        FUN_141166570(param_1 + 0x1738,pdVar4,pdVar4,local_258);
        FUN_141166570(param_1 + 6000,pdVar4,pdVar4,local_258);
        lVar5 = param_1 + 0x17a8;
        pdVar8 = pdVar4;
LAB_14118d5ad:
        FUN_141166570(lVar5,pdVar8,pdVar4,local_258);
      }
      else if (*(int *)(param_1 + 0x16f8) == 2) {
        lVar5 = param_1 + 0x1700;
        pdVar8 = (double *)*param_2;
        goto LAB_14118d5ad;
      }
      lVar5 = FUN_141166a30(local_290,local_270);
      if (lVar5 != 0) {
        FUN_141167080(local_290);
        FUN_141166570(param_1 + 0x17e0,lVar5,lVar5,local_278);
        local_2bc = uVar16 - uVar19;
        dVar21 = auVar43._0_8_;
        if (local_278 < (int)((int)dVar21 * local_2d0 + -1 + uVar19)) {
          local_2d0 = local_2d0 + -1;
        }
        if (local_2d0 < 1) {
          lVar6 = FUN_141169190(param_1 + 0x1948,1,1);
          if (lVar6 != 0) {
            **(undefined1 **)(param_1 + 0x1948) = 0;
            *(undefined8 *)(param_1 + 0x1930) =
                 *(undefined8 *)(lVar5 + -8 + (longlong)(int)local_2c4 * 8);
            *(double *)(param_1 + 0x1940) = local_250;
            *(uint *)(param_1 + 0x1938) = local_2c8;
            uVar12 = 1;
            goto LAB_14118dbd3;
          }
        }
        else {
          local_230 = FUN_141169190(param_1 + 0x1948,local_2d0 << 4,1);
          if (local_230 != 0) {
            local_2b0 = 0;
            auVar37 = ZEXT1664(ZEXT816(0));
            uVar16 = 0;
            if (local_2d0 < 2) {
LAB_14118da11:
              *(uint *)(param_1 + 0x1938) = uVar16;
              if (1 < (int)param_7) {
                uVar16 = local_2c8;
              }
              FUN_141166570(param_1 + 0x1818,pdVar4,pdVar4,local_258);
              FUN_141166570(param_1 + 0x1850,pdVar4,pdVar4,local_258);
              FUN_141166570(param_1 + 0x1888,pdVar4,pdVar4,local_258);
              FUN_141166570(param_1 + 0x18c0,pdVar4,pdVar4,local_258);
              pdVar4 = (double *)FUN_141166a30(local_290,local_270);
              local_2b8 = pdVar4;
              if (pdVar4 != (double *)0x0) {
                FUN_141167080(local_290);
                FUN_141166570(param_1 + 0x18f8,pdVar4,pdVar4);
                lVar5 = (longlong)(int)(local_2c4 - 1);
                *(double *)(param_1 + 0x1930) = pdVar4[lVar5];
                if (3 < lVar5) {
                  pdVar8 = pdVar4 + lVar5 + -2;
                  lVar6 = (lVar5 - 4U >> 2) + 1;
                  lVar5 = lVar5 + lVar6 * -4;
                  do {
                    pdVar8[2] = pdVar8[2] - pdVar8[1];
                    pdVar8[1] = pdVar8[1] - *pdVar8;
                    *pdVar8 = *pdVar8 - pdVar8[-1];
                    pdVar8[-1] = pdVar8[-1] - pdVar8[-2];
                    pdVar8 = pdVar8 + -4;
                    lVar6 = lVar6 + -1;
                  } while (lVar6 != 0);
                }
                for (; 0 < lVar5; lVar5 = lVar5 + -1) {
                  pdVar4[lVar5] = pdVar4[lVar5] - pdVar4[lVar5 + -1];
                }
                *pdVar4 = *pdVar4 - auVar38._0_8_;
                if (((int)uVar16 < (int)uVar19) && ((int)(local_2c0 - uVar16) <= local_278)) {
                  lVar5 = FUN_141166bc0(local_198,
                                        pdVar4 + ((longlong)(int)uVar19 - (longlong)(int)uVar16) +
                                                 -1,local_2bc + 1);
                  if ((lVar5 == 0) ||
                     (pdVar4 = (double *)FUN_141166a30(local_290,local_198), local_2b8 = pdVar4,
                     pdVar4 == (double *)0x0)) goto LAB_14118dbd1;
                }
                else if (((int)((local_2c0 - uVar16) + -1) <= local_278) &&
                        ((int)uVar19 < (int)uVar16)) {
                  lVar5 = FUN_141166bc0(local_198,pdVar4);
                  if ((lVar5 == 0) ||
                     ((local_278 < (int)local_2bc &&
                      (pdVar4 = (double *)FUN_1411667d0(local_290,local_2bc,0), local_2b8 = pdVar4,
                      pdVar4 == (double *)0x0)))) goto LAB_14118dbd1;
                  iVar3 = (uVar16 - uVar19) + 1;
                  if (0 < iVar3) {
                    FUN_142db0ac0(pdVar4,0,(longlong)iVar3 << 3);
                  }
                  pdVar8 = pdVar4 + (longlong)(int)(uVar16 - uVar19) + 1;
                  iVar3 = 0;
                  if (0 < local_180) {
                    lVar5 = lVar5 - (longlong)pdVar8;
                    do {
                      *pdVar8 = *(double *)(lVar5 + (longlong)pdVar8);
                      iVar3 = iVar3 + 1;
                      pdVar8 = pdVar8 + 1;
                    } while (iVar3 < local_180);
                  }
                }
                piVar11 = (int *)(longlong)(int)param_5;
                lVar5 = 0;
                local_2e0 = 0;
                local_2b0 = 10;
                local_2bc = 0xb;
                local_2c0 = 0xc;
                local_2c4 = 0xd;
                local_2c8 = 0xe;
                local_2cc = 0xf;
                local_2d8 = local_250;
                local_2a0 = piVar11;
                do {
                  lVar6 = 0;
                  local_218[0] = 0.0;
                  local_218[1] = 0.0;
                  local_218[2] = 0.0;
                  local_218[3] = 0.0;
                  local_218[4] = 0.0;
                  local_218[5] = 0.0;
                  uStack_1e8 = 0;
                  uStack_1e0 = 0;
                  local_1d8 = 0;
                  uStack_1d0 = 0;
                  uStack_1c8 = 0;
                  uStack_1c0 = 0;
                  local_1b8 = 0;
                  uStack_1b0 = 0;
                  uStack_1a8 = 0;
                  uStack_1a0 = 0;
                  uVar18 = (ulonglong)piVar11 & 0xffffffff;
                  do {
                    uVar13 = 0;
                    if ((int)local_240[3] < (int)uVar18) goto LAB_14118e102;
                    dVar21 = (double)(int)(*(int *)(local_238 + -4 +
                                                   ((longlong)local_2a0 + lVar5 * 4 + lVar6) * 4) -
                                          local_298);
                    uVar14 = uVar13;
                    do {
                      iVar3 = (int)((double)(int)uVar13 * dVar36 + dVar21 + dVar35);
                      if (local_278 < iVar3 + iVar20) goto LAB_14118e102;
                      auVar37 = ZEXT1664(auVar37._0_16_);
                      lVar7 = FUN_141166c30(local_158,pdVar4 + iVar3,iVar20);
                      if (lVar7 == 0) goto LAB_14118dbd1;
                      dVar22 = (double)FUN_141167280(local_158);
                      local_218[uVar14 + lVar6 * 4] = dVar22;
                      uVar16 = (int)uVar13 + 1;
                      uVar13 = (ulonglong)uVar16;
                      uVar14 = uVar14 + 1;
                    } while ((int)uVar16 < 4);
                    uVar18 = (ulonglong)((int)uVar18 + 1);
                    lVar6 = lVar6 + 1;
                  } while (lVar6 < 4);
                  iVar20 = 0;
                  auVar38 = ZEXT864(uStack_1a0);
                  auVar39 = ZEXT864(uStack_1a8);
                  auVar43 = ZEXT864(uStack_1b0);
                  auVar41 = ZEXT864(local_1b8);
                  auVar40 = ZEXT864(uStack_1c0);
                  auVar34 = ZEXT864(uStack_1c8);
                  auVar42 = ZEXT864(uStack_1d0);
                  auVar44 = ZEXT864(local_1d8);
                  auVar46 = ZEXT864(uStack_1e0);
                  auVar45 = ZEXT864(uStack_1e8);
                  do {
                    auVar26._8_8_ = 0;
                    auVar26._0_8_ = local_218[1];
                    auVar32._8_8_ = 0;
                    auVar32._0_8_ = local_218[0];
                    auVar31 = vcmpsd_avx(auVar32,auVar26,1);
                    auVar31 = vblendvpd_avx(auVar32,auVar26,auVar31);
                    uVar16 = (uint)(local_218[0] < local_218[1]);
                    auVar27._8_8_ = 0;
                    auVar27._0_8_ = local_218[2];
                    if (auVar31._0_8_ < local_218[2]) {
                      uVar16 = 2;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar27,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar27,auVar26);
                    auVar28._8_8_ = 0;
                    auVar28._0_8_ = local_218[3];
                    if (auVar31._0_8_ < local_218[3]) {
                      uVar16 = 3;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar28,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar28,auVar26);
                    auVar24._8_8_ = 0;
                    auVar24._0_8_ = local_218[4];
                    if (auVar31._0_8_ < local_218[4]) {
                      uVar16 = 4;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar24,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar24,auVar26);
                    auVar29._8_8_ = 0;
                    auVar29._0_8_ = local_218[5];
                    if (auVar31._0_8_ < local_218[5]) {
                      uVar16 = 5;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar29,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar29,auVar26);
                    if (auVar31._0_8_ < auVar45._0_8_) {
                      uVar16 = 6;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar45._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar45._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar46._0_8_) {
                      uVar16 = 7;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar46._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar46._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar44._0_8_) {
                      uVar16 = 8;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar44._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar44._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar42._0_8_) {
                      uVar16 = 9;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar42._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar42._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar34._0_8_) {
                      uVar16 = local_2b0;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar34._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar34._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar40._0_8_) {
                      uVar16 = local_2bc;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar40._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar40._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar41._0_8_) {
                      uVar16 = local_2c0;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar41._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar41._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar43._0_8_) {
                      uVar16 = local_2c4;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar43._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar43._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar39._0_8_) {
                      uVar16 = local_2c8;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar39._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar39._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar38._0_8_) {
                      uVar16 = local_2cc;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar38._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar38._0_16_,auVar26);
                    auVar30._0_8_ = -(ulonglong)(iVar20 == 0);
                    auVar30._8_8_ = 0xffffffffffffffff;
                    auVar26 = vblendvpd_avx(auVar37._0_16_,auVar31,auVar30);
                    auVar37 = ZEXT1664(auVar26);
                    auVar25._8_8_ = 0;
                    auVar25._0_8_ = local_2d8;
                    dVar21 = auVar31._0_8_;
                    if (((DAT_145414da0 < dVar21) && (auVar26._0_8_ * dVar35 < dVar21)) &&
                       (local_2d8 * DAT_143538390 < dVar21)) {
                      *(undefined1 *)((int)(uVar16 + local_2e0) + local_230) = 1;
                      local_218[(int)uVar16] = 0.0;
                      auVar38 = ZEXT864(uStack_1a0);
                      auVar39 = ZEXT864(uStack_1a8);
                      auVar43 = ZEXT864(uStack_1b0);
                      auVar41 = ZEXT864(local_1b8);
                      auVar40 = ZEXT864(uStack_1c0);
                      auVar34 = ZEXT864(uStack_1c8);
                      auVar42 = ZEXT864(uStack_1d0);
                      auVar44 = ZEXT864(local_1d8);
                      auVar46 = ZEXT864(uStack_1e0);
                      auVar45 = ZEXT864(uStack_1e8);
                    }
                    iVar20 = iVar20 + 1;
                  } while (iVar20 < 0xc);
                  auVar31 = vmaxsd_avx(auVar26,auVar25);
                  local_2d8 = auVar31._0_8_;
                  param_5 = param_5 + 4;
                  piVar11 = (int *)(ulonglong)param_5;
                  local_2e0 = local_2e0 + 0x10;
                  lVar5 = lVar5 + 1;
                  pdVar4 = local_2b8;
                  iVar20 = local_294;
                  dVar36 = local_228;
                } while (lVar5 < local_2d0);
LAB_14118e102:
                *(double *)(local_2a8 + 0x1940) = local_2d8;
                uVar12 = 1;
                goto LAB_14118dbd3;
              }
            }
            else {
              uVar9 = 0;
              local_2cc = 0;
              local_118[0] = 0;
              local_118[1] = 0;
              local_118[2] = 0;
              local_118[3] = 0;
              local_118[4] = 0;
              local_118[5] = 0;
              local_118[6] = 0;
              local_118[7] = 0;
              iVar3 = (local_2d0 + -1) * (int)dVar21;
              auVar37 = ZEXT1664(ZEXT816(0));
              auVar38 = ZEXT1664(auVar39._0_16_);
              auVar39 = ZEXT1664(ZEXT816(0));
              auVar43 = ZEXT1664(auVar43._0_16_);
              auVar45 = ZEXT1664(auVar45._0_16_);
              lVar6 = FUN_1411667d0(local_178,iVar3,1);
              if (lVar6 != 0) {
                local_248 = (longlong)(int)uVar19;
                local_2e8 = FUN_141166bc0(local_178,
                                          lVar5 + ((longlong)(int)auVar43._0_8_ + -1 + local_248) *
                                                  8,iVar3);
                if (local_2e8 != 0) {
                  local_2e0 = 0;
                  piVar11 = local_118;
                  auVar41 = ZEXT864(DAT_143537ec0);
                  do {
                    if ((0 < local_2e0) && (piVar11[-1] <= local_160)) {
                      *(undefined8 *)(local_2e8 + -8 + (longlong)piVar11[-1] * 8) = 0;
                    }
                    local_2a0 = piVar11;
                    auVar40._0_8_ = (double)FUN_1411672c0(local_178,local_118 + local_2e0,1);
                    auVar40._8_56_ = extraout_var;
                    auVar34 = ZEXT1664(auVar40._0_16_);
                    iVar3 = 0xffff;
                    dVar21 = auVar43._0_8_;
                    if (dVar21 != auVar37._0_8_) {
                      iVar3 = (int)((double)*piVar11 / dVar21);
                    }
                    iVar3 = (iVar3 + 1) * (int)dVar21;
                    dVar22 = *(double *)(lVar5 + (longlong)(local_278 + -1) * 8);
                    lVar6 = (longlong)(int)(iVar3 + uVar19);
                    if ((int)(iVar3 + uVar19 + -1) <= local_278) {
                      dVar22 = *(double *)(lVar5 + -0x10 + lVar6 * 8);
                    }
                    if ((auVar40._0_8_ * DAT_1454151e0 < dVar22) &&
                       (iVar3 < local_160 - (int)dVar21)) {
                      lVar7 = FUN_141166c30(local_138);
                      if (lVar7 == 0) goto LAB_14118dbd1;
                      dVar21 = *(double *)(lVar5 + -8 + lVar6 * 8);
                      if (((dVar22 <= dVar21) &&
                          (dVar22 = *(double *)(lVar5 + lVar6 * 8), dVar21 <= dVar22)) &&
                         (bVar2 = *(double *)(lVar5 + 8 + lVar6 * 8) < dVar22, !bVar2)) {
                        iVar15 = 0;
                        iVar10 = 0;
                        if (0 < iVar20) {
                          pdVar4 = (double *)
                                   (lVar5 + ((longlong)iVar3 + -2 + (longlong)(int)uVar19) * 8);
                          pdVar8 = pdVar4;
                          do {
                            iVar15 = iVar15 + 1;
                            if (bVar2) {
                              dVar21 = pdVar4[-2] - pdVar4[-1];
                            }
                            else {
                              dVar21 = pdVar8[2] - pdVar8[1];
                            }
                            if (dVar21 < auVar37._0_8_) {
                              if (bVar2) {
                                iVar15 = -iVar15;
                              }
                              uVar9 = iVar15 + iVar3;
                              auVar39 = ZEXT864(*(ulonglong *)
                                                 (lVar5 + -0x10 +
                                                 (longlong)(int)(uVar9 + uVar19) * 8));
                              break;
                            }
                            iVar10 = iVar10 + 1;
                            pdVar4 = pdVar4 + -1;
                            pdVar8 = pdVar8 + 1;
                          } while (iVar10 < iVar20);
                        }
                        if (auVar34._0_8_ * DAT_143538398 < auVar39._0_8_) {
                          dVar21 = (double)FUN_141167340();
                          dVar22 = auVar34._0_8_;
                          bVar2 = false;
                          iVar15 = 0;
                          iVar17 = 0;
                          iVar10 = (int)(auVar45._0_8_ * DAT_143538128);
                          if (0 < iVar10) {
                            lVar6 = 0;
                            pdVar4 = (double *)
                                     (lVar5 + ((longlong)(int)uVar19 + -3 + (longlong)(int)uVar9) *
                                              8);
                            uVar16 = uVar9;
                            do {
                              uVar16 = uVar16 - 1;
                              lVar6 = lVar6 + 1;
                              pdVar4 = pdVar4 + -1;
                              dVar1 = *(double *)
                                       (lVar5 + -0x10 + (((int)uVar9 - lVar6) + local_248) * 8);
                              if (bVar2) {
                                dVar33 = auVar34._0_8_;
                                if (dVar1 <= dVar33 * auVar41._0_8_) {
                                  iVar15 = iVar15 + 1;
                                  if ((int)(dVar36 * auVar38._0_8_) <= iVar15) {
                                    if (dVar33 == auVar37._0_8_) {
                                      uVar16 = 0;
                                      goto LAB_14118da01;
                                    }
                                    if (DAT_1435383a8 < auVar39._0_8_ / dVar33) {
                                      uVar16 = (uint)(((double)iVar3 - (double)(int)local_2cc) +
                                                     dVar35);
                                      auVar38 = ZEXT864((ulonglong)local_2d8);
                                      pdVar4 = local_2b8;
                                      param_1 = local_2a8;
                                      if ((int)uVar19 <= (int)uVar16) {
                                        uVar16 = uVar19;
                                      }
                                      goto LAB_14118da11;
                                    }
                                    break;
                                  }
                                }
                                else {
                                  bVar2 = false;
                                  iVar15 = 0;
                                }
                              }
                              else if ((dVar1 <= *pdVar4) && (dVar1 < (dVar22 + dVar21) * dVar35)) {
                                bVar2 = true;
                                auVar34 = ZEXT864((ulonglong)dVar1);
                                local_2cc = uVar16;
                              }
                              iVar17 = iVar17 + 1;
                            } while (iVar17 < iVar10);
                          }
                        }
                      }
                    }
                    local_2e0 = local_2e0 + 1;
                    piVar11 = local_2a0 + 1;
                    uVar16 = local_2b0;
                    local_2a0 = piVar11;
                  } while (local_2e0 < 8);
LAB_14118da01:
                  auVar38 = ZEXT864((ulonglong)local_2d8);
                  pdVar4 = local_2b8;
                  param_1 = local_2a8;
                  goto LAB_14118da11;
                }
              }
            }
          }
        }
      }
    }
  }
LAB_14118dbd1:
  uVar12 = 0;
LAB_14118dbd3:
  FUN_141166790(local_158);
  FUN_141166790(local_198);
  FUN_141166790(local_138);
  FUN_141166790(local_178);
  FUN_141166790(local_290);
  FUN_141166790(local_270);
  return uVar12;
}


==== FUN_141166a30 @ 141166a30 (prof 1)
