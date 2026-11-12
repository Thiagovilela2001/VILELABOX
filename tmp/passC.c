==== FUN_1411965b0 @ 1411965b0 (prof 0)

bool FUN_1411965b0(longlong param_1,longlong param_2)

{
  int *piVar1;
  undefined1 auVar2 [32];
  int iVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  undefined1 (*pauVar7) [32];
  undefined1 (*pauVar8) [32];
  undefined1 (*pauVar9) [32];
  double dVar10;
  undefined1 auVar11 [32];
  char cVar12;
  int iVar13;
  int iVar14;
  int iVar15;
  longlong lVar16;
  longlong lVar17;
  longlong lVar18;
  undefined1 (*pauVar19) [32];
  int *piVar20;
  uint uVar21;
  int iVar22;
  int iVar23;
  uint uVar24;
  int iVar25;
  uint uVar26;
  ulonglong uVar28;
  double dVar29;
  double dVar30;
  undefined1 auVar31 [32];
  undefined1 auVar32 [32];
  undefined1 auVar33 [32];
  undefined1 auVar34 [64];
  undefined8 uVar35;
  undefined1 auVar36 [64];
  double dVar37;
  double dVar38;
  double dVar39;
  double dVar40;
  undefined8 uVar41;
  undefined8 uVar42;
  int local_res8;
  undefined8 uVar43;
  undefined8 uVar44;
  undefined8 in_stack_fffffffffffffef8;
  undefined4 uVar45;
  undefined4 uVar46;
  undefined8 uVar47;
  undefined8 in_stack_ffffffffffffff08;
  undefined4 uVar48;
  int local_e4;
  ulonglong uVar27;
  
  uVar48 = (undefined4)((ulonglong)in_stack_ffffffffffffff08 >> 0x20);
  uVar45 = (undefined4)((ulonglong)in_stack_fffffffffffffef8 >> 0x20);
  auVar34 = ZEXT864(*(ulonglong *)(param_1 + 0xb0));
  auVar36 = ZEXT864(*(ulonglong *)(param_1 + 0x98));
  dVar29 = *(double *)(param_1 + 0x88);
  uVar27 = 0;
  pauVar7 = *(undefined1 (**) [32])(param_1 + 0x1120);
  uVar26 = 0;
  iVar6 = 0;
  local_res8 = 0;
  local_e4 = 0;
  uVar41 = *(undefined8 *)(param_1 + 0xa0);
  uVar42 = *(undefined8 *)(param_1 + 0x90);
  iVar15 = *(int *)(param_1 + 0xc0);
  if (((pauVar7 != (undefined1 (*) [32])0x0) &&
      (pauVar8 = *(undefined1 (**) [32])(param_1 + 0x1140), pauVar8 != (undefined1 (*) [32])0x0)) &&
     (pauVar9 = *(undefined1 (**) [32])(param_1 + 0x1160), pauVar9 != (undefined1 (*) [32])0x0)) {
    iVar3 = *(int *)(param_1 + 0xb98);
    iVar4 = *(int *)(param_1 + 0xb90);
    iVar5 = *(int *)(param_1 + 0x1180);
    dVar37 = *(double *)(param_1 + 0xb48);
    iVar14 = *(int *)(param_2 + 0x18);
    cVar12 = FUN_14118ae80(param_1 + 0x768,param_2,param_1 + 0xa20);
    if ((((cVar12 != '\0') &&
         (cVar12 = FUN_1411913c0(param_1 + 0x850,param_2,param_1 + 0xa40), cVar12 != '\0')) &&
        ((cVar12 = FUN_1411913c0(param_1 + 0x938,param_2,param_1 + 0xa60), cVar12 != '\0' &&
         ((lVar16 = FUN_141166cc0(param_1 + 0xa80,param_1 + 0xa20), lVar16 != 0 &&
          (lVar16 = FUN_141166cc0(param_1 + 0xaa0,param_1 + 0xa40), lVar16 != 0)))))) &&
       (lVar16 = FUN_141166cc0(param_1 + 0xac0,param_1 + 0xa60), dVar30 = DAT_145415208,
       dVar10 = DAT_1435384e0, lVar16 != 0)) {
      iVar25 = *(int *)(param_1 + 0xa38) + *(int *)(param_1 + 0xb68);
      *(int *)(param_1 + 0xb7c) = *(int *)(param_1 + 0xb7c) + iVar14;
      dVar40 = (double)iVar4;
      dVar39 = dVar40 * DAT_145414e88;
      iVar13 = (int)((-(*(double *)(param_1 + 0xb48) * (double)(*(int *)(param_1 + 0xb98) + -1)) +
                     (dVar10 - dVar39)) * (dVar30 / *(double *)(param_1 + 0xb48)));
      dVar29 = (double)iVar14 / dVar29;
      dVar10 = dVar29 + *(double *)(param_1 + 0xb58);
      iVar14 = *(int *)(param_1 + 0xb98) + iVar13 * 4;
      *(int *)(param_1 + 0xb74) = iVar13;
      iVar13 = iVar14 + -1;
      *(int *)(param_1 + 0xb6c) = *(int *)(param_1 + 0xb6c) + *(int *)(param_1 + 0xa58);
      *(int *)(param_1 + 0xb70) = *(int *)(param_1 + 0xb70) + *(int *)(param_1 + 0xa78);
      *(double *)(param_1 + 0xb60) = dVar29 + *(double *)(param_1 + 0xb60);
      dVar30 = DAT_145415378;
      *(double *)(param_1 + 0xb58) = dVar10;
      *(int *)(param_1 + 0xb78) = iVar13;
      *(int *)(param_1 + 0xb68) = iVar25;
      dVar38 = DAT_1454154c0;
      dVar30 = dVar30 / dVar37;
      dVar29 = dVar10;
      if (*(int *)(param_1 + 0xb80) == 1) {
        dVar29 = dVar10 - dVar39;
      }
      if ((dVar37 * (double)iVar13 + dVar39 <= dVar10) &&
         (*(int *)(param_1 + 0xde0) + iVar13 <= iVar5)) {
        lVar16 = (longlong)iVar14;
        *(undefined4 *)(param_1 + 0xb40) = 1;
        iVar14 = *(int *)(pauVar8[-1] + lVar16 * 4 + 0x1c) + -1;
        iVar23 = *(int *)(pauVar9[-1] + lVar16 * 4 + 0x1c) + -1;
        iVar22 = *(int *)(pauVar7[-1] + lVar16 * 4 + 0x1c) + -1;
        iVar6 = iVar25;
        if (iVar22 < iVar25) {
          lVar16 = FUN_141166bc0(param_1 + 0xae0,
                                 *(longlong *)(param_1 + 0xa80) + (longlong)iVar22 * 8,
                                 iVar25 - iVar22);
          if (lVar16 == 0) {
            return false;
          }
          lVar16 = FUN_141166910(param_1 + 0xa80,iVar22,0);
          iVar6 = iVar22;
          if (lVar16 == 0) {
            return false;
          }
        }
        local_res8 = *(int *)(param_1 + 0xb6c);
        if (((iVar14 < local_res8) && (iVar14 < *(int *)(param_1 + 0xab8))) &&
           (local_res8 <= *(int *)(param_1 + 0xab8))) {
          lVar16 = FUN_141166bc0(param_1 + 0xb00,
                                 *(longlong *)(param_1 + 0xaa0) + (longlong)iVar14 * 8,
                                 local_res8 - iVar14);
          if (lVar16 == 0) {
            return false;
          }
          lVar16 = FUN_141166910(param_1 + 0xaa0,iVar14,0);
          local_res8 = iVar14;
          if (lVar16 == 0) {
            return false;
          }
        }
        local_e4 = *(int *)(param_1 + 0xb70);
        if (((iVar23 < local_e4) && (iVar23 < *(int *)(param_1 + 0xad8))) &&
           (local_e4 <= *(int *)(param_1 + 0xad8))) {
          lVar16 = FUN_141166bc0(param_1 + 0xb20,
                                 *(longlong *)(param_1 + 0xac0) + (longlong)iVar23 * 8,
                                 local_e4 - iVar23);
          if (lVar16 == 0) {
            return false;
          }
          lVar16 = FUN_141166910(param_1 + 0xac0,iVar23,0);
          local_e4 = iVar23;
          if (lVar16 == 0) {
            return false;
          }
        }
        dVar10 = *(double *)(param_1 + 0xb58) -
                 ((double)*(int *)(param_1 + 0xb74) * dVar37 * dVar38 +
                 dVar37 * (double)(iVar3 + -1) + dVar39);
        *(double *)(param_1 + 0xb50) = dVar10;
        *(double *)(param_1 + 0xb58) = dVar10;
      }
      if (iVar15 <= *(int *)(param_1 + 0xb7c)) {
        if (*(int *)(param_1 + 0xb40) == 0) {
          iVar6 = *(int *)(param_1 + 0xb68);
          local_res8 = *(int *)(param_1 + 0xb6c);
          local_e4 = *(int *)(param_1 + 0xb70);
          lVar16 = FUN_141166910(param_1 + 0xa80,iVar6,0);
          if (lVar16 == 0) {
            return false;
          }
          lVar16 = FUN_141166910(param_1 + 0xaa0,local_res8,0);
          if (lVar16 == 0) {
            return false;
          }
          lVar16 = FUN_141166910(param_1 + 0xac0,local_e4,0);
          if (lVar16 == 0) {
            return false;
          }
          dVar37 = *(double *)(param_1 + 0xb60) -
                   ((double)*(int *)(param_1 + 0xb74) * dVar37 * dVar38 + dVar40);
          *(double *)(param_1 + 0xb50) = dVar37;
          *(double *)(param_1 + 0xb58) = dVar37;
          iVar13 = (int)(dVar29 * dVar30);
        }
        *(undefined4 *)(param_1 + 0xb40) = 2;
      }
      if (((*(int *)(param_1 + 0xb40) < 1) || (iVar13 < 1)) || (iVar5 <= *(int *)(param_1 + 0xde0)))
      {
        if (*(int *)(param_1 + 0xb40) != 2) {
          return true;
        }
        cVar12 = FUN_141197890(param_1,iVar5,iVar3);
        return cVar12 != '\0';
      }
      iVar15 = FUN_1411c5cb0(param_1,param_1 + 0xac0,*(undefined4 *)(param_1 + 0xdf0));
      *(int *)(param_1 + 0xdf0) = iVar15;
      if (-1 < iVar15) {
        iVar15 = *(int *)(param_1 + 0xde0) + iVar13;
        if (iVar5 < iVar15) {
          iVar13 = iVar5 - *(int *)(param_1 + 0xde0);
          iVar15 = iVar5;
        }
        *(int *)(param_1 + 0xde0) = iVar15;
        uVar35 = CONCAT44(uVar45,iVar13);
        cVar12 = FUN_141188050(param_1,param_1 + 0xac0,auVar34._0_8_,iVar4,uVar41,param_1 + 0x1160,
                               uVar35,*(undefined8 *)(param_1 + 0xb88));
        uVar45 = (undefined4)((ulonglong)uVar35 >> 0x20);
        if (cVar12 != '\0') {
          FUN_142db0e80(*(longlong *)(param_1 + 0xc00) +
                        (longlong)(*(int *)(param_1 + 0xde0) - *(int *)(param_1 + 0x1210)) * 8,
                        *(undefined8 *)(param_1 + 0x11f8),(longlong)*(int *)(param_1 + 0x1210) << 3)
          ;
          FUN_142db0e80(*(longlong *)(param_1 + 0xc40) +
                        (longlong)(*(int *)(param_1 + 0xde0) * 4 - *(int *)(param_1 + 0x1230)) * 8,
                        *(undefined8 *)(param_1 + 0x1218),(longlong)*(int *)(param_1 + 0x1230) << 3)
          ;
          FUN_142db0e80(*(longlong *)(param_1 + 0xc20) +
                        (longlong)(*(int *)(param_1 + 0xde0) - *(int *)(param_1 + 0x1250)) * 8,
                        *(undefined8 *)(param_1 + 0x1238),(longlong)*(int *)(param_1 + 0x1250) << 3)
          ;
          uVar35 = CONCAT44(uVar45,iVar13);
          *(undefined8 *)(param_1 + 0xb88) =
               *(undefined8 *)
                (*(longlong *)(param_1 + 0xc00) + -8 + (longlong)*(int *)(param_1 + 0xde0) * 8);
          cVar12 = FUN_141187b40(param_1,param_1 + 0xac0,auVar34._0_8_,iVar4,uVar41,param_1 + 0x1160
                                 ,uVar35,(undefined8 *)(param_1 + 0xc80));
          uVar45 = (undefined4)((ulonglong)uVar35 >> 0x20);
          if (cVar12 != '\0') {
            FUN_142db0e80(*(longlong *)(param_1 + 0xca0) +
                          (longlong)(*(int *)(param_1 + 0xde0) - *(int *)(param_1 + 0xc98)) * 8,
                          *(undefined8 *)(param_1 + 0xc80),(longlong)*(int *)(param_1 + 0xc98) << 3)
            ;
            uVar35 = CONCAT44(uVar45,iVar13);
            cVar12 = FUN_1411885f0(param_1,param_1 + 0xac0,auVar34._0_8_,iVar4,uVar41,
                                   param_1 + 0x1160,uVar35);
            if (cVar12 != '\0') {
              uVar35 = CONCAT44((int)((ulonglong)uVar35 >> 0x20),iVar13);
              cVar12 = FUN_141188d40(param_1,param_1 + 0xac0,auVar34._0_8_,iVar4,uVar41,
                                     param_1 + 0x1160,uVar35);
              uVar45 = (undefined4)((ulonglong)uVar35 >> 0x20);
              if (cVar12 != '\0') {
                FUN_142db0e80(*(longlong *)(param_1 + 0xba0) +
                              (longlong)(*(int *)(param_1 + 0xde0) * 4 - *(int *)(param_1 + 0x3f0))
                              * 8,*(undefined8 *)(param_1 + 0x3d8),
                              (longlong)*(int *)(param_1 + 0x3f0) << 3);
                FUN_142db0e80(*(longlong *)(param_1 + 0xd20) +
                              (longlong)(*(int *)(param_1 + 0xde0) * 4 - *(int *)(param_1 + 0x1350))
                              * 8,*(undefined8 *)(param_1 + 0x1338),
                              (longlong)*(int *)(param_1 + 0x1350) << 3);
                lVar16 = param_1 + 0x1140;
                uVar35 = CONCAT44(uVar45,iVar13);
                cVar12 = FUN_141189500(param_1,param_1 + 0xaa0,auVar34._0_8_,iVar4,auVar36._0_8_,
                                       lVar16,uVar35);
                uVar46 = (undefined4)((ulonglong)uVar35 >> 0x20);
                uVar45 = (undefined4)((ulonglong)lVar16 >> 0x20);
                if (cVar12 != '\0') {
                  FUN_142db0e80(*(longlong *)(param_1 + 0xcc0) +
                                (longlong)(*(int *)(param_1 + 0xde0) - *(int *)(param_1 + 0x1400)) *
                                8,*(undefined8 *)(param_1 + 0x13e8),
                                (longlong)*(int *)(param_1 + 0x1400) << 3);
                  cVar12 = FUN_141189ab0(param_1,param_1 + 0xaa0,auVar34._0_8_,auVar36._0_8_,
                                         param_1 + 0x1140,CONCAT44(uVar45,iVar13));
                  if (cVar12 != '\0') {
                    FUN_142db0e80(*(longlong *)(param_1 + 0xc60) +
                                  (longlong)(*(int *)(param_1 + 0xde0) - *(int *)(param_1 + 0x1538))
                                  * 8,*(undefined8 *)(param_1 + 0x1520),
                                  (longlong)*(int *)(param_1 + 0x1538) << 3);
                    uVar35 = CONCAT44(uVar46,iVar13);
                    cVar12 = FUN_1411711f0(param_1,param_1 + 0xaa0,auVar34._0_8_,iVar4,auVar36._0_8_
                                           ,param_1 + 0x1140,uVar35);
                    uVar45 = (undefined4)((ulonglong)uVar35 >> 0x20);
                    if (cVar12 != '\0') {
                      FUN_142db0e80(*(longlong *)(param_1 + 0xd40) +
                                    (longlong)
                                    (*(int *)(param_1 + 0xde0) - *(int *)(param_1 + 0x1558)) * 8,
                                    *(undefined8 *)(param_1 + 0x1540),
                                    (longlong)*(int *)(param_1 + 0x1558) << 3);
                      uVar43 = CONCAT44(uVar45,iVar13);
                      uVar35 = auVar36._0_8_;
                      cVar12 = FUN_141171510(param_1,param_1 + 0xaa0,auVar34._0_8_,iVar4,uVar35,
                                             param_1 + 0x1140,uVar43);
                      uVar45 = (undefined4)((ulonglong)uVar35 >> 0x20);
                      uVar46 = (undefined4)((ulonglong)uVar43 >> 0x20);
                      if (cVar12 != '\0') {
                        FUN_142db0e80(*(longlong *)(param_1 + 0xbc0) +
                                      (longlong)
                                      (*(int *)(param_1 + 0xde0) * 4 - *(int *)(param_1 + 0x1578)) *
                                      8,*(undefined8 *)(param_1 + 0x1560),
                                      (longlong)*(int *)(param_1 + 0x1578) << 3);
                        FUN_142db0e80(*(longlong *)(param_1 + 0xbe0) +
                                      (longlong)
                                      (*(int *)(param_1 + 0xde0) * 4 - *(int *)(param_1 + 0x1598)) *
                                      8,*(undefined8 *)(param_1 + 0x1580),
                                      (longlong)*(int *)(param_1 + 0x1598) << 3);
                        uVar44 = CONCAT44(uVar48,iVar13);
                        uVar43 = CONCAT44(uVar46,*(undefined4 *)(param_1 + 0xb80));
                        uVar35 = CONCAT44(uVar45,*(undefined4 *)(param_1 + 0xb98));
                        cVar12 = FUN_14118e450(param_1,param_1 + 0xaa0,auVar34._0_8_,iVar4,uVar35,
                                               auVar36._0_8_,uVar43,param_1 + 0x1140,uVar44);
                        uVar45 = (undefined4)((ulonglong)uVar35 >> 0x20);
                        uVar48 = (undefined4)((ulonglong)uVar43 >> 0x20);
                        uVar46 = (undefined4)((ulonglong)uVar44 >> 0x20);
                        if (cVar12 != '\0') {
                          iVar14 = *(int *)(param_1 + 0x16f0);
                          iVar25 = ((int)((double)((*(int *)(param_1 + 0xde0) -
                                                   *(int *)(param_1 + 0xb94)) + 1) * DAT_1435384e8)
                                    * -0x10 + 1) - iVar14;
                          iVar15 = iVar25 + -1;
                          dVar29 = DAT_1435384e8;
                          FUN_142db0e80(*(longlong *)(param_1 + 0xd80) + (longlong)iVar15,
                                        *(undefined8 *)(param_1 + 0x16d8),(longlong)iVar14);
                          if ((iVar25 < 1) || (iVar15 + iVar14 < 1)) {
                            lVar16 = FUN_141169190(param_1 + 0xd80,1,1);
                            if (lVar16 == 0) {
                              return false;
                            }
                          }
                          else {
                            FUN_142db0e80((longlong)iVar15 + *(longlong *)(param_1 + 0xd80),
                                          *(undefined8 *)(param_1 + 0x16d8),
                                          (longlong)*(int *)(param_1 + 0x16f0));
                          }
                          lVar16 = param_1 + 0x1140;
                          uVar35 = CONCAT44(uVar48,*(undefined4 *)(param_1 + 0xb80));
                          cVar12 = FUN_14118d310(param_1,param_1 + 0xaa0,auVar34._0_8_,iVar4,
                                                 CONCAT44(uVar45,*(undefined4 *)(param_1 + 0xb98)),
                                                 auVar36._0_8_,uVar35,lVar16,CONCAT44(uVar46,iVar13)
                                                );
                          uVar45 = (undefined4)((ulonglong)uVar35 >> 0x20);
                          if (cVar12 != '\0') {
                            iVar14 = *(int *)(param_1 + 0x1960);
                            iVar25 = ((int)((double)((*(int *)(param_1 + 0xde0) -
                                                     *(int *)(param_1 + 0xb94)) + 1) * dVar29) *
                                      -0x10 + 1) - iVar14;
                            iVar15 = iVar25 + -1;
                            FUN_142db0e80(*(longlong *)(param_1 + 0xda0) + (longlong)iVar15,
                                          *(undefined8 *)(param_1 + 0x1948),(longlong)iVar14);
                            if ((iVar25 < 1) || (iVar15 + iVar14 < 1)) {
                              lVar17 = FUN_141169190(param_1 + 0xda0,1,1);
                              if (lVar17 == 0) {
                                return false;
                              }
                            }
                            else {
                              FUN_142db0e80((longlong)iVar15 + *(longlong *)(param_1 + 0xda0),
                                            *(undefined8 *)(param_1 + 0x1948),
                                            (longlong)*(int *)(param_1 + 0x1960));
                            }
                            uVar43 = CONCAT44(uVar45,iVar13);
                            uVar35 = uVar42;
                            cVar12 = FUN_14117e850(param_1,param_1 + 0xa80,auVar34._0_8_,iVar4,
                                                   uVar42,param_1 + 0x1120,uVar43);
                            uVar45 = (undefined4)((ulonglong)uVar35 >> 0x20);
                            uVar48 = (undefined4)((ulonglong)uVar43 >> 0x20);
                            if (cVar12 != '\0') {
                              FUN_142db0e80((longlong)
                                            (*(int *)(param_1 + 0xde0) - *(int *)(param_1 + 0x1990))
                                            + *(longlong *)(param_1 + 0xdc0),
                                            *(undefined8 *)(param_1 + 0x1978),
                                            (longlong)*(int *)(param_1 + 0x1990));
                              uVar35 = auVar36._0_8_;
                              uVar43 = CONCAT44(uVar45,iVar3);
                              cVar12 = FUN_1411803f0(param_1,param_1 + 0xaa0,auVar34._0_8_,0,uVar43,
                                                     uVar35,CONCAT44(uVar48,*(undefined4 *)
                                                                             (param_1 + 0xb40)));
                              if (cVar12 != '\0') {
                                if (*(int *)(param_1 + 0xb80) == 1) {
                                  *(undefined4 *)(param_1 + 0xdec) =
                                       *(undefined4 *)(param_1 + 0x1a34);
                                }
                                *(int *)(param_1 + 0xde8) =
                                     *(int *)(param_1 + 0xde8) + *(int *)(param_1 + 0x1a30);
                                uVar28 = uVar27;
                                do {
                                  lVar17 = FUN_141169a20(param_1 + 0xdf8,uVar28);
                                  if (lVar17 == 0) {
                                    return false;
                                  }
                                  iVar15 = *(int *)(param_1 + 0xde8);
                                  iVar14 = *(int *)(param_1 + 0x1a2c);
                                  lVar18 = FUN_141169a20(param_1 + 0x1a18,uVar28);
                                  if (lVar18 == 0) {
                                    return false;
                                  }
                                  FUN_142db0e80(lVar17 + (longlong)(iVar15 - iVar14) * 8,lVar18,
                                                (longlong)*(int *)(param_1 + 0x1a2c) << 3);
                                  uVar24 = (int)uVar28 + 1;
                                  uVar28 = (ulonglong)uVar24;
                                } while ((int)uVar24 < 0x48);
                                uVar47 = CONCAT44((int)((ulonglong)lVar16 >> 0x20),iVar13);
                                uVar44 = CONCAT44((int)((ulonglong)uVar35 >> 0x20),iVar3);
                                uVar35 = CONCAT44((int)((ulonglong)uVar43 >> 0x20),iVar4);
                                cVar12 = FUN_14117b1b0(param_1,param_1 + 0xaa0,auVar36._0_8_,
                                                       auVar34._0_8_,uVar35,uVar44,param_1 + 0x1140,
                                                       uVar47);
                                uVar45 = (undefined4)((ulonglong)uVar35 >> 0x20);
                                uVar48 = (undefined4)((ulonglong)uVar44 >> 0x20);
                                uVar46 = (undefined4)((ulonglong)uVar47 >> 0x20);
                                if (cVar12 != '\0') {
                                  iVar15 = *(int *)(param_1 + 0x1a80);
                                  iVar14 = *(int *)(param_1 + 0xde0);
                                  iVar25 = *(int *)(param_1 + 0xb94);
                                  uVar28 = uVar27;
                                  if (0 < iVar15) {
                                    do {
                                      lVar16 = FUN_141169a20(param_1 + 0xe10,
                                                             (int)uVar28 +
                                                             ((int)((double)((iVar14 - iVar25) + 1)
                                                                   * dVar29) * -2 - iVar15));
                                      if (lVar16 == 0) {
                                        return false;
                                      }
                                      lVar17 = FUN_141169a20(param_1 + 0x1a70,uVar28);
                                      if (lVar17 == 0) {
                                        return false;
                                      }
                                      FUN_142db0e80(lVar16,lVar17,
                                                    (longlong)*(int *)(param_1 + 0x1a84) << 3);
                                      uVar45 = (undefined4)((ulonglong)uVar35 >> 0x20);
                                      uVar48 = (undefined4)((ulonglong)uVar44 >> 0x20);
                                      uVar46 = (undefined4)((ulonglong)uVar47 >> 0x20);
                                      uVar24 = (int)uVar28 + 1;
                                      uVar28 = (ulonglong)uVar24;
                                    } while ((int)uVar24 < iVar15);
                                  }
                                  uVar43 = CONCAT44(uVar45,iVar4);
                                  cVar12 = FUN_14117ac80(param_1,param_1 + 0xaa0,auVar36._0_8_,
                                                         auVar34._0_8_,uVar43,CONCAT44(uVar48,iVar3)
                                                         ,param_1 + 0x1140,CONCAT44(uVar46,iVar13));
                                  uVar45 = (undefined4)((ulonglong)uVar43 >> 0x20);
                                  uVar35 = auVar36._0_8_;
                                  if (cVar12 != '\0') {
                                    iVar15 = *(int *)(param_1 + 0x1ad0);
                                    iVar14 = (*(int *)(param_1 + 0xde0) - *(int *)(param_1 + 0xb94))
                                             + 1;
                                    uVar28 = uVar27;
                                    if (0 < iVar15) {
                                      do {
                                        lVar16 = FUN_141169a20(param_1 + 0xe28,
                                                               (int)uVar28 +
                                                               ((iVar14 + (iVar14 >> 0x1f & 3U) &
                                                                0xfffffffc) - iVar15));
                                        if (lVar16 == 0) {
                                          return false;
                                        }
                                        lVar17 = FUN_141169a20(param_1 + 0x1ac0,uVar28);
                                        if (lVar17 == 0) {
                                          return false;
                                        }
                                        FUN_142db0e80(lVar16,lVar17,
                                                      (longlong)*(int *)(param_1 + 0x1ad4) << 3);
                                        uVar45 = (undefined4)((ulonglong)uVar43 >> 0x20);
                                        uVar35 = auVar36._0_8_;
                                        uVar24 = (int)uVar28 + 1;
                                        uVar28 = (ulonglong)uVar24;
                                      } while ((int)uVar24 < iVar15);
                                    }
                                    cVar12 = FUN_14117a200(param_1,param_1 + 0xac0,param_1 + 0xaa0,
                                                           auVar34._0_8_,CONCAT44(uVar45,iVar4),
                                                           uVar41,uVar35,param_1 + 0x1160,
                                                           param_1 + 0x1140,iVar13);
                                    uVar45 = (undefined4)((ulonglong)uVar41 >> 0x20);
                                    if (cVar12 != '\0') {
                                      FUN_142db0e80(*(longlong *)(param_1 + 0xce0) +
                                                    ((longlong)*(int *)(param_1 + 0xde0) -
                                                    (longlong)*(int *)(param_1 + 0x1c20)) * 8,
                                                    *(undefined8 *)(param_1 + 0x1c08),
                                                    (longlong)*(int *)(param_1 + 0x1c20) * 8);
                                      FUN_142db0e80(*(longlong *)(param_1 + 0xd00) +
                                                    ((longlong)*(int *)(param_1 + 0xde0) -
                                                    (longlong)*(int *)(param_1 + 0x1c40)) * 8,
                                                    *(undefined8 *)(param_1 + 0x1c28),
                                                    (longlong)*(int *)(param_1 + 0x1c40) * 8);
                                      cVar12 = FUN_14117e3a0(param_1,param_1 + 0xa80,uVar42,
                                                             auVar34._0_8_,param_1 + 0x1120,
                                                             CONCAT44(uVar45,iVar13));
                                      if (cVar12 != '\0') {
                                        FUN_142db0e80(*(longlong *)(param_1 + 0xd60) +
                                                      ((longlong)(*(int *)(param_1 + 0xde0) << 3) -
                                                      (longlong)*(int *)(param_1 + 0x1d78)) * 8,
                                                      *(undefined8 *)(param_1 + 0x1d60),
                                                      (longlong)*(int *)(param_1 + 0x1d78) * 8);
                                        if (*(int *)(param_1 + 0xb40) == 1) {
                                          if (iVar6 < *(int *)(param_1 + 0xb68)) {
                                            lVar16 = FUN_141166a30(param_1 + 0xa80,param_1 + 0xae0);
                                          }
                                          else {
                                            lVar16 = FUN_141166910(param_1 + 0xa80,0,0);
                                          }
                                          if (lVar16 != 0) {
                                            if (local_res8 < *(int *)(param_1 + 0xb6c)) {
                                              lVar16 = FUN_141166a30(param_1 + 0xaa0,param_1 + 0xb00
                                                                    );
                                            }
                                            else {
                                              lVar16 = FUN_141166910(param_1 + 0xaa0,0,0);
                                            }
                                            if (lVar16 != 0) {
                                              if (local_e4 < *(int *)(param_1 + 0xb70)) {
                                                lVar16 = FUN_141166a30(param_1 + 0xac0,
                                                                       param_1 + 0xb20);
                                              }
                                              else {
                                                lVar16 = FUN_141166910(param_1 + 0xac0,0,0);
                                              }
                                              if (lVar16 != 0) {
                                                *(int *)(param_1 + 0xb6c) =
                                                     *(int *)(param_1 + 0xb6c) - local_res8;
                                                *(int *)(param_1 + 0xb70) =
                                                     *(int *)(param_1 + 0xb70) - local_e4;
                                                *(int *)(param_1 + 0xb68) =
                                                     *(int *)(param_1 + 0xb68) - iVar6;
                                                lVar16 = (longlong)iVar13;
                                                *(undefined4 *)(param_1 + 0xb40) = 0;
                                                *(undefined4 *)(param_1 + 0xb90) = 0;
                                                uVar24 = iVar5 + 1;
                                                *(undefined4 *)(param_1 + 0xb98) = 1;
                                                iVar15 = *(int *)(*pauVar7 + lVar16 * 4);
                                                iVar6 = *(int *)(*pauVar8 + lVar16 * 4);
                                                iVar3 = *(int *)(*pauVar9 + lVar16 * 4);
                                                auVar33._4_4_ = iVar3;
                                                auVar33._0_4_ = iVar3;
                                                auVar33._8_4_ = iVar3;
                                                auVar33._12_4_ = iVar3;
                                                auVar33._16_4_ = iVar3;
                                                auVar33._20_4_ = iVar3;
                                                auVar33._24_4_ = iVar3;
                                                auVar33._28_4_ = iVar3;
                                                auVar31._4_4_ = iVar15;
                                                auVar31._0_4_ = iVar15;
                                                auVar31._8_4_ = iVar15;
                                                auVar31._12_4_ = iVar15;
                                                auVar31._16_4_ = iVar15;
                                                auVar31._20_4_ = iVar15;
                                                auVar31._24_4_ = iVar15;
                                                auVar31._28_4_ = iVar15;
                                                auVar32._4_4_ = iVar6;
                                                auVar32._0_4_ = iVar6;
                                                auVar32._8_4_ = iVar6;
                                                auVar32._12_4_ = iVar6;
                                                auVar32._16_4_ = iVar6;
                                                auVar32._20_4_ = iVar6;
                                                auVar32._24_4_ = iVar6;
                                                auVar32._28_4_ = iVar6;
                                                if ((0 < (int)uVar24) && (7 < uVar24)) {
                                                  lVar16 = (longlong)iVar5;
                                                  if (((((undefined1 (*) [32])
                                                         (*pauVar8 + lVar16 * 4) < pauVar9) ||
                                                       ((undefined1 (*) [32])(*pauVar9 + lVar16 * 4)
                                                        < pauVar8)) &&
                                                      (((undefined1 (*) [32])(*pauVar7 + lVar16 * 4)
                                                        < pauVar9 ||
                                                       ((undefined1 (*) [32])(*pauVar9 + lVar16 * 4)
                                                        < pauVar7)))) &&
                                                     (((undefined1 (*) [32])(*pauVar7 + lVar16 * 4)
                                                       < pauVar8 ||
                                                      ((undefined1 (*) [32])(*pauVar8 + lVar16 * 4)
                                                       < pauVar7)))) {
                                                    uVar21 = uVar24 & 0x80000007;
                                                    if ((int)uVar21 < 0) {
                                                      uVar21 = (uVar21 - 1 | 0xfffffff8) + 1;
                                                    }
                                                    pauVar19 = pauVar8;
                                                    uVar28 = uVar27;
                                                    do {
                                                      auVar11 = vpsubd_avx2(*(undefined1 (*) [32])
                                                                             ((longlong)pauVar19 +
                                                                             ((longlong)pauVar7 -
                                                                             (longlong)pauVar8)),
                                                                            auVar31);
                                                      auVar2 = *pauVar19;
                                                      *(undefined1 (*) [32])
                                                       ((longlong)pauVar19 +
                                                       ((longlong)pauVar7 - (longlong)pauVar8)) =
                                                           auVar11;
                                                      auVar11 = vpsubd_avx2(auVar2,auVar32);
                                                      auVar2 = *(undefined1 (*) [32])
                                                                ((longlong)pauVar19 +
                                                                ((longlong)pauVar9 -
                                                                (longlong)pauVar8));
                                                      *pauVar19 = auVar11;
                                                      auVar2 = vpsubd_avx2(auVar2,auVar33);
                                                      uVar26 = (int)uVar27 + 8;
                                                      uVar27 = (ulonglong)uVar26;
                                                      uVar28 = uVar28 + 8;
                                                      *(undefined1 (*) [32])
                                                       ((longlong)pauVar19 +
                                                       ((longlong)pauVar9 - (longlong)pauVar8)) =
                                                           auVar2;
                                                      pauVar19 = pauVar19 + 1;
                                                    } while ((longlong)uVar28 <
                                                             (longlong)(int)(uVar24 - uVar21));
                                                  }
                                                }
                                                lVar16 = (longlong)(int)uVar26;
                                                if (lVar16 < (int)uVar24) {
                                                  piVar20 = (int *)(*pauVar8 + lVar16 * 4);
                                                  lVar16 = (int)uVar24 - lVar16;
                                                  do {
                                                    piVar1 = (int *)((longlong)piVar20 +
                                                                    ((longlong)pauVar7 -
                                                                    (longlong)pauVar8));
                                                    *piVar1 = *piVar1 - iVar15;
                                                    *piVar20 = *piVar20 - iVar6;
                                                    piVar1 = (int *)((longlong)piVar20 +
                                                                    ((longlong)pauVar9 -
                                                                    (longlong)pauVar8));
                                                    *piVar1 = *piVar1 - iVar3;
                                                    piVar20 = piVar20 + 1;
                                                    lVar16 = lVar16 + -1;
                                                  } while (lVar16 != 0);
                                                }
                                                iVar15 = (iVar5 - iVar13) + 1;
                                                lVar16 = (longlong)iVar15;
                                                if (0 < iVar15) {
                                                  piVar20 = (int *)(*pauVar8 + (longlong)iVar13 * 4)
                                                  ;
                                                  do {
                                                    piVar1 = piVar20 + 1;
                                                    *(int *)((longlong)pauVar7 +
                                                             ((longlong)iVar13 * -4 -
                                                             (longlong)pauVar8) + -4 +
                                                            (longlong)piVar1) =
                                                         *(int *)(((longlong)pauVar7 -
                                                                  (longlong)pauVar8) +
                                                                 (longlong)piVar20) + 1;
                                                    piVar20[-(longlong)iVar13] = *piVar20 + 1;
                                                    *(int *)((longlong)piVar20 +
                                                            (longlong)
                                                            ((longlong)pauVar9 +
                                                            ((longlong)iVar13 * -4 -
                                                            (longlong)pauVar8))) =
                                                         *(int *)((undefined1 *)
                                                                  ((longlong)pauVar9 +
                                                                  (-4 - (longlong)pauVar8)) +
                                                                 (longlong)piVar1) + 1;
                                                    lVar16 = lVar16 + -1;
                                                    piVar20 = piVar1;
                                                  } while (lVar16 != 0);
                                                }
                                                *(int *)(param_1 + 0xb80) =
                                                     *(int *)(param_1 + 0xb80) + 1;
                                                return true;
                                              }
                                            }
                                          }
                                        }
                                        else {
                                          if (*(int *)(param_1 + 0xb40) != 2) {
                                            return true;
                                          }
                                          cVar12 = FUN_141197890(param_1,iVar5,iVar3);
                                          if (cVar12 != '\0') {
                                            return true;
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


==== FUN_142db0e80 @ 142db0e80 (prof 1)

void FUN_142db0e80(undefined8 *param_1,undefined8 *param_2,ulonglong param_3)

{
  undefined8 *puVar1;
  undefined8 *puVar2;
  undefined1 auVar3 [32];
  undefined1 auVar4 [32];
  undefined1 auVar5 [32];
  undefined1 auVar6 [32];
  undefined1 uVar7;
  undefined2 uVar8;
  undefined4 uVar9;
  undefined8 uVar10;
  undefined8 uVar11;
  undefined8 uVar12;
  undefined8 uVar13;
  undefined8 uVar14;
  undefined8 uVar15;
  undefined8 uVar16;
  undefined8 uVar17;
  undefined8 uVar18;
  undefined8 uVar19;
  undefined8 uVar20;
  undefined8 uVar21;
  undefined8 uVar22;
  undefined1 (*pauVar23) [32];
  undefined1 (*pauVar24) [32];
  undefined8 *puVar25;
  undefined8 *puVar26;
  undefined1 (*pauVar27) [32];
  undefined1 (*pauVar28) [32];
  ulonglong uVar29;
  longlong lVar30;
  ulonglong uVar31;
  undefined8 uVar32;
  undefined8 uVar33;
  
  switch(param_3) {
  case 0:
    return;
  case 1:
    *(undefined1 *)param_1 = *(undefined1 *)param_2;
    return;
  case 2:
    *(undefined2 *)param_1 = *(undefined2 *)param_2;
    return;
  case 3:
    uVar7 = *(undefined1 *)((longlong)param_2 + 2);
    *(undefined2 *)param_1 = *(undefined2 *)param_2;
    *(undefined1 *)((longlong)param_1 + 2) = uVar7;
    return;
  case 4:
    *(undefined4 *)param_1 = *(undefined4 *)param_2;
    return;
  case 5:
    uVar7 = *(undefined1 *)((longlong)param_2 + 4);
    *(undefined4 *)param_1 = *(undefined4 *)param_2;
    *(undefined1 *)((longlong)param_1 + 4) = uVar7;
    return;
  case 6:
    uVar8 = *(undefined2 *)((longlong)param_2 + 4);
    *(undefined4 *)param_1 = *(undefined4 *)param_2;
    *(undefined2 *)((longlong)param_1 + 4) = uVar8;
    return;
  case 7:
    uVar8 = *(undefined2 *)((longlong)param_2 + 4);
    uVar7 = *(undefined1 *)((longlong)param_2 + 6);
    *(undefined4 *)param_1 = *(undefined4 *)param_2;
    *(undefined2 *)((longlong)param_1 + 4) = uVar8;
    *(undefined1 *)((longlong)param_1 + 6) = uVar7;
    return;
  case 8:
    *param_1 = *param_2;
    return;
  case 9:
    uVar7 = *(undefined1 *)(param_2 + 1);
    *param_1 = *param_2;
    *(undefined1 *)(param_1 + 1) = uVar7;
    return;
  case 10:
    uVar8 = *(undefined2 *)(param_2 + 1);
    *param_1 = *param_2;
    *(undefined2 *)(param_1 + 1) = uVar8;
    return;
  case 0xb:
    uVar8 = *(undefined2 *)(param_2 + 1);
    uVar7 = *(undefined1 *)((longlong)param_2 + 10);
    *param_1 = *param_2;
    *(undefined2 *)(param_1 + 1) = uVar8;
    *(undefined1 *)((longlong)param_1 + 10) = uVar7;
    return;
  case 0xc:
    uVar9 = *(undefined4 *)(param_2 + 1);
    *param_1 = *param_2;
    *(undefined4 *)(param_1 + 1) = uVar9;
    return;
  case 0xd:
    uVar9 = *(undefined4 *)(param_2 + 1);
    uVar7 = *(undefined1 *)((longlong)param_2 + 0xc);
    *param_1 = *param_2;
    *(undefined4 *)(param_1 + 1) = uVar9;
    *(undefined1 *)((longlong)param_1 + 0xc) = uVar7;
    return;
  case 0xe:
    uVar9 = *(undefined4 *)(param_2 + 1);
    uVar8 = *(undefined2 *)((longlong)param_2 + 0xc);
    *param_1 = *param_2;
    *(undefined4 *)(param_1 + 1) = uVar9;
    *(undefined2 *)((longlong)param_1 + 0xc) = uVar8;
    return;
  case 0xf:
    uVar9 = *(undefined4 *)(param_2 + 1);
    uVar8 = *(undefined2 *)((longlong)param_2 + 0xc);
    uVar7 = *(undefined1 *)((longlong)param_2 + 0xe);
    *param_1 = *param_2;
    *(undefined4 *)(param_1 + 1) = uVar9;
    *(undefined2 *)((longlong)param_1 + 0xc) = uVar8;
    *(undefined1 *)((longlong)param_1 + 0xe) = uVar7;
    return;
  }
  if (param_3 < 0x21) {
    uVar10 = param_2[1];
    puVar26 = (undefined8 *)((longlong)param_2 + (param_3 - 0x10));
    uVar11 = *puVar26;
    uVar12 = puVar26[1];
    *param_1 = *param_2;
    param_1[1] = uVar10;
    param_1 = (undefined8 *)((longlong)param_1 + (param_3 - 0x10));
    *param_1 = uVar11;
    param_1[1] = uVar12;
    return;
  }
  puVar26 = (undefined8 *)((longlong)param_2 + param_3);
  if (param_1 <= param_2) {
    puVar26 = param_1;
  }
  if (puVar26 <= param_1) {
    if (DAT_1459ca718 < 3) {
      if ((param_3 < 0x801) || (((byte)DAT_145d35c44 & 2) == 0)) {
        if (0x80 < param_3) {
          lVar30 = ((ulonglong)param_1 & 0xf) - 0x10;
          param_1 = (undefined8 *)((longlong)param_1 - lVar30);
          param_2 = (undefined8 *)((longlong)param_2 - lVar30);
          param_3 = param_3 + lVar30;
          if (0x80 < param_3) {
            do {
              uVar10 = param_2[1];
              uVar11 = param_2[2];
              uVar12 = param_2[3];
              uVar13 = param_2[4];
              uVar32 = param_2[5];
              uVar33 = param_2[6];
              uVar14 = param_2[7];
              *param_1 = *param_2;
              param_1[1] = uVar10;
              param_1[2] = uVar11;
              param_1[3] = uVar12;
              param_1[4] = uVar13;
              param_1[5] = uVar32;
              param_1[6] = uVar33;
              param_1[7] = uVar14;
              uVar10 = param_2[9];
              uVar11 = param_2[10];
              uVar12 = param_2[0xb];
              uVar13 = param_2[0xc];
              uVar32 = param_2[0xd];
              uVar33 = param_2[0xe];
              uVar14 = param_2[0xf];
              param_1[8] = param_2[8];
              param_1[9] = uVar10;
              param_1[10] = uVar11;
              param_1[0xb] = uVar12;
              param_1[0xc] = uVar13;
              param_1[0xd] = uVar32;
              param_1[0xe] = uVar33;
              param_1[0xf] = uVar14;
              param_1 = param_1 + 0x10;
              param_2 = param_2 + 0x10;
              param_3 = param_3 - 0x80;
            } while (0x7f < param_3);
          }
        }
                    /* WARNING: Could not recover jumptable at 0x000142db1396. Too many branches */
                    /* WARNING: Treating indirect jump as call */
        (*(code *)(switchD_141c7d5cb::caseD_3.e_magic +
                  *(uint *)(&DAT_1454175f8 + (param_3 + 0xf >> 4) * 4)))();
        return;
      }
    }
    else if (((param_3 < 0x2001) || (0x180000 < param_3)) || (((byte)DAT_145d35c44 & 2) == 0)) {
      uVar10 = *param_2;
      uVar11 = param_2[1];
      uVar12 = param_2[2];
      uVar13 = param_2[3];
      puVar26 = (undefined8 *)((longlong)param_2 + (param_3 - 0x20));
      uVar32 = *puVar26;
      uVar33 = puVar26[1];
      uVar14 = puVar26[2];
      uVar15 = puVar26[3];
      if (0x100 < param_3) {
        lVar30 = ((ulonglong)param_1 & 0x1f) - 0x20;
        pauVar23 = (undefined1 (*) [32])((longlong)param_1 - lVar30);
        pauVar27 = (undefined1 (*) [32])((longlong)param_2 - lVar30);
        param_3 = param_3 + lVar30;
        if (0x100 < param_3) {
          if (0x180000 < param_3) {
            do {
              uVar29 = param_3;
              pauVar28 = pauVar27;
              pauVar24 = pauVar23;
              auVar3 = pauVar28[1];
              auVar4 = pauVar28[2];
              auVar5 = pauVar28[3];
              auVar6 = vmovntdq_avx(*pauVar28);
              *pauVar24 = auVar6;
              auVar3 = vmovntdq_avx(auVar3);
              pauVar24[1] = auVar3;
              auVar3 = vmovntdq_avx(auVar4);
              pauVar24[2] = auVar3;
              auVar3 = vmovntdq_avx(auVar5);
              pauVar24[3] = auVar3;
              auVar3 = pauVar28[5];
              auVar4 = pauVar28[6];
              auVar5 = pauVar28[7];
              auVar6 = vmovntdq_avx(pauVar28[4]);
              pauVar24[4] = auVar6;
              auVar3 = vmovntdq_avx(auVar3);
              pauVar24[5] = auVar3;
              auVar3 = vmovntdq_avx(auVar4);
              pauVar24[6] = auVar3;
              auVar3 = vmovntdq_avx(auVar5);
              pauVar24[7] = auVar3;
              pauVar23 = pauVar24 + 8;
              pauVar27 = pauVar28 + 8;
              param_3 = uVar29 - 0x100;
            } while (0xff < uVar29 - 0x100);
            uVar31 = uVar29 - 0xe1 & 0xffffffffffffffe0;
            switch(uVar29) {
            case 0x1e1:
            case 0x1e2:
            case 0x1e3:
            case 0x1e4:
            case 0x1e5:
            case 0x1e6:
            case 0x1e7:
            case 0x1e8:
            case 0x1e9:
            case 0x1ea:
            case 0x1eb:
            case 0x1ec:
            case 0x1ed:
            case 0x1ee:
            case 0x1ef:
            case 0x1f0:
            case 0x1f1:
            case 0x1f2:
            case 499:
            case 500:
            case 0x1f5:
            case 0x1f6:
            case 0x1f7:
            case 0x1f8:
            case 0x1f9:
            case 0x1fa:
            case 0x1fb:
            case 0x1fc:
            case 0x1fd:
            case 0x1fe:
            case 0x1ff:
              auVar3 = vmovntdq_avx(*(undefined1 (*) [32])(*pauVar28 + uVar31));
              *(undefined1 (*) [32])(*pauVar24 + uVar31) = auVar3;
            case 0x1c1:
            case 0x1c2:
            case 0x1c3:
            case 0x1c4:
            case 0x1c5:
            case 0x1c6:
            case 0x1c7:
            case 0x1c8:
            case 0x1c9:
            case 0x1ca:
            case 0x1cb:
            case 0x1cc:
            case 0x1cd:
            case 0x1ce:
            case 0x1cf:
            case 0x1d0:
            case 0x1d1:
            case 0x1d2:
            case 0x1d3:
            case 0x1d4:
            case 0x1d5:
            case 0x1d6:
            case 0x1d7:
            case 0x1d8:
            case 0x1d9:
            case 0x1da:
            case 0x1db:
            case 0x1dc:
            case 0x1dd:
            case 0x1de:
            case 0x1df:
            case 0x1e0:
              auVar3 = vmovntdq_avx(*(undefined1 (*) [32])(pauVar28[1] + uVar31));
              *(undefined1 (*) [32])(pauVar24[1] + uVar31) = auVar3;
            case 0x1a1:
            case 0x1a2:
            case 0x1a3:
            case 0x1a4:
            case 0x1a5:
            case 0x1a6:
            case 0x1a7:
            case 0x1a8:
            case 0x1a9:
            case 0x1aa:
            case 0x1ab:
            case 0x1ac:
            case 0x1ad:
            case 0x1ae:
            case 0x1af:
            case 0x1b0:
            case 0x1b1:
            case 0x1b2:
            case 0x1b3:
            case 0x1b4:
            case 0x1b5:
            case 0x1b6:
            case 0x1b7:
            case 0x1b8:
            case 0x1b9:
            case 0x1ba:
            case 0x1bb:
            case 0x1bc:
            case 0x1bd:
            case 0x1be:
            case 0x1bf:
            case 0x1c0:
              auVar3 = vmovntdq_avx(*(undefined1 (*) [32])(pauVar28[2] + uVar31));
              *(undefined1 (*) [32])(pauVar24[2] + uVar31) = auVar3;
            case 0x181:
            case 0x182:
            case 0x183:
            case 0x184:
            case 0x185:
            case 0x186:
            case 0x187:
            case 0x188:
            case 0x189:
            case 0x18a:
            case 0x18b:
            case 0x18c:
            case 0x18d:
            case 0x18e:
            case 399:
            case 400:
            case 0x191:
            case 0x192:
            case 0x193:
            case 0x194:
            case 0x195:
            case 0x196:
            case 0x197:
            case 0x198:
            case 0x199:
            case 0x19a:
            case 0x19b:
            case 0x19c:
            case 0x19d:
            case 0x19e:
            case 0x19f:
            case 0x1a0:
              auVar3 = vmovntdq_avx(*(undefined1 (*) [32])(pauVar28[3] + uVar31));
              *(undefined1 (*) [32])(pauVar24[3] + uVar31) = auVar3;
            case 0x161:
            case 0x162:
            case 0x163:
            case 0x164:
            case 0x165:
            case 0x166:
            case 0x167:
            case 0x168:
            case 0x169:
            case 0x16a:
            case 0x16b:
            case 0x16c:
            case 0x16d:
            case 0x16e:
            case 0x16f:
            case 0x170:
            case 0x171:
            case 0x172:
            case 0x173:
            case 0x174:
            case 0x175:
            case 0x176:
            case 0x177:
            case 0x178:
            case 0x179:
            case 0x17a:
            case 0x17b:
            case 0x17c:
            case 0x17d:
            case 0x17e:
            case 0x17f:
            case 0x180:
              auVar3 = vmovntdq_avx(*(undefined1 (*) [32])(pauVar28[4] + uVar31));
              *(undefined1 (*) [32])(pauVar24[4] + uVar31) = auVar3;
            case 0x141:
            case 0x142:
            case 0x143:
            case 0x144:
            case 0x145:
            case 0x146:
            case 0x147:
            case 0x148:
            case 0x149:
            case 0x14a:
            case 0x14b:
            case 0x14c:
            case 0x14d:
            case 0x14e:
            case 0x14f:
            case 0x150:
            case 0x151:
            case 0x152:
            case 0x153:
            case 0x154:
            case 0x155:
            case 0x156:
            case 0x157:
            case 0x158:
            case 0x159:
            case 0x15a:
            case 0x15b:
            case 0x15c:
            case 0x15d:
            case 0x15e:
            case 0x15f:
            case 0x160:
              auVar3 = vmovntdq_avx(*(undefined1 (*) [32])(pauVar28[5] + uVar31));
              *(undefined1 (*) [32])(pauVar24[5] + uVar31) = auVar3;
            case 0x121:
            case 0x122:
            case 0x123:
            case 0x124:
            case 0x125:
            case 0x126:
            case 0x127:
            case 0x128:
            case 0x129:
            case 0x12a:
            case 299:
            case 300:
            case 0x12d:
            case 0x12e:
            case 0x12f:
            case 0x130:
            case 0x131:
            case 0x132:
            case 0x133:
            case 0x134:
            case 0x135:
            case 0x136:
            case 0x137:
            case 0x138:
            case 0x139:
            case 0x13a:
            case 0x13b:
            case 0x13c:
            case 0x13d:
            case 0x13e:
            case 0x13f:
            case 0x140:
              auVar3 = vmovntdq_avx(*(undefined1 (*) [32])(pauVar28[6] + uVar31));
              *(undefined1 (*) [32])(pauVar24[6] + uVar31) = auVar3;
            default:
              puVar26 = (undefined8 *)(pauVar24[-1] + uVar29);
              *puVar26 = uVar32;
              puVar26[1] = uVar33;
              puVar26[2] = uVar14;
              puVar26[3] = uVar15;
            case 0x100:
              *param_1 = uVar10;
              param_1[1] = uVar11;
              param_1[2] = uVar12;
              param_1[3] = uVar13;
              return;
            }
          }
          do {
            uVar10 = *(undefined8 *)(*pauVar27 + 8);
            uVar11 = *(undefined8 *)(*pauVar27 + 0x10);
            uVar12 = *(undefined8 *)(*pauVar27 + 0x18);
            uVar13 = *(undefined8 *)pauVar27[1];
            uVar32 = *(undefined8 *)(pauVar27[1] + 8);
            uVar33 = *(undefined8 *)(pauVar27[1] + 0x10);
            uVar14 = *(undefined8 *)(pauVar27[1] + 0x18);
            uVar15 = *(undefined8 *)pauVar27[2];
            uVar16 = *(undefined8 *)(pauVar27[2] + 8);
            uVar17 = *(undefined8 *)(pauVar27[2] + 0x10);
            uVar18 = *(undefined8 *)(pauVar27[2] + 0x18);
            uVar19 = *(undefined8 *)pauVar27[3];
            uVar20 = *(undefined8 *)(pauVar27[3] + 8);
            uVar21 = *(undefined8 *)(pauVar27[3] + 0x10);
            uVar22 = *(undefined8 *)(pauVar27[3] + 0x18);
            *(undefined8 *)*pauVar23 = *(undefined8 *)*pauVar27;
            *(undefined8 *)(*pauVar23 + 8) = uVar10;
            *(undefined8 *)(*pauVar23 + 0x10) = uVar11;
            *(undefined8 *)(*pauVar23 + 0x18) = uVar12;
            *(undefined8 *)pauVar23[1] = uVar13;
            *(undefined8 *)(pauVar23[1] + 8) = uVar32;
            *(undefined8 *)(pauVar23[1] + 0x10) = uVar33;
            *(undefined8 *)(pauVar23[1] + 0x18) = uVar14;
            *(undefined8 *)pauVar23[2] = uVar15;
            *(undefined8 *)(pauVar23[2] + 8) = uVar16;
            *(undefined8 *)(pauVar23[2] + 0x10) = uVar17;
            *(undefined8 *)(pauVar23[2] + 0x18) = uVar18;
            *(undefined8 *)pauVar23[3] = uVar19;
            *(undefined8 *)(pauVar23[3] + 8) = uVar20;
            *(undefined8 *)(pauVar23[3] + 0x10) = uVar21;
            *(undefined8 *)(pauVar23[3] + 0x18) = uVar22;
            uVar10 = *(undefined8 *)(pauVar27[4] + 8);
            uVar11 = *(undefined8 *)(pauVar27[4] + 0x10);
            uVar12 = *(undefined8 *)(pauVar27[4] + 0x18);
            uVar13 = *(undefined8 *)pauVar27[5];
            uVar32 = *(undefined8 *)(pauVar27[5] + 8);
            uVar33 = *(undefined8 *)(pauVar27[5] + 0x10);
            uVar14 = *(undefined8 *)(pauVar27[5] + 0x18);
            uVar15 = *(undefined8 *)pauVar27[6];
            uVar16 = *(undefined8 *)(pauVar27[6] + 8);
            uVar17 = *(undefined8 *)(pauVar27[6] + 0x10);
            uVar18 = *(undefined8 *)(pauVar27[6] + 0x18);
            uVar19 = *(undefined8 *)pauVar27[7];
            uVar20 = *(undefined8 *)(pauVar27[7] + 8);
            uVar21 = *(undefined8 *)(pauVar27[7] + 0x10);
            uVar22 = *(undefined8 *)(pauVar27[7] + 0x18);
            *(undefined8 *)pauVar23[4] = *(undefined8 *)pauVar27[4];
            *(undefined8 *)(pauVar23[4] + 8) = uVar10;
            *(undefined8 *)(pauVar23[4] + 0x10) = uVar11;
            *(undefined8 *)(pauVar23[4] + 0x18) = uVar12;
            *(undefined8 *)pauVar23[5] = uVar13;
            *(undefined8 *)(pauVar23[5] + 8) = uVar32;
            *(undefined8 *)(pauVar23[5] + 0x10) = uVar33;
            *(undefined8 *)(pauVar23[5] + 0x18) = uVar14;
            *(undefined8 *)pauVar23[6] = uVar15;
            *(undefined8 *)(pauVar23[6] + 8) = uVar16;
            *(undefined8 *)(pauVar23[6] + 0x10) = uVar17;
            *(undefined8 *)(pauVar23[6] + 0x18) = uVar18;
            *(undefined8 *)pauVar23[7] = uVar19;
            *(undefined8 *)(pauVar23[7] + 8) = uVar20;
            *(undefined8 *)(pauVar23[7] + 0x10) = uVar21;
            *(undefined8 *)(pauVar23[7] + 0x18) = uVar22;
            pauVar23 = pauVar23 + 8;
            pauVar27 = pauVar27 + 8;
            param_3 = param_3 - 0x100;
          } while (0xff < param_3);
        }
      }
                    /* WARNING: Could not recover jumptable at 0x000142db10f2. Too many branches */
                    /* WARNING: Treating indirect jump as call */
      (*(code *)(switchD_141c7d5cb::caseD_3.e_magic +
                *(uint *)(&DAT_1454175b0 + (param_3 + 0x1f >> 5) * 4)))();
      return;
    }
    for (; param_3 != 0; param_3 = param_3 - 1) {
      *(undefined1 *)param_1 = *(undefined1 *)param_2;
      param_2 = (undefined8 *)((longlong)param_2 + 1);
      param_1 = (undefined8 *)((longlong)param_1 + 1);
    }
    return;
  }
  uVar10 = *param_2;
  uVar11 = param_2[1];
  lVar30 = (longlong)param_2 - (longlong)param_1;
  puVar26 = (undefined8 *)((longlong)param_1 + lVar30 + (param_3 - 0x10));
  uVar12 = *puVar26;
  uVar13 = puVar26[1];
  puVar25 = (undefined8 *)((longlong)param_1 + (param_3 - 0x10));
  uVar29 = param_3 - 0x10;
  puVar26 = puVar25;
  uVar32 = uVar12;
  uVar33 = uVar13;
  if (((ulonglong)puVar25 & 0xf) != 0) {
    puVar26 = (undefined8 *)((ulonglong)puVar25 & 0xfffffffffffffff0);
    uVar32 = *(undefined8 *)((longlong)puVar26 + lVar30);
    uVar33 = ((undefined8 *)((longlong)puVar26 + lVar30))[1];
    *puVar25 = uVar12;
    *(undefined8 *)((longlong)param_1 + (param_3 - 8)) = uVar13;
    uVar29 = (longlong)puVar26 - (longlong)param_1;
  }
  uVar31 = uVar29 >> 7;
  if (uVar31 != 0) {
    *puVar26 = uVar32;
    puVar26[1] = uVar33;
    puVar25 = puVar26;
    while( true ) {
      puVar1 = (undefined8 *)((longlong)puVar25 + lVar30 + -0x10);
      uVar12 = puVar1[1];
      puVar26 = (undefined8 *)((longlong)puVar25 + lVar30 + -0x20);
      uVar13 = *puVar26;
      uVar32 = puVar26[1];
      puVar26 = puVar25 + -0x10;
      puVar25[-2] = *puVar1;
      puVar25[-1] = uVar12;
      puVar25[-4] = uVar13;
      puVar25[-3] = uVar32;
      puVar1 = (undefined8 *)((longlong)puVar25 + lVar30 + -0x30);
      uVar12 = puVar1[1];
      puVar2 = (undefined8 *)((longlong)puVar25 + lVar30 + -0x40);
      uVar13 = *puVar2;
      uVar32 = puVar2[1];
      uVar31 = uVar31 - 1;
      puVar25[-6] = *puVar1;
      puVar25[-5] = uVar12;
      puVar25[-8] = uVar13;
      puVar25[-7] = uVar32;
      puVar1 = (undefined8 *)((longlong)puVar25 + lVar30 + -0x50);
      uVar12 = puVar1[1];
      puVar2 = (undefined8 *)((longlong)puVar25 + lVar30 + -0x60);
      uVar13 = *puVar2;
      uVar32 = puVar2[1];
      puVar25[-10] = *puVar1;
      puVar25[-9] = uVar12;
      puVar25[-0xc] = uVar13;
      puVar25[-0xb] = uVar32;
      puVar1 = (undefined8 *)((longlong)puVar25 + lVar30 + -0x70);
      uVar12 = *puVar1;
      uVar13 = puVar1[1];
      uVar32 = *(undefined8 *)((longlong)puVar26 + lVar30);
      uVar33 = ((undefined8 *)((longlong)puVar26 + lVar30))[1];
      if (uVar31 == 0) break;
      puVar25[-0xe] = uVar12;
      puVar25[-0xd] = uVar13;
      *puVar26 = uVar32;
      puVar25[-0xf] = uVar33;
      puVar25 = puVar26;
    }
    puVar25[-0xe] = uVar12;
    puVar25[-0xd] = uVar13;
    uVar29 = uVar29 & 0x7f;
  }
  for (uVar31 = uVar29 >> 4; uVar31 != 0; uVar31 = uVar31 - 1) {
    *puVar26 = uVar32;
    puVar26[1] = uVar33;
    puVar26 = puVar26 + -2;
    uVar32 = *(undefined8 *)((longlong)puVar26 + lVar30);
    uVar33 = ((undefined8 *)((longlong)puVar26 + lVar30))[1];
  }
  if ((uVar29 & 0xf) != 0) {
    *param_1 = uVar10;
    param_1[1] = uVar11;
  }
  *puVar26 = uVar32;
  puVar26[1] = uVar33;
  return;
}


==== FUN_14118ae80 @ 14118ae80 (prof 1)

undefined1 FUN_14118ae80(longlong param_1)

{
  undefined1 uVar1;
  
  if (*(int *)(param_1 + 0xe0) < 2) {
    return 0;
  }
  uVar1 = FUN_14117cbb0();
  return uVar1;
}


==== FUN_141188d40 @ 141188d40 (prof 1)

undefined8
FUN_141188d40(longlong param_1,undefined8 *param_2,double param_3,undefined8 param_4,double param_5,
             undefined8 *param_6,int param_7)

{
  uint uVar1;
  longlong lVar2;
  double *pdVar3;
  longlong lVar4;
  double *pdVar5;
  ulonglong uVar6;
  int iVar7;
  undefined8 uVar8;
  double *pdVar9;
  int *piVar10;
  ulonglong uVar11;
  int iVar12;
  int iVar13;
  double dVar14;
  double dVar15;
  undefined1 local_90 [24];
  undefined4 local_78;
  undefined1 local_70 [72];
  
  FUN_141166770(local_90);
  FUN_141166770(local_70);
  piVar10 = (int *)*param_6;
  if ((piVar10 != (int *)0x0) &&
     (lVar2 = FUN_1411667d0(local_90,*(undefined4 *)(param_2 + 3),0), lVar2 != 0)) {
    FUN_141166570(param_1 + 0x1258,*param_2,lVar2,local_78);
    FUN_141166570(param_1 + 0x1290,lVar2,lVar2,local_78);
    FUN_141166570(param_1 + 0x12c8,lVar2,lVar2,local_78);
    FUN_141166570(param_1 + 0x1300,lVar2,lVar2,local_78);
    dVar15 = param_5 * (DAT_145415378 / param_3) * DAT_145415638 + DAT_1454152a0;
    iVar13 = (int)(param_5 * (DAT_145415378 / param_3) * DAT_145415578 + DAT_1454152a0);
    pdVar3 = (double *)FUN_1411667d0(param_1 + 0x1338,param_7 * 4,1);
    if (pdVar3 != (double *)0x0) {
      uVar6 = 0;
      uVar11 = uVar6;
      pdVar5 = pdVar3;
      if (0 < param_7) {
        do {
          iVar7 = *piVar10;
          uVar1 = (int)uVar11 + 1;
          pdVar9 = pdVar5;
          uVar11 = uVar6;
          if ((int)uVar1 < param_7) {
            iVar12 = piVar10[1];
          }
          else {
            iVar12 = (int)dVar15 + -1 + iVar7;
          }
          do {
            lVar4 = FUN_141166c30(local_70,lVar2 + ((longlong)iVar7 + -1) * 8,iVar13);
            if (lVar4 == 0) goto LAB_14118908c;
            iVar7 = iVar7 + iVar13;
            dVar14 = (double)FUN_1411671d0(local_70);
            *pdVar9 = dVar14;
            uVar11 = uVar11 + 1;
            pdVar9 = pdVar9 + 1;
          } while ((longlong)uVar11 < 3);
          lVar4 = FUN_141166c30(local_70,lVar2 + ((longlong)iVar7 + -1) * 8,iVar12 - iVar7);
          if (lVar4 == 0) goto LAB_14118908c;
          dVar14 = (double)FUN_1411671d0(local_70);
          pdVar5[3] = dVar14;
          piVar10 = piVar10 + 1;
          uVar11 = (ulonglong)uVar1;
          pdVar5 = pdVar5 + 4;
        } while ((int)uVar1 < param_7);
      }
      dVar15 = DAT_1454154c0;
      if (((param_5 <= DAT_143538358) || (DAT_143538360 <= param_5)) &&
         (pdVar5 = pdVar3, uVar11 = uVar6, 0 < *(int *)(param_1 + 0x1350))) {
        do {
          *pdVar5 = dVar15 * *pdVar5;
          uVar1 = (int)uVar11 + 1;
          pdVar5 = pdVar5 + 1;
          uVar11 = (ulonglong)uVar1;
        } while ((int)uVar1 < *(int *)(param_1 + 0x1350));
      }
      iVar13 = (int)(param_7 + (param_7 >> 0x1f & 3U)) >> 2;
      lVar2 = FUN_1411667d0(param_1 + 0x1358,iVar13,1);
      if (lVar2 != 0) {
        pdVar5 = pdVar3;
        uVar11 = uVar6;
        if (0 < iVar13) {
          do {
            *(double *)(lVar2 + uVar11 * 8) =
                 *(double *)(uVar6 + 8 + (longlong)pdVar3) + pdVar5[2] +
                 *(double *)(lVar2 + uVar11 * 8) + *(double *)(uVar6 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x20 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x18 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x28 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x30 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x38 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x40 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x50 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x48 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x60 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x58 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x68 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x78 + (longlong)pdVar3) +
                 *(double *)(uVar6 + 0x70 + (longlong)pdVar3);
            uVar11 = uVar11 + 1;
            uVar6 = uVar6 + 0x80;
            pdVar5 = pdVar5 + 0x10;
          } while ((longlong)uVar11 < (longlong)iVar13);
        }
        uVar8 = 1;
        goto LAB_14118908e;
      }
    }
  }
LAB_14118908c:
  uVar8 = 0;
LAB_14118908e:
  FUN_141166790(local_70);
  FUN_141166790(local_90);
  return uVar8;
}


==== FUN_1411913c0 @ 1411913c0 (prof 1)

undefined1 FUN_1411913c0(longlong param_1)

{
  undefined1 uVar1;
  
  if (*(int *)(param_1 + 0xe0) < 2) {
    return 0;
  }
  uVar1 = FUN_14117cbb0();
  return uVar1;
}


==== FUN_141166cc0 @ 141166cc0 (prof 1)

longlong FUN_141166cc0(longlong param_1,longlong *param_2)

{
  int iVar1;
  longlong lVar2;
  
  iVar1 = *(int *)(param_1 + 0x18);
  lVar2 = FUN_141166910(param_1,(int)param_2[3] + iVar1,0);
  if (lVar2 == 0) {
    return 0;
  }
  if (0 < (int)param_2[3]) {
    if (*param_2 == 0) {
      return 0;
    }
    FUN_142db0e80(lVar2 + (longlong)iVar1 * 8,*param_2,(longlong)(int)param_2[3] << 3);
  }
  return lVar2;
}


==== FUN_141169190 @ 141169190 (prof 1)

longlong FUN_141169190(longlong *param_1,int param_2,char param_3)

{
  ulonglong *puVar1;
  longlong *plVar2;
  longlong lVar3;
  ulonglong uVar4;
  
  uVar4 = (ulonglong)param_2;
  if (param_2 < 0) {
    return 0;
  }
  puVar1 = (ulonglong *)(param_1 + 2);
  plVar2 = param_1 + 3;
  if (*(char *)((longlong)param_1 + 0x1c) != '\0') {
    *param_1 = 0;
    param_1[1] = 0;
    *(undefined4 *)plVar2 = 0;
    *puVar1 = 0;
    *(undefined1 *)((longlong)param_1 + 0x1c) = 0;
  }
  lVar3 = *param_1;
  if (lVar3 != 0) {
    if (uVar4 <= *puVar1) {
      *(int *)plVar2 = param_2;
      goto LAB_14116928b;
    }
    thunk_FUN_1427bd480(lVar3);
    *param_1 = 0;
  }
  if (param_1[1] != 0) {
    thunk_FUN_1427bd480();
    param_1[1] = 0;
  }
  *(int *)plVar2 = param_2;
  *puVar1 = uVar4;
  if (param_2 == 0) {
    uVar4 = 1;
    *puVar1 = 1;
  }
  lVar3 = _malloc_base(uVar4);
  *param_1 = lVar3;
  if (lVar3 == 0) {
    *(undefined4 *)(param_1 + 3) = 0;
    param_1[2] = 0;
    return 0;
  }
LAB_14116928b:
  if (param_3 != '\0') {
    FUN_142db0ac0(lVar3,0,(longlong)(int)param_1[3]);
  }
  return *param_1;
}


==== FUN_141169a20 @ 141169a20 (prof 1)

longlong FUN_141169a20(longlong *param_1,int param_2)

{
  if (((*param_1 != 0) && (-1 < param_2)) && (param_2 < (int)param_1[2])) {
    return *param_1 + (longlong)(param_2 * *(int *)((longlong)param_1 + 0x14)) * 8;
  }
  return 0;
}


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

int FUN_1411c5cb0(undefined8 param_1,longlong *param_2,int param_3)

{
  longlong lVar1;
  
  if (*param_2 == 0) {
    return -1;
  }
  if ((param_3 == 0) && (0 < (longlong)(int)param_2[3])) {
    lVar1 = 0;
    while (*(double *)(*param_2 + lVar1 * 8) == 0.0) {
      lVar1 = lVar1 + 1;
      if ((int)param_2[3] <= lVar1) {
        return 0;
      }
    }
    param_3 = 1;
  }
  return param_3;
}


==== FUN_141166bc0 @ 141166bc0 (prof 1)

longlong FUN_141166bc0(undefined8 param_1,longlong param_2,int param_3)

{
  longlong lVar1;
  
  if (param_2 != 0) {
    lVar1 = FUN_1411667d0(param_1,param_3,0);
    if (lVar1 != 0) {
      FUN_142db0e80(lVar1,param_2,(longlong)param_3 << 3);
      return lVar1;
    }
  }
  return 0;
}


==== FUN_141171510 @ 141171510 (prof 1)

undefined8
FUN_141171510(longlong param_1,longlong *param_2,double param_3,undefined8 param_4,double param_5,
             undefined8 *param_6,int param_7)

{
  longlong lVar1;
  longlong lVar2;
  longlong lVar3;
  longlong lVar4;
  longlong lVar5;
  int iVar6;
  int iVar7;
  longlong lVar8;
  undefined8 *puVar9;
  int *piVar10;
  int iVar11;
  int iVar12;
  undefined8 uVar13;
  double dVar14;
  undefined1 local_68 [48];
  
  FUN_141166770(local_68);
  dVar14 = param_5 * (DAT_145415378 / param_3) * DAT_145415638 + DAT_1454152a0;
  iVar11 = (int)(param_5 * (DAT_145415378 / param_3) * DAT_145415578 + DAT_1454152a0);
  lVar2 = FUN_1411667d0(param_1 + 0x1560,param_7 * 4,1);
  if (lVar2 != 0) {
    lVar3 = FUN_1411667d0(param_1 + 0x1580,param_7 * 4,1);
    if (((lVar3 != 0) && (piVar10 = (int *)*param_6, piVar10 != (int *)0x0)) &&
       (lVar1 = *param_2, lVar1 != 0)) {
      param_5._0_4_ = 0;
      if (0 < param_7) {
        lVar4 = 0;
        do {
          iVar6 = *piVar10;
          if (param_5._0_4_ < param_7 + -1) {
            iVar12 = piVar10[1];
          }
          else {
            iVar12 = (int)dVar14 + -1 + iVar6;
          }
          iVar7 = 0;
          puVar9 = (undefined8 *)(lVar3 + lVar4 * 8);
          do {
            lVar8 = lVar4;
            lVar4 = FUN_141166bc0(local_68,lVar1 + -8 + (longlong)iVar6 * 8,iVar11);
            if (lVar4 == 0) goto LAB_141171731;
            iVar6 = iVar6 + iVar11;
            uVar13 = FUN_1411671d0(local_68);
            *(undefined8 *)((longlong)puVar9 + (lVar2 - lVar3)) = uVar13;
            FUN_141167080(local_68);
            uVar13 = FUN_141167280(local_68);
            *puVar9 = uVar13;
            lVar4 = lVar8 + 1;
            puVar9 = puVar9 + 1;
            iVar7 = iVar7 + 1;
          } while (iVar7 < 3);
          lVar5 = FUN_141166bc0(local_68,lVar1 + -8 + (longlong)iVar6 * 8,iVar12 - iVar6);
          if (lVar5 == 0) goto LAB_141171731;
          uVar13 = FUN_1411671d0(local_68);
          *(undefined8 *)(lVar2 + lVar4 * 8) = uVar13;
          FUN_141167080(local_68);
          uVar13 = FUN_141167280(local_68);
          *(undefined8 *)(lVar3 + lVar4 * 8) = uVar13;
          lVar4 = lVar8 + 2;
          param_5._0_4_ = param_5._0_4_ + 1;
          piVar10 = piVar10 + 1;
        } while (param_5._0_4_ < param_7);
      }
      uVar13 = 1;
      goto LAB_141171733;
    }
  }
LAB_141171731:
  uVar13 = 0;
LAB_141171733:
  FUN_141166790(local_68);
  return uVar13;
}


==== FUN_14117a200 @ 14117a200 (prof 1)

undefined8 FUN_14117a200(longlong param_1,undefined8 *param_2,undefined8 *param_3)

{
  longlong lVar1;
  int *piVar2;
  int *piVar3;
  longlong lVar4;
  longlong lVar5;
  undefined8 *puVar6;
  int *piVar7;
  undefined8 *puVar8;
  longlong lVar9;
  undefined8 uVar10;
  longlong *in_stack_00000040;
  undefined8 *in_stack_00000048;
  int in_stack_00000050;
  undefined1 local_70 [24];
  int local_58;
  undefined1 local_50 [24];
  int local_38;
  
  FUN_141166770(local_50);
  FUN_141166770(local_70);
  lVar1 = *in_stack_00000040;
  if (((lVar1 != 0) && (piVar2 = (int *)*in_stack_00000048, piVar2 != (int *)0x0)) &&
     (lVar4 = FUN_1411667d0(local_50,*(undefined4 *)(param_2 + 3),0), lVar4 != 0)) {
    FUN_141166570(param_1 + 0x1b28,*param_2,lVar4,local_38);
    FUN_141167080(local_50);
    lVar5 = FUN_1411667d0(local_70,*(undefined4 *)(param_3 + 3),0);
    if (lVar5 != 0) {
      FUN_141166570(param_1 + 0x1b60,*param_3,lVar5,local_58);
      FUN_141167080(local_70);
      FUN_141166430(param_1 + 0x1b98,lVar4,lVar4,local_38);
      FUN_141166430(param_1 + 0x1bd0,lVar5,lVar5,local_58);
      lVar4 = FUN_1411667d0(param_1 + 0x1c08,in_stack_00000050,0);
      if ((lVar4 != 0) &&
         (puVar6 = (undefined8 *)FUN_1411667d0(param_1 + 0x1c28,in_stack_00000050,0),
         puVar6 != (undefined8 *)0x0)) {
        lVar5 = (longlong)in_stack_00000050 + -1;
        if (0 < lVar5) {
          puVar8 = puVar6;
          lVar9 = lVar5;
          piVar3 = piVar2;
          do {
            piVar7 = piVar3 + 1;
            uVar10 = FUN_141167180(local_50,*(int *)((longlong)piVar3 + (lVar1 - (longlong)piVar2))
                                            + -1,
                                   *(int *)((longlong)piVar7 + (lVar1 - (longlong)piVar2)) + -2);
            *(undefined8 *)((lVar4 - (longlong)puVar6) + (longlong)puVar8) = uVar10;
            uVar10 = FUN_141167180(local_70,*piVar3 + -1,*piVar7 + -2);
            *puVar8 = uVar10;
            puVar8 = puVar8 + 1;
            lVar9 = lVar9 + -1;
            piVar3 = piVar7;
          } while (lVar9 != 0);
        }
        uVar10 = FUN_141167180(local_50,*(int *)(lVar1 + lVar5 * 4) + -1,local_38 + -1);
        *(undefined8 *)(lVar4 + lVar5 * 8) = uVar10;
        uVar10 = FUN_141167180(local_70,piVar2[lVar5] + -1,local_58 + -1);
        puVar6[lVar5] = uVar10;
        uVar10 = 1;
        goto LAB_14117a427;
      }
    }
  }
  uVar10 = 0;
LAB_14117a427:
  FUN_141166790(local_70);
  FUN_141166790(local_50);
  return uVar10;
}


==== FUN_141166910 @ 141166910 (prof 1)

longlong FUN_141166910(longlong *param_1,int param_2,char param_3)

{
  int iVar1;
  longlong lVar2;
  longlong lVar3;
  ulonglong uVar4;
  
  lVar3 = *param_1;
  iVar1 = (int)param_1[3];
  if ((param_2 < 0) || (*(char *)((longlong)param_1 + 0x1c) != '\0')) {
LAB_1411669b5:
    lVar3 = 0;
  }
  else {
    uVar4 = (longlong)param_2 << 3;
    if (lVar3 == 0) {
LAB_141166969:
      if (param_1[1] != 0) {
        thunk_FUN_1427bd480();
        param_1[1] = 0;
      }
      *(int *)(param_1 + 3) = param_2;
      param_1[2] = uVar4;
      if (uVar4 == 0) {
        uVar4 = 8;
        param_1[2] = 8;
      }
      lVar2 = _malloc_base(uVar4);
      *param_1 = lVar2;
      if (lVar2 == 0) {
        *(undefined4 *)(param_1 + 3) = 0;
        param_1[2] = 0;
        if (lVar3 != 0) {
          thunk_FUN_1427bd480(lVar3);
        }
        goto LAB_1411669b5;
      }
      if (lVar3 != 0) {
        FUN_142db0e80(lVar2,lVar3,(longlong)iVar1 << 3);
        thunk_FUN_1427bd480(lVar3);
      }
    }
    else {
      if ((ulonglong)param_1[2] < uVar4) {
        *param_1 = 0;
        goto LAB_141166969;
      }
      *(int *)(param_1 + 3) = param_2;
    }
    if ((param_3 != '\0') && (iVar1 < (int)param_1[3])) {
      FUN_142db0ac0(*param_1 + (longlong)iVar1 * 8,0,(longlong)((int)param_1[3] - iVar1) << 3);
    }
    lVar3 = *param_1;
  }
  return lVar3;
}


==== FUN_141189500 @ 141189500 (prof 1)

undefined8
FUN_141189500(longlong param_1,undefined8 *param_2,double param_3,undefined8 param_4,double param_5,
             longlong *param_6,int param_7)

{
  int iVar1;
  longlong lVar2;
  longlong lVar3;
  int *piVar4;
  longlong lVar5;
  ulonglong uVar6;
  uint uVar7;
  undefined8 uVar9;
  undefined8 uVar10;
  int local_res10 [2];
  undefined1 local_98 [24];
  int local_80;
  undefined1 local_78 [32];
  undefined1 local_58 [48];
  ulonglong uVar8;
  
  FUN_141166770(local_98);
  FUN_141166770(local_58);
  FUN_141167d60(local_78);
  lVar2 = FUN_1411667d0(local_98,*(undefined4 *)(param_2 + 3),0);
  if (lVar2 != 0) {
    FUN_141166570(param_1 + 0x1378,*param_2,lVar2,local_80);
    FUN_141167080(local_98);
    FUN_141166430(param_1 + 0x13b0,lVar2,lVar2,local_80);
    local_res10[0] =
         *(int *)(*param_6 + -4 + (longlong)param_7 * 4) -
         (int)((DAT_143537ea0 / param_3) * param_5);
    if (local_80 < local_res10[0] + -1) {
      local_res10[0] = local_80;
    }
    lVar3 = FUN_141167dc0(local_78,(int)param_6[3] + 1,0);
    if ((((lVar3 != 0) && (lVar3 = FUN_141168020(local_78,param_6), lVar3 != 0)) &&
        (piVar4 = (int *)FUN_141168220(local_78,local_res10,1), piVar4 != (int *)0x0)) &&
       (lVar3 = FUN_1411667d0(param_1 + 0x13e8,param_7,0), lVar3 != 0)) {
      uVar6 = 0;
      if (0 < param_7) {
        uVar8 = uVar6;
        uVar9 = 0;
        do {
          iVar1 = *piVar4;
          uVar10 = uVar9;
          if (((0 < iVar1) && (iVar1 <= local_80)) && (piVar4[1] + -1 <= local_80)) {
            lVar5 = FUN_141166c30(local_58,lVar2 + -8 + (longlong)iVar1 * 8,piVar4[1] - iVar1);
            if (lVar5 == 0) goto LAB_1411896d5;
            uVar10 = uVar9;
            uVar9 = FUN_141167150(local_58);
          }
          piVar4 = piVar4 + 1;
          *(undefined8 *)(lVar3 + uVar6) = uVar9;
          uVar7 = (int)uVar8 + 1;
          uVar8 = (ulonglong)uVar7;
          uVar6 = uVar6 + 8;
          uVar9 = uVar10;
        } while ((int)uVar7 < param_7);
      }
      uVar9 = 1;
      goto LAB_1411896d7;
    }
  }
LAB_1411896d5:
  uVar9 = 0;
LAB_1411896d7:
  FUN_141167d80(local_78);
  FUN_141166790(local_58);
  FUN_141166790(local_98);
  return uVar9;
}


==== FUN_1411711f0 @ 1411711f0 (prof 1)

undefined8
FUN_1411711f0(longlong param_1,longlong param_2,double param_3,undefined8 param_4,double param_5,
             longlong *param_6,int param_7)

{
  int iVar1;
  longlong lVar2;
  longlong lVar3;
  int *piVar4;
  longlong lVar5;
  ulonglong uVar6;
  uint uVar7;
  undefined8 uVar9;
  undefined8 uVar10;
  int local_98 [2];
  undefined1 local_90 [32];
  undefined1 local_70 [24];
  int local_58;
  undefined1 local_50 [56];
  ulonglong uVar8;
  
  FUN_141166770(local_70);
  FUN_141166770(local_50);
  FUN_141167d60(local_90);
  lVar2 = FUN_141166a30(local_70,param_2);
  if (lVar2 != 0) {
    FUN_141167080(local_70);
    local_98[0] = *(int *)(*param_6 + -4 + (longlong)param_7 * 4) -
                  (int)((DAT_143537ea0 / param_3) * param_5);
    if (*(int *)(param_2 + 0x18) < local_98[0] + -1) {
      local_98[0] = *(int *)(param_2 + 0x18);
    }
    lVar3 = FUN_141167dc0(local_90,(int)param_6[3] + 1,0);
    if ((((lVar3 != 0) && (lVar3 = FUN_141168020(local_90,param_6), lVar3 != 0)) &&
        (piVar4 = (int *)FUN_141168220(local_90,local_98,1), piVar4 != (int *)0x0)) &&
       (lVar3 = FUN_1411667d0(param_1 + 0x1540,param_7,0), lVar3 != 0)) {
      uVar6 = 0;
      if (0 < param_7) {
        uVar8 = uVar6;
        uVar9 = 0;
        do {
          iVar1 = *piVar4;
          uVar10 = uVar9;
          if (((0 < iVar1) && (iVar1 <= local_58)) && (piVar4[1] + -1 <= local_58)) {
            lVar5 = FUN_141166c30(local_50,lVar2 + -8 + (longlong)iVar1 * 8,piVar4[1] - iVar1);
            if (lVar5 == 0) goto LAB_141171385;
            uVar10 = uVar9;
            uVar9 = FUN_141167280(local_50);
          }
          piVar4 = piVar4 + 1;
          *(undefined8 *)(lVar3 + uVar6) = uVar9;
          uVar7 = (int)uVar8 + 1;
          uVar8 = (ulonglong)uVar7;
          uVar6 = uVar6 + 8;
          uVar9 = uVar10;
        } while ((int)uVar7 < param_7);
      }
      uVar9 = 1;
      goto LAB_141171387;
    }
  }
LAB_141171385:
  uVar9 = 0;
LAB_141171387:
  FUN_141167d80(local_90);
  FUN_141166790(local_50);
  FUN_141166790(local_70);
  return uVar9;
}


==== FUN_141197890 @ 141197890 (prof 1)

undefined8 FUN_141197890(longlong param_1,undefined4 param_2,undefined4 param_3)

{
  char cVar1;
  longlong lVar2;
  
  lVar2 = FUN_141166f90(param_1 + 0xe58,param_1 + 0xba0);
  if (lVar2 != 0) {
    lVar2 = FUN_141166f90(param_1 + 0xe78,param_1 + 0xbc0);
    if (lVar2 != 0) {
      *(undefined4 *)(param_1 + 0xe98) = param_2;
      lVar2 = FUN_141166f90(param_1 + 0xea0,param_1 + 0xbe0);
      if (lVar2 != 0) {
        lVar2 = FUN_141166f90(param_1 + 0xec0,param_1 + 0xc00);
        if (lVar2 != 0) {
          *(undefined4 *)(param_1 + 0xee0) = param_3;
          cVar1 = FUN_141169970(param_1 + 0xee8,param_1 + 0xe28);
          if (cVar1 != '\0') {
            lVar2 = FUN_141166f90(param_1 + 0xf00,param_1 + 0xc60);
            if (lVar2 != 0) {
              lVar2 = FUN_141166f90(param_1 + 0xf20,param_1 + 0xc20);
              if (lVar2 != 0) {
                lVar2 = FUN_141169670(param_1 + 0xf40,param_1 + 0xdc0);
                if (lVar2 != 0) {
                  lVar2 = FUN_141169670(param_1 + 0xf60,param_1 + 0xd80);
                  if (lVar2 != 0) {
                    lVar2 = FUN_141166f90(param_1 + 0xf80,param_1 + 0xca0);
                    if (lVar2 != 0) {
                      lVar2 = FUN_141169670(param_1 + 4000,param_1 + 0xda0);
                      if (lVar2 != 0) {
                        lVar2 = FUN_141166f90(param_1 + 0xfc0,param_1 + 0xc40);
                        if (lVar2 != 0) {
                          lVar2 = FUN_141166f90(param_1 + 0xfe0,param_1 + 0xcc0);
                          if (lVar2 != 0) {
                            lVar2 = FUN_141166f90(param_1 + 0x1000,param_1 + 0xce0);
                            if (lVar2 != 0) {
                              lVar2 = FUN_141166f90(param_1 + 0x1020,param_1 + 0xd00);
                              if (lVar2 != 0) {
                                cVar1 = FUN_141169970(param_1 + 0x1040,param_1 + 0xe40);
                                if (cVar1 != '\0') {
                                  cVar1 = FUN_141169970(param_1 + 0x1058,param_1 + 0xe10);
                                  if (cVar1 != '\0') {
                                    *(undefined4 *)(param_1 + 0x1070) = param_2;
                                    lVar2 = FUN_141166f90(param_1 + 0x1078,param_1 + 0xd20);
                                    if (lVar2 != 0) {
                                      cVar1 = FUN_141169970(param_1 + 0x1098,param_1 + 0xdf8);
                                      if (cVar1 != '\0') {
                                        *(undefined4 *)(param_1 + 0x10b0) =
                                             *(undefined4 *)(param_1 + 0xde8);
                                        *(undefined4 *)(param_1 + 0x10b4) =
                                             *(undefined4 *)(param_1 + 0xdec);
                                        lVar2 = FUN_141166f90(param_1 + 0x10b8,param_1 + 0xd40);
                                        if (lVar2 != 0) {
                                          lVar2 = FUN_141166f90(param_1 + 0x10d8,param_1 + 0xd60);
                                          if (lVar2 != 0) {
                                            *(undefined4 *)(param_1 + 0x10f8) =
                                                 *(undefined4 *)(param_1 + 0xdf0);
                                            lVar2 = FUN_1411684f0(param_1 + 0x1100,param_1 + 0x1140)
                                            ;
                                            if (lVar2 != 0) {
                                              FUN_141167030(param_1 + 0xa20);
                                              FUN_141167030(param_1 + 0xa40);
                                              FUN_141167030(param_1 + 0xa60);
                                              FUN_141167030(param_1 + 0xa80);
                                              FUN_141167030(param_1 + 0xaa0);
                                              FUN_141167030(param_1 + 0xac0);
                                              FUN_141167030(param_1 + 0xae0);
                                              FUN_141167030(param_1 + 0xb00);
                                              FUN_141167030(param_1 + 0xb20);
                                              FUN_141167030(param_1 + 0xc80);
                                              FUN_141167030(param_1 + 0x3d8);
                                              FUN_141167030(param_1 + 0x3f8);
                                              FUN_141168590(param_1 + 0x1120);
                                              FUN_141168590(param_1 + 0x1160);
                                              FUN_141167030(param_1 + 0x11f8);
                                              FUN_141167030(param_1 + 0x1218);
                                              FUN_141167030(param_1 + 0x1238);
                                              FUN_141167030(param_1 + 0x1338);
                                              FUN_141167030(param_1 + 0x1358);
                                              FUN_141167030(param_1 + 0x13e8);
                                              FUN_141167030(param_1 + 0x1520);
                                              FUN_141167030(param_1 + 0x1540);
                                              FUN_141167030(param_1 + 0x1560);
                                              FUN_141167030(param_1 + 0x1580);
                                              FUN_141169710(param_1 + 0x16d8);
                                              FUN_141169710(param_1 + 0x1948);
                                              FUN_141169710(param_1 + 0x1978);
                                              FUN_141167030(param_1 + 0x1998);
                                              FUN_141168590(param_1 + 0x19c0);
                                              FUN_141167030(param_1 + 0x19e0);
                                              FUN_1411699e0(param_1 + 0x1a00);
                                              FUN_1411699e0(param_1 + 0x1a18);
                                              FUN_1411645f0(param_1 + 0x1a38,0);
                                              FUN_1411699e0(param_1 + 0x1a70);
                                              FUN_1411645f0(param_1 + 0x1a88,0);
                                              FUN_1411699e0(param_1 + 0x1ac0);
                                              FUN_1411645f0(param_1 + 0x1ad8,0);
                                              FUN_1411699e0(param_1 + 0x1b10);
                                              FUN_141167030(param_1 + 0x1c08);
                                              FUN_141167030(param_1 + 0x1c28);
                                              FUN_141167030(param_1 + 0x1d60);
                                              return 1;
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
  return 0;
}


==== FUN_1411885f0 @ 1411885f0 (prof 1)

undefined8
FUN_1411885f0(longlong param_1,undefined8 *param_2,double param_3,undefined8 param_4,double param_5,
             longlong *param_6,uint param_7)

{
  longlong lVar1;
  double *pdVar2;
  longlong lVar3;
  double *pdVar4;
  ulonglong uVar5;
  uint uVar6;
  int iVar7;
  undefined8 uVar8;
  int iVar9;
  double *pdVar10;
  longlong lVar11;
  ulonglong uVar12;
  int iVar13;
  double dVar14;
  double dVar15;
  int local_res10;
  longlong local_98 [3];
  undefined4 local_80;
  undefined1 local_78 [64];
  
  FUN_141166770(local_98);
  FUN_141166770(local_78);
  lVar1 = FUN_1411667d0(local_98,*(undefined4 *)(param_2 + 3),0);
  if (lVar1 != 0) {
    FUN_141166570(param_1 + 0x3a0,*param_2,local_98[0],local_80);
    dVar15 = param_5 * (DAT_145415378 / param_3) * DAT_145415638 + DAT_1454152a0;
    iVar9 = (int)(param_5 * (DAT_145415378 / param_3) * DAT_145415578 + DAT_1454152a0);
    pdVar2 = (double *)FUN_1411667d0(param_1 + 0x3d8,param_7 * 4,0);
    if (pdVar2 != (double *)0x0) {
      lVar1 = 0;
      if (0 < (int)param_7) {
        local_res10 = 2;
        pdVar4 = pdVar2;
        uVar5 = (longlong)(int)param_7;
        do {
          lVar11 = 0;
          iVar7 = *(int *)(*param_6 + lVar1 * 4);
          pdVar10 = pdVar4;
          if (local_res10 < (int)uVar5) {
            iVar13 = *(int *)(*param_6 + 4 + lVar1 * 4);
          }
          else {
            iVar13 = (int)dVar15 + -1 + iVar7;
          }
          do {
            lVar3 = FUN_141166c30(local_78,local_98[0] + -8 + (longlong)iVar7 * 8,iVar9);
            if (lVar3 == 0) goto LAB_141188909;
            iVar7 = iVar7 + iVar9;
            dVar14 = (double)FUN_1411671d0(local_78);
            *pdVar10 = dVar14;
            lVar11 = lVar11 + 1;
            pdVar10 = pdVar10 + 1;
          } while (lVar11 < 3);
          lVar11 = FUN_141166c30(local_78,local_98[0] + -8 + (longlong)iVar7 * 8,iVar13 - iVar7);
          if (lVar11 == 0) goto LAB_141188909;
          dVar14 = (double)FUN_1411671d0(local_78);
          pdVar4[3] = dVar14;
          local_res10 = local_res10 + 1;
          lVar1 = lVar1 + 1;
          pdVar4 = pdVar4 + 4;
          uVar5 = (ulonglong)param_7;
        } while (lVar1 < (int)param_7);
      }
      dVar15 = DAT_1454154c0;
      uVar5 = 0;
      if (((param_5 <= DAT_143538358) || (DAT_143538360 <= param_5)) &&
         (pdVar4 = pdVar2, uVar12 = uVar5, 0 < *(int *)(param_1 + 0x3f0))) {
        do {
          *pdVar4 = dVar15 * *pdVar4;
          uVar6 = (int)uVar12 + 1;
          pdVar4 = pdVar4 + 1;
          uVar12 = (ulonglong)uVar6;
        } while ((int)uVar6 < *(int *)(param_1 + 0x3f0));
      }
      iVar9 = (int)(param_7 + ((int)param_7 >> 0x1f & 3U)) >> 2;
      lVar1 = FUN_1411667d0(param_1 + 0x3f8,iVar9,1);
      if (lVar1 != 0) {
        pdVar4 = pdVar2;
        uVar12 = uVar5;
        if (0 < iVar9) {
          do {
            *(double *)(lVar1 + uVar12 * 8) =
                 pdVar4[2] + *(double *)(uVar5 + 8 + (longlong)pdVar2) +
                 *(double *)(lVar1 + uVar12 * 8) + *(double *)(uVar5 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x20 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x18 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x28 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x30 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x38 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x40 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x50 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x48 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x60 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x58 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x68 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x78 + (longlong)pdVar2) +
                 *(double *)(uVar5 + 0x70 + (longlong)pdVar2);
            uVar12 = uVar12 + 1;
            uVar5 = uVar5 + 0x80;
            pdVar4 = pdVar4 + 0x10;
          } while ((longlong)uVar12 < (longlong)iVar9);
        }
        uVar8 = 1;
        goto LAB_14118890b;
      }
    }
  }
LAB_141188909:
  uVar8 = 0;
LAB_14118890b:
  FUN_141166790(local_78);
  FUN_141166790(local_98);
  return uVar8;
}


==== FUN_14117ac80 @ 14117ac80 (prof 1)

undefined8
FUN_14117ac80(longlong param_1,longlong *param_2,double param_3,double param_4,undefined8 param_5,
             int param_6,longlong *param_7,int param_8)

{
  uint uVar1;
  ulonglong uVar2;
  longlong lVar3;
  longlong lVar4;
  char cVar5;
  int iVar6;
  int iVar7;
  ulonglong uVar8;
  double *pdVar9;
  double *pdVar10;
  undefined8 *puVar11;
  int iVar12;
  longlong lVar13;
  longlong lVar14;
  double *pdVar15;
  undefined8 uVar16;
  longlong lVar17;
  int iVar18;
  longlong lVar19;
  longlong lVar20;
  int iVar21;
  int *piVar22;
  double dVar23;
  double dVar24;
  double dVar25;
  double dVar26;
  int local_e8;
  longlong local_d8;
  undefined1 local_b0 [24];
  int local_98;
  undefined1 local_90 [88];
  
  FUN_141166770(local_b0);
  FUN_141166770(local_90);
  lVar3 = *param_7;
  if ((lVar3 != 0) && (lVar4 = *param_2, lVar4 != 0)) {
    iVar6 = (param_8 - param_6) + 1;
    iVar7 = (int)(iVar6 + (iVar6 >> 0x1f & 3U)) >> 2;
    param_3 = (DAT_145415638 / param_4) * param_3;
    iVar12 = (int)param_3;
    iVar6 = iVar12 + 1;
    if (param_3 - (double)iVar12 <= 0.0) {
      iVar6 = iVar12;
    }
    iVar6 = iVar6 * 4 + 0x801;
    uVar8 = FUN_1411667d0(local_b0,iVar6,1);
    if (((uVar8 != 0) &&
        (pdVar9 = (double *)FUN_1411667d0(local_90,0x1000,1), pdVar9 != (double *)0x0)) &&
       (cVar5 = FUN_1411697a0(param_1 + 0x1ac0,iVar7 * 4,0x400,1), cVar5 != '\0')) {
      if (0 < iVar7) {
        local_d8 = 1;
        param_7._0_4_ = 0;
        local_e8 = 0;
        piVar22 = (int *)(lVar3 + ((longlong)param_6 + 3) * 4);
        dVar24 = DAT_143538108;
        dVar25 = DAT_143538100;
        dVar26 = DAT_1454155b0;
        do {
          if ((local_98 < iVar6) && (uVar8 = FUN_1411667d0(local_b0,iVar6,0), uVar8 == 0))
          goto LAB_14117b129;
          if (0 < iVar6) {
            FUN_142db0ac0(uVar8,0,(longlong)iVar6 << 3);
          }
          iVar12 = piVar22[-4];
          if (local_d8 < iVar7) {
            iVar21 = *piVar22;
          }
          else {
            iVar21 = iVar12 + local_98;
          }
          iVar18 = (int)param_2[3];
          if (iVar21 + -1 <= (int)param_2[3]) {
            iVar18 = iVar21 + -1;
          }
          iVar18 = iVar18 - iVar12;
          uVar1 = iVar18 + 1;
          if ((local_98 < (int)uVar1) && (uVar8 = FUN_1411667d0(local_b0,uVar1,0), uVar8 == 0))
          goto LAB_14117b129;
          lVar14 = lVar4 + (longlong)iVar12 * 8;
          uVar2 = lVar14 - 8;
          lVar17 = 0;
          lVar19 = (longlong)(int)uVar1;
          if ((((int)uVar1 < 1) || (uVar1 < 4)) ||
             ((uVar8 <= uVar2 + (longlong)iVar18 * 8 && (uVar2 <= uVar8 + (longlong)iVar18 * 8)))) {
            if (0 < lVar19) {
              if (3 < lVar19) {
                puVar11 = (undefined8 *)(lVar14 + 8);
                lVar13 = uVar8 - uVar2;
                lVar20 = (lVar19 - 4U >> 2) + 1;
                lVar17 = lVar20 * 4;
                do {
                  *(undefined8 *)(lVar13 + -0x10 + (longlong)puVar11) = puVar11[-2];
                  *(undefined8 *)(lVar13 + -8 + (longlong)puVar11) =
                       *(undefined8 *)((longlong)puVar11 + (lVar14 - uVar2) + -0x10);
                  *(undefined8 *)(lVar13 + (longlong)puVar11) = *puVar11;
                  *(undefined8 *)(lVar13 + 8 + (longlong)puVar11) = puVar11[1];
                  puVar11 = puVar11 + 4;
                  lVar20 = lVar20 + -1;
                } while (lVar20 != 0);
                if (lVar19 <= lVar17) goto LAB_14117af5d;
              }
              puVar11 = (undefined8 *)(uVar8 + lVar17 * 8);
              lVar19 = lVar19 - lVar17;
              do {
                *puVar11 = *(undefined8 *)((longlong)puVar11 + (uVar2 - uVar8));
                puVar11 = puVar11 + 1;
                lVar19 = lVar19 + -1;
              } while (lVar19 != 0);
            }
          }
          else {
            FUN_142db0e80(uVar8,uVar2,lVar19 * 8);
          }
LAB_14117af5d:
          iVar21 = (int)param_7;
          do {
            lVar17 = uVar8 + (longlong)
                             (*(int *)(lVar3 + -0x14 + (longlong)(param_6 + 4 + iVar21) * 4) -
                             iVar12) * 8;
            lVar14 = 0x18;
            pdVar10 = pdVar9 + 2;
            do {
              pdVar10[-2] = *(double *)(lVar17 + -0x18 + lVar14);
              pdVar10[-1] = 0.0;
              *pdVar10 = *(double *)(lVar14 + -0x10 + lVar17);
              pdVar10[1] = 0.0;
              pdVar10[2] = *(double *)(lVar17 + -8 + lVar14);
              pdVar10[3] = 0.0;
              pdVar10[4] = *(double *)(lVar14 + lVar17);
              pdVar10[5] = 0.0;
              pdVar10[6] = *(double *)(lVar17 + 8 + lVar14);
              pdVar10[7] = 0.0;
              pdVar10[8] = *(double *)(lVar17 + 0x10 + lVar14);
              pdVar10[9] = 0.0;
              pdVar10[10] = *(double *)(lVar17 + 0x18 + lVar14);
              pdVar10[0xb] = 0.0;
              pdVar10[0xc] = *(double *)(lVar17 + 0x20 + lVar14);
              pdVar10[0xd] = 0.0;
              lVar14 = lVar14 + 0x40;
              pdVar10 = pdVar10 + 0x10;
            } while (lVar14 < 0x4018);
            cVar5 = FUN_1411655f0(param_1 + 0x1a88,pdVar9);
            if (((cVar5 == '\0') || (cVar5 = FUN_141164e90(param_1 + 0x1a88,pdVar9), cVar5 == '\0'))
               || (pdVar10 = (double *)FUN_141169a20(param_1 + 0x1ac0,iVar21),
                  pdVar10 == (double *)0x0)) goto LAB_14117b129;
            lVar17 = 0x400;
            pdVar15 = pdVar9;
            dVar23 = dVar24;
            do {
              dVar24 = dVar23;
              if (dVar25 <= SQRT(pdVar15[1] * pdVar15[1] + *pdVar15 * *pdVar15)) {
                dVar23 = (double)FUN_1427a25f0();
                dVar23 = dVar23 * dVar26;
              }
              *pdVar10 = dVar23;
              pdVar15 = pdVar15 + 2;
              pdVar10 = pdVar10 + 1;
              lVar17 = lVar17 + -1;
              dVar23 = dVar24;
            } while (lVar17 != 0);
            iVar21 = iVar21 + 1;
          } while (iVar21 + local_e8 < 4);
          param_7._0_4_ = (int)param_7 + 4;
          local_e8 = local_e8 + -4;
          local_d8 = local_d8 + 1;
          piVar22 = piVar22 + 4;
        } while (local_d8 <= iVar7);
      }
      uVar16 = 1;
      goto LAB_14117b12b;
    }
  }
LAB_14117b129:
  uVar16 = 0;
LAB_14117b12b:
  FUN_141166790(local_90);
  FUN_141166790(local_b0);
  return uVar16;
}


==== FUN_141187b40 @ 141187b40 (prof 1)

undefined8
FUN_141187b40(longlong param_1,undefined8 *param_2,double param_3,undefined8 param_4,double param_5,
             longlong *param_6,int param_7,longlong param_8)

{
  int iVar1;
  double dVar2;
  longlong lVar3;
  longlong lVar4;
  int *piVar5;
  double *pdVar6;
  longlong lVar7;
  int iVar8;
  int iVar9;
  undefined8 uVar10;
  undefined8 uVar11;
  undefined1 local_98 [24];
  int local_80;
  undefined1 local_78 [32];
  undefined1 local_58 [48];
  
  FUN_141166770(local_98);
  FUN_141166770(local_58);
  FUN_141167d60(local_78);
  lVar3 = FUN_1411667d0(local_98,*(undefined4 *)(param_2 + 3),0);
  if (lVar3 != 0) {
    FUN_141166570(param_1 + 0x470,*param_2,lVar3,local_80);
    FUN_141167080(local_98);
    FUN_141166430(param_1 + 0x4a8,lVar3,lVar3,local_80);
    iVar8 = *(int *)(*param_6 + -4 + (longlong)param_7 * 4) -
            (int)((DAT_143537ea0 / param_3) * param_5);
    if (local_80 < iVar8 + -1) {
      iVar8 = local_80;
    }
    lVar4 = FUN_141167dc0(local_78,param_7 + 1,0);
    if ((lVar4 != 0) && (lVar4 = FUN_1411680a0(local_78,*param_6,param_7), lVar4 != 0)) {
      piVar5 = (int *)FUN_141167f00(local_78,param_7 + 1,0);
      if (piVar5 != (int *)0x0) {
        piVar5[param_7] = iVar8;
        pdVar6 = (double *)FUN_1411667d0(param_8,param_7,1);
        if (pdVar6 != (double *)0x0) {
          iVar9 = 0;
          iVar8 = 0;
          if (0 < param_7) {
            lVar4 = 0;
            uVar10 = 0;
            do {
              iVar1 = *piVar5;
              uVar11 = uVar10;
              if (((0 < iVar1) && (iVar1 <= local_80)) && (piVar5[1] + -1 <= local_80)) {
                lVar7 = FUN_141166c30(local_58,lVar3 + -8 + (longlong)iVar1 * 8,piVar5[1] - iVar1);
                if (lVar7 == 0) goto LAB_141187d49;
                uVar11 = uVar10;
                uVar10 = FUN_141167150(local_58);
              }
              piVar5 = piVar5 + 1;
              *(undefined8 *)(lVar4 + (longlong)pdVar6) = uVar10;
              iVar8 = iVar8 + 1;
              lVar4 = lVar4 + 8;
              uVar10 = uVar11;
            } while (iVar8 < param_7);
          }
          dVar2 = DAT_1454154c0;
          if (0 < *(int *)(param_8 + 0x18)) {
            do {
              *pdVar6 = dVar2 * *pdVar6;
              iVar9 = iVar9 + 1;
              pdVar6 = pdVar6 + 1;
            } while (iVar9 < *(int *)(param_8 + 0x18));
          }
          uVar10 = 1;
          goto LAB_141187d4b;
        }
      }
    }
  }
LAB_141187d49:
  uVar10 = 0;
LAB_141187d4b:
  FUN_141167d80(local_78);
  FUN_141166790(local_58);
  FUN_141166790(local_98);
  return uVar10;
}


==== FUN_141188050 @ 141188050 (prof 1)

undefined8
FUN_141188050(longlong param_1,undefined8 *param_2,double param_3,undefined8 param_4,double param_5,
             undefined8 *param_6,int param_7,double param_8)

{
  double dVar1;
  double dVar2;
  int iVar3;
  int iVar4;
  double dVar5;
  longlong lVar6;
  longlong lVar7;
  int *piVar8;
  longlong lVar9;
  double *pdVar10;
  double *pdVar11;
  int iVar12;
  int *piVar13;
  int iVar14;
  int iVar15;
  int iVar16;
  longlong lVar17;
  int iVar18;
  int iVar19;
  undefined8 uVar20;
  undefined1 auVar21 [64];
  double dVar22;
  double dVar23;
  int local_res10 [2];
  longlong local_180;
  undefined1 local_168 [24];
  int local_150;
  undefined1 local_148 [24];
  int local_130;
  undefined1 local_128 [32];
  undefined1 local_108 [32];
  undefined1 local_e8 [32];
  undefined1 local_c8 [32];
  undefined1 local_a8 [32];
  undefined1 local_88 [80];
  
  FUN_141166770(local_168);
  FUN_141166770(local_88);
  FUN_141166770(local_128);
  FUN_141166770(local_a8);
  FUN_141166770(local_c8);
  FUN_141166770(local_148);
  FUN_141166770(local_e8);
  FUN_141167d60(local_108);
  lVar6 = FUN_1411667d0(local_168,*(undefined4 *)(param_2 + 3),0);
  if (lVar6 != 0) {
    FUN_141166570(param_1 + 0x1188,*param_2,lVar6,local_150);
    FUN_141167080(local_168);
    FUN_141166430(param_1 + 0x11c0,lVar6,lVar6,local_150);
    param_3 = DAT_145415638 / param_3;
    lVar7 = FUN_141167dc0(local_108,param_7 + 1,0);
    if (lVar7 != 0) {
      lVar7 = FUN_1411680a0(local_108,*param_6,param_7);
      if (lVar7 != 0) {
        local_res10[0] = *(int *)(lVar7 + -4 + (longlong)param_7 * 4) + (int)(param_3 * param_5);
        piVar8 = (int *)FUN_141168220(local_108,local_res10,1);
        if (piVar8 != (int *)0x0) {
          lVar7 = FUN_1411667d0(local_88,param_7,0);
          if (lVar7 != 0) {
            iVar15 = 0;
            auVar21 = ZEXT1664(ZEXT816(0));
            if (0 < param_7) {
              lVar17 = 0;
              piVar13 = piVar8;
              do {
                iVar3 = *piVar13;
                if (iVar3 < 1) {
LAB_141188234:
                  uVar20 = auVar21._0_8_;
                }
                else {
                  if (local_150 < iVar3) goto LAB_141188234;
                  if (local_150 < piVar13[1] + -1) goto LAB_141188234;
                  lVar9 = FUN_141166c30(local_128,lVar6 + -8 + (longlong)iVar3 * 8,
                                        piVar13[1] - iVar3);
                  if (lVar9 == 0) goto LAB_1411884da;
                  uVar20 = FUN_141167280(local_128);
                }
                piVar13 = piVar13 + 1;
                *(undefined8 *)(lVar17 + lVar7) = uVar20;
                iVar15 = iVar15 + 1;
                lVar17 = lVar17 + 8;
              } while (iVar15 < param_7);
            }
            lVar7 = FUN_1411667d0(local_a8,param_7 * 4,0);
            if (lVar7 != 0) {
              iVar15 = 0;
              if (0 < param_7) {
                local_180 = 0;
                dVar22 = DAT_145415208;
                dVar23 = DAT_1454152a0;
                do {
                  iVar3 = *piVar8;
                  piVar8 = piVar8 + 1;
                  iVar4 = *piVar8;
                  iVar19 = (int)((double)(iVar4 - iVar3) * dVar22 + dVar23);
                  lVar17 = local_180;
                  iVar14 = 0;
                  do {
                    iVar12 = iVar14 * iVar19 + iVar3;
                    iVar16 = iVar14 + 1;
                    iVar18 = iVar4;
                    if (iVar14 != 3) {
                      iVar18 = iVar16 * iVar19 + iVar3;
                    }
                    if (((iVar12 < 1) || (local_150 < iVar12)) || (local_150 < iVar18 + -1)) {
                      uVar20 = auVar21._0_8_;
                    }
                    else {
                      lVar9 = FUN_141166c30(local_128,lVar6 + -8 + (longlong)iVar12 * 8,
                                            ((iVar18 + -1) - iVar12) + 1);
                      if (lVar9 == 0) goto LAB_1411884da;
                      uVar20 = FUN_141167280(local_128);
                    }
                    *(undefined8 *)(lVar17 + lVar7) = uVar20;
                    lVar17 = lVar17 + 8;
                    iVar14 = iVar16;
                  } while (iVar16 < 4);
                  iVar15 = iVar15 + 1;
                  local_180 = local_180 + 0x20;
                } while (iVar15 < param_7);
              }
              lVar6 = FUN_141166f90(local_c8,local_a8);
              if (lVar6 != 0) {
                pdVar10 = (double *)FUN_141166f90(local_148,local_88);
                if (pdVar10 != (double *)0x0) {
                  pdVar11 = (double *)FUN_1411667d0(local_e8,local_130,1);
                  dVar5 = DAT_145415338;
                  dVar23 = DAT_143538350;
                  dVar22 = DAT_143538238;
                  if (pdVar11 != (double *)0x0) {
                    if (param_8 != auVar21._0_8_) {
                      dVar1 = *pdVar10;
                      if (dVar1 <= param_8) {
                        if (dVar1 < param_8) {
                          if (dVar1 + dVar1 < param_8) {
                            *pdVar11 = 0.5;
                          }
                        }
                      }
                      else if (param_8 * DAT_143538350 < dVar1) {
                        *pdVar11 = 1.0;
                      }
                      if ((dVar1 <= dVar22) && (param_8 <= dVar22)) {
                        *pdVar11 = dVar5 * *pdVar11;
                      }
                    }
                    if (1 < local_130) {
                      lVar6 = (longlong)pdVar10 - (longlong)pdVar11;
                      iVar15 = 1;
                      do {
                        pdVar11 = pdVar11 + 1;
                        dVar1 = *(double *)(lVar6 + (longlong)pdVar11);
                        dVar2 = *(double *)(lVar6 + -8 + (longlong)pdVar11);
                        if (dVar1 <= dVar2) {
                          if (dVar1 < dVar2) {
                            if (dVar1 + dVar1 < dVar2) {
                              *pdVar11 = 0.5;
                            }
                          }
                        }
                        else if (dVar2 * dVar23 < dVar1) {
                          *pdVar11 = 1.0;
                        }
                        if ((dVar1 <= dVar22) && (dVar2 <= dVar22)) {
                          *pdVar11 = dVar5 * *pdVar11;
                        }
                        iVar15 = iVar15 + 1;
                      } while (iVar15 < local_130);
                    }
                    lVar6 = FUN_141166f90(param_1 + 0x11f8,local_148);
                    if (lVar6 != 0) {
                      lVar6 = FUN_141166f90(param_1 + 0x1218,local_c8);
                      if (lVar6 != 0) {
                        lVar6 = FUN_141166f90(param_1 + 0x1238,local_e8);
                        if (lVar6 != 0) {
                          uVar20 = 1;
                          goto LAB_1411884dc;
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
LAB_1411884da:
  uVar20 = 0;
LAB_1411884dc:
  FUN_141167d80(local_108);
  FUN_141166790(local_e8);
  FUN_141166790(local_148);
  FUN_141166790(local_c8);
  FUN_141166790(local_a8);
  FUN_141166790(local_128);
  FUN_141166790(local_88);
  FUN_141166790(local_168);
  return uVar20;
}


==== FUN_141189ab0 @ 141189ab0 (prof 1)

undefined8
FUN_141189ab0(longlong param_1,undefined8 *param_2,double param_3,double param_4,longlong *param_5,
             int param_6)

{
  int iVar1;
  longlong lVar2;
  longlong lVar3;
  int *piVar4;
  longlong lVar5;
  int iVar6;
  longlong lVar7;
  undefined8 uVar8;
  undefined8 uVar9;
  int local_res10 [2];
  undefined1 local_c8 [24];
  undefined4 local_b0;
  undefined1 local_a8 [32];
  undefined1 local_88 [24];
  int local_70;
  undefined1 local_68 [64];
  
  FUN_141166770(local_c8);
  FUN_141166770(local_88);
  FUN_141166770(local_68);
  FUN_141167d60(local_a8);
  lVar2 = FUN_1411667d0(local_c8,*(undefined4 *)(param_2 + 3),0);
  if ((lVar2 != 0) && (lVar3 = FUN_1411667d0(local_88,*(undefined4 *)(param_2 + 3),0), lVar3 != 0))
  {
    FUN_141166570(param_1 + 0x1408,*param_2,lVar2,local_b0);
    FUN_141166570(param_1 + 0x1440,lVar2,lVar2,local_b0);
    FUN_141166570(param_1 + 0x1478,lVar2,lVar2,local_b0);
    FUN_141166570(param_1 + 0x14b0,lVar2,lVar2,local_b0);
    FUN_141167080(local_c8);
    FUN_141166430(param_1 + 0x14e8,lVar2,lVar3,local_70);
    local_res10[0] =
         *(int *)(*param_5 + -4 + (longlong)param_6 * 4) -
         (int)((DAT_143537ea0 / param_3) * param_4);
    if (*(int *)(param_2 + 3) < local_res10[0] + -1) {
      local_res10[0] = *(int *)(param_2 + 3);
    }
    lVar2 = FUN_141167dc0(local_a8,(int)param_5[3] + 1,0);
    if ((((lVar2 != 0) && (lVar2 = FUN_141168020(local_a8,param_5), lVar2 != 0)) &&
        (piVar4 = (int *)FUN_141168220(local_a8,local_res10,1), piVar4 != (int *)0x0)) &&
       (lVar2 = FUN_1411667d0(param_1 + 0x1520,param_6,0), lVar2 != 0)) {
      iVar6 = 0;
      if (0 < param_6) {
        lVar7 = 0;
        uVar8 = 0;
        do {
          iVar1 = *piVar4;
          uVar9 = uVar8;
          if (((0 < iVar1) && (iVar1 <= local_70)) && (piVar4[1] + -1 <= local_70)) {
            lVar5 = FUN_141166c30(local_68,lVar3 + -8 + (longlong)iVar1 * 8,piVar4[1] - iVar1);
            if (lVar5 == 0) goto LAB_141189cd1;
            uVar9 = uVar8;
            uVar8 = FUN_141167280(local_68);
          }
          piVar4 = piVar4 + 1;
          *(undefined8 *)(lVar7 + lVar2) = uVar8;
          iVar6 = iVar6 + 1;
          lVar7 = lVar7 + 8;
          uVar8 = uVar9;
        } while (iVar6 < param_6);
      }
      uVar8 = 1;
      goto LAB_141189cd3;
    }
  }
LAB_141189cd1:
  uVar8 = 0;
LAB_141189cd3:
  FUN_141167d80(local_a8);
  FUN_141166790(local_68);
  FUN_141166790(local_88);
  FUN_141166790(local_c8);
  return uVar8;
}


==== FUN_14117e850 @ 14117e850 (prof 1)

undefined8
FUN_14117e850(longlong param_1,longlong param_2,double param_3,undefined8 param_4,double param_5,
             undefined8 *param_6,int param_7)

{
  int *piVar1;
  double dVar2;
  double dVar3;
  longlong lVar4;
  longlong lVar5;
  uint *puVar6;
  uint *puVar7;
  char *pcVar8;
  char *pcVar9;
  uint uVar10;
  ulonglong uVar11;
  int *piVar13;
  ulonglong uVar14;
  uint *puVar15;
  double *pdVar16;
  undefined8 uVar17;
  longlong lVar18;
  longlong lVar19;
  int iVar20;
  ulonglong uVar21;
  ulonglong uVar22;
  double dVar23;
  undefined1 auVar24 [16];
  undefined1 auVar25 [16];
  undefined1 auVar26 [16];
  undefined1 auVar27 [16];
  double dVar28;
  undefined1 auVar29 [16];
  undefined1 auVar30 [16];
  undefined1 auVar31 [16];
  undefined1 auVar32 [16];
  undefined1 auVar33 [64];
  undefined1 auVar34 [16];
  undefined1 auVar35 [64];
  undefined1 auVar36 [16];
  undefined1 auVar37 [16];
  undefined1 auVar38 [16];
  undefined1 auVar39 [64];
  undefined1 auVar40 [16];
  undefined1 auVar41 [64];
  undefined1 auVar42 [16];
  undefined1 auVar43 [64];
  undefined1 auVar44 [16];
  undefined1 auVar45 [64];
  undefined1 auVar46 [16];
  undefined1 auVar47 [64];
  undefined1 auVar48 [16];
  undefined1 auVar49 [64];
  undefined1 local_118 [24];
  int local_100;
  undefined1 local_f8 [24];
  undefined4 local_e0;
  undefined1 local_d8 [192];
  ulonglong uVar12;
  
  FUN_141166770(local_f8);
  FUN_141167d60(local_118);
  FUN_141167d60(local_d8);
  auVar39 = ZEXT864(*(ulonglong *)(param_1 + 0x1968));
  auVar41 = ZEXT864(*(ulonglong *)(param_1 + 0x1970));
  piVar1 = (int *)*param_6;
  if (piVar1 != (int *)0x0) {
    iVar20 = *(int *)(param_2 + 0x18);
    lVar18 = (longlong)iVar20;
    dVar23 = DAT_145415378 / param_5;
    auVar43 = ZEXT864((ulonglong)(dVar23 * DAT_143538188));
    auVar45 = ZEXT864((ulonglong)(dVar23 * DAT_1454155b0));
    auVar47 = ZEXT864((ulonglong)(dVar23 * DAT_1454157b8));
    auVar49 = ZEXT864((ulonglong)(dVar23 * DAT_143537b30));
    param_5 = (DAT_143538180 / param_3) * param_5;
    dVar23 = DAT_145415378;
    lVar4 = FUN_141166a30(local_f8,param_2);
    if (lVar4 != 0) {
      FUN_141167080(local_f8);
      lVar5 = FUN_141167dc0(local_118,local_e0,1);
      dVar3 = DAT_143538178;
      if (lVar5 != 0) {
        auVar33 = ZEXT1664(ZEXT816(0));
        auVar35 = ZEXT1664(ZEXT816(0));
        uVar12 = 0;
        auVar38 = auVar39._0_16_;
        auVar40 = auVar41._0_16_;
        auVar42 = auVar43._0_16_;
        auVar44 = auVar45._0_16_;
        auVar46 = auVar47._0_16_;
        auVar48 = auVar49._0_16_;
        uVar11 = uVar12;
        uVar14 = uVar12;
        uVar21 = uVar12;
        if (lVar18 < 4) {
          uVar22 = uVar12;
          if (0 < iVar20) goto LAB_14117eb74;
        }
        else {
          iVar20 = 2;
          pdVar16 = (double *)(lVar4 + 0x10);
          lVar19 = (lVar18 - 4U >> 2) + 1;
          uVar22 = lVar19 * 4;
          do {
            auVar29 = auVar33._0_16_;
            auVar36 = auVar35._0_16_;
            if ((int)uVar21 < 1) {
              auVar29 = auVar38;
              auVar36 = auVar40;
            }
            dVar23 = pdVar16[-2];
            auVar24._8_8_ = 0;
            auVar24._0_8_ = dVar23;
            auVar32 = vcmpsd_avx(auVar29,auVar24,2);
            auVar32 = vblendvpd_avx(auVar44,auVar42,auVar32);
            auVar24 = vcmpsd_avx(auVar36,auVar24,2);
            auVar24 = vblendvpd_avx(auVar48,auVar46,auVar24);
            dVar28 = auVar32._0_8_ * dVar23 + -(auVar29._0_8_ * auVar32._0_8_) + auVar29._0_8_;
            auVar29._8_8_ = auVar29._8_8_;
            auVar29._0_8_ = dVar28;
            dVar23 = auVar24._0_8_ * dVar23 + -(auVar36._0_8_ * auVar24._0_8_) + auVar36._0_8_;
            auVar36._8_8_ = auVar36._8_8_;
            auVar36._0_8_ = dVar23;
            if ((dVar23 < dVar28) && (dVar3 < dVar28)) {
              *(int *)(lVar5 + uVar11 * 4) = iVar20 + -1;
              uVar14 = (ulonglong)((int)uVar14 + 1);
              uVar11 = uVar11 + 1;
            }
            if (iVar20 + -1 < 1) {
              auVar29 = auVar38;
              auVar36 = auVar40;
            }
            dVar23 = pdVar16[-1];
            auVar25._8_8_ = 0;
            auVar25._0_8_ = dVar23;
            auVar32 = vcmpsd_avx(auVar29,auVar25,2);
            auVar32 = vblendvpd_avx(auVar44,auVar42,auVar32);
            auVar24 = vcmpsd_avx(auVar36,auVar25,2);
            auVar24 = vblendvpd_avx(auVar48,auVar46,auVar24);
            dVar28 = auVar32._0_8_ * dVar23 + -(auVar29._0_8_ * auVar32._0_8_) + auVar29._0_8_;
            auVar32._8_8_ = auVar29._8_8_;
            auVar32._0_8_ = dVar28;
            dVar23 = auVar24._0_8_ * dVar23 + -(auVar36._0_8_ * auVar24._0_8_) + auVar36._0_8_;
            auVar34._8_8_ = auVar36._8_8_;
            auVar34._0_8_ = dVar23;
            if ((dVar23 < dVar28) && (dVar3 < dVar28)) {
              *(int *)(lVar5 + uVar11 * 4) = iVar20;
              uVar14 = (ulonglong)((int)uVar14 + 1);
              uVar11 = uVar11 + 1;
            }
            if (iVar20 < 1) {
              auVar32 = auVar38;
              auVar34 = auVar40;
            }
            dVar23 = *pdVar16;
            auVar26._8_8_ = 0;
            auVar26._0_8_ = dVar23;
            auVar29 = vcmpsd_avx(auVar32,auVar26,2);
            auVar29 = vblendvpd_avx(auVar44,auVar42,auVar29);
            auVar36 = vcmpsd_avx(auVar34,auVar26,2);
            auVar36 = vblendvpd_avx(auVar48,auVar46,auVar36);
            dVar28 = auVar29._0_8_ * dVar23 + -(auVar32._0_8_ * auVar29._0_8_) + auVar32._0_8_;
            auVar30._8_8_ = auVar32._8_8_;
            auVar30._0_8_ = dVar28;
            dVar23 = auVar36._0_8_ * dVar23 + -(auVar34._0_8_ * auVar36._0_8_) + auVar34._0_8_;
            auVar37._8_8_ = auVar34._8_8_;
            auVar37._0_8_ = dVar23;
            if ((dVar23 < dVar28) && (dVar3 < dVar28)) {
              *(int *)(lVar5 + uVar11 * 4) = iVar20 + 1;
              uVar14 = (ulonglong)((int)uVar14 + 1);
              uVar11 = uVar11 + 1;
            }
            if (iVar20 + 1 < 1) {
              auVar30 = auVar38;
              auVar37 = auVar40;
            }
            dVar23 = pdVar16[1];
            auVar27._8_8_ = 0;
            auVar27._0_8_ = dVar23;
            auVar29 = vcmpsd_avx(auVar30,auVar27,2);
            auVar29 = vblendvpd_avx(auVar44,auVar42,auVar29);
            auVar36 = vcmpsd_avx(auVar37,auVar27,2);
            auVar36 = vblendvpd_avx(auVar48,auVar46,auVar36);
            dVar28 = auVar29._0_8_ * dVar23 + -(auVar30._0_8_ * auVar29._0_8_) + auVar30._0_8_;
            auVar33 = ZEXT1664(CONCAT88(auVar30._8_8_,dVar28));
            dVar23 = auVar36._0_8_ * dVar23 + -(auVar37._0_8_ * auVar36._0_8_) + auVar37._0_8_;
            auVar35 = ZEXT1664(CONCAT88(auVar37._8_8_,dVar23));
            if ((dVar23 < dVar28) && (dVar3 < dVar28)) {
              *(int *)(lVar5 + uVar11 * 4) = iVar20 + 2;
              uVar14 = (ulonglong)((int)uVar14 + 1);
              uVar11 = uVar11 + 1;
            }
            uVar21 = (ulonglong)((int)uVar21 + 4);
            iVar20 = iVar20 + 4;
            pdVar16 = pdVar16 + 4;
            lVar19 = lVar19 + -1;
          } while (lVar19 != 0);
          dVar23 = DAT_145415378;
          if ((longlong)uVar22 < lVar18) {
LAB_14117eb74:
            piVar13 = (int *)(lVar5 + uVar11 * 4);
            do {
              iVar20 = (int)uVar21;
              auVar29 = auVar33._0_16_;
              auVar36 = auVar35._0_16_;
              if (iVar20 < 1) {
                auVar29 = auVar38;
                auVar36 = auVar40;
              }
              dVar28 = *(double *)(lVar4 + uVar22 * 8);
              auVar31._8_8_ = 0;
              auVar31._0_8_ = dVar28;
              auVar32 = vcmpsd_avx(auVar29,auVar31,2);
              auVar32 = vblendvpd_avx(auVar44,auVar42,auVar32);
              auVar24 = vcmpsd_avx(auVar36,auVar31,2);
              auVar24 = vblendvpd_avx(auVar48,auVar46,auVar24);
              dVar2 = (dVar23 - auVar32._0_8_) * auVar29._0_8_ + dVar28 * auVar32._0_8_;
              auVar33 = ZEXT864((ulonglong)dVar2);
              dVar28 = (dVar23 - auVar24._0_8_) * auVar36._0_8_ + dVar28 * auVar24._0_8_;
              auVar35 = ZEXT864((ulonglong)dVar28);
              if ((dVar28 < dVar2) && (dVar3 < dVar2)) {
                *piVar13 = iVar20 + 1;
                uVar14 = (ulonglong)((int)uVar14 + 1);
                piVar13 = piVar13 + 1;
              }
              uVar21 = (ulonglong)(iVar20 + 1);
              uVar22 = uVar22 + 1;
            } while ((longlong)uVar22 < lVar18);
          }
        }
        puVar6 = (uint *)FUN_141167f00(local_118,uVar14,0);
        if (puVar6 != (uint *)0x0) {
          puVar7 = (uint *)FUN_141167dc0(local_d8,local_100,1);
          if (puVar7 != (uint *)0x0) {
            iVar20 = 0;
            puVar15 = puVar7;
            uVar14 = uVar12;
            uVar11 = uVar12;
            if (0 < local_100) {
              do {
                if (param_5 < (double)(int)(*puVar6 - (int)uVar14)) {
                  *puVar15 = *puVar6;
                  uVar14 = (ulonglong)*puVar6;
                  uVar11 = (ulonglong)((int)uVar11 + 1);
                  puVar15 = puVar15 + 1;
                }
                iVar20 = (int)uVar11;
                uVar10 = (int)uVar12 + 1;
                uVar12 = (ulonglong)uVar10;
                puVar6 = puVar6 + 1;
              } while ((int)uVar10 < local_100);
            }
            pcVar8 = (char *)FUN_141169190(param_1 + 0x1978,param_7,1);
            if (pcVar8 != (char *)0x0) {
              lVar4 = (longlong)iVar20;
              if (0 < iVar20) {
                do {
                  pcVar9 = pcVar8;
                  piVar13 = piVar1;
                  lVar18 = (longlong)(param_7 + -1);
                  if (0 < (longlong)(param_7 + -1)) {
                    do {
                      if ((*piVar13 <= (int)*puVar7) && ((int)*puVar7 < piVar13[1])) {
                        *pcVar9 = *pcVar9 + '\x01';
                      }
                      lVar18 = lVar18 + -1;
                      pcVar9 = pcVar9 + 1;
                      piVar13 = piVar13 + 1;
                    } while (lVar18 != 0);
                  }
                  if (piVar1[(longlong)param_7 + -1] <= (int)*puVar7) {
                    pcVar8[(longlong)param_7 + -1] = pcVar8[(longlong)param_7 + -1] + '\x01';
                  }
                  puVar7 = puVar7 + 1;
                  lVar4 = lVar4 + -1;
                } while (lVar4 != 0);
              }
              uVar17 = 1;
              goto LAB_14117ece9;
            }
          }
        }
      }
    }
  }
  uVar17 = 0;
LAB_14117ece9:
  FUN_141167d80(local_d8);
  FUN_141167d80(local_118);
  FUN_141166790(local_f8);
  return uVar17;
}


==== FUN_14117e3a0 @ 14117e3a0 (prof 1)

undefined8
FUN_14117e3a0(longlong param_1,undefined8 *param_2,double param_3,double param_4,undefined8 param_5,
             int param_6)

{
  longlong lVar1;
  longlong lVar2;
  longlong lVar3;
  int iVar4;
  int iVar5;
  int iVar6;
  undefined8 *puVar7;
  longlong lVar8;
  int iVar9;
  undefined8 uVar10;
  int local_c8;
  undefined1 local_c0 [24];
  int local_a8;
  longlong local_a0;
  longlong local_98;
  undefined1 local_90 [32];
  undefined1 local_70 [72];
  
  FUN_141166770(local_c0);
  FUN_141166770(local_70);
  FUN_141167d60(local_90);
  lVar1 = FUN_141168020(local_90,param_5);
  if (lVar1 != 0) {
    lVar2 = FUN_1411667d0(local_c0,*(undefined4 *)(param_2 + 3),0);
    local_a0 = lVar2;
    if (lVar2 != 0) {
      FUN_141166570(param_1 + 0x1c48,*param_2,lVar2,local_a8);
      FUN_141166570(param_1 + 0x1c80,lVar2,lVar2,local_a8);
      FUN_141166570(param_1 + 0x1cb8,lVar2,lVar2,local_a8);
      FUN_141166570(param_1 + 0x1cf0,lVar2,lVar2,local_a8);
      FUN_141167080(local_c0);
      FUN_141166430(param_1 + 0x1d28,lVar2,lVar2,local_a8);
      param_3 = (DAT_145415638 / param_4) * param_3;
      iVar4 = *(int *)(lVar1 + -4 + (longlong)param_6 * 4) + (int)param_3;
      if (local_a8 < iVar4 + -1) {
        iVar4 = local_a8;
      }
      lVar1 = FUN_141167f00(local_90,param_6 + 1,0);
      if (lVar1 != 0) {
        *(int *)(lVar1 + (longlong)param_6 * 4) = iVar4;
        local_98 = FUN_1411667d0(param_1 + 0x1d60,param_6 * 8,1);
        if (local_98 != 0) {
          iVar9 = (int)((double)(int)(param_3 + DAT_1454152a0) * DAT_145415198 + DAT_1454152a0);
          iVar5 = 0;
          iVar4 = 0;
          local_c8 = 0;
          if (0 < param_6) {
            lVar2 = 0;
            lVar8 = 0;
            do {
              iVar6 = 0;
              puVar7 = (undefined8 *)(local_98 + (lVar8 + -1) * 8);
              do {
                if (iVar6 == 0) {
                  iVar5 = *(int *)(lVar1 + lVar2 * 4);
                  iVar4 = iVar5 + iVar9;
                }
                else {
                  if (iVar6 - 1U < 6) {
                    iVar4 = *(int *)(lVar1 + lVar2 * 4);
                    iVar5 = iVar4 + 1 + iVar6 * iVar9;
                    iVar4 = (iVar6 + 1) * iVar9 + iVar4;
                  }
                  if (iVar6 == 7) {
                    iVar5 = iVar9 * 7 + 1 + *(int *)(lVar1 + lVar2 * 4);
                    iVar4 = *(int *)(lVar1 + 4 + lVar2 * 4) + -1;
                  }
                }
                lVar3 = FUN_141166c30(local_70,local_a0 + -8 + (longlong)iVar5 * 8,
                                      (iVar4 - iVar5) + 1);
                if (lVar3 == 0) goto LAB_14117e631;
                lVar8 = lVar8 + 1;
                puVar7 = puVar7 + 1;
                uVar10 = FUN_141167280(local_70);
                *puVar7 = uVar10;
                iVar6 = iVar6 + 1;
              } while (iVar6 < 8);
              local_c8 = local_c8 + 1;
              lVar2 = lVar2 + 1;
            } while (local_c8 < param_6);
          }
          uVar10 = 1;
          goto LAB_14117e633;
        }
      }
    }
  }
LAB_14117e631:
  uVar10 = 0;
LAB_14117e633:
  FUN_141167d80(local_90);
  FUN_141166790(local_70);
  FUN_141166790(local_c0);
  return uVar10;
}


==== FUN_14117b1b0 @ 14117b1b0 (prof 1)

undefined8
FUN_14117b1b0(longlong param_1,longlong *param_2,double param_3,double param_4,undefined8 param_5,
             int param_6,longlong *param_7,int param_8)

{
  uint uVar1;
  ulonglong uVar2;
  longlong lVar3;
  longlong lVar4;
  char cVar5;
  int iVar6;
  int iVar7;
  ulonglong uVar8;
  double *pdVar9;
  double *pdVar10;
  undefined8 *puVar11;
  double *pdVar12;
  int iVar13;
  longlong lVar14;
  longlong lVar15;
  undefined8 uVar16;
  longlong lVar17;
  int iVar18;
  int *piVar19;
  longlong lVar20;
  longlong lVar21;
  int iVar22;
  double dVar23;
  double dVar24;
  double dVar25;
  double dVar26;
  longlong local_d8;
  undefined1 local_b0 [24];
  int local_98;
  undefined1 local_90 [88];
  
  FUN_141166770(local_b0);
  FUN_141166770(local_90);
  lVar3 = *param_7;
  if ((lVar3 != 0) && (lVar4 = *param_2, lVar4 != 0)) {
    iVar6 = (param_8 - param_6) + 1;
    iVar7 = (int)(iVar6 + (iVar6 >> 0x1f & 3U)) >> 2;
    param_3 = (DAT_145415638 / param_4) * param_3;
    iVar13 = (int)param_3;
    iVar6 = iVar13 + 1;
    if (param_3 - (double)iVar13 <= 0.0) {
      iVar6 = iVar13;
    }
    iVar6 = iVar6 * 4 + 0x1001;
    uVar8 = FUN_1411667d0(local_b0,iVar6,1);
    if (((uVar8 != 0) &&
        (pdVar9 = (double *)FUN_1411667d0(local_90,0x2000,1), pdVar9 != (double *)0x0)) &&
       (cVar5 = FUN_1411697a0(param_1 + 0x1a70,iVar7 * 2,0x800,1), cVar5 != '\0')) {
      if (0 < iVar7) {
        local_d8 = 1;
        piVar19 = (int *)((longlong)param_6 * 4 + 0xc + lVar3);
        dVar24 = DAT_143538108;
        dVar25 = DAT_143538100;
        dVar26 = DAT_1454155b0;
        do {
          param_6 = param_6 + 4;
          if ((local_98 < iVar6) && (uVar8 = FUN_1411667d0(local_b0,iVar6,0), uVar8 == 0))
          goto LAB_14117b646;
          if (0 < iVar6) {
            FUN_142db0ac0(uVar8,0,(longlong)iVar6 << 3);
          }
          iVar13 = piVar19[-4];
          if (local_d8 < iVar7) {
            iVar22 = *piVar19;
          }
          else {
            iVar22 = iVar13 + local_98;
          }
          iVar18 = (int)param_2[3];
          if (iVar22 + -1 <= (int)param_2[3]) {
            iVar18 = iVar22 + -1;
          }
          iVar18 = iVar18 - iVar13;
          uVar1 = iVar18 + 1;
          if ((local_98 < (int)uVar1) && (uVar8 = FUN_1411667d0(local_b0,uVar1,0), uVar8 == 0))
          goto LAB_14117b646;
          lVar15 = lVar4 + (longlong)iVar13 * 8;
          uVar2 = lVar15 - 8;
          lVar17 = 0;
          lVar20 = (longlong)(int)uVar1;
          if ((((int)uVar1 < 1) || (uVar1 < 4)) ||
             ((uVar8 <= uVar2 + (longlong)iVar18 * 8 && (uVar2 <= uVar8 + (longlong)iVar18 * 8)))) {
            if (0 < lVar20) {
              if (3 < lVar20) {
                puVar11 = (undefined8 *)(lVar15 + 8);
                lVar14 = uVar8 - uVar2;
                lVar21 = (lVar20 - 4U >> 2) + 1;
                lVar17 = lVar21 * 4;
                do {
                  *(undefined8 *)(lVar14 + -0x10 + (longlong)puVar11) = puVar11[-2];
                  *(undefined8 *)(lVar14 + -8 + (longlong)puVar11) =
                       *(undefined8 *)((longlong)puVar11 + (lVar15 - uVar2) + -0x10);
                  *(undefined8 *)(lVar14 + (longlong)puVar11) = *puVar11;
                  *(undefined8 *)(lVar14 + 8 + (longlong)puVar11) = puVar11[1];
                  puVar11 = puVar11 + 4;
                  lVar21 = lVar21 + -1;
                } while (lVar21 != 0);
                if (lVar20 <= lVar17) goto LAB_14117b49d;
              }
              puVar11 = (undefined8 *)(uVar8 + lVar17 * 8);
              lVar20 = lVar20 - lVar17;
              do {
                *puVar11 = *(undefined8 *)((longlong)puVar11 + (uVar2 - uVar8));
                puVar11 = puVar11 + 1;
                lVar20 = lVar20 + -1;
              } while (lVar20 != 0);
            }
          }
          else {
            FUN_142db0e80(uVar8,uVar2,lVar20 * 8);
          }
LAB_14117b49d:
          iVar18 = 0;
          iVar22 = param_6;
          do {
            pdVar12 = pdVar9 + 2;
            lVar17 = uVar8 + (longlong)(*(int *)(lVar3 + -0x14 + (longlong)iVar22 * 4) - iVar13) * 8
            ;
            lVar15 = 0x18;
            do {
              pdVar12[-2] = *(double *)(lVar17 + -0x18 + lVar15);
              pdVar12[-1] = 0.0;
              *pdVar12 = *(double *)(lVar15 + -0x10 + lVar17);
              pdVar12[1] = 0.0;
              pdVar12[2] = *(double *)(lVar17 + -8 + lVar15);
              pdVar12[3] = 0.0;
              pdVar12[4] = *(double *)(lVar15 + lVar17);
              pdVar12[5] = 0.0;
              pdVar12[6] = *(double *)(lVar17 + 8 + lVar15);
              pdVar12[7] = 0.0;
              pdVar12[8] = *(double *)(lVar17 + 0x10 + lVar15);
              pdVar12[9] = 0.0;
              pdVar12[10] = *(double *)(lVar17 + 0x18 + lVar15);
              pdVar12[0xb] = 0.0;
              pdVar12[0xc] = *(double *)(lVar17 + 0x20 + lVar15);
              pdVar12[0xd] = 0.0;
              lVar15 = lVar15 + 0x40;
              pdVar12 = pdVar12 + 0x10;
            } while (lVar15 < 0x8018);
            cVar5 = FUN_1411655f0(param_1 + 0x1a38,pdVar9);
            if ((cVar5 == '\0') || (cVar5 = FUN_141164e90(param_1 + 0x1a38,pdVar9), cVar5 == '\0'))
            goto LAB_14117b646;
            pdVar10 = (double *)FUN_141169a20(param_1 + 0x1a70);
            lVar17 = 0x800;
            pdVar12 = pdVar9;
            dVar23 = dVar24;
            do {
              dVar24 = dVar23;
              if (dVar25 <= SQRT(pdVar12[1] * pdVar12[1] + *pdVar12 * *pdVar12)) {
                dVar23 = (double)FUN_1427a25f0();
                dVar23 = dVar23 * dVar26;
              }
              *pdVar10 = dVar23;
              pdVar12 = pdVar12 + 2;
              pdVar10 = pdVar10 + 1;
              lVar17 = lVar17 + -1;
              dVar23 = dVar24;
            } while (lVar17 != 0);
            iVar18 = iVar18 + 1;
            iVar22 = iVar22 + 2;
          } while (iVar18 < 2);
          local_d8 = local_d8 + 1;
          piVar19 = piVar19 + 4;
        } while (local_d8 <= iVar7);
      }
      uVar16 = 1;
      goto LAB_14117b648;
    }
  }
LAB_14117b646:
  uVar16 = 0;
LAB_14117b648:
  FUN_141166790(local_90);
  FUN_141166790(local_b0);
  return uVar16;
}


==== FUN_1411803f0 @ 1411803f0 (prof 1)

undefined8
FUN_1411803f0(longlong param_1,longlong param_2,double param_3,int param_4,int param_5,
             double param_6,int param_7)

{
  double *pdVar1;
  int *piVar2;
  double dVar3;
  double dVar4;
  char cVar5;
  uint uVar6;
  longlong lVar7;
  double *pdVar8;
  ulonglong uVar9;
  undefined8 *puVar10;
  int iVar11;
  double *pdVar12;
  undefined8 uVar13;
  double *pdVar14;
  int *piVar15;
  int iVar16;
  ulonglong uVar17;
  longlong lVar18;
  longlong lVar19;
  int iVar20;
  int iVar21;
  longlong lVar22;
  undefined1 auVar23 [16];
  double dVar24;
  double dVar25;
  double dVar26;
  longlong local_b8;
  undefined1 local_88 [24];
  int local_70;
  
  FUN_141166770(local_88);
  piVar2 = *(int **)(param_1 + 0x19c0);
  if (((piVar2 != (int *)0x0) && (lVar19 = *(longlong *)(param_1 + 0x19e0), lVar19 != 0)) &&
     (uVar9 = *(ulonglong *)(param_1 + 0x1998), uVar9 != 0)) {
    auVar23._8_8_ = 0;
    auVar23._0_8_ =
         ((DAT_145415378 / param_3) * (double)(param_5 + -1) * DAT_1435381b8 +
         (double)param_4 * DAT_1435381b0) * param_6;
    auVar23 = vroundsd_avx(ZEXT816(0) << 0x40,auVar23,2);
    *(int *)(param_1 + 0x1a34) = (int)auVar23._0_8_;
    local_b8 = FUN_1411667d0(local_88,*(int *)(param_2 + 0x18) + *(int *)(param_1 + 0x19b8),1);
    if (local_b8 != 0) {
      if (*(int *)(param_1 + 0x19b8) == 0) {
        local_b8 = FUN_141166a30(local_88,param_2);
      }
      else if ((0 < *(int *)(param_1 + 0x19b8)) &&
              ((lVar22 = FUN_141166bc0(local_88,*(undefined8 *)(param_1 + 0x1998)), lVar22 == 0 ||
               (local_b8 = FUN_141166cc0(local_88,param_2), local_b8 == 0)))) goto LAB_14118087e;
      iVar20 = (int)((double)(local_70 - *piVar2) * DAT_145414f10);
      cVar5 = FUN_1411697a0(param_1 + 0x1a18,0x48,iVar20,1);
      if (cVar5 != '\0') {
        dVar3 = (double)*piVar2 * DAT_1454152a0;
        if (0 < iVar20) {
          lVar22 = 0;
          iVar21 = 1;
          dVar26 = DAT_1454152a0;
          do {
            iVar16 = 0x17;
            pdVar14 = (double *)(lVar19 + 0xb8);
            piVar15 = piVar2 + 0x17;
            do {
              dVar4 = (double)*piVar15 * dVar26;
              lVar7 = FUN_141169a20(param_1 + 0x1a18,iVar16);
              if (lVar7 == 0) goto LAB_14118087e;
              dVar24 = 0.0;
              dVar25 = 0.0;
              pdVar12 = (double *)
                        (local_b8 + -8 +
                        ((longlong)((int)dVar3 - (int)dVar4) + (longlong)iVar21) * 8);
              pdVar8 = (double *)FUN_141169a20(param_1 + 0x1a00,iVar16);
              if (pdVar8 == (double *)0x0) goto LAB_14118087e;
              iVar11 = *piVar15;
              if (3 < iVar11) {
                uVar6 = (iVar11 - 4U >> 2) + 1;
                uVar17 = (ulonglong)uVar6;
                iVar11 = iVar11 + uVar6 * -4;
                do {
                  dVar25 = pdVar12[3] * pdVar8[7] +
                           pdVar12[2] * pdVar8[5] +
                           pdVar12[1] * pdVar8[3] + dVar25 + *pdVar12 * pdVar8[1];
                  dVar24 = pdVar12[3] * pdVar8[6] +
                           pdVar12[2] * pdVar8[4] +
                           pdVar12[1] * pdVar8[2] + dVar24 + *pdVar12 * *pdVar8;
                  pdVar8 = pdVar8 + 8;
                  pdVar12 = pdVar12 + 4;
                  uVar17 = uVar17 - 1;
                } while (uVar17 != 0);
              }
              for (; 0 < iVar11; iVar11 = iVar11 + -1) {
                pdVar1 = pdVar8 + 1;
                dVar24 = dVar24 + *pdVar12 * *pdVar8;
                pdVar8 = pdVar8 + 2;
                dVar25 = dVar25 + *pdVar12 * *pdVar1;
                pdVar12 = pdVar12 + 1;
              }
              *(double *)(lVar7 + lVar22 * 8) = SQRT(dVar25 * dVar25 + dVar24 * dVar24) * *pdVar14;
              iVar16 = iVar16 + 1;
              piVar15 = piVar15 + 1;
              pdVar14 = pdVar14 + 1;
            } while (iVar16 < 0x48);
            iVar21 = iVar21 + 0x100;
            lVar22 = lVar22 + 1;
          } while (lVar22 < iVar20);
        }
        iVar21 = 0;
        if (param_7 < 2) {
          uVar6 = local_70 + iVar20 * -0x100;
          if ((*(int *)(param_1 + 0x19b0) < (int)uVar6) &&
             (uVar9 = FUN_1411667d0(param_1 + 0x1998,uVar6,0), uVar9 == 0)) goto LAB_14118087e;
          uVar17 = local_b8 + (longlong)(iVar20 * 0x100) * 8;
          if (((0 < (int)uVar6) && (3 < uVar6)) &&
             ((uVar17 + (longlong)(int)(uVar6 - 1) * 8 < uVar9 ||
              (uVar9 + (longlong)(int)(uVar6 - 1) * 8 < uVar17)))) {
            FUN_142db0e80(uVar9,uVar17,(longlong)(int)uVar6 * 8);
            do {
              iVar21 = iVar21 + 1;
            } while (iVar21 < (int)uVar6);
          }
          lVar19 = (longlong)iVar21;
          lVar22 = (longlong)(int)uVar6;
          if (lVar19 < lVar22) {
            if (3 < lVar22 - lVar19) {
              puVar10 = (undefined8 *)(uVar9 + (lVar19 + 1) * 8);
              lVar7 = uVar17 - uVar9;
              lVar18 = ((lVar22 - lVar19) - 4U >> 2) + 1;
              lVar19 = lVar19 + lVar18 * 4;
              do {
                puVar10[-1] = *(undefined8 *)(lVar7 + -8 + (longlong)puVar10);
                *puVar10 = *(undefined8 *)(lVar7 + (longlong)puVar10);
                puVar10[1] = *(undefined8 *)(lVar7 + 8 + (longlong)puVar10);
                puVar10[2] = *(undefined8 *)(lVar7 + 0x10 + (longlong)puVar10);
                puVar10 = puVar10 + 4;
                lVar18 = lVar18 + -1;
              } while (lVar18 != 0);
              if (lVar22 <= lVar19) goto LAB_14118086c;
            }
            puVar10 = (undefined8 *)(uVar9 + lVar19 * 8);
            lVar22 = lVar22 - lVar19;
            do {
              *puVar10 = *(undefined8 *)((uVar17 - uVar9) + (longlong)puVar10);
              puVar10 = puVar10 + 1;
              lVar22 = lVar22 + -1;
            } while (lVar22 != 0);
          }
LAB_14118086c:
          *(uint *)(param_1 + 0x19b8) = uVar6;
        }
        *(int *)(param_1 + 0x1a30) = iVar20;
        uVar13 = 1;
        goto LAB_141180880;
      }
    }
  }
LAB_14118087e:
  uVar13 = 0;
LAB_141180880:
  FUN_141166790(local_88);
  return uVar13;
}


==== FUN_14118d310 @ 14118d310 (prof 1)

/* WARNING: Function: __security_check_cookie replaced with injection: security_check_cookie */

undefined8
FUN_14118d310(longlong param_1,undefined8 *param_2,double param_3,int param_4,uint param_5,
             double param_6,uint param_7,longlong *param_8,int param_9)

{
  double dVar1;
  bool bVar2;
  int iVar3;
  double *pdVar4;
  longlong lVar5;
  longlong lVar6;
  longlong lVar7;
  double *pdVar8;
  uint uVar9;
  int iVar10;
  int *piVar11;
  undefined8 uVar12;
  ulonglong uVar13;
  ulonglong uVar14;
  int iVar15;
  uint uVar16;
  int iVar17;
  ulonglong uVar18;
  uint uVar19;
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
  undefined1 auVar30 [16];
  undefined1 auVar31 [16];
  undefined1 auVar32 [16];
  double dVar33;
  undefined1 auVar34 [64];
  double dVar35;
  double dVar36;
  undefined1 auVar37 [64];
  undefined1 auVar38 [64];
  undefined1 auVar39 [64];
  undefined1 auVar40 [64];
  undefined1 in_ZMM12 [64];
  undefined1 auVar41 [64];
  undefined1 in_ZMM13 [64];
  undefined1 auVar42 [64];
  undefined1 auVar43 [64];
  undefined1 auVar44 [64];
  undefined1 auVar45 [64];
  undefined1 auVar46 [64];
  undefined1 auStack_308 [32];
  longlong local_2e8;
  int local_2e0;
  double local_2d8;
  int local_2d0;
  uint local_2cc;
  uint local_2c8;
  uint local_2c4;
  uint local_2c0;
  uint local_2bc;
  double *local_2b8;
  uint local_2b0;
  longlong local_2a8;
  int *local_2a0;
  uint local_298;
  int local_294;
  undefined1 local_290 [24];
  int local_278;
  undefined1 local_270 [24];
  undefined4 local_258;
  double local_250;
  longlong local_248;
  longlong *local_240;
  longlong local_238;
  longlong local_230;
  double local_228;
  double local_218 [6];
  ulonglong uStack_1e8;
  ulonglong uStack_1e0;
  ulonglong local_1d8;
  ulonglong uStack_1d0;
  ulonglong uStack_1c8;
  ulonglong uStack_1c0;
  ulonglong local_1b8;
  ulonglong uStack_1b0;
  ulonglong uStack_1a8;
  ulonglong uStack_1a0;
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
  local_f8 = DAT_1459ca6c0 ^ (ulonglong)auStack_308;
  local_240 = param_8;
  local_2a8 = param_1;
  FUN_141166770(local_270);
  FUN_141166770(local_290);
  FUN_141166770(local_178);
  FUN_141166770(local_138);
  FUN_141166770(local_198);
  FUN_141166770(local_158);
  dVar35 = DAT_1454152a0;
  local_2d8 = *(double *)(param_1 + 0x1930);
  auVar38 = ZEXT864((ulonglong)local_2d8);
  local_2c8 = *(uint *)(param_1 + 0x1938);
  auVar31._8_8_ = 0;
  auVar31._0_8_ = *(ulonglong *)(param_1 + 0x1940);
  local_238 = *param_8;
  if (local_238 != 0) {
    local_2c4 = *(uint *)(param_2 + 3);
    dVar21 = (DAT_145415638 / param_3) * param_6;
    auVar45 = ZEXT864((ulonglong)dVar21);
    auVar39 = ZEXT864((ulonglong)DAT_145415208);
    dVar36 = dVar21 * DAT_145415208;
    iVar20 = (int)dVar36;
    auVar43 = ZEXT864((ulonglong)(dVar21 * DAT_1454154c0));
    uVar19 = (uint)(dVar21 * (double)(int)(param_5 - 1) +
                   (double)((int)(param_6 * (double)param_4 * DAT_145414e88 + DAT_1454152a0) + 1));
    uVar16 = *(uint *)(local_238 + -4 + (longlong)param_9 * 4);
    uVar9 = (uVar16 - 1) + (int)dVar21;
    if ((int)uVar9 <= (int)local_2c4) {
      uVar16 = uVar9;
    }
    iVar3 = (param_9 - param_5) + 1;
    local_2d0 = (int)(iVar3 + (iVar3 >> 0x1f & 3U)) >> 2;
    auVar26 = vpcmpgtd_avx(ZEXT416(param_7),ZEXT416(1));
    auVar26 = vshufps_avx(auVar26,auVar26,0);
    auVar23._8_8_ = 0;
    auVar23._0_8_ = DAT_145414da0;
    auVar31 = vblendvpd_avx(auVar23,auVar31,auVar26);
    local_250 = auVar31._0_8_;
    *(undefined4 *)(param_1 + 0x1938) = 0;
    local_2c0 = uVar16;
    local_298 = uVar19;
    local_294 = iVar20;
    local_228 = dVar36;
    pdVar4 = (double *)FUN_1411667d0(local_270,*(undefined4 *)(param_2 + 3),0);
    local_2b8 = pdVar4;
    if (pdVar4 != (double *)0x0) {
      if (*(int *)(param_1 + 0x16f8) == 4) {
        FUN_141166570(param_1 + 0x1700,*param_2,pdVar4,local_258);
        FUN_141166570(param_1 + 0x1738,pdVar4,pdVar4,local_258);
        FUN_141166570(param_1 + 6000,pdVar4,pdVar4,local_258);
        lVar5 = param_1 + 0x17a8;
        pdVar8 = pdVar4;
LAB_14118d5ad:
        FUN_141166570(lVar5,pdVar8,pdVar4,local_258);
      }
      else if (*(int *)(param_1 + 0x16f8) == 2) {
        lVar5 = param_1 + 0x1700;
        pdVar8 = (double *)*param_2;
        goto LAB_14118d5ad;
      }
      lVar5 = FUN_141166a30(local_290,local_270);
      if (lVar5 != 0) {
        FUN_141167080(local_290);
        FUN_141166570(param_1 + 0x17e0,lVar5,lVar5,local_278);
        local_2bc = uVar16 - uVar19;
        dVar21 = auVar43._0_8_;
        if (local_278 < (int)((int)dVar21 * local_2d0 + -1 + uVar19)) {
          local_2d0 = local_2d0 + -1;
        }
        if (local_2d0 < 1) {
          lVar6 = FUN_141169190(param_1 + 0x1948,1,1);
          if (lVar6 != 0) {
            **(undefined1 **)(param_1 + 0x1948) = 0;
            *(undefined8 *)(param_1 + 0x1930) =
                 *(undefined8 *)(lVar5 + -8 + (longlong)(int)local_2c4 * 8);
            *(double *)(param_1 + 0x1940) = local_250;
            *(uint *)(param_1 + 0x1938) = local_2c8;
            uVar12 = 1;
            goto LAB_14118dbd3;
          }
        }
        else {
          local_230 = FUN_141169190(param_1 + 0x1948,local_2d0 << 4,1);
          if (local_230 != 0) {
            local_2b0 = 0;
            auVar37 = ZEXT1664(ZEXT816(0));
            uVar16 = 0;
            if (local_2d0 < 2) {
LAB_14118da11:
              *(uint *)(param_1 + 0x1938) = uVar16;
              if (1 < (int)param_7) {
                uVar16 = local_2c8;
              }
              FUN_141166570(param_1 + 0x1818,pdVar4,pdVar4,local_258);
              FUN_141166570(param_1 + 0x1850,pdVar4,pdVar4,local_258);
              FUN_141166570(param_1 + 0x1888,pdVar4,pdVar4,local_258);
              FUN_141166570(param_1 + 0x18c0,pdVar4,pdVar4,local_258);
              pdVar4 = (double *)FUN_141166a30(local_290,local_270);
              local_2b8 = pdVar4;
              if (pdVar4 != (double *)0x0) {
                FUN_141167080(local_290);
                FUN_141166570(param_1 + 0x18f8,pdVar4,pdVar4);
                lVar5 = (longlong)(int)(local_2c4 - 1);
                *(double *)(param_1 + 0x1930) = pdVar4[lVar5];
                if (3 < lVar5) {
                  pdVar8 = pdVar4 + lVar5 + -2;
                  lVar6 = (lVar5 - 4U >> 2) + 1;
                  lVar5 = lVar5 + lVar6 * -4;
                  do {
                    pdVar8[2] = pdVar8[2] - pdVar8[1];
                    pdVar8[1] = pdVar8[1] - *pdVar8;
                    *pdVar8 = *pdVar8 - pdVar8[-1];
                    pdVar8[-1] = pdVar8[-1] - pdVar8[-2];
                    pdVar8 = pdVar8 + -4;
                    lVar6 = lVar6 + -1;
                  } while (lVar6 != 0);
                }
                for (; 0 < lVar5; lVar5 = lVar5 + -1) {
                  pdVar4[lVar5] = pdVar4[lVar5] - pdVar4[lVar5 + -1];
                }
                *pdVar4 = *pdVar4 - auVar38._0_8_;
                if (((int)uVar16 < (int)uVar19) && ((int)(local_2c0 - uVar16) <= local_278)) {
                  lVar5 = FUN_141166bc0(local_198,
                                        pdVar4 + ((longlong)(int)uVar19 - (longlong)(int)uVar16) +
                                                 -1,local_2bc + 1);
                  if ((lVar5 == 0) ||
                     (pdVar4 = (double *)FUN_141166a30(local_290,local_198), local_2b8 = pdVar4,
                     pdVar4 == (double *)0x0)) goto LAB_14118dbd1;
                }
                else if (((int)((local_2c0 - uVar16) + -1) <= local_278) &&
                        ((int)uVar19 < (int)uVar16)) {
                  lVar5 = FUN_141166bc0(local_198,pdVar4);
                  if ((lVar5 == 0) ||
                     ((local_278 < (int)local_2bc &&
                      (pdVar4 = (double *)FUN_1411667d0(local_290,local_2bc,0), local_2b8 = pdVar4,
                      pdVar4 == (double *)0x0)))) goto LAB_14118dbd1;
                  iVar3 = (uVar16 - uVar19) + 1;
                  if (0 < iVar3) {
                    FUN_142db0ac0(pdVar4,0,(longlong)iVar3 << 3);
                  }
                  pdVar8 = pdVar4 + (longlong)(int)(uVar16 - uVar19) + 1;
                  iVar3 = 0;
                  if (0 < local_180) {
                    lVar5 = lVar5 - (longlong)pdVar8;
                    do {
                      *pdVar8 = *(double *)(lVar5 + (longlong)pdVar8);
                      iVar3 = iVar3 + 1;
                      pdVar8 = pdVar8 + 1;
                    } while (iVar3 < local_180);
                  }
                }
                piVar11 = (int *)(longlong)(int)param_5;
                lVar5 = 0;
                local_2e0 = 0;
                local_2b0 = 10;
                local_2bc = 0xb;
                local_2c0 = 0xc;
                local_2c4 = 0xd;
                local_2c8 = 0xe;
                local_2cc = 0xf;
                local_2d8 = local_250;
                local_2a0 = piVar11;
                do {
                  lVar6 = 0;
                  local_218[0] = 0.0;
                  local_218[1] = 0.0;
                  local_218[2] = 0.0;
                  local_218[3] = 0.0;
                  local_218[4] = 0.0;
                  local_218[5] = 0.0;
                  uStack_1e8 = 0;
                  uStack_1e0 = 0;
                  local_1d8 = 0;
                  uStack_1d0 = 0;
                  uStack_1c8 = 0;
                  uStack_1c0 = 0;
                  local_1b8 = 0;
                  uStack_1b0 = 0;
                  uStack_1a8 = 0;
                  uStack_1a0 = 0;
                  uVar18 = (ulonglong)piVar11 & 0xffffffff;
                  do {
                    uVar13 = 0;
                    if ((int)local_240[3] < (int)uVar18) goto LAB_14118e102;
                    dVar21 = (double)(int)(*(int *)(local_238 + -4 +
                                                   ((longlong)local_2a0 + lVar5 * 4 + lVar6) * 4) -
                                          local_298);
                    uVar14 = uVar13;
                    do {
                      iVar3 = (int)((double)(int)uVar13 * dVar36 + dVar21 + dVar35);
                      if (local_278 < iVar3 + iVar20) goto LAB_14118e102;
                      auVar37 = ZEXT1664(auVar37._0_16_);
                      lVar7 = FUN_141166c30(local_158,pdVar4 + iVar3,iVar20);
                      if (lVar7 == 0) goto LAB_14118dbd1;
                      dVar22 = (double)FUN_141167280(local_158);
                      local_218[uVar14 + lVar6 * 4] = dVar22;
                      uVar16 = (int)uVar13 + 1;
                      uVar13 = (ulonglong)uVar16;
                      uVar14 = uVar14 + 1;
                    } while ((int)uVar16 < 4);
                    uVar18 = (ulonglong)((int)uVar18 + 1);
                    lVar6 = lVar6 + 1;
                  } while (lVar6 < 4);
                  iVar20 = 0;
                  auVar38 = ZEXT864(uStack_1a0);
                  auVar39 = ZEXT864(uStack_1a8);
                  auVar43 = ZEXT864(uStack_1b0);
                  auVar41 = ZEXT864(local_1b8);
                  auVar40 = ZEXT864(uStack_1c0);
                  auVar34 = ZEXT864(uStack_1c8);
                  auVar42 = ZEXT864(uStack_1d0);
                  auVar44 = ZEXT864(local_1d8);
                  auVar46 = ZEXT864(uStack_1e0);
                  auVar45 = ZEXT864(uStack_1e8);
                  do {
                    auVar26._8_8_ = 0;
                    auVar26._0_8_ = local_218[1];
                    auVar32._8_8_ = 0;
                    auVar32._0_8_ = local_218[0];
                    auVar31 = vcmpsd_avx(auVar32,auVar26,1);
                    auVar31 = vblendvpd_avx(auVar32,auVar26,auVar31);
                    uVar16 = (uint)(local_218[0] < local_218[1]);
                    auVar27._8_8_ = 0;
                    auVar27._0_8_ = local_218[2];
                    if (auVar31._0_8_ < local_218[2]) {
                      uVar16 = 2;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar27,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar27,auVar26);
                    auVar28._8_8_ = 0;
                    auVar28._0_8_ = local_218[3];
                    if (auVar31._0_8_ < local_218[3]) {
                      uVar16 = 3;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar28,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar28,auVar26);
                    auVar24._8_8_ = 0;
                    auVar24._0_8_ = local_218[4];
                    if (auVar31._0_8_ < local_218[4]) {
                      uVar16 = 4;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar24,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar24,auVar26);
                    auVar29._8_8_ = 0;
                    auVar29._0_8_ = local_218[5];
                    if (auVar31._0_8_ < local_218[5]) {
                      uVar16 = 5;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar29,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar29,auVar26);
                    if (auVar31._0_8_ < auVar45._0_8_) {
                      uVar16 = 6;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar45._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar45._0_16_,auVar26);
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
                    if (auVar31._0_8_ < auVar34._0_8_) {
                      uVar16 = local_2b0;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar34._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar34._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar40._0_8_) {
                      uVar16 = local_2bc;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar40._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar40._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar41._0_8_) {
                      uVar16 = local_2c0;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar41._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar41._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar43._0_8_) {
                      uVar16 = local_2c4;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar43._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar43._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar39._0_8_) {
                      uVar16 = local_2c8;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar39._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar39._0_16_,auVar26);
                    if (auVar31._0_8_ < auVar38._0_8_) {
                      uVar16 = local_2cc;
                    }
                    auVar26 = vcmpsd_avx(auVar31,auVar38._0_16_,1);
                    auVar31 = vblendvpd_avx(auVar31,auVar38._0_16_,auVar26);
                    auVar30._0_8_ = -(ulonglong)(iVar20 == 0);
                    auVar30._8_8_ = 0xffffffffffffffff;
                    auVar26 = vblendvpd_avx(auVar37._0_16_,auVar31,auVar30);
                    auVar37 = ZEXT1664(auVar26);
                    auVar25._8_8_ = 0;
                    auVar25._0_8_ = local_2d8;
                    dVar21 = auVar31._0_8_;
                    if (((DAT_145414da0 < dVar21) && (auVar26._0_8_ * dVar35 < dVar21)) &&
                       (local_2d8 * DAT_143538390 < dVar21)) {
                      *(undefined1 *)((int)(uVar16 + local_2e0) + local_230) = 1;
                      local_218[(int)uVar16] = 0.0;
                      auVar38 = ZEXT864(uStack_1a0);
                      auVar39 = ZEXT864(uStack_1a8);
                      auVar43 = ZEXT864(uStack_1b0);
                      auVar41 = ZEXT864(local_1b8);
                      auVar40 = ZEXT864(uStack_1c0);
                      auVar34 = ZEXT864(uStack_1c8);
                      auVar42 = ZEXT864(uStack_1d0);
                      auVar44 = ZEXT864(local_1d8);
                      auVar46 = ZEXT864(uStack_1e0);
                      auVar45 = ZEXT864(uStack_1e8);
                    }
                    iVar20 = iVar20 + 1;
                  } while (iVar20 < 0xc);
                  auVar31 = vmaxsd_avx(auVar26,auVar25);
                  local_2d8 = auVar31._0_8_;
                  param_5 = param_5 + 4;
                  piVar11 = (int *)(ulonglong)param_5;
                  local_2e0 = local_2e0 + 0x10;
                  lVar5 = lVar5 + 1;
                  pdVar4 = local_2b8;
                  iVar20 = local_294;
                  dVar36 = local_228;
                } while (lVar5 < local_2d0);
LAB_14118e102:
                *(double *)(local_2a8 + 0x1940) = local_2d8;
                uVar12 = 1;
                goto LAB_14118dbd3;
              }
            }
            else {
              uVar9 = 0;
              local_2cc = 0;
              local_118[0] = 0;
              local_118[1] = 0;
              local_118[2] = 0;
              local_118[3] = 0;
              local_118[4] = 0;
              local_118[5] = 0;
              local_118[6] = 0;
              local_118[7] = 0;
              iVar3 = (local_2d0 + -1) * (int)dVar21;
              auVar37 = ZEXT1664(ZEXT816(0));
              auVar38 = ZEXT1664(auVar39._0_16_);
              auVar39 = ZEXT1664(ZEXT816(0));
              auVar43 = ZEXT1664(auVar43._0_16_);
              auVar45 = ZEXT1664(auVar45._0_16_);
              lVar6 = FUN_1411667d0(local_178,iVar3,1);
              if (lVar6 != 0) {
                local_248 = (longlong)(int)uVar19;
                local_2e8 = FUN_141166bc0(local_178,
                                          lVar5 + ((longlong)(int)auVar43._0_8_ + -1 + local_248) *
                                                  8,iVar3);
                if (local_2e8 != 0) {
                  local_2e0 = 0;
                  piVar11 = local_118;
                  auVar41 = ZEXT864(DAT_143537ec0);
                  do {
                    if ((0 < local_2e0) && (piVar11[-1] <= local_160)) {
                      *(undefined8 *)(local_2e8 + -8 + (longlong)piVar11[-1] * 8) = 0;
                    }
                    local_2a0 = piVar11;
                    auVar40._0_8_ = (double)FUN_1411672c0(local_178,local_118 + local_2e0,1);
                    auVar40._8_56_ = extraout_var;
                    auVar34 = ZEXT1664(auVar40._0_16_);
                    iVar3 = 0xffff;
                    dVar21 = auVar43._0_8_;
                    if (dVar21 != auVar37._0_8_) {
                      iVar3 = (int)((double)*piVar11 / dVar21);
                    }
                    iVar3 = (iVar3 + 1) * (int)dVar21;
                    dVar22 = *(double *)(lVar5 + (longlong)(local_278 + -1) * 8);
                    lVar6 = (longlong)(int)(iVar3 + uVar19);
                    if ((int)(iVar3 + uVar19 + -1) <= local_278) {
                      dVar22 = *(double *)(lVar5 + -0x10 + lVar6 * 8);
                    }
                    if ((auVar40._0_8_ * DAT_1454151e0 < dVar22) &&
                       (iVar3 < local_160 - (int)dVar21)) {
                      lVar7 = FUN_141166c30(local_138);
                      if (lVar7 == 0) goto LAB_14118dbd1;
                      dVar21 = *(double *)(lVar5 + -8 + lVar6 * 8);
                      if (((dVar22 <= dVar21) &&
                          (dVar22 = *(double *)(lVar5 + lVar6 * 8), dVar21 <= dVar22)) &&
                         (bVar2 = *(double *)(lVar5 + 8 + lVar6 * 8) < dVar22, !bVar2)) {
                        iVar15 = 0;
                        iVar10 = 0;
                        if (0 < iVar20) {
                          pdVar4 = (double *)
                                   (lVar5 + ((longlong)iVar3 + -2 + (longlong)(int)uVar19) * 8);
                          pdVar8 = pdVar4;
                          do {
                            iVar15 = iVar15 + 1;
                            if (bVar2) {
                              dVar21 = pdVar4[-2] - pdVar4[-1];
                            }
                            else {
                              dVar21 = pdVar8[2] - pdVar8[1];
                            }
                            if (dVar21 < auVar37._0_8_) {
                              if (bVar2) {
                                iVar15 = -iVar15;
                              }
                              uVar9 = iVar15 + iVar3;
                              auVar39 = ZEXT864(*(ulonglong *)
                                                 (lVar5 + -0x10 +
                                                 (longlong)(int)(uVar9 + uVar19) * 8));
                              break;
                            }
                            iVar10 = iVar10 + 1;
                            pdVar4 = pdVar4 + -1;
                            pdVar8 = pdVar8 + 1;
                          } while (iVar10 < iVar20);
                        }
                        if (auVar34._0_8_ * DAT_143538398 < auVar39._0_8_) {
                          dVar21 = (double)FUN_141167340();
                          dVar22 = auVar34._0_8_;
                          bVar2 = false;
                          iVar15 = 0;
                          iVar17 = 0;
                          iVar10 = (int)(auVar45._0_8_ * DAT_143538128);
                          if (0 < iVar10) {
                            lVar6 = 0;
                            pdVar4 = (double *)
                                     (lVar5 + ((longlong)(int)uVar19 + -3 + (longlong)(int)uVar9) *
                                              8);
                            uVar16 = uVar9;
                            do {
                              uVar16 = uVar16 - 1;
                              lVar6 = lVar6 + 1;
                              pdVar4 = pdVar4 + -1;
                              dVar1 = *(double *)
                                       (lVar5 + -0x10 + (((int)uVar9 - lVar6) + local_248) * 8);
                              if (bVar2) {
                                dVar33 = auVar34._0_8_;
                                if (dVar1 <= dVar33 * auVar41._0_8_) {
                                  iVar15 = iVar15 + 1;
                                  if ((int)(dVar36 * auVar38._0_8_) <= iVar15) {
                                    if (dVar33 == auVar37._0_8_) {
                                      uVar16 = 0;
                                      goto LAB_14118da01;
                                    }
                                    if (DAT_1435383a8 < auVar39._0_8_ / dVar33) {
                                      uVar16 = (uint)(((double)iVar3 - (double)(int)local_2cc) +
                                                     dVar35);
                                      auVar38 = ZEXT864((ulonglong)local_2d8);
                                      pdVar4 = local_2b8;
                                      param_1 = local_2a8;
                                      if ((int)uVar19 <= (int)uVar16) {
                                        uVar16 = uVar19;
                                      }
                                      goto LAB_14118da11;
                                    }
                                    break;
                                  }
                                }
                                else {
                                  bVar2 = false;
                                  iVar15 = 0;
                                }
                              }
                              else if ((dVar1 <= *pdVar4) && (dVar1 < (dVar22 + dVar21) * dVar35)) {
                                bVar2 = true;
                                auVar34 = ZEXT864((ulonglong)dVar1);
                                local_2cc = uVar16;
                              }
                              iVar17 = iVar17 + 1;
                            } while (iVar17 < iVar10);
                          }
                        }
                      }
                    }
                    local_2e0 = local_2e0 + 1;
                    piVar11 = local_2a0 + 1;
                    uVar16 = local_2b0;
                    local_2a0 = piVar11;
                  } while (local_2e0 < 8);
LAB_14118da01:
                  auVar38 = ZEXT864((ulonglong)local_2d8);
                  pdVar4 = local_2b8;
                  param_1 = local_2a8;
                  goto LAB_14118da11;
                }
              }
            }
          }
        }
      }
    }
  }
LAB_14118dbd1:
  uVar12 = 0;
LAB_14118dbd3:
  FUN_141166790(local_158);
  FUN_141166790(local_198);
  FUN_141166790(local_138);
  FUN_141166790(local_178);
  FUN_141166790(local_290);
  FUN_141166790(local_270);
  return uVar12;
}


==== FUN_141166a30 @ 141166a30 (prof 1)

longlong FUN_141166a30(longlong param_1,longlong *param_2)

{
  longlong lVar1;
  
  lVar1 = FUN_1411667d0(param_1,(int)param_2[3],0);
  if (lVar1 == 0) {
    return 0;
  }
  if (0 < (int)param_2[3]) {
    if (*param_2 == 0) {
      return 0;
    }
    FUN_142db0e80(lVar1,*param_2,(longlong)*(int *)(param_1 + 0x18) << 3);
  }
  return lVar1;
}


