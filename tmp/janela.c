==== FUN_14117dfc0 @ 14117dfc0 (prof 0)

undefined8 FUN_14117dfc0(undefined8 param_1,uint param_2,undefined8 param_3)

{
  int iVar1;
  double *pdVar2;
  int iVar3;
  longlong lVar4;
  double *pdVar5;
  double *pdVar6;
  double extraout_XMM0_Qa;
  double dVar7;
  double extraout_XMM0_Qa_00;
  double dVar8;
  double dVar9;
  double dVar10;
  double dVar11;
  double dVar12;
  
  pdVar2 = (double *)FUN_1411667d0(param_3,param_2,0);
  if (pdVar2 != (double *)0x0) {
    iVar3 = 1;
    if ((param_2 & 1) == 0) {
      lVar4 = (longlong)((int)param_2 / 2);
      if (0 < lVar4) {
        dVar9 = DAT_145415378 / (double)(int)(param_2 + 1);
        pdVar5 = pdVar2 + (longlong)(int)param_2 + -1;
        dVar10 = DAT_145415378;
        dVar8 = DAT_1454154f8;
        dVar12 = DAT_1454152a0;
        do {
          FUN_1427a2fa0((double)iVar3 * dVar8 * dVar9);
          dVar11 = (dVar10 - extraout_XMM0_Qa) * dVar12;
          *pdVar2 = dVar11;
          iVar3 = iVar3 + 1;
          pdVar2 = pdVar2 + 1;
          *pdVar5 = dVar11;
          pdVar5 = pdVar5 + -1;
          lVar4 = lVar4 + -1;
        } while (lVar4 != 0);
      }
    }
    else {
      iVar1 = (int)(param_2 + 1) / 2;
      dVar12 = (double)(int)(param_2 + 1);
      dVar10 = DAT_1454154f8;
      dVar8 = DAT_1454152a0;
      if (0 < iVar1 + -1) {
        dVar11 = DAT_145415378 / dVar12;
        pdVar6 = pdVar2 + (longlong)(int)param_2 + -1;
        pdVar5 = pdVar2;
        dVar9 = DAT_145415378;
        do {
          dVar7 = (double)FUN_1427a2fa0((double)iVar3 * dVar10 * dVar11);
          dVar7 = (dVar9 - dVar7) * dVar8;
          *pdVar5 = dVar7;
          iVar3 = iVar3 + 1;
          pdVar5 = pdVar5 + 1;
          *pdVar6 = dVar7;
          pdVar6 = pdVar6 + -1;
        } while (iVar3 <= iVar1 + -1);
      }
      FUN_1427a2fa0(((double)iVar1 * dVar10) / dVar12);
      pdVar2[(longlong)iVar1 + -1] = -(extraout_XMM0_Qa_00 * dVar8) + dVar8;
    }
    return 1;
  }
  return 0;
}


==== FUN_141169a20 @ 141169a20 (prof 0)

longlong FUN_141169a20(longlong *param_1,int param_2)

{
  if (((*param_1 != 0) && (-1 < param_2)) && (param_2 < (int)param_1[2])) {
    return *param_1 + (longlong)(param_2 * *(int *)((longlong)param_1 + 0x14)) * 8;
  }
  return 0;
}


==== FUN_1411697a0 @ 1411697a0 (prof 0)

undefined8 FUN_1411697a0(longlong *param_1,int param_2,int param_3,char param_4)

{
  longlong lVar1;
  ulonglong uVar2;
  
  if ((param_2 < 0) || (param_3 < 0)) {
    return 0;
  }
  lVar1 = *param_1;
  uVar2 = (longlong)param_2 * (longlong)param_3 * 8;
  *(int *)(param_1 + 2) = param_2;
  *(int *)((longlong)param_1 + 0x14) = param_3;
  if (lVar1 != 0) {
    if (uVar2 < (ulonglong)param_1[1]) goto LAB_14116985e;
    thunk_FUN_1427bd480();
  }
  param_1[1] = uVar2;
  if (uVar2 == 0) {
    uVar2 = 8;
    param_1[1] = 8;
  }
  lVar1 = _malloc_base(uVar2);
  *param_1 = lVar1;
  if (lVar1 == 0) {
    param_1[1] = 0;
    *(undefined4 *)(param_1 + 2) = 0;
    *(undefined4 *)((longlong)param_1 + 0x14) = 0;
    return 0;
  }
LAB_14116985e:
  if (param_4 != '\0') {
    FUN_142db0ac0(lVar1,0,(longlong)(*(int *)((longlong)param_1 + 0x14) * (int)param_1[2]) << 3);
  }
  return 1;
}


