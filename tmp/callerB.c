==== FUN_141195ed0 @ 141195ed0 (prof 0)

bool FUN_141195ed0(longlong param_1,int param_2,int param_3)

{
  int iVar1;
  int iVar2;
  int iVar3;
  char cVar4;
  int iVar5;
  longlong lVar6;
  undefined4 uVar7;
  double dVar8;
  double dVar9;
  double dVar10;
  undefined8 uVar11;
  double dVar12;
  double dVar13;
  
  iVar1 = *(int *)(param_1 + 0x78);
  iVar5 = *(int *)(param_1 + 0x7c);
  iVar2 = *(int *)(param_1 + 0x80);
  iVar3 = *(int *)(param_1 + 0xc0);
  dVar8 = *(double *)(param_1 + 0x98);
  dVar9 = *(double *)(param_1 + 0xa0);
  uVar11 = *(undefined8 *)(param_1 + 0x88);
  dVar12 = *(double *)(param_1 + 0xb0);
  dVar13 = *(double *)(param_1 + 0x90);
  cVar4 = FUN_141170590();
  if (cVar4 != '\0') {
    dVar10 = DAT_145415608;
    lVar6 = FUN_1411667d0(param_1 + 0xa80,(int)(dVar13 * DAT_145415608),0);
    if (lVar6 != 0) {
      uVar7 = 0;
      lVar6 = FUN_141166910(param_1 + 0xa80,0,0);
      if (lVar6 != 0) {
        lVar6 = FUN_1411667d0(param_1 + 0xaa0,CONCAT44(uVar7,(int)(dVar8 * dVar10)),0);
        if (lVar6 != 0) {
          uVar7 = 0;
          lVar6 = FUN_141166910(param_1 + 0xaa0,0,0);
          if (lVar6 != 0) {
            lVar6 = FUN_1411667d0(param_1 + 0xac0,CONCAT44(uVar7,(int)(dVar9 * dVar10)),0);
            if (lVar6 != 0) {
              lVar6 = FUN_141166910(param_1 + 0xac0,0,0);
              if (((lVar6 != 0) &&
                  (cVar4 = FUN_14118ad30(param_1 + 0x768,dVar13 * DAT_1454152d0,uVar11,iVar1),
                  cVar4 != '\0')) &&
                 (dVar10 = DAT_145415250,
                 cVar4 = FUN_141191270(param_1 + 0x850,dVar8 * DAT_145415250,uVar11,iVar5),
                 cVar4 != '\0')) {
                cVar4 = FUN_141191270(param_1 + 0x938,dVar9 * dVar10,uVar11,iVar2);
                if (cVar4 != '\0') {
                  *(double *)(param_1 + 0xb48) = DAT_145415638 / dVar12;
                  *(double *)(param_1 + 0xb50) = (double)param_2 * DAT_145414e88;
                  dVar10 = (double)iVar3;
                  *(undefined4 *)(param_1 + 0xb40) = 0;
                  *(undefined8 *)(param_1 + 0xb60) = 0;
                  *(undefined8 *)(param_1 + 0xb58) = 0;
                  *(undefined8 *)(param_1 + 0xb68) = 0;
                  *(undefined4 *)(param_1 + 0xb70) = 0;
                  *(undefined4 *)(param_1 + 0xb80) = 1;
                  *(undefined8 *)(param_1 + 0xb88) = 0;
                  cVar4 = FUN_141173370(param_1,dVar10 / (double)iVar1,dVar13,dVar10 / (double)iVar5
                                        ,dVar8,dVar10 / (double)iVar2,dVar9,dVar12,param_2,param_3);
                  if (cVar4 != '\0') {
                    iVar1 = *(int *)(param_1 + 0x1180);
                    *(undefined1 *)(param_1 + 200) = 1;
                    iVar5 = (iVar1 - param_3) + 1;
                    if (iVar5 < 0x20) {
                      *(undefined1 *)(param_1 + 200) = 0;
                    }
                    *(int *)(param_1 + 0xb90) = param_2;
                    *(int *)(param_1 + 0xb98) = param_3;
                    iVar2 = iVar1 * 4;
                    *(int *)(param_1 + 0xb94) = param_3;
                    iVar5 = (int)((iVar5 >> 0x1f & 3U) + iVar5) >> 2;
                    lVar6 = FUN_1411667d0(param_1 + 0xba0,iVar2,1);
                    if (lVar6 != 0) {
                      lVar6 = FUN_1411667d0(param_1 + 0xbc0,iVar2,1);
                      if (lVar6 != 0) {
                        *(undefined4 *)(param_1 + 0xde0) = 0;
                        lVar6 = FUN_1411667d0(param_1 + 0xbe0,iVar2,1);
                        if (lVar6 != 0) {
                          lVar6 = FUN_1411667d0(param_1 + 0xc00,iVar1,1);
                          if (lVar6 != 0) {
                            cVar4 = FUN_1411697a0(param_1 + 0xe28,iVar5 * 4,0x400,1);
                            if (cVar4 != '\0') {
                              lVar6 = FUN_1411667d0(param_1 + 0xc60,iVar1,1);
                              if (lVar6 != 0) {
                                lVar6 = FUN_1411667d0(param_1 + 0xc20,iVar1,1);
                                if (lVar6 != 0) {
                                  lVar6 = FUN_141169190(param_1 + 0xdc0,iVar1,1);
                                  if (lVar6 != 0) {
                                    lVar6 = FUN_141169190(param_1 + 0xd80,iVar5 << 4,1);
                                    if (lVar6 != 0) {
                                      lVar6 = FUN_1411667d0(param_1 + 0xca0,iVar1,1);
                                      if (lVar6 != 0) {
                                        lVar6 = FUN_141169190(param_1 + 0xda0,iVar5 << 4,1);
                                        if (lVar6 != 0) {
                                          lVar6 = FUN_1411667d0(param_1 + 0xc40,iVar2,1);
                                          if (lVar6 != 0) {
                                            lVar6 = FUN_1411667d0(param_1 + 0xcc0,iVar1,1);
                                            if (lVar6 != 0) {
                                              lVar6 = FUN_1411667d0(param_1 + 0xce0,iVar1,1);
                                              if (lVar6 != 0) {
                                                lVar6 = FUN_1411667d0(param_1 + 0xd00,iVar1,1);
                                                if (lVar6 != 0) {
                                                  cVar4 = FUN_1411697a0(param_1 + 0xe40,1,1,1);
                                                  if (cVar4 != '\0') {
                                                    cVar4 = FUN_1411697a0(param_1 + 0xe10,iVar5 * 2,
                                                                          0x800,1);
                                                    if (cVar4 != '\0') {
                                                      *(undefined4 *)(param_1 + 0xde4) = 0;
                                                      lVar6 = FUN_1411667d0(param_1 + 0xd20,iVar2,1)
                                                      ;
                                                      if (lVar6 != 0) {
                                                        cVar4 = FUN_1411697a0(param_1 + 0xdf8,0x48,
                                                                              (int)((dVar10 * 
                                                  DAT_145415198 -
                                                  (double)(int)(dVar8 * DAT_1435384d8)) *
                                                  DAT_145414f10),1);
                                                  if (cVar4 != '\0') {
                                                    *(undefined8 *)(param_1 + 0xde8) = 0;
                                                    lVar6 = FUN_1411667d0(param_1 + 0xd40,iVar1,1);
                                                    if (lVar6 != 0) {
                                                      lVar6 = FUN_1411667d0(param_1 + 0xd60,
                                                                            iVar1 * 8,1);
                                                      if (lVar6 != 0) {
                                                        *(undefined4 *)(param_1 + 0xdf0) = 0;
                                                        *(undefined4 *)(param_1 + 0xb7c) = 0;
                                                        cVar4 = FUN_141187f60(param_1,dVar9);
                                                        if ((((cVar4 != '\0') &&
                                                             (cVar4 = FUN_141187a50(param_1,dVar9),
                                                             cVar4 != '\0')) &&
                                                            ((cVar4 = FUN_141188560(param_1,dVar9),
                                                             cVar4 != '\0' &&
                                                             ((cVar4 = FUN_141188c50(param_1,dVar9),
                                                              cVar4 != '\0' &&
                                                              (cVar4 = FUN_141189410(param_1,dVar8),
                                                              cVar4 != '\0')))))) &&
                                                           ((cVar4 = FUN_1411898c0(param_1,dVar8),
                                                            cVar4 != '\0' &&
                                                            (((((uVar11 = DAT_145414da0,
                                                                cVar4 = FUN_14118e120(param_1,0,0,
                                                                                      DAT_145414da0,
                                                                                      dVar8),
                                                                cVar4 != '\0' &&
                                                                (cVar4 = FUN_14118cf70(param_1,0,0,
                                                                                       uVar11,dVar8)
                                                                , cVar4 != '\0')) &&
                                                               (cVar4 = FUN_14117e830(param_1,0,0),
                                                               cVar4 != '\0')) &&
                                                              ((cVar4 = FUN_14117ff60(param_1,dVar8)
                                                               , cVar4 != '\0' &&
                                                               (cVar4 = FUN_14117b170(param_1),
                                                               cVar4 != '\0')))) &&
                                                             ((cVar4 = FUN_14117ac40(param_1),
                                                              cVar4 != '\0' &&
                                                              ((cVar4 = FUN_141194730(param_1),
                                                               cVar4 != '\0' &&
                                                               (cVar4 = FUN_14117a050(param_1,dVar9,
                                                                                      dVar8),
                                                               cVar4 != '\0')))))))))) {
                                                          cVar4 = FUN_14117e1b0(param_1,dVar13);
                                                          return cVar4 != '\0';
                                                        }
                                                      }
                                                    }
                                                  }
                                                  }
                                                  }
                                                  }
                                                }
                                              }
                                            }
                                          }
                                        }
                                      }
                                    }
                                  }
                                }
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  return false;
}


