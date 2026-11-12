==== FUN_141173370 @ 141173370 (prof 0)

undefined8
FUN_141173370(longlong param_1,double param_2,double param_3,double param_4,double param_5,
             double param_6,double param_7,double param_8,int param_9)

{
  double dVar1;
  int iVar2;
  longlong lVar3;
  int *piVar4;
  longlong lVar5;
  longlong lVar6;
  undefined8 uVar7;
  int iVar8;
  int *piVar9;
  int iVar10;
  double dVar11;
  double dVar12;
  double dVar13;
  double dVar14;
  double dVar15;
  double dVar16;
  
  param_8 = DAT_145415638 / param_8;
  dVar13 = param_8 * param_5;
  dVar14 = param_8 * param_7;
  param_8 = param_8 * param_3;
  dVar12 = (double)param_9 * DAT_145414e88;
  dVar15 = (double)(int)(dVar12 * param_3 + DAT_1454152a0);
  iVar10 = (int)((param_2 - dVar15) / param_8);
  dVar16 = (double)(int)(dVar12 * param_5 + DAT_1454152a0);
  dVar12 = (double)(int)(dVar12 * param_7 + DAT_1454152a0);
  iVar8 = (int)((param_4 - dVar16) / dVar13);
  iVar2 = (int)((param_6 - dVar12) / dVar14);
  if (iVar10 < iVar8) {
    if (iVar2 <= iVar10) {
      iVar10 = iVar2;
    }
  }
  else {
    iVar10 = iVar2;
    if (iVar8 < iVar2) {
      iVar10 = iVar8;
    }
  }
  *(int *)(param_1 + 0x1180) = iVar10;
  lVar3 = FUN_141167dc0(param_1 + 0x1120,iVar10 + 1,0);
  if (((lVar3 == 0) ||
      (piVar4 = (int *)FUN_141167dc0(param_1 + 0x1140,*(int *)(param_1 + 0x1180) + 1,0),
      piVar4 == (int *)0x0)) ||
     (lVar5 = FUN_141167dc0(param_1 + 0x1160,*(int *)(param_1 + 0x1180) + 1,0),
     dVar1 = DAT_145415408, lVar5 == 0)) {
    uVar7 = 0;
  }
  else {
    iVar10 = *(int *)(param_1 + 0x1180);
    iVar2 = 0;
    lVar6 = lVar5;
    if (0 < iVar10 + 1) {
      piVar9 = piVar4;
      do {
        dVar11 = (double)iVar2;
        *(int *)((longlong)piVar9 + (lVar3 - (longlong)piVar4)) =
             (int)(dVar11 * param_8 + dVar15 + dVar1);
        *piVar9 = (int)(dVar11 * dVar13 + dVar16 + dVar1);
        iVar2 = iVar2 + 1;
        *(int *)((longlong)piVar9 + (lVar5 - (longlong)piVar4)) =
             (int)(dVar11 * dVar14 + dVar12 + dVar1);
        iVar10 = *(int *)(param_1 + 0x1180);
        lVar6 = 0;
        piVar9 = piVar9 + 1;
      } while (iVar2 < iVar10 + 1);
    }
    if ((int)param_2 < *(int *)(lVar3 + -4 + (longlong)iVar10 * 4)) {
      *(int *)(lVar3 + -4 + (longlong)iVar10 * 4) = (int)param_2;
    }
    if ((int)param_4 < piVar4[(longlong)*(int *)(param_1 + 0x1180) + -1]) {
      piVar4[(longlong)*(int *)(param_1 + 0x1180) + -1] = (int)param_4;
    }
    iVar10 = (int)param_6;
    if (iVar10 < *(int *)(lVar5 + -4 + (longlong)*(int *)(param_1 + 0x1180) * 4)) {
      *(int *)(lVar5 + -4 + (longlong)*(int *)(param_1 + 0x1180) * 4) = iVar10;
    }
    uVar7 = CONCAT71((int7)(CONCAT44((int)((ulonglong)lVar6 >> 0x20),iVar10) >> 8),1);
  }
  return uVar7;
}


==== FUN_141167dc0 @ 141167dc0 (prof 1)

longlong FUN_141167dc0(longlong *param_1,int param_2,char param_3)

{
  longlong *plVar1;
  ulonglong *puVar2;
  longlong lVar3;
  ulonglong uVar4;
  
  if (param_2 < 0) {
    return 0;
  }
  plVar1 = param_1 + 3;
  puVar2 = (ulonglong *)(param_1 + 2);
  if (*(char *)((longlong)param_1 + 0x1c) != '\0') {
    *param_1 = 0;
    param_1[1] = 0;
    *(undefined4 *)plVar1 = 0;
    *puVar2 = 0;
    *(undefined1 *)((longlong)param_1 + 0x1c) = 0;
  }
  lVar3 = *param_1;
  uVar4 = (longlong)param_2 * 4;
  if (lVar3 != 0) {
    if (uVar4 < *puVar2 || uVar4 - *puVar2 == 0) {
      *(int *)plVar1 = param_2;
      goto LAB_141167e35;
    }
    thunk_FUN_1427bd480();
    *param_1 = 0;
  }
  if (param_1[1] != 0) {
    thunk_FUN_1427bd480();
    param_1[1] = 0;
  }
  *(int *)plVar1 = param_2;
  *puVar2 = uVar4;
  if (uVar4 == 0) {
    uVar4 = 4;
    param_1[2] = 4;
  }
  lVar3 = _malloc_base(uVar4);
  *param_1 = lVar3;
  if (lVar3 == 0) {
    *(undefined4 *)(param_1 + 3) = 0;
    param_1[2] = 0;
    return 0;
  }
LAB_141167e35:
  if (param_3 != '\0') {
    FUN_142db0ac0(lVar3,0,(longlong)(int)param_1[3] << 2);
  }
  return *param_1;
}


