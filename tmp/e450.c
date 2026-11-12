==== FUN_14118e450 @ 14118e450 (prof 1)

/* WARNING: Function: __security_check_cookie replaced with injection: security_check_cookie */

undefined8
FUN_14118e450(longlong param_1,undefined8 *param_2,double param_3,int param_4,uint param_5,
             double param_6,uint param_7,longlong *param_8,int param_9)

{
  double dVar1;
  bool bVar2;
  double *pdVar3;
  longlong lVar4;
  longlong lVar5;
  double *pdVar6;
  longlong lVar7;
  double *pdVar8;
  ulonglong uVar9;
  int iVar10;
  int *piVar11;
  uint uVar12;
  undefined8 uVar13;
  ulonglong uVar14;
  int iVar15;
  uint uVar16;
  ulonglong uVar17;
  uint uVar18;
  int iVar19;
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
  double dVar30;
  undefined1 auVar31 [16];
  undefined1 auVar32 [16];
  double dVar33;
  undefined1 auVar34 [64];
  double dVar35;
  undefined1 auVar36 [64];
  undefined1 auVar37 [64];
  undefined1 auVar38 [64];
  undefined1 auVar39 [64];
  undefined1 in_ZMM12 [64];
  undefined1 auVar40 [64];
  undefined1 auVar41 [64];
  undefined1 in_ZMM13 [64];
  undefined1 auVar42 [64];
  undefined1 auVar43 [64];
  undefined1 auVar44 [64];
  undefined1 auVar45 [64];
  undefined1 auVar46 [64];
  undefined1 auStack_2d8 [32];
  double local_2b8;
  uint local_2b0;
  int local_2ac;
  double local_2a8;
  int local_2a0;
  uint local_29c;
  uint local_298;
  uint local_294;
  uint local_290;
  uint local_28c;
  int *local_288;
  double local_280;
  longlong local_278;
  undefined1 local_270 [24];
  int local_258;
  uint local_250;
  int local_24c;
  double *local_248;
  longlong local_240;
  double local_238 [5];
  ulonglong uStack_210;
  ulonglong uStack_208;
  ulonglong uStack_200;
  ulonglong local_1f8;
  ulonglong uStack_1f0;
  ulonglong uStack_1e8;
  ulonglong uStack_1e0;
  ulonglong local_1d8;
  ulonglong uStack_1d0;
  ulonglong uStack_1c8;
  ulonglong uStack_1c0;
  longlong *local_1b8;
  longlong local_1b0;
  longlong local_1a8;
  double local_1a0;
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
  local_f8 = DAT_1459ca6c0 ^ (ulonglong)auStack_2d8;
  local_1b8 = param_8;
  local_278 = param_1;
  FUN_141166770(local_270);
  FUN_141166770(local_138);
  FUN_141166770(local_178);
  FUN_141166770(local_198);
  FUN_141166770(local_158);
  local_280 = *(double *)(param_1 + 0x16c0);
  auVar37 = ZEXT864((ulonglong)local_280);
  local_294 = *(uint *)(param_1 + 0x16c8);
  auVar31._8_8_ = 0;
  auVar31._0_8_ = *(ulonglong *)(param_1 + 0x16d0);
  local_1b0 = *param_8;
  if (local_1b0 != 0) {
    local_290 = *(uint *)(param_2 + 3);
    dVar30 = (DAT_145415638 / param_3) * param_6;
    auVar45 = ZEXT864((ulonglong)dVar30);
    auVar39 = ZEXT864((ulonglong)DAT_145415208);
    dVar35 = dVar30 * DAT_145415208;
    iVar20 = (int)dVar35;
    auVar43 = ZEXT864((ulonglong)(dVar30 * DAT_1454154c0));
    uVar12 = (uint)(dVar30 * (double)(int)(param_5 - 1) +
                   (double)((int)(param_6 * (double)param_4 * DAT_145414e88 + DAT_1454152a0) + 1));
    iVar15 = *(int *)(local_1b0 + -4 + (longlong)param_9 * 4);
    iVar10 = iVar15 + -1 + (int)dVar30;
    if (iVar10 <= (int)local_290) {
      iVar15 = iVar10;
    }
    iVar10 = (param_9 - param_5) + 1;
    iVar10 = (int)((iVar10 >> 0x1f & 3U) + iVar10) >> 2;
    auVar26 = vpcmpgtd_avx(ZEXT416(param_7),ZEXT416(1));
    auVar26 = vshufps_avx(auVar26,auVar26,0);
    auVar23._8_8_ = 0;
    auVar23._0_8_ = DAT_145414da0;
    auVar31 = vblendvpd_avx(auVar23,auVar31,auVar26);
    local_2b8 = auVar31._0_8_;
    dVar30 = DAT_1454152a0;
    local_2b0 = uVar12;
    local_2ac = iVar15;
    local_2a0 = iVar10;
    local_24c = iVar20;
    local_1a0 = dVar35;
    pdVar3 = (double *)FUN_1411667d0(local_270,local_290,0);
    lVar5 = local_278;
    local_248 = pdVar3;
    if (pdVar3 != (double *)0x0) {
      if (*(int *)(local_278 + 0x15a0) == 4) {
        FUN_141166570(local_278 + 0x15a8,*param_2,pdVar3,local_258);
        FUN_141166570(lVar5 + 0x15e0,pdVar3,pdVar3,local_258);
        FUN_141166570(lVar5 + 0x1618,pdVar3,pdVar3,local_258);
        lVar4 = lVar5 + 0x1650;
        pdVar6 = pdVar3;
LAB_14118e6d2:
        FUN_141166570(lVar4,pdVar6,pdVar3,local_258);
      }
      else if (*(int *)(local_278 + 0x15a0) == 2) {
        lVar4 = local_278 + 0x15a8;
        pdVar6 = (double *)*param_2;
        goto LAB_14118e6d2;
      }
      FUN_141167080(local_270);
      FUN_141166570(lVar5 + 0x1688,pdVar3,pdVar3);
      lVar5 = local_278;
      local_2a8 = (double)CONCAT44(local_2a8._4_4_,iVar15 - uVar12);
      if (local_258 < (int)((int)auVar43._0_8_ * iVar10 + -1 + uVar12)) {
        iVar10 = iVar10 + -1;
        local_2a0 = iVar10;
      }
      if (iVar10 < 1) {
        lVar4 = FUN_141169190(local_278 + 0x16d8,1,1);
        if (lVar4 != 0) {
          *(double *)(lVar5 + 0x16c0) = pdVar3[(longlong)(int)local_290 + -1];
          *(uint *)(lVar5 + 0x16c8) = local_294;
          *(double *)(lVar5 + 0x16d0) = local_2b8;
          uVar13 = 1;
          goto LAB_14118f10d;
        }
      }
      else {
        local_1a8 = FUN_141169190(local_278 + 0x16d8,iVar10 << 4,1);
        if (local_1a8 != 0) {
          uVar18 = 0;
          local_28c = 0;
          local_250 = 2;
          auVar36 = ZEXT1664(ZEXT816(0));
          iVar19 = iVar15 - uVar12;
          uVar16 = 0;
          dVar21 = auVar37._0_8_;
          if (iVar10 < 2) {
LAB_14118eb49:
            *(uint *)(local_278 + 0x16c8) = uVar16;
            if (1 < (int)param_7) {
              uVar16 = local_294;
            }
            lVar5 = (longlong)(int)(local_290 - 1);
            *(double *)(local_278 + 0x16c0) = pdVar3[lVar5];
            if (3 < lVar5) {
              pdVar6 = pdVar3 + lVar5 + -2;
              lVar4 = (lVar5 - 4U >> 2) + 1;
              lVar5 = lVar5 + lVar4 * -4;
              do {
                pdVar6[2] = pdVar6[2] - pdVar6[1];
                pdVar6[1] = pdVar6[1] - *pdVar6;
                *pdVar6 = *pdVar6 - pdVar6[-1];
                pdVar6[-1] = pdVar6[-1] - pdVar6[-2];
                pdVar6 = pdVar6 + -4;
                lVar4 = lVar4 + -1;
              } while (lVar4 != 0);
            }
            for (; 0 < lVar5; lVar5 = lVar5 + -1) {
              pdVar3[lVar5] = pdVar3[lVar5] - pdVar3[lVar5 + -1];
            }
            *pdVar3 = *pdVar3 - dVar21;
            if (((int)uVar16 < (int)uVar12) && ((int)(iVar15 - uVar16) <= local_258)) {
              lVar5 = FUN_141166bc0(local_198,
                                    pdVar3 + ((longlong)(int)uVar12 - (longlong)(int)uVar16) + -1,
                                    iVar19 + 1);
              if ((lVar5 != 0) &&
                 (pdVar3 = (double *)FUN_141166a30(local_270,local_198), local_248 = pdVar3,
                 pdVar3 != (double *)0x0)) {
LAB_14118ed12:
                lVar5 = 0;
                piVar11 = (int *)(longlong)(int)param_5;
                local_240 = (longlong)iVar10;
                local_2ac = 0;
                local_28c = 0xb;
                local_290 = 0xc;
                local_294 = 0xd;
                local_298 = 0xe;
                local_29c = 0xf;
                local_2a8 = local_2b8;
                local_288 = piVar11;
                do {
                  lVar4 = 0;
                  local_238[0] = 0.0;
                  local_238[1] = 0.0;
                  local_238[2] = 0.0;
                  local_238[3] = 0.0;
                  local_238[4] = 0.0;
                  uStack_210 = 0;
                  uStack_208 = 0;
                  uStack_200 = 0;
                  local_1f8 = 0;
                  uStack_1f0 = 0;
                  uStack_1e8 = 0;
                  uStack_1e0 = 0;
                  local_1d8 = 0;
                  uStack_1d0 = 0;
                  uStack_1c8 = 0;
                  uStack_1c0 = 0;
                  uVar9 = (ulonglong)piVar11 & 0xffffffff;
                  do {
                    uVar14 = 0;
                    if ((int)local_1b8[3] < (int)uVar9) goto LAB_14118f0fa;
                    dVar21 = (double)(int)(*(int *)(local_1b0 + -4 +
                                                   ((longlong)local_288 + lVar5 * 4 + lVar4) * 4) -
                                          uVar12);
                    uVar17 = uVar14;
                    do {
                      iVar15 = (int)((double)(int)uVar14 * dVar35 + dVar21 + dVar30);
                      if (local_258 < iVar15 + iVar20) goto LAB_14118f0fa;
                      auVar36 = ZEXT1664(auVar36._0_16_);
                      lVar7 = FUN_141166c30(local_158,pdVar3 + iVar15,iVar20);
                      if (lVar7 == 0) goto LAB_14118f10b;
                      dVar22 = (double)FUN_141167280(local_158);
                      local_238[uVar17 + lVar4 * 4] = dVar22;
                      lVar7 = local_1a8;
                      uVar12 = (int)uVar14 + 1;
                      uVar14 = (ulonglong)uVar12;
                      uVar17 = uVar17 + 1;
                    } while ((int)uVar12 < 4);
                    uVar9 = (ulonglong)((int)uVar9 + 1);
                    lVar4 = lVar4 + 1;
                    uVar12 = local_2b0;
                  } while (lVar4 < 4);
                  iVar15 = 0;
                  auVar37 = ZEXT864(uStack_1c0);
                  auVar39 = ZEXT864(uStack_1c8);
                  auVar43 = ZEXT864(uStack_1d0);
                  auVar38 = ZEXT864(local_1d8);
                  auVar34 = ZEXT864(uStack_1e0);
                  auVar41 = ZEXT864(uStack_1e8);
                  auVar42 = ZEXT864(uStack_1f0);
                  auVar44 = ZEXT864(local_1f8);
                  auVar46 = ZEXT864(uStack_200);
                  auVar40 = ZEXT864(uStack_208);
                  auVar45 = ZEXT864(uStack_210);
                  do {
                    auVar26._8_8_ = 0;
                    auVar26._0_8_ = local_238[1];
                    auVar32._8_8_ = 0;
                    auVar32._0_8_ = local_238[0];
                    auVar31 = vcmpsd_avx(auVar32,auVar26,1);
                    auVar31 = vblendvpd_avx(auVar32,auVar26,auVar31);
                    auVar27._8_8_ = 0;
                    auVar27._0_8_ = local_238[2];
                    uVar16 = (uint)(local_238[0] < local_238[1]);
                    if (auVar31._0_8_ < local_238[2]) {
                      uVar16 = local_250;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar27,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar27,auVar26);
                    auVar28._8_8_ = 0;
                    auVar28._0_8_ = local_238[3];
                    if (auVar31._0_8_ < local_238[3]) {
                      uVar16 = 3;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar28,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar28,auVar26);
                    auVar24._8_8_ = 0;
                    auVar24._0_8_ = local_238[4];
                    if (auVar31._0_8_ < local_238[4]) {
                      uVar16 = 4;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar24,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar24,auVar26);
                    if (auVar31._0_8_ < auVar45._0_8_) {
                      uVar16 = 5;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar45._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar45._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar40._0_8_) {
                      uVar16 = 6;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar40._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar40._0_16_,auVar26);
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
                    if (auVar31._0_8_ < auVar41._0_8_) {
                      uVar16 = 10;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar41._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar41._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar34._0_8_) {
                      uVar16 = local_28c;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar34._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar34._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar38._0_8_) {
                      uVar16 = local_290;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar38._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar38._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar43._0_8_) {
                      uVar16 = local_294;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar43._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar43._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar39._0_8_) {
                      uVar16 = local_298;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar39._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar39._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar37._0_8_) {
                      uVar16 = local_29c;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar37._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar37._0_16_,auVar26);
                    auVar29._0_8_ = -(ulonglong)(iVar15 == 0);
                    auVar29._8_8_ = 0xffffffffffffffff;
                    auVar26 = vblendvpd_avx(auVar36._0_16_,auVar31,auVar29);
                    auVar36 = ZEXT1664(auVar26);
                    auVar25._8_8_ = 0;
                    auVar25._0_8_ = local_2a8;
                    dVar30 = auVar31._0_8_;
                    if (((DAT_145414da0 < dVar30) && (auVar26._0_8_ * DAT_1435383a0 < dVar30)) &&
                       (local_2a8 * DAT_143538390 < dVar30)) {
                      *(undefined1 *)((int)(uVar16 + local_2ac) + lVar7) = 1;
                      local_238[(int)uVar16] = 0.0;
                      auVar37 = ZEXT864(uStack_1c0);
                      auVar39 = ZEXT864(uStack_1c8);
                      auVar43 = ZEXT864(uStack_1d0);
                      auVar38 = ZEXT864(local_1d8);
                      auVar34 = ZEXT864(uStack_1e0);
                      auVar41 = ZEXT864(uStack_1e8);
                      auVar42 = ZEXT864(uStack_1f0);
                      auVar44 = ZEXT864(local_1f8);
                      auVar46 = ZEXT864(uStack_200);
                      auVar40 = ZEXT864(uStack_208);
                      auVar45 = ZEXT864(uStack_210);
                    }
                    iVar15 = iVar15 + 1;
                  } while (iVar15 < 6);
                  auVar31 = vmaxsd_avx(auVar26,auVar25);
                  local_2a8 = auVar31._0_8_;
                  param_5 = param_5 + 4;
                  piVar11 = (int *)(ulonglong)param_5;
                  local_2ac = local_2ac + 0x10;
                  lVar5 = lVar5 + 1;
                  pdVar3 = local_248;
                  iVar20 = local_24c;
                  dVar35 = local_1a0;
                  dVar30 = DAT_1454152a0;
                } while (lVar5 < local_240);
LAB_14118f0fa:
                *(double *)(local_278 + 0x16d0) = local_2a8;
                uVar13 = 1;
                goto LAB_14118f10d;
              }
            }
            else {
              if ((local_258 < (int)((iVar15 - uVar16) + -1)) || ((int)uVar16 <= (int)uVar12))
              goto LAB_14118ed12;
              lVar5 = FUN_141166bc0(local_198,pdVar3);
              if (lVar5 != 0) {
                if ((local_2a8._0_4_ <= local_258) ||
                   (pdVar3 = (double *)FUN_1411667d0(local_270,(ulonglong)local_2a8 & 0xffffffff,0),
                   local_248 = pdVar3, pdVar3 != (double *)0x0)) {
                  iVar15 = (uVar16 - uVar12) + 1;
                  if (0 < iVar15) {
                    FUN_142db0ac0(pdVar3,0,(longlong)iVar15 << 3);
                  }
                  pdVar6 = pdVar3 + (longlong)(int)(uVar16 - uVar12) + 1;
                  iVar15 = 0;
                  if (0 < local_180) {
                    lVar5 = lVar5 - (longlong)pdVar6;
                    do {
                      *pdVar6 = *(double *)(lVar5 + (longlong)pdVar6);
                      iVar15 = iVar15 + 1;
                      pdVar6 = pdVar6 + 1;
                    } while (iVar15 < local_180);
                  }
                  goto LAB_14118ed12;
                }
              }
            }
          }
          else {
            local_298 = 0;
            local_118[0] = 0;
            local_118[1] = 0;
            local_118[2] = 0;
            local_118[3] = 0;
            local_118[4] = 0;
            local_118[5] = 0;
            local_118[6] = 0;
            local_118[7] = 0;
            iVar19 = (int)auVar43._0_8_;
            iVar15 = (iVar10 + -1) * iVar19;
            auVar36 = ZEXT1664(ZEXT816(0));
            auVar37 = ZEXT1664(auVar39._0_16_);
            auVar39 = ZEXT1664(ZEXT816(0));
            auVar43 = ZEXT1664(auVar43._0_16_);
            auVar45 = ZEXT1664(auVar45._0_16_);
            lVar5 = FUN_1411667d0(local_178,iVar15,1);
            if (lVar5 != 0) {
              lVar5 = (longlong)(int)uVar12;
              local_240 = FUN_141166bc0(local_178,pdVar3 + (longlong)iVar19 + -1 + lVar5,iVar15);
              if (local_240 != 0) {
                local_29c = 0;
                local_288 = local_118;
                auVar40 = ZEXT864(DAT_143537ec0);
                do {
                  if ((0 < (int)local_29c) && (local_288[-1] <= local_160)) {
                    *(undefined8 *)(local_240 + -8 + (longlong)local_288[-1] * 8) = 0;
                  }
                  auVar38._0_8_ = (double)FUN_1411672c0(local_178,local_118 + (int)local_29c,1);
                  uVar16 = local_2b0;
                  auVar38._8_56_ = extraout_var;
                  auVar34 = ZEXT1664(auVar38._0_16_);
                  iVar15 = 0xffff;
                  if (auVar43._0_8_ != auVar36._0_8_) {
                    iVar15 = (int)((double)*local_288 / auVar43._0_8_);
                  }
                  iVar10 = (iVar15 + 1) * iVar19;
                  dVar21 = pdVar3[local_258 + -1];
                  iVar15 = iVar10 + uVar12;
                  if (iVar15 + -1 <= local_258) {
                    dVar21 = pdVar3[(longlong)iVar15 + -2];
                  }
                  if ((auVar38._0_8_ * DAT_1454151e0 < dVar21) && (iVar10 < local_160 - iVar19)) {
                    lVar4 = FUN_141166c30(local_138);
                    if (lVar4 == 0) goto LAB_14118f10b;
                    dVar22 = pdVar3[(longlong)iVar15 + -1];
                    dVar1 = pdVar3[iVar15];
                    if (dVar21 <= dVar22) {
                      if ((dVar22 <= dVar1) && (dVar1 <= pdVar3[(longlong)iVar15 + 1])) {
                        bVar2 = true;
                        goto LAB_14118e95a;
                      }
                    }
                    else if ((dVar1 < dVar22) && (pdVar3[(longlong)iVar15 + 1] < dVar1)) {
                      bVar2 = false;
LAB_14118e95a:
                      iVar15 = 0;
                      iVar19 = 0;
                      if (0 < iVar20) {
                        pdVar6 = pdVar3 + (longlong)iVar10 + -2 + lVar5;
                        pdVar8 = pdVar3 + (longlong)iVar10 + -2 + lVar5;
                        do {
                          iVar15 = iVar15 + 1;
                          if (bVar2) {
                            dVar21 = pdVar8[2] - pdVar8[1];
                          }
                          else {
                            dVar21 = pdVar6[-2] - pdVar6[-1];
                          }
                          if (dVar21 < auVar36._0_8_) {
                            if (!bVar2) {
                              iVar15 = -iVar15;
                            }
                            uVar18 = iVar15 + iVar10;
                            auVar39 = ZEXT864((ulonglong)
                                              pdVar3[(longlong)(int)(uVar18 + uVar16) + -2]);
                            break;
                          }
                          iVar19 = iVar19 + 1;
                          pdVar6 = pdVar6 + -1;
                          pdVar8 = pdVar8 + 1;
                        } while (iVar19 < iVar20);
                      }
                      if (auVar34._0_8_ * DAT_143538398 < auVar39._0_8_) {
                        dVar21 = (double)FUN_141167340(local_138);
                        dVar22 = auVar34._0_8_;
                        bVar2 = false;
                        uVar9 = 0;
                        iVar15 = (int)(auVar45._0_8_ * DAT_143538128);
                        if (iVar15 < 1) {
                          iVar19 = (int)auVar43._0_8_;
                          goto LAB_14118ea97;
                        }
                        pdVar6 = pdVar3 + (longlong)(int)uVar18 + -3 + lVar5;
                        uVar14 = uVar9;
                        uVar17 = uVar9;
                        uVar12 = uVar18;
                        do {
                          uVar12 = uVar12 - 1;
                          uVar17 = uVar17 + 1;
                          pdVar6 = pdVar6 + -1;
                          dVar1 = pdVar3[((longlong)(int)uVar18 - uVar17) + lVar5 + -2];
                          if (bVar2) {
                            dVar33 = auVar34._0_8_;
                            if (dVar1 <= dVar33 * auVar40._0_8_) {
                              uVar16 = (int)uVar9 + 1;
                              uVar9 = (ulonglong)uVar16;
                              if ((int)(dVar35 * auVar37._0_8_) <= (int)uVar16) {
                                if (dVar33 == auVar36._0_8_) {
                                  uVar16 = 0;
                                  goto LAB_14118eb35;
                                }
                                if (DAT_1435383a8 < auVar39._0_8_ / dVar33) {
                                  uVar16 = (uint)(((double)iVar10 - (double)(int)local_298) + dVar30
                                                 );
                                  iVar10 = local_2a0;
                                  iVar19 = local_2a8._0_4_;
                                  uVar12 = local_2b0;
                                  iVar15 = local_2ac;
                                  dVar21 = local_280;
                                  if ((int)local_2b0 <= (int)uVar16) {
                                    uVar16 = local_2b0;
                                  }
                                  goto LAB_14118eb49;
                                }
                                break;
                              }
                            }
                            else {
                              bVar2 = false;
                              uVar9 = 0;
                            }
                          }
                          else if ((dVar1 <= *pdVar6) && (dVar1 < (dVar22 + dVar21) * dVar30)) {
                            bVar2 = true;
                            auVar34 = ZEXT864((ulonglong)dVar1);
                            local_298 = uVar12;
                          }
                          uVar16 = (int)uVar14 + 1;
                          uVar14 = (ulonglong)uVar16;
                        } while ((int)uVar16 < iVar15);
                      }
                    }
                    iVar19 = (int)auVar43._0_8_;
                  }
LAB_14118ea97:
                  local_29c = local_29c + 1;
                  local_288 = local_288 + 1;
                  uVar12 = local_2b0;
                  uVar16 = local_28c;
                } while ((int)local_29c < 8);
LAB_14118eb35:
                iVar10 = local_2a0;
                iVar19 = local_2a8._0_4_;
                uVar12 = local_2b0;
                iVar15 = local_2ac;
                dVar21 = local_280;
                goto LAB_14118eb49;
              }
            }
          }
        }
      }
    }
  }
LAB_14118f10b:
  uVar13 = 0;
LAB_14118f10d:
  FUN_141166790(local_158);
  FUN_141166790(local_198);
  FUN_141166790(local_178);
  FUN_141166790(local_138);
  FUN_141166790(local_270);
  return uVar13;
}


==== FUN_1411c5cb0 @ 1411c5cb0 (prof 1)
