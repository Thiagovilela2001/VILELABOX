==== FUN_141170430 @ 141170430 (prof 0)

undefined4 FUN_141170430(longlong *param_1)

{
  int iVar1;
  char cVar2;
  undefined4 uVar3;
  longlong lVar4;
  int iVar5;
  
  if (((((int)param_1[0x15] < 3) || (4 < (int)param_1[0x15])) || ((char)param_1[0x19] == '\0')) ||
     (cVar2 = FUN_1411705c0(), cVar2 == '\0')) {
    return 0xffffffff;
  }
  lVar4 = *param_1;
  do {
    iVar1 = *(int *)((longlong)param_1 + 0xc4);
    iVar5 = (int)param_1[3];
    if (iVar5 < iVar1) {
      if ((int)param_1[4] < (int)param_1[0x18]) goto LAB_141170532;
    }
    else if (iVar1 < iVar5) {
      iVar5 = iVar1;
    }
    lVar4 = FUN_141166bc0(param_1 + 9,lVar4,iVar5);
    if ((lVar4 == 0) || (lVar4 = FUN_141166dc0(param_1,0,iVar5), lVar4 == 0)) goto LAB_141170567;
    if ((int)param_1[0x15] == 3) {
      cVar2 = FUN_1411965b0(param_1,param_1 + 9);
      if (cVar2 == '\0') goto LAB_141170567;
      if ((int)param_1[0x168] == 2) {
        FUN_141167030(param_1);
        FUN_141167030(param_1 + 9);
        *(undefined4 *)(param_1 + 0x15) = 4;
      }
    }
  } while ((int)param_1[0x15] != 4);
  cVar2 = FUN_141197d60(param_1);
  if (cVar2 == '\0') {
LAB_141170567:
    *(undefined4 *)(param_1 + 0x15) = 0xffffffff;
    uVar3 = 0xffffffff;
  }
  else {
    *(undefined4 *)(param_1 + 0x15) = 5;
LAB_141170532:
    if (4 < (int)param_1[0x15]) {
      FUN_141167030(param_1);
      FUN_141167030(param_1 + 9);
    }
    uVar3 = (undefined4)param_1[0x15];
  }
  return uVar3;
}


