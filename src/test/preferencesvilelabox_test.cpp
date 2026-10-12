#include <QCheckBox>
#include <QSpinBox>

#include "control/controlobject.h"
#include "preferences/dialog/dlgprefvilelabox.h"
#include "test/mixxxtest.h"

class VilelaboxPreferencesTest : public MixxxTest {};

TEST_F(VilelaboxPreferencesTest, ApplyUpdatesLoadedSkinAndCancelPreservesSettings) {
    const ConfigKey key("[RbxOriginal]", "show_mixer");
    ControlObject mixer(key);
    mixer.set(1);
    DlgPrefVilelabox page(nullptr, config());
    auto* checkbox = page.findChild<QCheckBox*>("RbxOriginal_show_mixer");
    auto* limit = page.findChild<QSpinBox*>("relatedLimit");
    ASSERT_NE(nullptr, checkbox);
    ASSERT_NE(nullptr, limit);
    EXPECT_TRUE(checkbox->isChecked());

    checkbox->setChecked(false);
    limit->setValue(250);
    EXPECT_DOUBLE_EQ(1, mixer.get()); // Editing does not change the live layout.
    page.slotCancel();
    EXPECT_TRUE(checkbox->isChecked());
    EXPECT_EQ(100, limit->value());

    checkbox->setChecked(false);
    limit->setValue(250);
    page.slotApply();
    EXPECT_DOUBLE_EQ(0, mixer.get());
    EXPECT_FALSE(config()->getValue(key, true));
    EXPECT_EQ(250, config()->getValue(ConfigKey("[Vilelabox]", "RelatedLimit"), 0));
}

TEST_F(VilelaboxPreferencesTest, DefaultsRemainPendingAndInactiveSkinSettingsPersist) {
    const ConfigKey limitKey("[Vilelabox]", "RelatedLimit");
    const ConfigKey timeoutKey("[Vilelabox]", "VocalTimeoutSeconds");
    const ConfigKey skinKey("[RbxFlow]", "show_pitch");
    config()->setValue(limitKey, 400);
    config()->setValue(timeoutKey, 90);
    config()->setValue(skinKey, false);
    DlgPrefVilelabox page(nullptr, config());
    page.slotResetToDefaults();
    EXPECT_EQ(400, config()->getValue(limitKey, 0));
    EXPECT_EQ(90, config()->getValue(timeoutKey, 0));
    EXPECT_FALSE(config()->getValue(skinKey, true));
    page.slotApply();
    EXPECT_EQ(100, config()->getValue(limitKey, 0));
    EXPECT_EQ(900, config()->getValue(timeoutKey, 0));
    EXPECT_TRUE(config()->getValue(skinKey, false));
}
