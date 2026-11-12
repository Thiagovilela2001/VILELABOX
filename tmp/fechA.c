==== FUN_141195480 @ 141195480 (prof 0)

undefined8 FUN_141195480(longlong param_1,undefined8 *param_2)

{
  undefined1 auVar1 [16];
  int iVar2;
  char cVar3;
  int iVar4;
  longlong lVar5;
  ulonglong uVar6;
  undefined8 uVar7;
  ulonglong *puVar8;
  ulonglong uVar9;
  int local_res8 [2];
  undefined4 local_res10 [2];
  undefined4 local_res18 [2];
  undefined4 local_res20 [2];
  undefined8 in_stack_fffffffffffffd48;
  undefined4 uVar11;
  undefined4 *puVar10;
  undefined8 in_stack_fffffffffffffd50;
  undefined4 uVar14;
  undefined1 *puVar12;
  undefined8 uVar13;
  undefined1 *puVar15;
  undefined4 uVar17;
  undefined8 uVar16;
  undefined4 uVar18;
  undefined1 *puVar19;
  undefined4 uVar21;
  undefined8 uVar20;
  undefined4 *puVar22;
  undefined4 *puVar23;
  undefined4 uVar24;
  undefined4 uVar25;
  int *piVar26;
  undefined4 uVar27;
  undefined4 local_218;
  undefined4 local_214;
  undefined4 local_210;
  int local_20c;
  int local_208;
  undefined4 local_204;
  undefined4 local_200;
  undefined4 local_1fc;
  undefined4 local_1f8;
  undefined4 local_1f4;
  undefined4 local_1f0 [2];
  undefined1 local_1e8 [32];
  undefined1 local_1c8 [32];
  undefined1 local_1a8 [32];
  undefined1 local_188 [4];
  undefined1 local_184 [4];
  undefined1 local_180 [4];
  undefined1 local_17c [4];
  undefined1 local_178 [4];
  undefined1 local_174 [4];
  undefined1 local_170 [4];
  undefined1 local_16c [4];
  undefined1 local_168 [4];
  undefined1 local_164 [4];
  undefined1 local_160 [8];
  undefined1 local_158 [32];
  undefined1 local_138 [32];
  undefined1 local_118 [32];
  undefined1 local_f8 [32];
  ulonglong local_d8;
  ulonglong *local_d0;
  undefined1 local_c8 [8];
  undefined1 local_c0 [32];
  undefined1 local_a0 [32];
  undefined1 local_80 [32];
  undefined1 local_60 [40];
  
  uVar11 = (undefined4)((ulonglong)in_stack_fffffffffffffd48 >> 0x20);
  uVar14 = (undefined4)((ulonglong)in_stack_fffffffffffffd50 >> 0x20);
  FUN_141166770(local_60);
  FUN_141166770(local_80);
  FUN_141166770(local_f8);
  FUN_141167d60(local_1c8);
  FUN_141167d60(local_118);
  FUN_141167d60(local_a0);
  FUN_141167d60(local_138);
  FUN_141169130(local_1a8);
  FUN_141169130(local_158);
  FUN_141166770(local_1e8);
  FUN_141166770(local_c0);
  iVar4 = (int)(*(double *)(param_1 + 0x98) * DAT_1435384d0);
  iVar2 = *(int *)(param_2 + 3);
  if (iVar4 <= *(int *)(param_2 + 3)) {
    iVar2 = iVar4;
  }
  lVar5 = FUN_141166c30(local_1e8,*param_2,iVar2);
  if (lVar5 != 0) {
    puVar15 = local_1c8;
    cVar3 = FUN_141173280(param_1,local_1e8,*(undefined8 *)(param_1 + 0x98),
                          *(undefined8 *)(param_1 + 0xb0),
                          CONCAT44(uVar11,*(undefined4 *)(param_1 + 0xac)),CONCAT44(uVar14,1),
                          puVar15,local_res8);
    uVar11 = (undefined4)((ulonglong)puVar15 >> 0x20);
    if (cVar3 != '\0') {
      cVar3 = FUN_14118aea0(param_1,0,0,DAT_145414da0,*(undefined8 *)(param_1 + 0x98));
      if (cVar3 != '\0') {
        uVar7 = CONCAT44(uVar11,1);
        cVar3 = FUN_14118b160(param_1,local_1e8,*(undefined8 *)(param_1 + 0xb0),
                              *(undefined4 *)(param_1 + 0xac),1,*(undefined8 *)(param_1 + 0x98),
                              uVar7,local_1a8);
        uVar11 = (undefined4)((ulonglong)uVar7 >> 0x20);
        if (((cVar3 != '\0') &&
            (cVar3 = FUN_141188560(param_1,*(undefined8 *)(param_1 + 0x98)), cVar3 != '\0')) &&
           (cVar3 = FUN_1411885f0(param_1,param_2,*(undefined8 *)(param_1 + 0xb0),
                                  *(undefined4 *)(param_1 + 0xac),*(undefined8 *)(param_1 + 0x98),
                                  local_1c8,CONCAT44(uVar11,local_res8[0])), cVar3 != '\0')) {
          FUN_141167030(param_1 + 0x3d8);
          puVar15 = local_168;
          puVar12 = local_160;
          puVar10 = &local_218;
          cVar3 = FUN_141176680(param_1,local_1a8,*(undefined8 *)(param_1 + 0xb0),param_1 + 0x3f8,
                                puVar10,puVar12,local_164,puVar15);
          uVar14 = (undefined4)((ulonglong)puVar12 >> 0x20);
          uVar17 = (undefined4)((ulonglong)puVar15 >> 0x20);
          uVar11 = (undefined4)((ulonglong)puVar10 >> 0x20);
          if (cVar3 != '\0') {
            FUN_141167030(param_1 + 0x3f8);
            cVar3 = FUN_141178330(param_1,local_1e8,*(undefined8 *)(param_1 + 0x98));
            if (cVar3 != '\0') {
              uVar9 = (ulonglong)((int)(local_res8[0] + (local_res8[0] >> 0x1f & 7U)) >> 3);
              auVar1._8_8_ = 0;
              auVar1._0_8_ = uVar9;
              uVar6 = SUB168(ZEXT816(0x20) * auVar1,0);
              if (SUB168(ZEXT816(0x20) * auVar1,8) != 0) {
                uVar6 = 0xffffffffffffffff;
              }
              lVar5 = uVar6 + 8;
              if (0xfffffffffffffff7 < uVar6) {
                lVar5 = -1;
              }
              local_d8 = uVar9;
              local_d0 = (ulonglong *)thunk_FUN_1426087a8(lVar5);
              if (local_d0 == (ulonglong *)0x0) {
                puVar8 = (ulonglong *)0x0;
              }
              else {
                *local_d0 = uVar9;
                puVar8 = local_d0 + 1;
                uVar11 = 1;
                _eh_vector_constructor_iterator_(puVar8,0x20,uVar9,FUN_141166770,FUN_141166790);
              }
              if (puVar8 != (ulonglong *)0x0) {
                puVar15 = local_118;
                uVar16 = CONCAT44(uVar17,local_res8[0]);
                uVar13 = CONCAT44(uVar14,1);
                uVar7 = CONCAT44(uVar11,*(undefined4 *)(param_1 + 0xac));
                cVar3 = FUN_1411786a0(param_1,param_1 + 0x418,*(undefined8 *)(param_1 + 0x98),
                                      *(undefined8 *)(param_1 + 0xb0),uVar7,uVar13,local_1c8,uVar16,
                                      puVar15,local_60,&local_210,&local_214,puVar8);
                uVar11 = (undefined4)((ulonglong)uVar7 >> 0x20);
                uVar14 = (undefined4)((ulonglong)uVar13 >> 0x20);
                uVar17 = (undefined4)((ulonglong)uVar16 >> 0x20);
                uVar18 = (undefined4)((ulonglong)puVar15 >> 0x20);
                if (cVar3 == '\0') {
                  _eh_vector_destructor_iterator_(puVar8,0x20,puVar8[-1],FUN_141166790);
                  thunk_FUN_142608cac(puVar8 + -1,puVar8[-1] * 0x20 + 8);
                }
                else {
                  FUN_141167030(local_60);
                  uVar13 = CONCAT44(uVar14,1);
                  uVar7 = CONCAT44(uVar11,*(undefined4 *)(param_1 + 0xac));
                  cVar3 = FUN_141179530(param_1,param_1 + 0x418,*(undefined8 *)(param_1 + 0x98),
                                        *(undefined8 *)(param_1 + 0xb0),uVar7,uVar13,local_1c8,
                                        CONCAT44(uVar17,local_res8[0]),CONCAT44(uVar18,local_218),
                                        local_1a8,puVar8,local_a0,local_80,&local_1f8,&local_1f4);
                  if (cVar3 == '\0') {
                    _eh_vector_destructor_iterator_(puVar8,0x20,puVar8[-1],FUN_141166790);
                    thunk_FUN_142608cac(puVar8 + -1,puVar8[-1] * 0x20 + 8);
                  }
                  else {
                    puVar19 = local_f8;
                    puVar15 = local_138;
                    puVar12 = local_1a8;
                    uVar13 = CONCAT44((int)((ulonglong)uVar13 >> 0x20),1);
                    uVar7 = CONCAT44((int)((ulonglong)uVar7 >> 0x20),*(undefined4 *)(param_1 + 0xac)
                                    );
                    cVar3 = FUN_141178df0(param_1,param_1 + 0x418,*(undefined8 *)(param_1 + 0x98),
                                          *(undefined8 *)(param_1 + 0xb0),uVar7,uVar13,local_1c8,
                                          local_res8[0],puVar12,puVar8,puVar15,puVar19,&local_20c,
                                          &local_200);
                    uVar11 = (undefined4)((ulonglong)uVar7 >> 0x20);
                    uVar14 = (undefined4)((ulonglong)uVar13 >> 0x20);
                    uVar17 = (undefined4)((ulonglong)puVar12 >> 0x20);
                    uVar18 = (undefined4)((ulonglong)puVar15 >> 0x20);
                    uVar21 = (undefined4)((ulonglong)puVar19 >> 0x20);
                    if (cVar3 == '\0') {
                      _eh_vector_destructor_iterator_(puVar8,0x20,puVar8[-1],FUN_141166790);
                      thunk_FUN_142608cac(puVar8 + -1,puVar8[-1] * 0x20 + 8);
                    }
                    else {
                      FUN_141167030(param_1 + 0x418);
                      _eh_vector_destructor_iterator_(puVar8,0x20,puVar8[-1],FUN_141166790);
                      thunk_FUN_142608cac(puVar8 + -1,puVar8[-1] * 0x20 + 8);
                      piVar26 = &local_208;
                      puVar10 = &local_1fc;
                      puVar23 = local_res18;
                      puVar22 = local_res10;
                      uVar20 = CONCAT44(uVar21,local_214);
                      uVar16 = CONCAT44(uVar17,local_res8[0]);
                      uVar13 = CONCAT44(uVar14,1);
                      uVar7 = CONCAT44(uVar11,*(undefined4 *)(param_1 + 0xac));
                      cVar3 = FUN_141172490(param_1,local_1e8,*(undefined8 *)(param_1 + 0x98),
                                            *(undefined8 *)(param_1 + 0xb0),uVar7,uVar13,local_1a8,
                                            local_1c8,uVar16,local_118,CONCAT44(uVar18,local_210),
                                            uVar20,puVar22,puVar23,puVar10,piVar26);
                      uVar17 = (undefined4)((ulonglong)uVar16 >> 0x20);
                      uVar18 = (undefined4)((ulonglong)uVar20 >> 0x20);
                      uVar21 = (undefined4)((ulonglong)puVar22 >> 0x20);
                      uVar24 = (undefined4)((ulonglong)puVar23 >> 0x20);
                      uVar25 = (undefined4)((ulonglong)puVar10 >> 0x20);
                      uVar27 = (undefined4)((ulonglong)piVar26 >> 0x20);
                      uVar11 = (undefined4)((ulonglong)uVar7 >> 0x20);
                      uVar14 = (undefined4)((ulonglong)uVar13 >> 0x20);
                      if (cVar3 != '\0') {
                        FUN_141168590(local_118);
                        if (*(char *)(param_1 + 0xca) != '\0') {
                          local_res10[0] = *(undefined4 *)(param_1 + 0xac);
                          local_res18[0] = 0;
                        }
                        puVar15 = local_1c8;
                        cVar3 = FUN_141173280(param_1,local_1e8,*(undefined8 *)(param_1 + 0x98),
                                              *(undefined8 *)(param_1 + 0xb0),
                                              CONCAT44(uVar11,local_res10[0]),CONCAT44(uVar14,1),
                                              puVar15,local_res8);
                        uVar11 = (undefined4)((ulonglong)puVar15 >> 0x20);
                        if (cVar3 != '\0') {
                          cVar3 = FUN_14118aea0(param_1,0,0,DAT_145414da0,
                                                *(undefined8 *)(param_1 + 0x98));
                          if (cVar3 != '\0') {
                            uVar7 = CONCAT44(uVar11,1);
                            cVar3 = FUN_14118b160(param_1,local_1e8,*(undefined8 *)(param_1 + 0xb0),
                                                  local_res10[0],1,*(undefined8 *)(param_1 + 0x98),
                                                  uVar7,local_1a8);
                            uVar11 = (undefined4)((ulonglong)uVar7 >> 0x20);
                            if (cVar3 != '\0') {
                              cVar3 = FUN_14118be60(param_1,0,0,DAT_145414da0,
                                                    *(undefined8 *)(param_1 + 0x98));
                              if (((cVar3 != '\0') &&
                                  (cVar3 = FUN_14118c1b0(param_1,local_1e8,
                                                         *(undefined8 *)(param_1 + 0xb0),
                                                         local_res10[0],1,
                                                         *(undefined8 *)(param_1 + 0x98),
                                                         CONCAT44(uVar11,1),local_158),
                                  cVar3 != '\0')) &&
                                 (cVar3 = FUN_141177b30(param_1,local_1a8,local_158,
                                                        *(undefined8 *)(param_1 + 0xb0),&local_204),
                                 cVar3 != '\0')) {
                                FUN_141169710(local_1a8);
                                FUN_141169710(local_158);
                                iVar4 = (int)(*(double *)(param_1 + 0x98) * DAT_1454155d8);
                                iVar2 = *(int *)(param_2 + 3);
                                if (iVar4 <= *(int *)(param_2 + 3)) {
                                  iVar2 = iVar4;
                                }
                                lVar5 = FUN_141166c30(local_c0,*param_2,iVar2);
                                if (lVar5 != 0) {
                                  cVar3 = FUN_141171760(param_1,local_c0,
                                                        *(undefined8 *)(param_1 + 0xb0),
                                                        local_res10[0],
                                                        *(undefined8 *)(param_1 + 0x98),local_184,
                                                        local_1f0,local_188);
                                  if (cVar3 != '\0') {
                                    if (((local_20c < 1) || (*(char *)(param_1 + 0xcb) != '\0')) ||
                                       (local_208 != 0)) {
                                      local_res20[0] = *(undefined4 *)(param_1 + 0xbc);
                                    }
                                    else {
                                      cVar3 = FUN_1411718e0(param_1,*(undefined8 *)(param_1 + 0x98),
                                                            local_1e8,
                                                            *(undefined8 *)(param_1 + 0xb0),
                                                            *(undefined4 *)(param_1 + 0xac),
                                                            local_res10[0],1,local_1c8,
                                                            CONCAT44(uVar17,local_res8[0]),local_138
                                                            ,local_f8,CONCAT44(uVar18,local_20c),
                                                            CONCAT44(uVar21,local_200),
                                                            CONCAT44(uVar24,*(undefined4 *)
                                                                             (param_1 + 0xbc)),
                                                            CONCAT44(uVar25,local_204),
                                                            CONCAT44(uVar27,local_res18[0]),
                                                            local_res20,local_16c,local_170,
                                                            local_174,local_178,local_c8,local_17c,
                                                            local_180);
                                      if (cVar3 == '\0') goto LAB_141195810;
                                    }
                                    FUN_141168590(local_1c8);
                                    FUN_141168590(local_138);
                                    FUN_141167030(local_f8);
                                    *(undefined4 *)(param_1 + 0x208) = local_res10[0];
                                    *(undefined4 *)(param_1 + 0x20c) = local_res20[0];
                                    *(undefined4 *)(param_1 + 0x210) = local_1fc;
                                    lVar5 = FUN_1411684f0(param_1 + 0x218,local_a0);
                                    if ((lVar5 != 0) &&
                                       (lVar5 = FUN_141166f90(param_1 + 0x238,local_80), lVar5 != 0)
                                       ) {
                                      *(undefined4 *)(param_1 + 600) = local_1f8;
                                      *(undefined4 *)(param_1 + 0x25c) = local_1f4;
                                      *(undefined4 *)(param_1 + 0x260) = local_1f0[0];
                                      uVar7 = 1;
                                      goto LAB_141195812;
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
LAB_141195810:
  uVar7 = 0;
LAB_141195812:
  FUN_141166790(local_c0);
  FUN_141166790(local_1e8);
  FUN_141169150(local_158);
  FUN_141169150(local_1a8);
  FUN_141167d80(local_138);
  FUN_141167d80(local_a0);
  FUN_141167d80(local_118);
  FUN_141167d80(local_1c8);
  FUN_141166790(local_f8);
  FUN_141166790(local_80);
  FUN_141166790(local_60);
  return uVar7;
}


