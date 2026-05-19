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

    // Character Stats parameterized by Player Index (0..7)
    PlayerHealth(usize),
    PlayerSpeed(usize),
    PlayerSize(usize),
    PlayerDamage(usize),
    PlayerBulletRange(usize),
    PlayerBulletSpeed(usize),
    PlayerBulletGravity(usize),
    PlayerBulletSizeMult(usize),
    PlayerBulletGrowth(usize),
    PlayerMaxAmmo(usize),
    PlayerReloadTime(usize),
    PlayerFireRate(usize),
    PlayerBounces(usize),
    PlayerBounceSpeedMultiplier(usize),
    PlayerBlockDuration(usize),
    PlayerBlockCooldown(usize),
    PlayerBlockBorderBoost(usize),

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
