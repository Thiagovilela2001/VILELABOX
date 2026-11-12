==== FUN_141167080 @ 141167080 (prof 0)

/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

longlong FUN_141167080(longlong *param_1)

{
  ulonglong uVar1;
  int iVar2;
  longlong lVar3;
  longlong lVar4;
  
  uVar1 = _DAT_145416930;
  if ((*param_1 == 0) || (*(char *)((longlong)param_1 + 0x1c) != '\0')) {
    return 0;
  }
  iVar2 = (int)param_1[3] + -1;
  lVar3 = (longlong)iVar2;
  if (-1 < iVar2) {
    if (2 < lVar3) {
      lVar4 = lVar3 * 8;
      do {
        lVar4 = lVar4 + -0x20;
        *(ulonglong *)(*param_1 + lVar3 * 8) = *(ulonglong *)(*param_1 + 0x20 + lVar4) & uVar1;
        *(ulonglong *)(*param_1 + -8 + lVar3 * 8) =
             *(ulonglong *)(*param_1 + -8 + lVar3 * 8) & uVar1;
        *(ulonglong *)(*param_1 + -0x10 + lVar3 * 8) =
             *(ulonglong *)(*param_1 + -0x10 + lVar3 * 8) & uVar1;
        *(ulonglong *)(*param_1 + -0x18 + lVar3 * 8) =
             *(ulonglong *)(*param_1 + -0x18 + lVar3 * 8) & uVar1;
        lVar3 = lVar3 + -4;
      } while (2 < lVar3);
      if (lVar3 < 0) goto LAB_141167136;
    }
    do {
      *(ulonglong *)(*param_1 + lVar3 * 8) = *(ulonglong *)(*param_1 + lVar3 * 8) & uVar1;
      lVar3 = lVar3 + -1;
    } while (-1 < lVar3);
  }
LAB_141167136:
  return *param_1;
}


==== FUN_141166430 @ 141166430 (prof 0)

void FUN_141166430(double *param_1,double *param_2,double *param_3,int param_4)

{
  double dVar1;
  double dVar2;
  double dVar3;
  double dVar4;
  double dVar5;
  uint uVar6;
  longlong lVar7;
  ulonglong uVar8;
  double dVar9;
  
  dVar9 = param_1[5];
  dVar1 = *param_1;
  dVar2 = param_1[1];
  dVar3 = param_1[3];
  if (3 < param_4) {
    uVar6 = (param_4 - 4U >> 2) + 1;
    uVar8 = (ulonglong)uVar6;
    param_4 = param_4 + uVar6 * -4;
    do {
      dVar4 = *param_2;
      dVar9 = dVar1 * dVar4 + dVar9;
      *param_3 = dVar9;
      dVar5 = param_2[1];
      dVar4 = (dVar2 * dVar4 - dVar9 * dVar3) + dVar1 * dVar5;
      param_3[1] = dVar4;
      dVar9 = param_2[2];
      dVar5 = (dVar2 * dVar5 - dVar4 * dVar3) + dVar1 * dVar9;
      param_3[2] = dVar5;
      dVar4 = param_2[3];
      dVar9 = (dVar2 * dVar9 - dVar5 * dVar3) + dVar1 * param_2[3];
      param_3[3] = dVar9;
      param_3 = param_3 + 4;
      param_2 = param_2 + 4;
      dVar9 = dVar2 * dVar4 - dVar9 * dVar3;
      uVar8 = uVar8 - 1;
    } while (uVar8 != 0);
  }
  if (0 < param_4) {
    lVar7 = (longlong)param_2 - (longlong)param_3;
    do {
      dVar9 = dVar1 * *(double *)(lVar7 + (longlong)param_3) + dVar9;
      dVar4 = *(double *)(lVar7 + (longlong)param_3);
      param_4 = param_4 + -1;
      *param_3 = dVar9;
      param_3 = param_3 + 1;
      dVar9 = dVar2 * dVar4 - dVar9 * dVar3;
    } while (0 < param_4);
  }
  param_1[5] = dVar9;
  return;
}


==== FUN_1411680a0 @ 1411680a0 (prof 0)

longlong FUN_1411680a0(undefined8 param_1,longlong param_2,int param_3)

{
  longlong lVar1;
  
  if (param_2 != 0) {
    lVar1 = FUN_141167dc0(param_1,param_3,0);
    if (lVar1 != 0) {
      FUN_142db0e80(lVar1,param_2,(longlong)param_3 << 2);
      return lVar1;
    }
  }
  return 0;
}


==== FUN_141167f00 @ 141167f00 (prof 0)

longlong FUN_141167f00(longlong *param_1,int param_2,char param_3)

{
  int iVar1;
  longlong lVar2;
  longlong lVar3;
  ulonglong uVar4;
  
  if ((param_2 < 0) || (*(char *)((longlong)param_1 + 0x1c) != '\0')) {
LAB_141167fa5:
    lVar3 = 0;
  }
  else {
    lVar3 = *param_1;
    iVar1 = (int)param_1[3];
    uVar4 = (longlong)param_2 << 2;
    if (lVar3 == 0) {
LAB_141167f59:
      if (param_1[1] != 0) {
        thunk_FUN_1427bd480();
        param_1[1] = 0;
      }
      *(int *)(param_1 + 3) = param_2;
      param_1[2] = uVar4;
      if (uVar4 == 0) {
        uVar4 = 4;
        param_1[2] = 4;
      }
      lVar2 = _malloc_base(uVar4);
      *param_1 = lVar2;
      if (lVar2 == 0) {
        *(undefined4 *)(param_1 + 3) = 0;
        param_1[2] = 0;
        if (lVar3 != 0) {
          thunk_FUN_1427bd480(lVar3);
        }
        goto LAB_141167fa5;
      }
      if (lVar3 != 0) {
        FUN_142db0e80(lVar2,lVar3,(longlong)iVar1 << 2);
        thunk_FUN_1427bd480(lVar3);
      }
    }
    else {
      if ((ulonglong)param_1[2] < uVar4) {
        *param_1 = 0;
        goto LAB_141167f59;
      }
      *(int *)(param_1 + 3) = param_2;
    }
    if ((param_3 != '\0') && (iVar1 < (int)param_1[3])) {
      FUN_142db0ac0(*param_1 + (longlong)iVar1 * 4,0,(longlong)((int)param_1[3] - iVar1) << 2);
    }
    lVar3 = *param_1;
  }
  return lVar3;
}


==== FUN_141166c30 @ 141166c30 (prof 0)

longlong FUN_141166c30(longlong *param_1,longlong param_2,int param_3)

{
  if ((param_2 != 0) && (-1 < param_3)) {
    if (*(char *)((longlong)param_1 + 0x1c) == '\0') {
      if (*param_1 != 0) {
        thunk_FUN_1427bd480();
      }
      if (param_1[1] != 0) {
        thunk_FUN_1427bd480();
      }
    }
    *param_1 = param_2;
    param_1[1] = 0;
    param_1[2] = 0;
    *(int *)(param_1 + 3) = param_3;
    *(undefined1 *)((longlong)param_1 + 0x1c) = 1;
    return param_2;
  }
  return 0;
}


==== FUN_141167150 @ 141167150 (prof 0)

undefined8 FUN_141167150(longlong *param_1)

{
  longlong lVar1;
  undefined8 uVar2;
  
  lVar1 = *param_1;
  if (lVar1 == 0) {
    return 0;
  }
  uVar2 = FUN_14116ef60(lVar1,lVar1 + (longlong)(int)param_1[3] * 8,0);
  return uVar2;
}


