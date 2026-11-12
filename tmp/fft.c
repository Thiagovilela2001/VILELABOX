==== FUN_1411645f0 @ 1411645f0 (prof 0)

undefined8 FUN_1411645f0(int *param_1,ulonglong param_2)

{
  uint uVar1;
  int iVar2;
  int iVar3;
  ulonglong uVar4;
  longlong lVar5;
  ulonglong uVar6;
  ulonglong uVar7;
  int iVar8;
  uint uVar9;
  undefined8 uVar10;
  double dVar11;
  
  uVar7 = 0;
  if (*(longlong *)(param_1 + 6) != 0) {
    thunk_FUN_1427bd480();
    param_1[6] = 0;
    param_1[7] = 0;
  }
  if (*(longlong *)(param_1 + 8) != 0) {
    thunk_FUN_1427bd480();
    param_1[8] = 0;
    param_1[9] = 0;
  }
  param_1[10] = 0;
  if (*(longlong *)(param_1 + 0xc) != 0) {
    thunk_FUN_1427bd480();
    param_1[0xc] = 0;
    param_1[0xd] = 0;
  }
  if (*(longlong *)(param_1 + 2) != 0) {
    thunk_FUN_1427bd480();
    param_1[2] = 0;
    param_1[3] = 0;
  }
  *param_1 = 0;
  iVar2 = (int)param_2;
  if (7 < iVar2) {
    uVar4 = param_2 & 0xffffffff;
    uVar6 = uVar7;
    do {
      uVar1 = (uint)uVar4 >> 1;
      uVar4 = (ulonglong)uVar1;
      uVar9 = (uint)uVar6;
      uVar6 = (ulonglong)(uVar9 + 1);
    } while (uVar1 != 0);
    if (1 << (uVar9 & 0x1f) == iVar2) {
      uVar9 = (uint)(param_2 >> 2) & 0x3fffffff;
      *(double *)(param_1 + 4) = DAT_145415378 / (double)iVar2;
      *param_1 = iVar2;
      lVar5 = _malloc_base((longlong)(int)(uVar9 + iVar2) << 3);
      *(longlong *)(param_1 + 0xc) = lVar5;
      if (lVar5 != 0) {
        iVar2 = *param_1;
        uVar6 = uVar7;
        uVar4 = uVar7;
        dVar11 = DAT_1454154f8;
        if (0 < (int)(iVar2 + uVar9)) {
          do {
            uVar10 = FUN_1427a51e0((dVar11 / (double)iVar2) * (double)(int)uVar6);
            uVar1 = (int)uVar6 + 1;
            *(undefined8 *)(uVar4 + *(longlong *)(param_1 + 0xc)) = uVar10;
            iVar2 = *param_1;
            uVar6 = (ulonglong)uVar1;
            uVar4 = uVar4 + 8;
          } while ((int)uVar1 < (int)(iVar2 + uVar9));
        }
        iVar2 = iVar2 / 2;
        lVar5 = _malloc_base((longlong)iVar2 << 2);
        *(longlong *)(param_1 + 6) = lVar5;
        if (lVar5 != 0) {
          lVar5 = _malloc_base((longlong)iVar2 << 2);
          *(longlong *)(param_1 + 8) = lVar5;
          if (lVar5 != 0) {
            iVar3 = *param_1;
            param_1[10] = 0;
            uVar6 = uVar7;
            uVar4 = uVar7;
            if (0 < iVar3) {
              while( true ) {
                uVar9 = (uint)uVar4;
                iVar8 = (int)uVar6;
                if (iVar8 < (int)uVar9) {
                  *(int *)(*(longlong *)(param_1 + 8) + (longlong)param_1[10] * 4) = iVar8;
                  *(uint *)(*(longlong *)(param_1 + 6) + (longlong)param_1[10] * 4) = uVar9;
                  param_1[10] = param_1[10] + 1;
                }
                iVar3 = *param_1;
                uVar6 = (ulonglong)(iVar8 + 1U);
                if (iVar3 <= (int)(iVar8 + 1U)) break;
                iVar3 = iVar2;
                if (iVar2 <= (int)uVar9) {
                  do {
                    uVar9 = (int)uVar4 - iVar3;
                    uVar4 = (ulonglong)uVar9;
                    iVar3 = iVar3 / 2;
                  } while (iVar3 <= (int)uVar9);
                }
                uVar4 = (ulonglong)(uVar9 + iVar3);
              }
            }
            lVar5 = _malloc_base((longlong)iVar3 << 3);
            *(longlong *)(param_1 + 2) = lVar5;
            if (lVar5 != 0) {
              uVar6 = uVar7;
              if (0 < *param_1) {
                do {
                  uVar9 = (int)uVar7 + 1;
                  uVar7 = (ulonglong)uVar9;
                  *(undefined8 *)(uVar6 + *(longlong *)(param_1 + 2)) = 0;
                  uVar6 = uVar6 + 8;
                } while ((int)uVar9 < *param_1);
              }
              return 1;
            }
            thunk_FUN_1427bd480(*(undefined8 *)(param_1 + 8));
            param_1[8] = 0;
            param_1[9] = 0;
          }
          thunk_FUN_1427bd480(*(undefined8 *)(param_1 + 6));
          param_1[6] = 0;
          param_1[7] = 0;
          param_1[10] = 0;
        }
        thunk_FUN_1427bd480(*(undefined8 *)(param_1 + 0xc));
        param_1[0xc] = 0;
        param_1[0xd] = 0;
      }
      *param_1 = 0;
      return 0;
    }
  }
  return 0;
}


==== FUN_141165390 @ 141165390 (prof 0)

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

undefined8 FUN_141165390(uint *param_1)

{
  uint uVar1;
  double *pdVar2;
  undefined1 auVar3 [32];
  undefined1 auVar4 [16];
  uint uVar5;
  undefined8 uVar6;
  int iVar7;
  double *pdVar8;
  longlong lVar9;
  double extraout_XMM0_Qa;
  double extraout_XMM0_Qa_00;
  undefined1 extraout_var [56];
  double dVar10;
  double dVar11;
  double dVar12;
  undefined1 auVar13 [32];
  undefined1 auVar14 [32];
  undefined1 auVar15 [32];
  double dVar16;
  double dVar17;
  int local_e0;
  int iStack_dc;
  int iStack_d8;
  int iStack_d4;
  
  uVar6 = 0;
  uVar1 = *param_1;
  if (7 < (int)uVar1) {
    pdVar2 = *(double **)(param_1 + 2);
    dVar10 = (double)(int)(uVar1 - 1) * DAT_1454152a0;
    iVar7 = 0;
    auVar13._8_8_ = dVar10;
    auVar13._0_8_ = dVar10;
    auVar13._16_8_ = dVar10;
    auVar13._24_8_ = dVar10;
    dVar16 = DAT_145415378;
    dVar17 = DAT_1454152a0;
    if (3 < uVar1) {
      dVar10 = dVar10 + DAT_145415378;
      auVar15._8_8_ = dVar10;
      auVar15._0_8_ = dVar10;
      auVar15._16_8_ = dVar10;
      auVar15._24_8_ = dVar10;
      uVar5 = uVar1 & 0x80000003;
      if ((int)uVar5 < 0) {
        uVar5 = (uVar5 - 1 | 0xfffffffc) + 1;
      }
      local_e0 = (int)_DAT_145416b40;
      iStack_dc = (int)((ulonglong)_DAT_145416b40 >> 0x20);
      iStack_d8 = (int)_UNK_145416b48;
      iStack_d4 = (int)((ulonglong)_UNK_145416b48 >> 0x20);
      pdVar8 = pdVar2;
      do {
        auVar4._0_4_ = iVar7 + local_e0;
        auVar4._4_4_ = iVar7 + iStack_dc;
        auVar4._8_4_ = iVar7 + iStack_d8;
        auVar4._12_4_ = iVar7 + iStack_d4;
        auVar3 = vcvtdq2pd_avx(auVar4);
        auVar3 = vsubpd_avx(auVar3,auVar13);
        auVar14._0_8_ = auVar3._0_8_ * _DAT_143537e00;
        auVar14._8_8_ = auVar3._8_8_ * _UNK_143537e08;
        auVar14._16_8_ = auVar3._16_8_ * _UNK_143537e10;
        auVar14._24_8_ = auVar3._24_8_ * _UNK_143537e18;
        auVar3 = vdivpd_avx(auVar14,auVar15);
        FUN_142608cd0(auVar3._0_8_);
        dVar10 = extraout_var._0_8_ * _UNK_145416f88 + _UNK_145416f88;
        dVar11 = extraout_var._8_8_ * _UNK_145416f90 + _UNK_145416f90;
        dVar12 = extraout_var._16_8_ * _UNK_145416f98 + _UNK_145416f98;
        iVar7 = iVar7 + 4;
        *pdVar8 = extraout_XMM0_Qa * _DAT_145416f80 + _DAT_145416f80;
        pdVar8[1] = dVar10;
        pdVar8[2] = dVar11;
        pdVar8[3] = dVar12;
        pdVar8 = pdVar8 + 4;
      } while (iVar7 < (int)(uVar1 - uVar5));
    }
    for (lVar9 = (longlong)iVar7; lVar9 < (int)uVar1; lVar9 = lVar9 + 1) {
      FUN_1427a2fa0();
      pdVar2[lVar9] = (extraout_XMM0_Qa_00 + dVar16) * dVar17;
    }
    uVar6 = 1;
  }
  return uVar6;
}


==== FUN_1411655f0 @ 1411655f0 (prof 0)

longlong FUN_1411655f0(int *param_1,longlong param_2)

{
  longlong lVar1;
  int iVar2;
  longlong lVar3;
  longlong lVar4;
  double *pdVar5;
  longlong lVar6;
  longlong lVar7;
  
  iVar2 = *param_1;
  lVar3 = (longlong)iVar2;
  if ((7 < iVar2) && (param_2 != 0)) {
    lVar6 = 0;
    lVar4 = lVar3;
    if (3 < lVar3) {
      pdVar5 = (double *)(param_2 + 0x10);
      lVar7 = lVar6;
      do {
        pdVar5[-2] = *(double *)(lVar7 + *(longlong *)(param_1 + 2)) * pdVar5[-2];
        pdVar5[-1] = *(double *)(*(longlong *)(param_1 + 2) + lVar6 * 8) * pdVar5[-1];
        *pdVar5 = *(double *)(*(longlong *)(param_1 + 2) + 8 + lVar6 * 8) * *pdVar5;
        pdVar5[1] = *(double *)(*(longlong *)(param_1 + 2) + 8 + lVar6 * 8) * pdVar5[1];
        pdVar5[2] = *(double *)(*(longlong *)(param_1 + 2) + 0x10 + lVar6 * 8) * pdVar5[2];
        pdVar5[3] = *(double *)(*(longlong *)(param_1 + 2) + 0x10 + lVar6 * 8) * pdVar5[3];
        pdVar5[4] = *(double *)(*(longlong *)(param_1 + 2) + 0x18 + lVar6 * 8) * pdVar5[4];
        lVar4 = *(longlong *)(param_1 + 2);
        lVar1 = lVar6 * 8;
        lVar6 = lVar6 + 4;
        pdVar5[5] = *(double *)(lVar4 + 0x18 + lVar1) * pdVar5[5];
        pdVar5 = pdVar5 + 8;
        lVar7 = lVar7 + 0x20;
      } while (lVar6 < lVar3 + -3);
    }
    if (lVar6 < lVar3) {
      pdVar5 = (double *)(lVar6 * 0x10 + param_2);
      do {
        *pdVar5 = *(double *)(*(longlong *)(param_1 + 2) + lVar6 * 8) * *pdVar5;
        lVar4 = *(longlong *)(param_1 + 2);
        lVar7 = lVar6 * 8;
        lVar6 = lVar6 + 1;
        pdVar5[1] = *(double *)(lVar4 + lVar7) * pdVar5[1];
        pdVar5 = pdVar5 + 2;
      } while (lVar6 < lVar3);
    }
    return CONCAT71((int7)((ulonglong)lVar4 >> 8),1);
  }
  return (ulonglong)(uint7)(int7)(int3)((uint)iVar2 >> 8) << 8;
}


==== FUN_141164e90 @ 141164e90 (prof 0)

ulonglong FUN_141164e90(int *param_1,longlong param_2)

{
  undefined8 uVar1;
  double dVar2;
  double dVar3;
  double dVar4;
  int iVar5;
  double *pdVar6;
  uint uVar7;
  uint uVar8;
  ulonglong in_RAX;
  longlong lVar10;
  longlong lVar11;
  longlong lVar12;
  double *pdVar13;
  int iVar14;
  longlong lVar15;
  ulonglong uVar16;
  ulonglong uVar17;
  longlong lVar18;
  double dVar19;
  double dVar20;
  double *pdVar9;
  
  iVar5 = *param_1;
  uVar16 = (ulonglong)iVar5;
  if (7 < iVar5) {
    iVar14 = 0;
    if (0 < param_1[10]) {
      lVar12 = 0;
      do {
        lVar12 = lVar12 + 4;
        iVar14 = iVar14 + 1;
        lVar11 = (longlong)*(int *)(*(longlong *)(param_1 + 8) + -4 + lVar12);
        lVar10 = (longlong)*(int *)(*(longlong *)(param_1 + 6) + -4 + lVar12);
        uVar1 = *(undefined8 *)(param_2 + lVar11 * 0x10);
        *(undefined8 *)(param_2 + lVar11 * 0x10) = *(undefined8 *)(param_2 + lVar10 * 0x10);
        *(undefined8 *)(param_2 + lVar10 * 0x10) = uVar1;
        uVar1 = *(undefined8 *)(param_2 + 8 + lVar11 * 0x10);
        *(undefined8 *)(param_2 + 8 + lVar11 * 0x10) = *(undefined8 *)(param_2 + 8 + lVar10 * 0x10);
        *(undefined8 *)(param_2 + 8 + lVar10 * 0x10) = uVar1;
      } while (iVar14 < param_1[10]);
    }
    uVar7 = (int)(iVar5 + (iVar5 >> 0x1f & 3U)) >> 2;
    pdVar9 = (double *)(ulonglong)uVar7;
    if (1 < (longlong)uVar16) {
      lVar12 = 1;
      uVar17 = uVar16;
      do {
        lVar10 = lVar12 * 2;
        lVar11 = 0;
        uVar8 = (int)uVar17 / 2;
        pdVar9 = (double *)(ulonglong)uVar8;
        uVar17 = (ulonglong)(int)uVar8;
        if (0 < lVar12) {
          lVar15 = 0;
          pdVar13 = (double *)(lVar12 * 0x10 + param_2);
          do {
            pdVar9 = (double *)(lVar15 + (longlong)(int)uVar7 * 8);
            dVar2 = *(double *)((longlong)pdVar9 + *(longlong *)(param_1 + 0xc));
            dVar3 = *(double *)(lVar15 + *(longlong *)(param_1 + 0xc));
            pdVar6 = pdVar13;
            for (lVar18 = lVar11; lVar18 < (longlong)uVar16; lVar18 = lVar18 + lVar10) {
              dVar19 = dVar3 * pdVar6[1] + dVar2 * *pdVar6;
              dVar20 = dVar2 * pdVar6[1] - dVar3 * *pdVar6;
              *pdVar6 = pdVar6[lVar12 * -2] - dVar19;
              dVar4 = pdVar6[lVar12 * -2 + 1];
              pdVar6[lVar12 * -2] = dVar19 + pdVar6[lVar12 * -2];
              pdVar6[1] = dVar4 - dVar20;
              pdVar6[lVar12 * -2 + 1] = dVar20 + pdVar6[lVar12 * -2 + 1];
              pdVar9 = pdVar6 + lVar12 * 4;
              pdVar6 = pdVar9;
            }
            lVar15 = lVar15 + uVar17 * 8;
            lVar11 = lVar11 + 1;
            pdVar13 = pdVar13 + 2;
          } while (lVar11 < lVar12);
          uVar17 = (ulonglong)uVar8;
        }
        lVar12 = lVar10;
      } while (lVar10 < (longlong)uVar16);
    }
    return CONCAT71((int7)((ulonglong)pdVar9 >> 8),1);
  }
  return in_RAX & 0xffffffffffffff00;
}


