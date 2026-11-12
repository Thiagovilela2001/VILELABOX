==== FUN_141187f60 @ 141187f60 (prof 0)

undefined1 FUN_141187f60(longlong param_1,undefined8 param_2)

{
  double dVar1;
  undefined8 uVar2;
  undefined8 uVar3;
  
  dVar1 = (double)FUN_141165dc0(param_1 + 0x1188,DAT_1434e55e8,param_2);
  uVar3 = DAT_145415378;
  FUN_141165eb0(param_1 + 0x1188,0,0,dVar1 * dVar1,DAT_145415378,dVar1 * DAT_1454153e8,dVar1 * dVar1
                ,param_2);
  FUN_141165da0(param_1 + 0x1188);
  uVar2 = FUN_141165dc0(param_1 + 0x11c0,DAT_1435380c8,param_2);
  FUN_141165e30(param_1 + 0x11c0,0,uVar2,uVar3,uVar2,param_2);
  FUN_141165da0(param_1 + 0x11c0);
  return 1;
}


==== FUN_141187a50 @ 141187a50 (prof 0)

undefined1 FUN_141187a50(longlong param_1,undefined8 param_2)

{
  double dVar1;
  undefined8 uVar2;
  undefined8 uVar3;
  
  dVar1 = (double)FUN_141165dc0(param_1 + 0x470,DAT_143538340,param_2);
  uVar3 = DAT_145415378;
  FUN_141165eb0(param_1 + 0x470,0,0,dVar1 * dVar1,DAT_145415378,dVar1 * DAT_1454153e8,dVar1 * dVar1,
                param_2);
  FUN_141165da0(param_1 + 0x470);
  uVar2 = FUN_141165dc0(param_1 + 0x4a8,DAT_1435380c8,param_2);
  FUN_141165e30(param_1 + 0x4a8,0,uVar2,uVar3,uVar2,param_2);
  FUN_141165da0(param_1 + 0x4a8);
  return 1;
}


==== FUN_141165dc0 @ 141165dc0 (prof 1)

double FUN_141165dc0(undefined8 param_1,double param_2,double param_3)

{
  double dVar1;
  
  param_3 = param_3 + param_3;
  dVar1 = (double)FUN_14279adf0(param_2 / param_3);
  return dVar1 * param_3;
}


==== FUN_141165da0 @ 141165da0 (prof 1)

void FUN_141165da0(longlong param_1)

{
  *(undefined8 *)(param_1 + 0x28) = 0;
  *(undefined8 *)(param_1 + 0x30) = 0;
  return;
}


==== FUN_141165e30 @ 141165e30 (prof 1)

void FUN_141165e30(double *param_1,double param_2,double param_3,double param_4,double param_5,
                  double param_6)

{
  double dVar1;
  
  param_6 = DAT_145415378 / param_6;
  dVar1 = DAT_145415378 / (param_6 * param_5 + param_4 + param_4);
  *param_1 = (param_3 * param_6 + param_2 + param_2) * dVar1;
  param_1[1] = (param_3 * param_6 - (param_2 + param_2)) * dVar1;
  param_1[3] = (param_6 * param_5 - (param_4 + param_4)) * dVar1;
  return;
}


==== FUN_141165eb0 @ 141165eb0 (prof 1)

void FUN_141165eb0(double *param_1,double param_2,double param_3,double param_4,double param_5,
                  double param_6,double param_7,double param_8)

{
  double dVar1;
  double dVar2;
  double dVar3;
  double dVar4;
  double dVar5;
  
  param_5 = DAT_1454154c0 * param_5;
  param_2 = param_2 * DAT_1454154c0;
  param_8 = DAT_145415378 / param_8;
  dVar5 = DAT_145415378 / (param_8 * (param_8 * param_7 + param_6 + param_6) + param_5);
  dVar1 = param_8 * param_4 * param_8;
  *param_1 = (param_8 * (param_8 * param_4 + param_3 + param_3) + param_2) * dVar5;
  dVar3 = dVar1 - param_2;
  dVar2 = param_8 * param_7 * param_8;
  dVar4 = dVar2 - param_5;
  param_1[2] = (-(param_8 * (param_3 + param_3)) + dVar1 + param_2) * dVar5;
  param_1[1] = (dVar3 + dVar3) * dVar5;
  param_1[4] = (-(param_8 * (param_6 + param_6)) + dVar2 + param_5) * dVar5;
  param_1[3] = (dVar4 + dVar4) * dVar5;
  return;
}


