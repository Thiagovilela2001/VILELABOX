==== FUN_1411702d0 @ 1411702d0 (prof 0)

undefined4 FUN_1411702d0(longlong *param_1)

{
  int iVar1;
  char cVar2;
  undefined4 uVar3;
  longlong lVar4;
  int iVar5;
  
  if ((((int)param_1[0x15] < 1) || (2 < (int)param_1[0x15])) ||
     (cVar2 = FUN_1411705c0(), cVar2 == '\0')) {
    return 0xffffffff;
  }
  lVar4 = *param_1;
  do {
    iVar1 = *(int *)((longlong)param_1 + 0xc4);
    iVar5 = (int)param_1[3];
    if (iVar5 < iVar1) {
      if ((int)param_1[4] < (int)param_1[0x18]) goto LAB_1411703ea;
    }
    else if (iVar1 < iVar5) {
      iVar5 = iVar1;
    }
    lVar4 = FUN_141166bc0(param_1 + 9,lVar4,iVar5);
    if ((lVar4 == 0) || (lVar4 = FUN_141166dc0(param_1,0,iVar5), lVar4 == 0)) goto LAB_141170405;
    if ((int)param_1[0x15] == 1) {
      cVar2 = FUN_1411953b0(param_1,param_1 + 9);
      if (cVar2 == '\0') goto LAB_141170405;
      if (0 < (int)param_1[0x3b]) {
        *(undefined4 *)(param_1 + 0x15) = 2;
      }
    }
  } while ((int)param_1[0x15] != 2);
  cVar2 = FUN_141195480(param_1,param_1 + 0x3d);
  if ((cVar2 == '\0') ||
     ((FUN_141167030(param_1 + 0x3d), *(char *)((longlong)param_1 + 0xc9) == '\0' &&
      (cVar2 = FUN_141195ed0(param_1,(int)param_1[0x41],*(undefined4 *)((longlong)param_1 + 0x20c)),
      cVar2 == '\0')))) {
LAB_141170405:
    *(undefined4 *)(param_1 + 0x15) = 0xffffffff;
    uVar3 = 0xffffffff;
  }
  else {
    *(undefined4 *)(param_1 + 0x15) = 3;
LAB_1411703ea:
    uVar3 = (undefined4)param_1[0x15];
  }
  return uVar3;
}


