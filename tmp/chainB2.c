==== FUN_141188c50 @ 141188c50 (prof 0)

undefined1 FUN_141188c50(longlong param_1,undefined8 param_2)

{
  longlong lVar1;
  double dVar2;
  
  lVar1 = param_1 + 0x1258;
  dVar2 = (double)FUN_141165dc0(lVar1,DAT_143538368,param_2);
  FUN_141165eb0(lVar1,0,0,dVar2 * dVar2,DAT_145415378,dVar2 * DAT_1454153e8,dVar2 * dVar2,param_2);
  FUN_141165da0(lVar1);
  FUN_141165d70(param_1 + 0x1290,lVar1);
  FUN_141165da0(param_1 + 0x1290);
  FUN_141165d70(param_1 + 0x12c8,lVar1);
  FUN_141165da0(param_1 + 0x12c8);
  FUN_141165d70(param_1 + 0x1300,lVar1);
  FUN_141165da0(param_1 + 0x1300);
  return 1;
}


==== FUN_141189410 @ 141189410 (prof 0)

undefined1 FUN_141189410(longlong param_1,undefined8 param_2)

{
  double dVar1;
  undefined8 uVar2;
  undefined8 uVar3;
  
  dVar1 = (double)FUN_141165dc0(param_1 + 0x1378,DAT_1435380f0,param_2);
  uVar3 = DAT_145415378;
  FUN_141165eb0(param_1 + 0x1378,DAT_145415378,0,0,DAT_145415378,dVar1 * DAT_1454153e8,dVar1 * dVar1
                ,param_2);
  FUN_141165da0(param_1 + 0x1378);
  uVar2 = FUN_141165dc0(param_1 + 0x13b0,DAT_1435380c8,param_2);
  FUN_141165e30(param_1 + 0x13b0,0,uVar2,uVar3,uVar2,param_2);
  FUN_141165da0(param_1 + 0x13b0);
  return 1;
}


==== FUN_1411898c0 @ 1411898c0 (prof 0)

undefined8 FUN_1411898c0(longlong param_1,undefined8 param_2)

{
  int iVar1;
  longlong lVar2;
  double dVar3;
  double dVar4;
  double dVar5;
  undefined8 uVar6;
  undefined8 uVar7;
  double dVar8;
  double dVar9;
  double dVar10;
  double dVar11;
  
  lVar2 = param_1 + 0x1408;
  dVar3 = (double)FUN_141165dc0(lVar2,DAT_1434e5608,param_2);
  dVar4 = (double)FUN_141165dc0(param_1 + 0x1478,DAT_143538378,param_2);
  iVar1 = 0;
  dVar8 = dVar4 * dVar4;
  dVar9 = dVar3 * dVar3;
  uVar7 = DAT_145415378;
  dVar10 = DAT_1435380b0;
  dVar11 = DAT_1435380a8;
  do {
    dVar5 = (double)FUN_1427a2fa0((double)iVar1 * dVar10 + dVar11);
    dVar5 = dVar5 + dVar5;
    FUN_141165eb0(lVar2,uVar7,0,0,uVar7,dVar5 * dVar3,dVar9,param_2);
    FUN_141165da0(lVar2);
    FUN_141165eb0(lVar2 + 0x70,0,0,dVar8,uVar7,dVar5 * dVar4,dVar8,param_2);
    FUN_141165da0(lVar2 + 0x70);
    iVar1 = iVar1 + 1;
    lVar2 = lVar2 + 0x38;
  } while (iVar1 < 2);
  uVar6 = FUN_141165dc0(param_1 + 0x14e8,DAT_143538160,param_2);
  FUN_141165e30(param_1 + 0x14e8,0,uVar6,uVar7,uVar6,param_2);
  FUN_141165da0(param_1 + 0x14e8);
  return 1;
}


==== FUN_14118e120 @ 14118e120 (prof 0)

bool FUN_14118e120(longlong param_1,undefined8 param_2,undefined4 param_3,undefined8 param_4,
                  ulonglong param_5)

{
  undefined1 auVar1 [16];
  int iVar2;
  longlong lVar3;
  double dVar4;
  double dVar5;
  double dVar6;
  undefined1 auVar7 [16];
  undefined1 auVar8 [64];
  undefined1 auVar9 [64];
  double dVar10;
  double dVar11;
  undefined8 uVar12;
  
  lVar3 = param_1 + 0x15a8;
  auVar8 = ZEXT864(param_5);
  dVar4 = (double)FUN_141165dc0(lVar3,DAT_1434e55f0,param_5);
  uVar12 = DAT_145415378;
  dVar10 = DAT_1435383f0;
  iVar2 = 0;
  dVar5 = auVar8._0_8_;
  *(undefined4 *)(param_1 + 0x15a0) = 0;
  dVar11 = DAT_1435380b0;
  dVar6 = DAT_1435380a8;
  auVar9 = ZEXT1664(ZEXT816(0));
  if (((dVar5 <= dVar10) || (DAT_1435383f8 <= dVar5)) &&
     ((dVar5 <= DAT_143538400 || (DAT_143538408 <= dVar5)))) {
    if (dVar5 <= DAT_1435383e0) {
      if (((DAT_1435383c0 < dVar5) && (dVar5 < DAT_1435383c8)) ||
         (((DAT_1435383d0 < dVar5 && (dVar5 < DAT_1435383d8)) ||
          ((DAT_1435383b0 < dVar5 && (dVar5 < DAT_1435383b8)))))) {
        dVar6 = dVar4 * DAT_1454153e8;
        *(undefined4 *)(param_1 + 0x15a0) = 2;
        FUN_141165eb0(lVar3,0,0,dVar4 * dVar4,uVar12,dVar6,dVar4 * dVar4,dVar5);
        FUN_141165da0(lVar3);
      }
      goto LAB_14118e338;
    }
    if (DAT_1435383e8 <= dVar5) goto LAB_14118e338;
  }
  *(undefined4 *)(param_1 + 0x15a0) = 4;
  dVar10 = dVar4 * dVar4;
  do {
    dVar5 = (double)FUN_1427a2fa0((double)iVar2 * dVar11 + dVar6);
    FUN_141165eb0(lVar3,auVar9._0_8_,auVar9._0_8_,dVar10,uVar12,(dVar5 + dVar5) * dVar4,dVar10,
                  auVar8._0_8_);
    FUN_141165da0(lVar3);
    FUN_141165d70(lVar3 + 0x70,lVar3);
    FUN_141165da0(lVar3 + 0x70);
    iVar2 = iVar2 + 1;
    lVar3 = lVar3 + 0x38;
  } while (iVar2 < 2);
LAB_14118e338:
  dVar6 = (double)FUN_141165dc0(param_1 + 0x1688,DAT_143538160,auVar8._0_8_);
  auVar7._8_8_ = 0;
  auVar7._0_8_ = DAT_1435380d0;
  auVar1 = vcmpsd_avx(auVar8._0_16_,auVar9._0_16_,0);
  auVar1 = vblendvpd_avx(auVar8._0_16_,auVar7,auVar1);
  FUN_141165eb0(param_1 + 0x1688,auVar9._0_8_,auVar9._0_8_,dVar6 * dVar6,uVar12,
                dVar6 * DAT_1454153e8,dVar6 * dVar6,auVar1._0_8_);
  FUN_141165da0(param_1 + 0x1688);
  *(undefined8 *)(param_1 + 0x16c0) = param_2;
  *(undefined8 *)(param_1 + 0x16d0) = param_4;
  *(undefined4 *)(param_1 + 0x16c8) = param_3;
  if (*(int *)(param_1 + 0x16f0) < 1) {
    return true;
  }
  lVar3 = FUN_1411692e0(param_1 + 0x16d8,0,0);
  return lVar3 != 0;
}


==== FUN_14118cf70 @ 14118cf70 (prof 0)

bool FUN_14118cf70(longlong param_1,undefined8 param_2,undefined4 param_3,undefined8 param_4,
                  double param_5)

{
  int iVar1;
  longlong lVar2;
  int iVar3;
  bool bVar4;
  double dVar5;
  double dVar6;
  double dVar7;
  undefined8 uVar8;
  double dVar9;
  double dVar10;
  
  lVar2 = param_1 + 0x1700;
  dVar5 = (double)FUN_141165dc0(lVar2,DAT_1434e55f0,param_5);
  uVar8 = DAT_145415378;
  dVar7 = DAT_1435383f0;
  dVar9 = DAT_1435380b0;
  dVar10 = DAT_1435380a8;
  iVar3 = 0;
  *(undefined4 *)(param_1 + 0x16f8) = 0;
  if (((param_5 <= dVar7) || (DAT_1435383f8 <= param_5)) &&
     ((param_5 <= DAT_143538400 || (DAT_143538408 <= param_5)))) {
    if (param_5 <= DAT_1435383e0) {
      if (((DAT_1435383c0 < param_5) && (param_5 < DAT_1435383c8)) ||
         (((DAT_1435383d0 < param_5 && (param_5 < DAT_1435383d8)) ||
          ((DAT_1435383b0 < param_5 && (param_5 < DAT_1435383b8)))))) {
        dVar7 = dVar5 * DAT_1454153e8;
        *(undefined4 *)(param_1 + 0x16f8) = 2;
        FUN_141165eb0(lVar2,0,0,dVar5 * dVar5,uVar8,dVar7,dVar5 * dVar5,param_5);
        FUN_141165da0(lVar2);
      }
      goto LAB_14118d15d;
    }
    if (DAT_1435383e8 <= param_5) goto LAB_14118d15d;
  }
  *(undefined4 *)(param_1 + 0x16f8) = 4;
  iVar1 = 0;
  dVar7 = dVar5 * dVar5;
  do {
    dVar6 = (double)FUN_1427a2fa0((double)iVar1 * dVar9 + dVar10);
    FUN_141165eb0(lVar2,0,0,dVar7,uVar8,(dVar6 + dVar6) * dVar5,dVar7,param_5);
    FUN_141165da0(lVar2);
    FUN_141165d70(lVar2 + 0x70,lVar2);
    FUN_141165da0(lVar2 + 0x70);
    iVar1 = iVar1 + 1;
    lVar2 = lVar2 + 0x38;
  } while (iVar1 < 2);
LAB_14118d15d:
  dVar7 = (double)FUN_141165dc0(param_1 + 0x17e0,DAT_143538160,param_5);
  FUN_141165eb0(param_1 + 0x17e0,0,0,dVar7 * dVar7,uVar8,dVar7 * DAT_1454153e8,dVar7 * dVar7,param_5
               );
  FUN_141165da0(param_1 + 0x17e0);
  dVar5 = (double)FUN_141165dc0(param_1 + 0x1888,DAT_143538410,param_5);
  dVar7 = dVar5 * dVar5;
  lVar2 = param_1 + 0x1818;
  do {
    dVar6 = (double)FUN_1427a2fa0((double)iVar3 * dVar9 + dVar10);
    FUN_141165eb0(lVar2,uVar8,0,0,uVar8,(dVar6 + dVar6) * dVar5,dVar7,param_5);
    FUN_141165da0(lVar2);
    FUN_141165d70(lVar2 + 0x70,lVar2);
    FUN_141165da0(lVar2 + 0x70);
    iVar3 = iVar3 + 1;
    lVar2 = lVar2 + 0x38;
  } while (iVar3 < 2);
  FUN_141165d70(param_1 + 0x18f8,param_1 + 0x17e0);
  FUN_141165da0(param_1 + 0x18f8);
  *(undefined4 *)(param_1 + 0x1938) = param_3;
  *(undefined8 *)(param_1 + 0x1930) = param_2;
  *(undefined8 *)(param_1 + 0x1940) = param_4;
  if (*(int *)(param_1 + 0x1960) < 1) {
    bVar4 = true;
  }
  else {
    lVar2 = FUN_1411692e0(param_1 + 0x1948,0,0);
    bVar4 = lVar2 != 0;
  }
  return bVar4;
}


==== FUN_14117e830 @ 14117e830 (prof 0)

undefined1 FUN_14117e830(longlong param_1,undefined8 param_2,undefined8 param_3)

{
  *(undefined8 *)(param_1 + 0x1968) = param_2;
  *(undefined8 *)(param_1 + 0x1970) = param_3;
  return 1;
}


==== FUN_14117ff60 @ 14117ff60 (prof 0)

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

undefined8 FUN_14117ff60(longlong param_1,double param_2)

{
  undefined1 auVar1 [32];
  undefined1 auVar2 [16];
  undefined8 uVar3;
  char cVar4;
  longlong lVar5;
  double *pdVar6;
  int *piVar7;
  undefined8 *puVar8;
  double *pdVar9;
  undefined8 *puVar10;
  longlong lVar11;
  longlong lVar12;
  double *pdVar13;
  int iVar14;
  int iVar15;
  undefined8 in_RDX;
  undefined4 uVar16;
  int iVar17;
  int *piVar18;
  double dVar19;
  undefined8 uVar20;
  undefined1 extraout_var [56];
  double dVar21;
  double dVar22;
  double dVar23;
  int iVar24;
  int iVar25;
  undefined1 local_f8 [24];
  int local_e0;
  int *local_d8;
  double *local_d0;
  double *local_c8;
  double *local_c0 [4];
  undefined1 local_a0 [32];
  undefined8 local_80;
  undefined8 uStack_78;
  undefined8 uStack_70;
  undefined8 uStack_68;
  
  uVar16 = (undefined4)((ulonglong)in_RDX >> 0x20);
  FUN_141166770(local_a0);
  FUN_141166770(local_f8);
  FUN_141166770(local_c0);
  lVar5 = FUN_1411667d0(param_1 + 0x1998,CONCAT44(uVar16,(int)(param_2 * DAT_1454154a0)),1);
  if (lVar5 != 0) {
    *(undefined4 *)(param_1 + 0x19b8) = 0;
    pdVar6 = (double *)FUN_1411667d0(local_a0,0x49,0);
    local_d0 = pdVar6;
    if (pdVar6 != (double *)0x0) {
      *pdVar6 = 27.5;
      local_80 = _DAT_145416b40;
      uVar20 = local_80;
      uStack_78 = _UNK_145416b48;
      uVar3 = uStack_78;
      uStack_70 = _UNK_145416b50;
      uStack_68 = _UNK_145416b58;
      iVar17 = 1;
      pdVar9 = pdVar6 + 1;
      local_80._0_4_ = (int)_DAT_145416b40;
      local_80._4_4_ = (int)((ulonglong)_DAT_145416b40 >> 0x20);
      uStack_78._0_4_ = (int)_UNK_145416b48;
      uStack_78._4_4_ = (int)((ulonglong)_UNK_145416b48 >> 0x20);
      pdVar13 = pdVar9;
      iVar15 = (int)local_80;
      iVar14 = local_80._4_4_;
      iVar24 = (int)uStack_78;
      iVar25 = uStack_78._4_4_;
      local_c8 = pdVar9;
      local_80 = uVar20;
      uStack_78 = uVar3;
      do {
        auVar2._0_4_ = iVar17 + iVar15;
        auVar2._4_4_ = iVar17 + iVar14;
        auVar2._8_4_ = iVar17 + iVar24;
        auVar2._12_4_ = iVar17 + iVar25;
        auVar1 = vcvtdq2pd_avx(auVar2);
        dVar19 = (double)FUN_1426086a0(_DAT_1435381e0,auVar1._0_8_ * _DAT_1435381c0);
        dVar21 = extraout_var._0_8_ * _UNK_143538208;
        dVar22 = extraout_var._8_8_ * _UNK_143538210;
        dVar23 = extraout_var._16_8_ * _UNK_143538218;
        *pdVar13 = dVar19 * _DAT_143538200;
        pdVar13[1] = dVar21;
        pdVar13[2] = dVar22;
        pdVar13[3] = dVar23;
        iVar17 = iVar17 + 4;
        pdVar13 = pdVar13 + 4;
      } while (iVar17 < 0x49);
      piVar7 = (int *)FUN_141167dc0(param_1 + 0x19c0,0x48,0);
      local_d8 = piVar7;
      if (piVar7 != (int *)0x0) {
        uVar16 = 0;
        puVar8 = (undefined8 *)FUN_1411667d0(param_1 + 0x19e0,0x48,0);
        if (puVar8 != (undefined8 *)0x0) {
          dVar19 = param_2 + param_2;
          iVar15 = (int)(dVar19 / (*pdVar9 - *pdVar6));
          *piVar7 = iVar15;
          pdVar9 = (double *)FUN_1411667d0(local_f8,CONCAT44(uVar16,iVar15),0);
          iVar14 = 0;
          iVar15 = *piVar7;
          if (0 < iVar15) {
            param_2 = DAT_145415378 / param_2;
            pdVar13 = pdVar9;
            do {
              iVar14 = iVar14 + 1;
              *pdVar13 = (double)iVar14 * param_2;
              pdVar13 = pdVar13 + 1;
              iVar15 = *piVar7;
            } while (iVar14 < iVar15);
          }
          lVar5 = param_1 + 0x1a00;
          cVar4 = FUN_1411697a0(lVar5,0x48,iVar15 * 2,1);
          if (cVar4 != '\0') {
            iVar15 = 0;
            dVar21 = DAT_1454154f8;
            do {
              puVar10 = (undefined8 *)FUN_141169a20(lVar5,iVar15);
              if (puVar10 == (undefined8 *)0x0) goto LAB_141180392;
              iVar14 = 0;
              pdVar13 = pdVar9;
              if (0 < local_e0) {
                do {
                  uVar20 = FUN_1427a2fa0(dVar21 * *pdVar6 * *pdVar13);
                  *puVar10 = uVar20;
                  uVar20 = FUN_1427a51e0(dVar21 * *pdVar6 * *pdVar13);
                  puVar10[1] = uVar20;
                  iVar14 = iVar14 + 1;
                  puVar10 = puVar10 + 2;
                  pdVar13 = pdVar13 + 1;
                } while (iVar14 < local_e0);
              }
              iVar15 = iVar15 + 1;
              pdVar6 = pdVar6 + 1;
            } while (iVar15 < 0xc);
            iVar15 = 0xc;
            do {
              lVar11 = FUN_141169a20(lVar5,iVar15);
              if ((lVar11 == 0) || (lVar12 = FUN_141169a20(lVar5), lVar12 == 0)) goto LAB_141180392;
              iVar14 = 0;
              if (0 < local_e0) {
                pdVar6 = (double *)(lVar12 + 8);
                do {
                  *(double *)((lVar11 - lVar12) + -8 + (longlong)pdVar6) =
                       pdVar6[-1] * pdVar6[-1] - *pdVar6 * *pdVar6;
                  *(double *)((lVar11 - lVar12) + (longlong)pdVar6) =
                       (pdVar6[-1] + pdVar6[-1]) * *pdVar6;
                  iVar14 = iVar14 + 1;
                  pdVar6 = pdVar6 + 2;
                } while (iVar14 < local_e0);
              }
              iVar15 = iVar15 + 1;
            } while (iVar15 < 0x48);
            pdVar6 = (double *)FUN_141169a20(lVar5,0);
            piVar7 = local_d8;
            if ((pdVar6 != (double *)0x0) &&
               (cVar4 = FUN_14117dfc0(param_1,*local_d8,local_c0), cVar4 != '\0')) {
              iVar15 = 0;
              pdVar9 = local_c0[0];
              if (0 < local_e0) {
                do {
                  *pdVar6 = *pdVar6 * *pdVar9;
                  pdVar6[1] = pdVar6[1] * *pdVar9;
                  iVar15 = iVar15 + 1;
                  pdVar6 = pdVar6 + 2;
                  pdVar9 = pdVar9 + 1;
                } while (iVar15 < local_e0);
              }
              *puVar8 = 0x3ff0000000000000;
              lVar11 = (longlong)puVar8 - (longlong)local_d0;
              iVar15 = 1;
              pdVar6 = local_c8;
              piVar18 = piVar7;
              do {
                piVar18 = piVar18 + 1;
                *piVar18 = (int)(dVar19 / (pdVar6[1] - *pdVar6));
                pdVar9 = (double *)FUN_141169a20(lVar5,iVar15);
                if ((pdVar9 == (double *)0x0) ||
                   (cVar4 = FUN_14117dfc0(param_1,*piVar18,local_c0), cVar4 == '\0'))
                goto LAB_141180392;
                iVar14 = 0;
                pdVar13 = local_c0[0];
                if (0 < local_e0) {
                  do {
                    *pdVar9 = *pdVar9 * *pdVar13;
                    pdVar9[1] = pdVar9[1] * *pdVar13;
                    iVar14 = iVar14 + 1;
                    pdVar9 = pdVar9 + 2;
                    pdVar13 = pdVar13 + 1;
                  } while (iVar14 < local_e0);
                }
                *(double *)(lVar11 + (longlong)pdVar6) = (double)*piVar7 / (double)*piVar18;
                iVar15 = iVar15 + 1;
                pdVar6 = pdVar6 + 1;
              } while (iVar15 < 0x48);
              uVar20 = 1;
              goto LAB_141180394;
            }
          }
        }
      }
    }
  }
LAB_141180392:
  uVar20 = 0;
LAB_141180394:
  FUN_141166790(local_c0);
  FUN_141166790(local_f8);
  FUN_141166790(local_a0);
  return uVar20;
}


==== FUN_14117b170 @ 14117b170 (prof 0)

bool FUN_14117b170(longlong param_1)

{
  char cVar1;
  
  cVar1 = FUN_1411645f0(param_1 + 0x1a38,0x1000);
  if (cVar1 != '\0') {
    cVar1 = FUN_141165390(param_1 + 0x1a38);
    return cVar1 != '\0';
  }
  return false;
}


==== FUN_14117ac40 @ 14117ac40 (prof 0)

bool FUN_14117ac40(longlong param_1)

{
  char cVar1;
  
  cVar1 = FUN_1411645f0(param_1 + 0x1a88,0x800);
  if (cVar1 != '\0') {
    cVar1 = FUN_141165390(param_1 + 0x1a88);
    return cVar1 != '\0';
  }
  return false;
}


==== FUN_141194730 @ 141194730 (prof 0)

bool FUN_141194730(longlong param_1)

{
  char cVar1;
  
  cVar1 = FUN_1411645f0(param_1 + 0x1ad8,0x100);
  if (cVar1 != '\0') {
    cVar1 = FUN_141165390(param_1 + 0x1ad8);
    return cVar1 != '\0';
  }
  return false;
}


==== FUN_14117a050 @ 14117a050 (prof 0)

undefined1 FUN_14117a050(longlong param_1,undefined8 param_2,undefined8 param_3)

{
  double dVar1;
  undefined8 uVar2;
  undefined8 uVar3;
  
  dVar1 = (double)FUN_141165dc0(param_1 + 0x1b28,DAT_1435380f0,param_2);
  uVar3 = DAT_145415378;
  FUN_141165eb0(param_1 + 0x1b28,0,0,dVar1 * dVar1,DAT_145415378,dVar1 * DAT_1454153e8,dVar1 * dVar1
                ,param_2);
  FUN_141165da0(param_1 + 0x1b28);
  dVar1 = (double)FUN_141165dc0(param_1 + 0x1b60,DAT_1435380f0,param_3);
  FUN_141165eb0(param_1 + 0x1b60,uVar3,0,0,uVar3,dVar1 * DAT_1454153e8,dVar1 * dVar1,param_3);
  FUN_141165da0(param_1 + 0x1b60);
  uVar2 = FUN_141165dc0(param_1 + 0x1b98,DAT_1435380c8,param_2);
  FUN_141165e30(param_1 + 0x1b98,0,uVar2,uVar3,uVar2,param_2);
  FUN_141165da0(param_1 + 0x1b98);
  uVar2 = FUN_141165dc0(param_1 + 0x1bd0,DAT_1435380c8,param_3);
  FUN_141165e30(param_1 + 0x1bd0,0,uVar2,uVar3,uVar2,param_3);
  FUN_141165da0(param_1 + 0x1bd0);
  return 1;
}


==== FUN_14117e1b0 @ 14117e1b0 (prof 0)

undefined8 FUN_14117e1b0(longlong param_1,undefined8 param_2)

{
  longlong lVar1;
  int iVar2;
  int iVar3;
  double dVar4;
  double dVar5;
  undefined8 uVar6;
  undefined8 uVar7;
  double dVar8;
  double dVar9;
  double dVar10;
  
  lVar1 = param_1 + 0x1c48;
  dVar4 = (double)FUN_141165dc0(lVar1,DAT_143538168,param_2);
  iVar3 = 0;
  iVar2 = 0;
  dVar8 = dVar4 * dVar4;
  uVar7 = DAT_145415378;
  dVar9 = DAT_1435380b0;
  dVar10 = DAT_1435380a8;
  do {
    dVar5 = (double)FUN_1427a2fa0((double)iVar2 * dVar9 + dVar10);
    FUN_141165eb0(lVar1,uVar7,0,0,uVar7,(dVar5 + dVar5) * dVar4,dVar8,param_2);
    FUN_141165da0(lVar1);
    iVar2 = iVar2 + 1;
    lVar1 = lVar1 + 0x38;
  } while (iVar2 < 2);
  lVar1 = param_1 + 0x1cb8;
  dVar4 = (double)FUN_141165dc0(lVar1,DAT_143538170,param_2);
  dVar8 = dVar4 * dVar4;
  do {
    dVar5 = (double)FUN_1427a2fa0((double)iVar3 * dVar9 + dVar10);
    FUN_141165eb0(lVar1,0,0,dVar8,uVar7,(dVar5 + dVar5) * dVar4,dVar8,param_2);
    FUN_141165da0(lVar1);
    iVar3 = iVar3 + 1;
    lVar1 = lVar1 + 0x38;
  } while (iVar3 < 2);
  uVar6 = FUN_141165dc0(param_1 + 0x1d28,DAT_143538160,param_2);
  FUN_141165e30(param_1 + 0x1d28,0,uVar6,uVar7,uVar6,param_2);
  FUN_141165da0(param_1 + 0x1d28);
  return 1;
}


