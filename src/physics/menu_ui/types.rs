use bevy::prelude::*;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Paused(pub bool);

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ActiveMenu {
    pub is_settings_open: bool,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct GameplayInputDelay(pub f32);

#[derive(Resource, Default, Debug, Clone)]
pub struct ActiveSettingInput {
    pub focused_setting: Option<SettingType>,
    pub current_text: String,
}

// --- Components ---

#[derive(Component)]
pub struct MainMenuContainer;

#[derive(Component)]
pub struct PauseMenuContainer;

#[derive(Component)]
pub struct SettingsMenuContainer;

#[derive(Component)]
pub struct SettingsInnerList {
    pub scroll_offset: f32,
}

#[derive(Component)]
pub enum MenuButton {
    StartGame,
    FindMatch,
    Continue,
    OpenSettings,
    CloseSettings,
    BackToMainMenu,
    Quit,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SettingType {
    // Physics
    Gravity,
    PlayerAccel,
    PlayerJumpForce,
    BoundaryRestitution,
    PlayerRestitution,
    AirFriction,
    GroundFriction,
    MovementStopFriction,
    WallSlideSpeed,
    WallJumpPushForce,
    FastFallAcceleration,
    AirAccel,
    PlayerBaseRadius,
    PlayerBaseMass,
    PlayerVisualOffset,
    PlayerAimOffsetY,
    BoundaryKnockbackSpeed,
    BoundaryDamageLockout,
    BoundaryDeflectLockout,
    SpawnInvincibilityGracePeriod,
    BoundaryHazardDamage,
    FastFallStickThreshold,
    FastFallVelocityLimit,
    WallClingStickThreshold,
    MaxJumpAllowance,
    CollisionPenetrationSkinBuffer,
    OverlappingPushFactor,
    GroundedSlopeThreshold,
    WallContactSlopeThreshold,
    BulletKnockbackConstant,

    // P1 Character
    P1Health,
    P1Speed,
    P1Size,
    P1Damage,
    P1BulletRange,
    P1BulletSpeed,
    P1BulletGravity,
    P1BulletSizeMult,
    P1BulletGrowth,
    P1MaxAmmo,
    P1ReloadTime,
    P1FireRate,
    P1Bounces,
    P1BounceSpeedMultiplier,
    P1BlockDuration,
    P1BlockCooldown,
    P1BlockBorderBoost,

    // P2 Character
    P2Health,
    P2Speed,
    P2Size,
    P2Damage,
    P2BulletRange,
    P2BulletSpeed,
    P2BulletGravity,
    P2BulletSizeMult,
    P2BulletGrowth,
    P2MaxAmmo,
    P2ReloadTime,
    P2FireRate,
    P2Bounces,
    P2BounceSpeedMultiplier,
    P2BlockDuration,
    P2BlockCooldown,
    P2BlockBorderBoost,

    // Keyboard Controls
    KbMoveLeft,
    KbMoveRight,
    KbJump,
    KbFastFall,
    KbBlock,
    KbShoot,
    KbReload,

    // Controller Controls
    CtrlJump,
    CtrlBlock,
    CtrlShoot,
    CtrlReload,
}

#[derive(Component)]
pub struct SettingInputBox(pub SettingType);

#[derive(Component)]
pub struct SettingValueText(pub SettingType);
